//! Rules engine error types

use thiserror::Error;

/// Rules engine errors
#[derive(Debug, Error)]
pub enum RulesError {
    /// Rule definition is invalid
    #[error("invalid rule: {0}")]
    InvalidRule(String),

    /// Rule evaluation failed
    #[error("evaluation error: {0}")]
    EvaluationError(String),

    /// Rule not found
    #[error("rule not found: {0}")]
    NotFound(String),

    /// Loop detected (rule triggered itself)
    #[error("loop detected: rule {0} triggered itself")]
    LoopDetected(String),

    /// Cooldown active
    #[error("rule {0} is in cooldown")]
    Cooldown(String),
}

/// Result type for rules operations
pub type Result<T> = std::result::Result<T, RulesError>;
