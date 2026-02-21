//! PUBLISH message handler -- broadcasts events to subscribers.

use clasp_core::{codec, Action, ErrorMessage, Message, SecurityMode, SignalType};
use tracing::{debug, warn};

use crate::gesture::GestureResult;
use crate::p2p::{analyze_address, P2PAddressType};
use super::{broadcast_to_subscriber_list, HandlerContext, MessageResult};

pub(crate) async fn handle(
    pub_msg: &clasp_core::PublishMessage,
    original_msg: &Message,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let session = ctx.session.as_ref()?;

    if ctx.security_mode == SecurityMode::Authenticated
        && !session.has_scope(Action::Write, &pub_msg.address)
    {
        warn!(
            "Session {} denied PUBLISH to {} - insufficient scope",
            session.id, pub_msg.address
        );
        let error = Message::Error(ErrorMessage {
            code: 301,
            message: "Insufficient scope for publish operation".to_string(),
            address: Some(pub_msg.address.clone()),
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    // SECURITY: Federation namespace enforcement -- prevents a compromised or
    // misconfigured peer from publishing events outside its declared namespaces.
    // Without this check, a peer could inject arbitrary events on the hub router.
    #[cfg(feature = "federation")]
    if session.is_federation_peer() {
        let namespaces = session.federation_namespaces();
        if !namespaces.is_empty() {
            let in_scope = namespaces
                .iter()
                .any(|ns| clasp_core::address::glob_match(ns, &pub_msg.address));
            if !in_scope {
                warn!(
                    "Federation peer {} denied PUBLISH to {} - outside declared namespaces",
                    session.id, pub_msg.address
                );
                let error = Message::Error(ErrorMessage {
                    code: 403,
                    message: "PUBLISH outside declared federation namespace".to_string(),
                    address: Some(pub_msg.address.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }
        }
    }

    if let Some(ref validator) = ctx.write_validator {
        let pub_value = pub_msg
            .value
            .as_ref()
            .cloned()
            .unwrap_or(clasp_core::Value::Null);
        if let Err(reason) =
            validator.validate_write(&pub_msg.address, &pub_value, session, ctx.state)
        {
            warn!(
                "Session {} denied PUBLISH to {} by write validator: {}",
                session.id, pub_msg.address, reason
            );
            let error = Message::Error(ErrorMessage {
                code: 403,
                message: reason,
                address: Some(pub_msg.address.clone()),
                correlation_id: None,
            });
            let bytes = codec::encode(&error).ok()?;
            return Some(MessageResult::Send(bytes));
        }
    }

    // Check for P2P signaling addresses
    match analyze_address(&pub_msg.address) {
        P2PAddressType::Signal { target_session } => {
            debug!("P2P signal from {} to {}", session.id, target_session);
            if let Ok(bytes) = codec::encode(original_msg) {
                if let Some(target) = ctx.sessions.get(&target_session) {
                    let _ = target.send(bytes).await;
                } else {
                    warn!("P2P signal target session not found: {}", target_session);
                    let error = Message::Error(ErrorMessage {
                        code: 404,
                        message: format!("Target session not found: {}", target_session),
                        address: Some(pub_msg.address.clone()),
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    return Some(MessageResult::Send(bytes));
                }
            }
            return Some(MessageResult::None);
        }
        P2PAddressType::Announce => {
            debug!("P2P announce from session {}", session.id);
            ctx.p2p_capabilities.register(&session.id);

            let subscribers = ctx.subscriptions.find_subscribers(&pub_msg.address, None);
            if let Ok(bytes) = codec::encode(original_msg) {
                broadcast_to_subscriber_list(
                    &bytes,
                    &subscribers,
                    ctx.sessions,
                    Some(&session.id),
                );
            }
            return Some(MessageResult::None);
        }
        P2PAddressType::NotP2P => {}
    }

    let signal_type = pub_msg.signal;

    // Check for gesture coalescing
    if let Some(registry) = ctx.gesture_registry {
        if signal_type == Some(SignalType::Gesture) {
            match registry.process(pub_msg) {
                GestureResult::Forward(messages) => {
                    for forward_msg in messages {
                        let msg_to_send = Message::Publish(forward_msg.clone());
                        let subscribers = ctx.subscriptions
                            .find_subscribers(&forward_msg.address, signal_type);
                        if let Ok(bytes) = codec::encode(&msg_to_send) {
                            broadcast_to_subscriber_list(
                                &bytes,
                                &subscribers,
                                ctx.sessions,
                                Some(&session.id),
                            );
                        }
                    }
                    return Some(MessageResult::None);
                }
                GestureResult::Buffered => {
                    return Some(MessageResult::None);
                }
                GestureResult::PassThrough => {}
            }
        }
    }

    let subscribers = ctx.subscriptions.find_subscribers(&pub_msg.address, signal_type);

    #[cfg(feature = "metrics")]
    metrics::histogram!("clasp_broadcast_fanout").record(subscribers.len() as f64);

    if let Ok(bytes) = codec::encode(original_msg) {
        broadcast_to_subscriber_list(
            &bytes,
            &subscribers,
            ctx.sessions,
            Some(&session.id),
        );
    }

    #[cfg(feature = "journal")]
    ctx.state.journal_publish(
        &pub_msg.address,
        signal_type.unwrap_or(SignalType::Event),
        pub_msg.value.as_ref(),
        &session.id,
    );

    #[cfg(feature = "rules")]
    if let Some(ref engine) = ctx.rules_engine {
        let pub_value = pub_msg
            .value
            .as_ref()
            .cloned()
            .unwrap_or(clasp_core::Value::Null);
        let actions = engine.lock().evaluate(
            &pub_msg.address,
            &pub_value,
            signal_type.unwrap_or(SignalType::Event),
            Some(session.id.as_str()),
            |addr| ctx.state.get(addr),
        );
        if !actions.is_empty() {
            crate::router::execute_rule_actions(
                actions, ctx.state, ctx.sessions, ctx.subscriptions,
            );
        }
    }

    Some(MessageResult::None)
}
