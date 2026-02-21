//! Federation leaf mode for CLASP relay.
//!
//! Connects to a hub router via WebSocket and bridges state between local
//! and remote routers using namespace-scoped forwarding.

use clasp_core::{codec, Message, SetMessage, SignalType};
use clasp_federation::{FederationConfig, FederationLink, FederationManager, LinkEvent};
use clasp_router::{session::{Session, SessionId}, RouterState, SubscriptionManager};
use clasp_transport::{Transport, WebSocketTransport};
use dashmap::DashMap;
use std::sync::Arc;

/// Run the federation leaf, connecting to a hub and bridging state.
///
/// This function connects to the hub, runs the federation protocol, and
/// processes events in a loop. On disconnect, it reconnects if configured.
pub async fn run_federation_leaf(
    config: FederationConfig,
    state: Arc<RouterState>,
    sessions: Arc<DashMap<SessionId, Arc<Session>>>,
    subscriptions: Arc<SubscriptionManager>,
) {
    let mut manager = FederationManager::new(config.clone());
    let event_tx = manager.event_sender();
    let mut event_rx = manager
        .take_event_receiver()
        .expect("event receiver already taken");

    let hub_endpoint = match &config.mode {
        clasp_federation::FederationMode::Leaf { hub_endpoint } => hub_endpoint.clone(),
        _ => {
            tracing::error!("Federation: expected Leaf mode");
            return;
        }
    };

    // Spawn connection task
    let conn_config = config.clone();
    let conn_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut attempt = 0u32;
        loop {
            tracing::info!("Federation: connecting to hub {}", hub_endpoint);
            match WebSocketTransport::connect(&hub_endpoint).await {
                Ok((sender, receiver)) => {
                    attempt = 0;
                    let link = FederationLink::new(
                        conn_config.clone(),
                        Arc::new(sender),
                        conn_tx.clone(),
                    );
                    if let Err(e) = link.run(Box::new(receiver)).await {
                        tracing::warn!("Federation link ended: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Federation: connection failed: {}", e);
                }
            }

            if !conn_config.auto_reconnect {
                tracing::info!("Federation: auto_reconnect disabled, stopping");
                break;
            }

            attempt += 1;
            if conn_config.max_reconnect_attempts > 0
                && attempt >= conn_config.max_reconnect_attempts
            {
                tracing::error!(
                    "Federation: max reconnect attempts ({}) reached",
                    conn_config.max_reconnect_attempts
                );
                break;
            }

            tracing::info!(
                "Federation: reconnecting in {:?} (attempt {})",
                conn_config.reconnect_delay,
                attempt
            );
            tokio::time::sleep(conn_config.reconnect_delay).await;
        }
    });

    // Process events from the federation link
    while let Some(event) = event_rx.recv().await {
        manager.process_event(&event).await;

        match event {
            LinkEvent::RemoteSet {
                address,
                value,
                revision,
                origin,
            } => {
                match state.set(&address, value.clone(), &origin, revision, false, false) {
                    Ok(rev) => {
                        let subscribers =
                            subscriptions.find_subscribers(&address, Some(SignalType::Param));
                        let set_msg = Message::Set(SetMessage {
                            address: address.clone(),
                            value,
                            revision: Some(rev),
                            lock: false,
                            unlock: false,
                        });
                        if let Ok(bytes) = codec::encode(&set_msg) {
                            for sub_session_id in &subscribers {
                                if let Some(sub_session) = sessions.get(sub_session_id) {
                                    let _ = sub_session.value().try_send(bytes.clone());
                                }
                            }
                        }
                        tracing::debug!(
                            "Federation: applied remote SET {} rev={} -> {} subscriber(s)",
                            address,
                            rev,
                            subscribers.len()
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Federation: failed to apply remote SET {}: {:?}",
                            address,
                            e
                        );
                    }
                }
            }
            LinkEvent::RemotePublish { message, origin } => {
                // Extract address from the publish message for subscriber lookup
                if let Message::Publish(ref pub_msg) = message {
                    let signal_type = pub_msg.signal.unwrap_or(SignalType::Event);
                    let subscribers =
                        subscriptions.find_subscribers(&pub_msg.address, Some(signal_type));
                    if let Ok(bytes) = codec::encode(&message) {
                        for sub_session_id in &subscribers {
                            if let Some(sub_session) = sessions.get(sub_session_id) {
                                let _ = sub_session.value().try_send(bytes.clone());
                            }
                        }
                    }
                    tracing::debug!(
                        "Federation: forwarded remote PUBLISH from {} to {} subscriber(s)",
                        origin,
                        subscribers.len()
                    );
                } else {
                    tracing::debug!("Federation: received non-publish remote message from {}", origin);
                }
            }
            LinkEvent::Connected { router_id } => {
                tracing::info!("Federation: connected to hub {}", router_id);
            }
            LinkEvent::Disconnected { router_id, reason } => {
                tracing::info!(
                    "Federation: disconnected from {} ({:?})",
                    router_id,
                    reason
                );
            }
            LinkEvent::PeerNamespaces {
                router_id,
                patterns,
            } => {
                tracing::info!(
                    "Federation: hub {} declared namespaces: {:?}",
                    router_id,
                    patterns
                );
            }
            LinkEvent::SyncComplete {
                router_id,
                pattern,
                revision,
            } => {
                tracing::info!(
                    "Federation: sync complete with {} for {} at rev {}",
                    router_id,
                    pattern,
                    revision
                );
            }
        }
    }
}
