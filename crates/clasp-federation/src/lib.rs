//! Router federation for CLASP
//!
//! This crate enables CLASP routers to connect to each other and share
//! state across network boundaries. Federation links appear as normal
//! client sessions, using the standard CLASP protocol for communication.
//!
//! # Architecture
//!
//! Federation works by:
//! 1. Each router declares its owned **namespace patterns** (e.g., `/site-a/**`)
//! 2. Routers connect to each other and exchange namespace declarations
//! 3. Messages matching a peer's namespace are forwarded via the federation link
//! 4. Loop prevention uses an `origin` field -- messages are never forwarded
//!    back to the router they came from
//!
//! # Modes
//!
//! - **Hub**: Central router that accepts leaf connections (star topology)
//! - **Leaf**: Edge router that connects to a single hub
//! - **Mesh**: Peer-to-peer connections between multiple routers
//!
//! # Example
//!
//! ```no_run
//! use clasp_federation::{FederationManager, FederationConfig, FederationMode};
//!
//! # async fn example() {
//! let config = FederationConfig {
//!     mode: FederationMode::Leaf {
//!         hub_endpoint: "wss://hub.example.com:7330".to_string(),
//!     },
//!     router_id: "site-a".to_string(),
//!     owned_namespaces: vec!["/site-a/**".to_string()],
//!     ..Default::default()
//! };
//!
//! let manager = FederationManager::new(config);
//! // Create links for each peer connection...
//! # }
//! ```

pub mod config;
pub mod error;
pub mod link;
pub mod manager;
pub mod namespace;

pub use config::{FederationConfig, FederationMode, PeerInfo, PeerState};
pub use error::{FederationError, Result};
pub use link::{FederationLink, LinkEvent};
pub use manager::FederationManager;
pub use namespace::NamespaceManager;
