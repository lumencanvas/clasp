//! Rules engine for CLASP
//!
//! Evaluates conditional rules triggered by state changes and events,
//! producing actions that the router executes. Rules are managed as
//! CLASP state at `/clasp/rules/{rule_id}`.
//!
//! # Features
//!
//! - **Trigger types**: OnChange (pattern), OnThreshold, OnEvent, OnInterval
//! - **Conditions**: Compare current state values before firing
//! - **Actions**: Set params, publish events, transform trigger values
//! - **Loop prevention**: Actions from rules are marked with `origin: "rule:{id}"`
//!   and skip rule evaluation
//! - **Cooldown**: Minimum time between rule firings
//!
//! # Example
//!
//! ```
//! use clasp_rules::{RulesEngine, Rule, Trigger, RuleAction, Transform};
//! use clasp_core::{SignalType, Value};
//!
//! let mut engine = RulesEngine::new();
//!
//! engine.add_rule(Rule {
//!     id: "motion-lights".to_string(),
//!     name: "Turn on lights when motion detected".to_string(),
//!     enabled: true,
//!     trigger: Trigger::OnChange {
//!         pattern: "/sensor/motion/**".to_string(),
//!     },
//!     conditions: vec![],
//!     actions: vec![RuleAction::Set {
//!         address: "/lights/room1/brightness".to_string(),
//!         value: Value::Float(1.0),
//!     }],
//!     cooldown: None,
//! }).unwrap();
//!
//! let actions = engine.evaluate(
//!     "/sensor/motion/room1",
//!     &Value::Bool(true),
//!     SignalType::Param,
//!     None,
//!     |_addr| None, // state lookup
//! );
//!
//! assert_eq!(actions.len(), 1);
//! ```

pub mod engine;
pub mod error;
pub mod rule;

pub use engine::{PendingAction, RulesEngine};
pub use error::{Result, RulesError};
pub use rule::{CompareOp, Condition, Rule, RuleAction, Transform, Trigger};
