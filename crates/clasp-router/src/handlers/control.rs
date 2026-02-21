//! Control message handlers -- PING, QUERY, REPLAY, ANNOUNCE, SYNC.
//!
//! Lightweight handlers for protocol housekeeping: heartbeat, signal discovery,
//! journal replay, signal announcement, and clock synchronization.

use clasp_core::{codec, AckMessage, Message};
#[cfg(feature = "journal")]
use clasp_core::{ErrorMessage, PublishMessage, SecurityMode, SetMessage};
use tracing::debug;
#[cfg(feature = "journal")]
use tracing::warn;

use super::{HandlerContext, MessageResult};

pub(crate) async fn handle_ping(
    _ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let pong = Message::Pong;
    let bytes = codec::encode(&pong).ok()?;
    Some(MessageResult::Send(bytes))
}

pub(crate) async fn handle_query(
    query: &clasp_core::QueryMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let signals = ctx.state.query_signals(&query.pattern);
    let result = Message::Result(clasp_core::ResultMessage { signals });
    let bytes = codec::encode(&result).ok()?;
    Some(MessageResult::Send(bytes))
}

#[cfg(feature = "journal")]
pub(crate) async fn handle_replay(
    replay: &clasp_core::ReplayMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let session = ctx.session.as_ref()?;

    if ctx.security_mode == SecurityMode::Authenticated
        && !session.has_strict_read_scope(&replay.pattern)
    {
        warn!(
            "Session {} denied REPLAY to {} - insufficient scope",
            session.id, replay.pattern
        );
        let error = Message::Error(ErrorMessage {
            code: 301,
            message: "Insufficient scope for replay".to_string(),
            address: Some(replay.pattern.clone()),
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    if let Some(journal) = ctx.state.journal() {
        match journal
            .query(
                &replay.pattern,
                replay.from,
                replay.to,
                replay.limit,
                &replay.types,
            )
            .await
        {
            Ok(entries) => {
                for entry in entries {
                    let msg = if entry.msg_type == 0x21 {
                        Message::Set(SetMessage {
                            address: entry.address,
                            value: entry.value,
                            revision: entry.revision,
                            lock: false,
                            unlock: false,
                        })
                    } else {
                        Message::Publish(PublishMessage {
                            address: entry.address,
                            signal: Some(entry.signal_type),
                            value: Some(entry.value),
                            payload: None,
                            samples: None,
                            rate: None,
                            id: None,
                            phase: None,
                            timestamp: None,
                            timeline: None,
                        })
                    };
                    if let Ok(bytes) = codec::encode(&msg) {
                        let _ = ctx.sender.send(bytes).await;
                    }
                }
            }
            Err(e) => {
                let error = Message::Error(ErrorMessage {
                    code: 500,
                    message: format!("Journal query failed: {}", e),
                    address: Some(replay.pattern.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }
        }
    } else {
        let error = Message::Error(ErrorMessage {
            code: 501,
            message: "Journal not configured on this router".to_string(),
            address: Some(replay.pattern.clone()),
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    Some(MessageResult::None)
}

pub(crate) async fn handle_announce(
    announce: &clasp_core::AnnounceMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    ctx.state.register_signals(announce.signals.clone());
    debug!(
        "Registered {} signals in namespace {}",
        announce.signals.len(),
        announce.namespace
    );
    let ack = Message::Ack(AckMessage {
        address: Some(announce.namespace.clone()),
        revision: None,
        locked: None,
        holder: None,
        correlation_id: None,
    });
    let bytes = codec::encode(&ack).ok()?;
    Some(MessageResult::Send(bytes))
}

pub(crate) async fn handle_sync(
    sync_msg: &clasp_core::SyncMessage,
    _ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let now = clasp_core::time::now();
    let response = Message::Sync(clasp_core::SyncMessage {
        t1: sync_msg.t1,
        t2: Some(now),
        t3: Some(now),
    });
    let bytes = codec::encode(&response).ok()?;
    Some(MessageResult::Send(bytes))
}
