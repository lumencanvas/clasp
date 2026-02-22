//! CLASP Relay — embeddable multi-protocol relay server.
//!
//! # Library usage
//!
//! ```rust,no_run
//! use clasp_relay::config::RelayConfig;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = RelayConfig {
//!         ws_port: 7330,
//!         auth_port: Some(7350),
//!         ..Default::default()
//!     };
//!     clasp_relay::server::run(config).await
//! }
//! ```
//!
//! # With app config (programmatic)
//!
//! ```rust,no_run
//! use clasp_relay::config::RelayConfig;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let json = std::fs::read_to_string("config/chat.json").unwrap();
//!     let app_config: clasp_relay::app_config::AppConfig = serde_json::from_str(&json).unwrap();
//!     let config = RelayConfig {
//!         ws_port: 7330,
//!         auth_port: Some(7350),
//!         app_config: Some(app_config),
//!         ..Default::default()
//!     };
//!     clasp_relay::server::run(config).await
//! }
//! ```

pub mod app_config;
pub mod auth;
pub mod config;
pub mod cpsk;
#[cfg(feature = "federation")]
pub mod federation;
pub mod health;
#[cfg(feature = "registry")]
pub mod registry;
pub mod server;
