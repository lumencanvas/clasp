//! SET message handler -- applies state changes and broadcasts to subscribers.

use clasp_core::{codec, AckMessage, Action, ErrorMessage, Message, SecurityMode, SignalType};
use tracing::warn;

use super::{broadcast_to_subscriber_list, HandlerContext, MessageResult};

pub(crate) async fn handle(
    set: &clasp_core::SetMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let session = ctx.session.as_ref()?;

    // See pentest PAT-05: Subscription Scope Escape
    if ctx.security_mode == SecurityMode::Authenticated
        && !session.has_scope(Action::Write, &set.address)
    {
        warn!(
            "Session {} denied SET to {} - insufficient scope",
            session.id, set.address
        );
        let error = Message::Error(ErrorMessage {
            code: 301,
            message: "Insufficient scope for write operation".to_string(),
            address: Some(set.address.clone()),
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    // SECURITY: Federation namespace enforcement -- prevents a compromised or
    // misconfigured peer from writing to addresses outside its declared namespaces.
    // Without this check, a peer could overwrite arbitrary state on the hub router.
    // See pentest FED-01: Namespace Escape, FED-10: Cross-Namespace Write
    #[cfg(feature = "federation")]
    if session.is_federation_peer() {
        let namespaces = session.federation_namespaces();
        if !namespaces.is_empty() {
            let in_scope = namespaces
                .iter()
                .any(|ns| clasp_core::address::glob_match(ns, &set.address));
            if !in_scope {
                warn!(
                    "Federation peer {} denied SET to {} - outside declared namespaces",
                    session.id, set.address
                );
                let error = Message::Error(ErrorMessage {
                    code: 403,
                    message: "SET outside declared federation namespace".to_string(),
                    address: Some(set.address.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }
        }
    }

    if let Some(ref validator) = ctx.write_validator {
        if let Err(reason) =
            validator.validate_write(&set.address, &set.value, session, ctx.state)
        {
            warn!(
                "Session {} denied SET to {} by write validator: {}",
                session.id, set.address, reason
            );
            let error = Message::Error(ErrorMessage {
                code: 403,
                message: reason,
                address: Some(set.address.clone()),
                correlation_id: None,
            });
            let bytes = codec::encode(&error).ok()?;
            return Some(MessageResult::Send(bytes));
        }
    }

    match ctx.state.apply_set(set, &session.id) {
        Ok(revision) => {
            let subscribers =
                ctx.subscriptions.find_subscribers(&set.address, Some(SignalType::Param));

            #[cfg(feature = "metrics")]
            metrics::histogram!("clasp_broadcast_fanout").record(subscribers.len() as f64);

            let mut updated_set = set.clone();
            updated_set.revision = Some(revision);
            let broadcast_msg = Message::Set(updated_set);

            if let Ok(bytes) = codec::encode(&broadcast_msg) {
                broadcast_to_subscriber_list(
                    &bytes,
                    &subscribers,
                    ctx.sessions,
                    None,
                );
            }

            #[cfg(feature = "rules")]
            if let Some(ref engine) = ctx.rules_engine {
                let actions = engine.lock().evaluate(
                    &set.address,
                    &set.value,
                    SignalType::Param,
                    Some(session.id.as_str()),
                    |addr| ctx.state.get(addr),
                );
                if !actions.is_empty() {
                    crate::router::execute_rule_actions(
                        actions, ctx.state, ctx.sessions, ctx.subscriptions,
                    );
                }
            }

            let ack = Message::Ack(AckMessage {
                address: Some(set.address.clone()),
                revision: Some(revision),
                locked: None,
                holder: None,
                correlation_id: None,
            });
            let ack_bytes = codec::encode(&ack).ok()?;
            Some(MessageResult::Send(ack_bytes))
        }
        Err(e) => {
            #[cfg(feature = "metrics")]
            metrics::counter!("clasp_errors_total", "code" => "400").increment(1);
            let error = Message::Error(ErrorMessage {
                code: 400,
                message: format!("{:?}", e),
                address: Some(set.address.clone()),
                correlation_id: None,
            });
            let bytes = codec::encode(&error).ok()?;
            Some(MessageResult::Send(bytes))
        }
    }
}
