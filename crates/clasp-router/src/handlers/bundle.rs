//! BUNDLE message handler -- atomic multi-message operations.
//!
//! Validates all inner SET/PUBLISH messages before applying any (two-phase
//! commit). If any message fails scope or write validation, the entire
//! bundle is rejected.

use clasp_core::{
    codec, AckMessage, Action, ErrorMessage, Message, SecurityMode, SetMessage, SignalType,
};
use tracing::{debug, error, warn};

use super::{broadcast_to_subscriber_list, HandlerContext, MessageResult};

pub(crate) async fn handle(
    bundle: &clasp_core::BundleMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let session = ctx.session.as_ref()?;

    // PHASE 1: Validate ALL messages first (atomic validation)
    let mut validated_sets: Vec<&SetMessage> = Vec::new();
    let mut validated_pubs: Vec<&clasp_core::PublishMessage> = Vec::new();

    for inner_msg in &bundle.messages {
        match inner_msg {
            Message::Set(set) => {
                if ctx.security_mode == SecurityMode::Authenticated
                    && !session.has_scope(Action::Write, &set.address)
                {
                    warn!(
                        "Session {} denied bundled SET to {} - rejecting entire bundle",
                        session.id, set.address
                    );
                    let err = Message::Error(ErrorMessage {
                        code: 403,
                        message: format!(
                            "Bundle rejected: insufficient scope for SET to {}",
                            set.address
                        ),
                        address: Some(set.address.clone()),
                        correlation_id: None,
                    });
                    let err_bytes = codec::encode(&err).ok()?;
                    return Some(MessageResult::Send(err_bytes));
                }

                if let Some(ref validator) = ctx.write_validator {
                    if let Err(reason) =
                        validator.validate_write(&set.address, &set.value, session, ctx.state)
                    {
                        warn!(
                            "Session {} denied bundled SET to {} by write validator - rejecting entire bundle: {}",
                            session.id, set.address, reason
                        );
                        let err = Message::Error(ErrorMessage {
                            code: 403,
                            message: format!("Bundle rejected: {}", reason),
                            address: Some(set.address.clone()),
                            correlation_id: None,
                        });
                        let err_bytes = codec::encode(&err).ok()?;
                        return Some(MessageResult::Send(err_bytes));
                    }
                }

                validated_sets.push(set);
            }
            Message::Publish(pub_msg) => {
                if ctx.security_mode == SecurityMode::Authenticated
                    && !session.has_scope(Action::Write, &pub_msg.address)
                {
                    warn!(
                        "Session {} denied bundled PUBLISH to {} - rejecting entire bundle",
                        session.id, pub_msg.address
                    );
                    let err = Message::Error(ErrorMessage {
                        code: 403,
                        message: format!(
                            "Bundle rejected: insufficient scope for PUBLISH to {}",
                            pub_msg.address
                        ),
                        address: Some(pub_msg.address.clone()),
                        correlation_id: None,
                    });
                    let err_bytes = codec::encode(&err).ok()?;
                    return Some(MessageResult::Send(err_bytes));
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
                            "Session {} denied bundled PUBLISH to {} by write validator - rejecting entire bundle: {}",
                            session.id, pub_msg.address, reason
                        );
                        let err = Message::Error(ErrorMessage {
                            code: 403,
                            message: format!("Bundle rejected: {}", reason),
                            address: Some(pub_msg.address.clone()),
                            correlation_id: None,
                        });
                        let err_bytes = codec::encode(&err).ok()?;
                        return Some(MessageResult::Send(err_bytes));
                    }
                }

                validated_pubs.push(pub_msg);
            }
            _ => {
                debug!("Ignoring {:?} message type in bundle", inner_msg);
            }
        }
    }

    // PHASE 2: Apply all validated changes atomically
    let mut applied_revisions: Vec<(String, u64)> = Vec::new();

    for set in &validated_sets {
        match ctx.state.apply_set(set, &session.id) {
            Ok(revision) => {
                applied_revisions.push((set.address.clone(), revision));

                let subscribers = ctx
                    .subscriptions
                    .find_subscribers(&set.address, Some(SignalType::Param));

                let mut updated_set: SetMessage = (*set).clone();
                updated_set.revision = Some(revision);
                let broadcast_msg = Message::Set(updated_set);

                if let Ok(bytes) = codec::encode(&broadcast_msg) {
                    broadcast_to_subscriber_list(&bytes, &subscribers, ctx.sessions, None);
                }
            }
            Err(e) => {
                error!("Bundle SET apply failed after validation: {}", e);
            }
        }
    }

    for pub_msg in &validated_pubs {
        let subscribers = ctx
            .subscriptions
            .find_subscribers(&pub_msg.address, pub_msg.signal);

        let inner_msg = Message::Publish((*pub_msg).clone());
        if let Ok(bytes) = codec::encode(&inner_msg) {
            broadcast_to_subscriber_list(&bytes, &subscribers, ctx.sessions, Some(&session.id));
        }
    }

    let ack = Message::Ack(AckMessage {
        address: None,
        revision: applied_revisions.last().map(|(_, r)| *r),
        locked: None,
        holder: None,
        correlation_id: None,
    });
    let ack_bytes = codec::encode(&ack).ok()?;
    Some(MessageResult::Send(ack_bytes))
}
