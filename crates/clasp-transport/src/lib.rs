//! CLASP Transport Layer
//!
//! This crate provides transport implementations for CLASP.
//! The protocol is transport-agnostic - any byte transport works.
//!
//! Available transports:
//! - WebSocket (recommended baseline for interoperability)
//! - UDP (LAN, low-latency, broadcast)
//! - QUIC (modern native apps, connection migration)
//! - Serial (direct hardware, lowest latency)
//! - BLE (Bluetooth Low Energy, wireless controllers)
//! - WebRTC (P2P, NAT traversal, low-latency)

pub mod error;
pub mod traits;

#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(feature = "udp")]
pub mod udp;

#[cfg(feature = "quic")]
pub mod quic;

#[cfg(feature = "serial")]
pub mod serial;

#[cfg(feature = "ble")]
pub mod ble;

#[cfg(feature = "webrtc")]
pub mod webrtc;

pub use error::{TransportError, Result};
pub use traits::{Transport, TransportEvent, TransportSender, TransportReceiver, TransportServer};

#[cfg(feature = "websocket")]
pub use websocket::{WebSocketTransport, WebSocketConfig, WebSocketServer};

#[cfg(feature = "udp")]
pub use udp::{UdpTransport, UdpConfig};

#[cfg(feature = "ble")]
pub use ble::{BleTransport, BleConfig};

#[cfg(feature = "webrtc")]
pub use webrtc::{WebRtcTransport, WebRtcConfig};

#[cfg(feature = "quic")]
pub use quic::{QuicTransport, QuicConfig, QuicConnection};

#[cfg(feature = "serial")]
pub use serial::{SerialTransport, SerialConfig};
