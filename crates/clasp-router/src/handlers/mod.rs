//! Message handler dispatch and shared context.
//!
//! Each CLASP message type has a dedicated handler module. The `handle_message()`
//! function dispatches to the appropriate handler based on message type.

pub mod bundle;
pub mod control;
#[cfg(feature = "federation")]
pub mod federation;
pub mod get;
pub mod hello;
pub mod publish;
pub mod set;
pub mod subscribe;

use bytes::Bytes;
use clasp_core::{
    codec, ErrorMessage, Frame, Message, SecurityMode, SnapshotMessage, TokenValidator,
};
#[cfg(feature = "rules")]
use clasp_rules::RulesEngine;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info, warn, Instrument};

use crate::{
    gesture::GestureRegistry,
    p2p::P2PCapabilities,
    router::{RouterConfig, SnapshotFilter, WriteValidator},
    session::{Session, SessionId},
    state::RouterState,
    subscription::SubscriptionManager,
};

/// Result of handling a message
pub(crate) enum MessageResult {
    NewSession(Arc<Session>),
    Send(Bytes),
    #[allow(dead_code)]
    Broadcast(Bytes, SessionId),
    Disconnect,
    None,
}

/// Maximum params per snapshot chunk to stay under frame size limit.
const MAX_SNAPSHOT_CHUNK_SIZE: usize = 800;

/// Shared context passed to all message handlers.
pub(crate) struct HandlerContext<'a> {
    pub session: &'a Option<Arc<Session>>,
    pub sender: &'a Arc<dyn clasp_transport::TransportSender>,
    pub sessions: &'a Arc<DashMap<SessionId, Arc<Session>>>,
    pub subscriptions: &'a Arc<SubscriptionManager>,
    pub state: &'a Arc<RouterState>,
    pub config: &'a RouterConfig,
    pub security_mode: SecurityMode,
    pub token_validator: &'a Option<Arc<dyn TokenValidator>>,
    pub p2p_capabilities: &'a Arc<P2PCapabilities>,
    pub gesture_registry: &'a Option<Arc<GestureRegistry>>,
    pub write_validator: &'a Option<Arc<dyn WriteValidator>>,
    pub snapshot_filter: &'a Option<Arc<dyn SnapshotFilter>>,
    #[cfg(feature = "rules")]
    pub rules_engine: &'a Option<Arc<parking_lot::Mutex<RulesEngine>>>,
}

/// Return a short uppercase label for a [`Message`] variant.
fn message_type_str(msg: &Message) -> &'static str {
    match msg {
        Message::Hello(_) => "HELLO",
        Message::Welcome(_) => "WELCOME",
        Message::Announce(_) => "ANNOUNCE",
        Message::Subscribe(_) => "SUBSCRIBE",
        Message::Unsubscribe(_) => "UNSUBSCRIBE",
        Message::Publish(_) => "PUBLISH",
        Message::Set(_) => "SET",
        Message::Get(_) => "GET",
        Message::Snapshot(_) => "SNAPSHOT",
        Message::Replay(_) => "REPLAY",
        Message::FederationSync(_) => "FEDERATION_SYNC",
        Message::Bundle(_) => "BUNDLE",
        Message::Sync(_) => "SYNC",
        Message::Ping => "PING",
        Message::Pong => "PONG",
        Message::Ack(_) => "ACK",
        Message::Error(_) => "ERROR",
        Message::Query(_) => "QUERY",
        Message::Result(_) => "RESULT",
    }
}

/// Return a short lowercase label for metrics recording.
#[cfg(feature = "metrics")]
fn metrics_type_str(msg: &Message) -> &'static str {
    match msg {
        Message::Hello(_) => "hello",
        Message::Welcome(_) => "welcome",
        Message::Announce(_) => "announce",
        Message::Subscribe(_) => "subscribe",
        Message::Unsubscribe(_) => "unsubscribe",
        Message::Publish(_) => "publish",
        Message::Set(_) => "set",
        Message::Get(_) => "get",
        Message::Snapshot(_) => "snapshot",
        Message::Replay(_) => "replay",
        Message::FederationSync(_) => "federation_sync",
        Message::Bundle(_) => "bundle",
        Message::Sync(_) => "sync",
        Message::Ping => "ping",
        Message::Pong => "pong",
        Message::Ack(_) => "ack",
        Message::Error(_) => "error",
        Message::Query(_) => "query",
        Message::Result(_) => "result",
    }
}

/// Dispatch an incoming message to the appropriate handler.
pub(crate) async fn handle_message(
    msg: &Message,
    _frame: &Frame,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let msg_type = message_type_str(msg);
    let span = tracing::debug_span!("handle_message", msg_type);

    #[cfg(feature = "metrics")]
    let start = std::time::Instant::now();
    #[cfg(feature = "metrics")]
    let metrics_label = metrics_type_str(msg);
    #[cfg(feature = "metrics")]
    metrics::counter!("clasp_messages_total", "type" => metrics_label).increment(1);

    let result = async {
        match msg {
            Message::Hello(hello) => hello::handle(hello, ctx).await,
            Message::Subscribe(sub) => subscribe::handle_subscribe(sub, ctx).await,
            Message::Unsubscribe(unsub) => subscribe::handle_unsubscribe(unsub, ctx).await,
            Message::Set(set) => set::handle(set, ctx).await,
            Message::Get(get) => get::handle(get, ctx).await,
            Message::Publish(pub_msg) => publish::handle(pub_msg, msg, ctx).await,
            Message::Bundle(bundle) => bundle::handle(bundle, ctx).await,
            Message::Ping => control::handle_ping(ctx).await,
            Message::Query(query) => control::handle_query(query, ctx).await,
            #[cfg(feature = "journal")]
            Message::Replay(replay) => control::handle_replay(replay, ctx).await,
            Message::Announce(announce) => control::handle_announce(announce, ctx).await,
            Message::Sync(sync_msg) => control::handle_sync(sync_msg, ctx).await,
            #[cfg(feature = "federation")]
            Message::FederationSync(fed_msg) => federation::handle(fed_msg, ctx).await,
            _ => Some(MessageResult::None),
        }
    }
    .instrument(span)
    .await;

    #[cfg(feature = "metrics")]
    {
        let elapsed = start.elapsed().as_secs_f64();
        metrics::histogram!("clasp_message_latency_seconds", "type" => metrics_label)
            .record(elapsed);
    }

    result
}

/// Send a snapshot, chunking if too large for a single frame.
pub(crate) async fn send_chunked_snapshot(
    sender: &Arc<dyn clasp_transport::TransportSender>,
    snapshot: SnapshotMessage,
) {
    let param_count = snapshot.params.len();

    if param_count <= MAX_SNAPSHOT_CHUNK_SIZE {
        let msg = Message::Snapshot(snapshot);
        if let Ok(bytes) = codec::encode(&msg) {
            let _ = sender.send(bytes).await;
        } else {
            warn!("Failed to encode snapshot ({} params)", param_count);
        }
        return;
    }

    let chunks = snapshot.params.chunks(MAX_SNAPSHOT_CHUNK_SIZE);
    let chunk_count = param_count.div_ceil(MAX_SNAPSHOT_CHUNK_SIZE);

    debug!(
        "Chunking snapshot of {} params into {} chunks",
        param_count, chunk_count
    );

    for (i, chunk) in chunks.enumerate() {
        let chunk_snapshot = SnapshotMessage {
            params: chunk.to_vec(),
        };
        let msg = Message::Snapshot(chunk_snapshot);
        match codec::encode(&msg) {
            Ok(bytes) => {
                if let Err(e) = sender.send(bytes).await {
                    warn!(
                        "Failed to send snapshot chunk {}/{}: {}",
                        i + 1,
                        chunk_count,
                        e
                    );
                    break;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to encode snapshot chunk {}/{}: {}",
                    i + 1,
                    chunk_count,
                    e
                );
            }
        }
    }
}

/// Subscriber count threshold above which broadcasts use concurrent fan-out
/// via `tokio::spawn` to reduce DashMap lock contention on the sending path.
const CONCURRENT_BROADCAST_THRESHOLD: usize = 10;

/// Try to send a message to a session with drop tracking.
pub(crate) fn try_send_with_drop_tracking_sync(
    session: &Arc<Session>,
    data: Bytes,
    session_id: &SessionId,
) {
    if let Err(e) = session.try_send(data) {
        warn!(
            "Failed to send to {}: {} (buffer full, dropping)",
            session_id, e
        );

        if session.record_drop() {
            let session = Arc::clone(session);
            let session_id = session_id.clone();
            let drops = session.drops_in_window();
            tokio::spawn(async move {
                let error = Message::Error(ErrorMessage {
                    code: 503,
                    message: format!(
                        "Buffer overflow: messages being dropped ({} drops in last 10 seconds)",
                        drops
                    ),
                    address: None,
                    correlation_id: None,
                });
                if let Ok(error_bytes) = codec::encode(&error) {
                    if let Err(e) = session.send(error_bytes).await {
                        warn!("Failed to send drop notification to {}: {}", session_id, e);
                    } else {
                        info!(
                            "Sent buffer overflow notification to session {} ({} drops)",
                            session_id, drops
                        );
                    }
                }
            });
        }
    }
}

/// Broadcast to all sessions except one (non-blocking).
///
/// Collects session handles first to minimize DashMap lock duration.
/// When the session count exceeds [`CONCURRENT_BROADCAST_THRESHOLD`],
/// fan-out is performed via `tokio::spawn` to avoid blocking the caller
/// on a large sequential send loop.
pub(crate) fn broadcast_to_subscribers(
    data: &Bytes,
    sessions: &Arc<DashMap<SessionId, Arc<Session>>>,
    exclude: &SessionId,
) {
    // Collect handles in one DashMap pass, then release all shard locks.
    let targets: Vec<(SessionId, Arc<Session>)> = sessions
        .iter()
        .filter(|entry| entry.key() != exclude)
        .map(|entry| (entry.key().clone(), Arc::clone(entry.value())))
        .collect();

    if targets.len() > CONCURRENT_BROADCAST_THRESHOLD {
        // Spawn a task to perform the fan-out off the caller's hot path.
        let data = data.clone();
        tokio::spawn(async move {
            for (session_id, session) in targets {
                try_send_with_drop_tracking_sync(&session, data.clone(), &session_id);
            }
        });
    } else {
        for (session_id, session) in targets {
            try_send_with_drop_tracking_sync(&session, data.clone(), &session_id);
        }
    }
}

/// Send pre-encoded bytes to a list of subscriber sessions, optionally excluding
/// one session (typically the sender).
///
/// Resolves subscriber session IDs against the sessions DashMap in a single pass
/// to collect `Arc<Session>` handles, then sends to all without holding DashMap
/// references. When the subscriber count exceeds [`CONCURRENT_BROADCAST_THRESHOLD`],
/// the sends are offloaded to a spawned task for concurrent fan-out.
///
/// Pass `Some(&session_id)` to skip the sender (e.g. for PUBLISH), or `None` to
/// broadcast to every subscriber (e.g. for SET where the sender also receives the
/// update via its subscription).
pub(crate) fn broadcast_to_subscriber_list(
    data: &Bytes,
    subscriber_ids: &[SessionId],
    sessions: &Arc<DashMap<SessionId, Arc<Session>>>,
    exclude: Option<&SessionId>,
) {
    // Collect Arc<Session> handles in one pass over the DashMap, then release locks.
    let targets: Vec<(SessionId, Arc<Session>)> = subscriber_ids
        .iter()
        .filter(|id| match exclude {
            Some(ex) => *id != ex,
            None => true,
        })
        .filter_map(|id| {
            sessions
                .get(id)
                .map(|entry| (id.clone(), Arc::clone(entry.value())))
        })
        .collect();

    if targets.len() > CONCURRENT_BROADCAST_THRESHOLD {
        let data = data.clone();
        tokio::spawn(async move {
            for (session_id, session) in targets {
                try_send_with_drop_tracking_sync(&session, data.clone(), &session_id);
            }
        });
    } else {
        for (session_id, session) in &targets {
            try_send_with_drop_tracking_sync(session, data.clone(), session_id);
        }
    }
}
