//! Tunnel DefraDB P2P sync over CLASP transports (WebSocket, WebRTC, QUIC).
//!
//! DefraDB uses libp2p for peer-to-peer networking, but libp2p has limited
//! browser support. CLASP has production WebSocket, WebRTC, and WASM clients.
//! This crate bridges the gap by wrapping DefraDB sync messages (DAG blocks,
//! head updates, collection sync requests) as CLASP binary payloads that flow
//! over any CLASP transport.
//!
//! # Architecture
//!
//! ```text
//! DefraDB (HTTP/GraphQL) <-> DefraTunnel <-> CLASP Transport <-> Remote Peer
//! ```
//!
//! The [`DefraTunnel`] polls the local DefraDB instance for new commits and
//! encodes them as [`TunnelMessage`] variants, which the caller sends via
//! CLASP's [`TransportSender`](clasp_core). On the receiving side, incoming
//! CLASP payloads are decoded and applied to the local DefraDB.
//!
//! # Address Namespace
//!
//! All tunnel traffic uses the `/defra/sync/` CLASP address namespace.
//! See the [`address`] module for helpers.

pub mod address;
pub mod error;
pub mod protocol;
pub mod sync;
pub mod tunnel;

pub use address::{
    block_channel, collection_channel, parse_sync_address, peer_channel, SyncAddress, DEFRA_SYNC_NS,
};
pub use error::{Result, TunnelError};
pub use protocol::TunnelMessage;
pub use sync::{apply_received_blocks, compute_sync_diff, BlockInfo};
pub use tunnel::DefraTunnel;
