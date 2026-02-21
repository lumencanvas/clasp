//! FederationSync message handler -- inter-router state synchronization.
//!
//! Handles namespace declaration, full/delta sync requests, and revision vector
//! exchange between federated CLASP routers. Only sessions with the `federation`
//! feature flag may use these operations.

use clasp_core::{
    codec, AckMessage, Action, ErrorMessage, Message, SecurityMode, SnapshotMessage,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::{send_chunked_snapshot, HandlerContext, MessageResult};

/// Resource limits for federation operations
const MAX_FEDERATION_PATTERNS: usize = 1000;
const MAX_REVISION_ENTRIES: usize = 10_000;

pub(crate) async fn handle(
    fed_msg: &clasp_core::FederationSyncMessage,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    let session = ctx.session.as_ref()?;

    if !session.is_federation_peer() {
        warn!(
            "Session {} sent FederationSync but is not a federation peer",
            session.id
        );
        let error = Message::Error(ErrorMessage {
            code: 403,
            message: "FederationSync requires federation feature".to_string(),
            address: None,
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    match fed_msg.op {
        clasp_core::FederationOp::DeclareNamespaces => {
            handle_declare_namespaces(fed_msg, session, ctx).await
        }
        clasp_core::FederationOp::RequestSync => {
            handle_request_sync(fed_msg, session, ctx).await
        }
        clasp_core::FederationOp::RevisionVector => {
            handle_revision_vector(fed_msg, session, ctx).await
        }
        clasp_core::FederationOp::SyncComplete => {
            info!(
                "Federation: sync complete from peer {}",
                session.federation_router_id().unwrap_or_default()
            );
            Some(MessageResult::None)
        }
    }
}

async fn handle_declare_namespaces(
    fed_msg: &clasp_core::FederationSyncMessage,
    session: &Arc<crate::session::Session>,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    if fed_msg.patterns.len() > MAX_FEDERATION_PATTERNS {
        warn!(
            "Federation peer {} declared {} namespaces (limit {})",
            session.id, fed_msg.patterns.len(), MAX_FEDERATION_PATTERNS
        );
        let error = Message::Error(ErrorMessage {
            code: 400,
            message: format!(
                "too many namespace patterns: {} (max {})",
                fed_msg.patterns.len(), MAX_FEDERATION_PATTERNS
            ),
            address: None,
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    let router_id = fed_msg
        .origin
        .clone()
        .unwrap_or_else(|| session.name.clone());

    if ctx.security_mode == SecurityMode::Authenticated {
        for pattern in &fed_msg.patterns {
            if !session.has_strict_read_scope(pattern) {
                warn!(
                    "Federation peer {} lacks read scope for namespace {}",
                    router_id, pattern
                );
                let error = Message::Error(ErrorMessage {
                    code: 403,
                    message: format!(
                        "insufficient scope for namespace: {}", pattern
                    ),
                    address: None,
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }
        }
    }

    info!(
        "Federation peer {} declares namespaces: {:?}",
        router_id, fed_msg.patterns
    );

    // Clean up previous federation subscriptions if re-declaring
    let old_namespaces = session.federation_namespaces();
    if !old_namespaces.is_empty() {
        for i in 0..old_namespaces.len() {
            let old_sub_id = 50000 + i as u32;
            ctx.subscriptions.remove(&session.id, old_sub_id);
            session.remove_subscription(old_sub_id);
        }
        debug!(
            "Federation: cleaned up {} old subscriptions for peer {}",
            old_namespaces.len(), router_id
        );
    }

    session.set_federation_router_id(router_id.clone());
    session.set_federation_namespaces(fed_msg.patterns.clone());

    for (i, pattern) in fed_msg.patterns.iter().enumerate() {
        let sub_id = 50000 + i as u32;
        match crate::subscription::Subscription::new(
            sub_id,
            session.id.clone(),
            pattern,
            vec![],
            Default::default(),
        ) {
            Ok(subscription) => {
                ctx.subscriptions.add(subscription);
                session.add_subscription(sub_id);
                debug!(
                    "Federation: auto-subscribed peer {} to {}",
                    router_id, pattern
                );
            }
            Err(e) => {
                warn!(
                    "Federation: failed to create subscription for {}: {:?}",
                    pattern, e
                );
            }
        }
    }

    let ack = Message::Ack(AckMessage {
        address: None,
        revision: None,
        locked: None,
        holder: None,
        correlation_id: None,
    });
    let bytes = codec::encode(&ack).ok()?;
    Some(MessageResult::Send(bytes))
}

async fn handle_request_sync(
    fed_msg: &clasp_core::FederationSyncMessage,
    session: &Arc<crate::session::Session>,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    if fed_msg.patterns.len() > MAX_FEDERATION_PATTERNS {
        warn!(
            "Federation RequestSync with {} patterns (limit {})",
            fed_msg.patterns.len(), MAX_FEDERATION_PATTERNS
        );
        let error = Message::Error(ErrorMessage {
            code: 400,
            message: format!(
                "too many sync patterns: {} (max {})",
                fed_msg.patterns.len(), MAX_FEDERATION_PATTERNS
            ),
            address: None,
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    let declared = session.federation_namespaces();
    for pattern in &fed_msg.patterns {
        let covered = declared.iter().any(|ns| {
            crate::router::federation_pattern_covered_by(pattern, ns)
        });
        if !covered {
            warn!(
                "Federation RequestSync pattern {} not covered by declared namespaces {:?}",
                pattern, declared
            );
            let error = Message::Error(ErrorMessage {
                code: 403,
                message: format!(
                    "pattern '{}' not covered by declared namespaces", pattern
                ),
                address: None,
                correlation_id: None,
            });
            let bytes = codec::encode(&error).ok()?;
            return Some(MessageResult::Send(bytes));
        }

        if ctx.security_mode == SecurityMode::Authenticated
            && !session.has_strict_read_scope(pattern)
        {
            warn!(
                "Federation RequestSync: session lacks read scope for {}",
                pattern
            );
            let error = Message::Error(ErrorMessage {
                code: 403,
                message: format!(
                    "insufficient scope for pattern: {}", pattern
                ),
                address: None,
                correlation_id: None,
            });
            let bytes = codec::encode(&error).ok()?;
            return Some(MessageResult::Send(bytes));
        }
    }

    for pattern in &fed_msg.patterns {
        let mut snapshot = ctx.state.snapshot(pattern);

        if let Some(since) = fed_msg.since_revision {
            snapshot.params.retain(|p| p.revision > since);
        }

        if let Some(ref filter) = ctx.snapshot_filter {
            snapshot.params =
                filter.filter_snapshot(snapshot.params, session, ctx.state);
        }

        send_chunked_snapshot(ctx.sender, snapshot).await;
    }

    let complete = Message::FederationSync(clasp_core::FederationSyncMessage {
        op: clasp_core::FederationOp::SyncComplete,
        patterns: fed_msg.patterns.clone(),
        revisions: std::collections::HashMap::new(),
        since_revision: None,
        origin: Some(ctx.config.name.clone()),
    });
    let bytes = codec::encode(&complete).ok()?;
    Some(MessageResult::Send(bytes))
}

async fn handle_revision_vector(
    fed_msg: &clasp_core::FederationSyncMessage,
    session: &Arc<crate::session::Session>,
    ctx: &HandlerContext<'_>,
) -> Option<MessageResult> {
    if fed_msg.revisions.len() > MAX_REVISION_ENTRIES {
        warn!(
            "Federation RevisionVector with {} entries (limit {})",
            fed_msg.revisions.len(), MAX_REVISION_ENTRIES
        );
        let error = Message::Error(ErrorMessage {
            code: 400,
            message: format!(
                "too many revision entries: {} (max {})",
                fed_msg.revisions.len(), MAX_REVISION_ENTRIES
            ),
            address: None,
            correlation_id: None,
        });
        let bytes = codec::encode(&error).ok()?;
        return Some(MessageResult::Send(bytes));
    }

    debug!(
        "Federation: received revision vector with {} entries from peer {}",
        fed_msg.revisions.len(),
        session.federation_router_id().unwrap_or_default()
    );

    let declared = session.federation_namespaces();

    let mut delta_params = Vec::new();
    for (addr, peer_rev) in &fed_msg.revisions {
        let covered = declared.iter().any(|ns| {
            clasp_core::address::glob_match(ns, addr)
        });
        if !covered {
            debug!(
                "Federation: skipping revision for {} (not in declared namespaces)",
                addr
            );
            continue;
        }

        if ctx.security_mode == SecurityMode::Authenticated
            && !session.has_scope(Action::Read, addr)
        {
            debug!(
                "Federation: skipping revision for {} (insufficient scope)",
                addr
            );
            continue;
        }

        if let Some(local_ps) = ctx.state.get_state(addr) {
            if local_ps.revision > *peer_rev {
                delta_params.push(clasp_core::ParamValue {
                    address: addr.clone(),
                    value: local_ps.value,
                    revision: local_ps.revision,
                    writer: Some(local_ps.writer),
                    timestamp: Some(local_ps.timestamp),
                });
            }
        }
    }

    if !delta_params.is_empty() {
        let snapshot = SnapshotMessage {
            params: delta_params,
        };
        send_chunked_snapshot(ctx.sender, snapshot).await;
    }

    Some(MessageResult::None)
}
