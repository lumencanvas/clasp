//! Abstraction traits for the CLASP side of the bridge.
//!
//! These traits decouple the bridge from a concrete CLASP client so the
//! crate can be tested without a running router.

use async_trait::async_trait;
use clasp_core::Value;

/// Trait for sending signals to a CLASP router.
#[async_trait]
pub trait SignalSender: Send + Sync {
    /// Send a SET signal (persistent parameter update).
    async fn set(
        &self,
        address: &str,
        value: Value,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Send an EMIT signal (one-shot event).
    async fn emit(
        &self,
        address: &str,
        value: Value,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Trait for receiving signals from a CLASP router.
#[async_trait]
pub trait SignalReceiver: Send + Sync {
    /// Subscribe to a CLASP address pattern.
    ///
    /// Returns a channel receiver that yields `(address, value)` pairs for
    /// every signal matching the pattern.
    async fn subscribe(
        &self,
        pattern: &str,
    ) -> std::result::Result<
        tokio::sync::mpsc::Receiver<(String, Value)>,
        Box<dyn std::error::Error + Send + Sync>,
    >;
}
