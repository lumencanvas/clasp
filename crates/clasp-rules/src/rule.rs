//! Rule definitions and types

use clasp_core::{SignalType, Value};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A rule definition that triggers actions based on signal changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique rule identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Whether the rule is active
    pub enabled: bool,
    /// What triggers the rule
    pub trigger: Trigger,
    /// Additional conditions that must be true for the rule to fire
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
    /// Actions to execute when the rule fires
    pub actions: Vec<RuleAction>,
    /// Minimum time between firings (None = no cooldown)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown: Option<Duration>,
}

/// What causes a rule to evaluate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
    /// Fires when a parameter matching the pattern changes
    OnChange { pattern: String },
    /// Fires when a parameter crosses a threshold
    OnThreshold {
        address: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        above: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        below: Option<f64>,
    },
    /// Fires when an event matching the pattern is published
    OnEvent { pattern: String },
    /// Fires periodically
    OnInterval { seconds: u64 },
}

/// A condition that must be true for a rule to fire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Address to check
    pub address: String,
    /// Comparison operator
    pub op: CompareOp,
    /// Value to compare against
    pub value: Value,
}

/// Comparison operators for conditions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompareOp {
    /// Equal
    Eq,
    /// Not equal
    Ne,
    /// Greater than
    Gt,
    /// Greater than or equal
    Gte,
    /// Less than
    Lt,
    /// Less than or equal
    Lte,
}

impl CompareOp {
    /// Evaluate the comparison
    pub fn evaluate(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Float(a), Value::Float(b)) => match self {
                CompareOp::Eq => (a - b).abs() < f64::EPSILON,
                CompareOp::Ne => (a - b).abs() >= f64::EPSILON,
                CompareOp::Gt => a > b,
                CompareOp::Gte => a >= b,
                CompareOp::Lt => a < b,
                CompareOp::Lte => a <= b,
            },
            (Value::Int(a), Value::Int(b)) => match self {
                CompareOp::Eq => a == b,
                CompareOp::Ne => a != b,
                CompareOp::Gt => a > b,
                CompareOp::Gte => a >= b,
                CompareOp::Lt => a < b,
                CompareOp::Lte => a <= b,
            },
            (Value::Bool(a), Value::Bool(b)) => match self {
                CompareOp::Eq => a == b,
                CompareOp::Ne => a != b,
                _ => false,
            },
            (Value::String(a), Value::String(b)) => match self {
                CompareOp::Eq => a == b,
                CompareOp::Ne => a != b,
                _ => false,
            },
            _ => false,
        }
    }
}

/// An action to execute when a rule fires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Set a parameter to a specific value
    Set { address: String, value: Value },
    /// Publish an event
    Publish {
        address: String,
        signal: SignalType,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<Value>,
    },
    /// Copy the trigger's value to another address, with optional transform
    SetFromTrigger {
        address: String,
        #[serde(default)]
        transform: Transform,
    },
    /// Delay before the next action in the sequence
    Delay { milliseconds: u64 },
}

/// Value transformations for SetFromTrigger
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Transform {
    /// Pass through unchanged
    #[default]
    Identity,
    /// Linear scale: output = input * scale + offset
    Scale { scale: f64, offset: f64 },
    /// Clamp to range
    Clamp { min: f64, max: f64 },
    /// Convert to boolean: true if input > threshold
    Threshold { value: f64 },
    /// Invert within range: output = max - (input - min)
    Invert { min: f64, max: f64 },
}

impl Transform {
    /// Apply the transform to a value
    pub fn apply(&self, value: &Value) -> Value {
        match self {
            Transform::Identity => value.clone(),
            Transform::Scale { scale, offset } => match value {
                Value::Float(f) => Value::Float(f * scale + offset),
                Value::Int(i) => Value::Float(*i as f64 * scale + offset),
                _ => value.clone(),
            },
            Transform::Clamp { min, max } => match value {
                Value::Float(f) => Value::Float(f.clamp(*min, *max)),
                Value::Int(i) => Value::Float((*i as f64).clamp(*min, *max)),
                _ => value.clone(),
            },
            Transform::Threshold { value: threshold } => match value {
                Value::Float(f) => Value::Bool(*f > *threshold),
                Value::Int(i) => Value::Bool(*i as f64 > *threshold),
                _ => value.clone(),
            },
            Transform::Invert { min, max } => match value {
                Value::Float(f) => Value::Float(max - (f - min)),
                Value::Int(i) => Value::Float(max - (*i as f64 - min)),
                _ => value.clone(),
            },
        }
    }
}

impl Trigger {
    /// Get the pattern this trigger listens to
    pub fn pattern(&self) -> Option<&str> {
        match self {
            Trigger::OnChange { pattern } => Some(pattern),
            Trigger::OnThreshold { address, .. } => Some(address),
            Trigger::OnEvent { pattern } => Some(pattern),
            Trigger::OnInterval { .. } => None,
        }
    }

    /// Check if this trigger matches a given address and signal type
    pub fn matches(&self, address: &str, signal_type: SignalType) -> bool {
        match self {
            Trigger::OnChange { pattern } => {
                signal_type == SignalType::Param
                    && clasp_core::address::glob_match(pattern, address)
            }
            Trigger::OnThreshold { address: addr, .. } => {
                signal_type == SignalType::Param && addr == address
            }
            Trigger::OnEvent { pattern } => {
                signal_type == SignalType::Event
                    && clasp_core::address::glob_match(pattern, address)
            }
            Trigger::OnInterval { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_identity() {
        let t = Transform::Identity;
        assert_eq!(t.apply(&Value::Float(0.5)), Value::Float(0.5));
    }

    #[test]
    fn test_transform_scale() {
        let t = Transform::Scale {
            scale: 2.0,
            offset: 1.0,
        };
        assert_eq!(t.apply(&Value::Float(0.5)), Value::Float(2.0));
    }

    #[test]
    fn test_transform_clamp() {
        let t = Transform::Clamp { min: 0.0, max: 1.0 };
        assert_eq!(t.apply(&Value::Float(1.5)), Value::Float(1.0));
        assert_eq!(t.apply(&Value::Float(-0.5)), Value::Float(0.0));
        assert_eq!(t.apply(&Value::Float(0.5)), Value::Float(0.5));
    }

    #[test]
    fn test_transform_threshold() {
        let t = Transform::Threshold { value: 0.5 };
        assert_eq!(t.apply(&Value::Float(0.8)), Value::Bool(true));
        assert_eq!(t.apply(&Value::Float(0.3)), Value::Bool(false));
    }

    #[test]
    fn test_transform_invert() {
        let t = Transform::Invert { min: 0.0, max: 1.0 };
        assert_eq!(t.apply(&Value::Float(0.25)), Value::Float(0.75));
        assert_eq!(t.apply(&Value::Float(0.0)), Value::Float(1.0));
    }

    #[test]
    fn test_compare_op() {
        assert!(CompareOp::Gt.evaluate(&Value::Float(1.0), &Value::Float(0.5)));
        assert!(!CompareOp::Gt.evaluate(&Value::Float(0.5), &Value::Float(1.0)));
        assert!(CompareOp::Eq.evaluate(&Value::Int(42), &Value::Int(42)));
        assert!(CompareOp::Ne.evaluate(
            &Value::String("a".to_string()),
            &Value::String("b".to_string())
        ));
    }

    #[test]
    fn test_trigger_matches() {
        let trigger = Trigger::OnChange {
            pattern: "/lights/**".to_string(),
        };
        assert!(trigger.matches("/lights/room1/brightness", SignalType::Param));
        assert!(!trigger.matches("/audio/volume", SignalType::Param));
        assert!(!trigger.matches("/lights/room1/brightness", SignalType::Event));
    }

    #[test]
    fn test_threshold_trigger_matches() {
        let trigger = Trigger::OnThreshold {
            address: "/sensor/temp".to_string(),
            above: Some(30.0),
            below: None,
        };
        assert!(trigger.matches("/sensor/temp", SignalType::Param));
        assert!(!trigger.matches("/sensor/humidity", SignalType::Param));
    }
}
