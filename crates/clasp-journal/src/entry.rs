//! Journal entry types

use clasp_core::{SignalType, Value};
use serde::{Deserialize, Serialize};

/// A single journal entry representing a state change or event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Monotonic sequence number (assigned by the journal)
    pub seq: u64,
    /// Wall clock timestamp (microseconds since epoch)
    pub timestamp: u64,
    /// Author of this entry (entity or session ID)
    pub author: String,
    /// CLASP address this entry applies to
    pub address: String,
    /// Signal type of the entry
    pub signal_type: SignalType,
    /// The value that was set or published
    pub value: Value,
    /// Param revision (for SET operations)
    pub revision: Option<u64>,
    /// Original message type code (0x20=PUBLISH, 0x21=SET, etc.)
    pub msg_type: u8,
}

impl JournalEntry {
    /// Create a new journal entry for a SET operation
    pub fn from_set(
        address: String,
        value: Value,
        revision: u64,
        author: String,
        timestamp: u64,
    ) -> Self {
        Self {
            seq: 0, // Assigned by journal
            timestamp,
            author,
            address,
            signal_type: SignalType::Param,
            value,
            revision: Some(revision),
            msg_type: 0x21, // SET
        }
    }

    /// Create a new journal entry for a PUBLISH operation
    pub fn from_publish(
        address: String,
        signal_type: SignalType,
        value: Value,
        author: String,
        timestamp: u64,
    ) -> Self {
        Self {
            seq: 0, // Assigned by journal
            timestamp,
            author,
            address,
            signal_type,
            value,
            revision: None,
            msg_type: 0x20, // PUBLISH
        }
    }
}

/// Serializable snapshot of param state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSnapshot {
    pub address: String,
    pub value: Value,
    pub revision: u64,
    pub writer: String,
    pub timestamp: u64,
}
