//! Rules evaluation engine
//!
//! The engine evaluates rules against incoming state changes and events,
//! producing actions that the router should execute.

use clasp_core::{SignalType, Value};
use std::collections::HashMap;
use std::time::Instant;

use crate::error::{Result, RulesError};
use crate::rule::{Rule, RuleAction, Trigger};

/// Output from rule evaluation -- an action the router should execute
#[derive(Debug, Clone)]
pub struct PendingAction {
    /// The rule that produced this action
    pub rule_id: String,
    /// The action to execute
    pub action: RuleAction,
    /// Origin tag for loop prevention
    pub origin: String,
}

/// The rules engine evaluates rules against state changes.
pub struct RulesEngine {
    /// Active rules
    rules: HashMap<String, Rule>,
    /// Last fire time for cooldown tracking
    last_fired: HashMap<String, Instant>,
    /// Currently evaluating rules (for loop detection)
    evaluating: Vec<String>,
}

impl RulesEngine {
    /// Create a new empty rules engine
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            last_fired: HashMap::new(),
            evaluating: Vec::new(),
        }
    }

    /// Add or update a rule
    pub fn add_rule(&mut self, rule: Rule) -> Result<()> {
        if rule.id.is_empty() {
            return Err(RulesError::InvalidRule("rule ID cannot be empty".into()));
        }
        if rule.actions.is_empty() {
            return Err(RulesError::InvalidRule(
                "rule must have at least one action".into(),
            ));
        }
        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, id: &str) -> Result<()> {
        self.rules
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| RulesError::NotFound(id.to_string()))
    }

    /// Get a rule by ID
    pub fn get_rule(&self, id: &str) -> Option<&Rule> {
        self.rules.get(id)
    }

    /// Get all rules
    pub fn rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.values()
    }

    /// Number of rules
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Enable or disable a rule
    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<()> {
        self.rules
            .get_mut(id)
            .map(|r| r.enabled = enabled)
            .ok_or_else(|| RulesError::NotFound(id.to_string()))
    }

    /// Evaluate rules triggered by a state change.
    ///
    /// `address`: the address that changed
    /// `value`: the new value
    /// `signal_type`: Param or Event
    /// `origin`: if set, skip rules that originated from this source (loop prevention)
    /// `state_lookup`: function to look up current param values (for conditions)
    ///
    /// Returns actions that should be executed by the router.
    pub fn evaluate<F>(
        &mut self,
        address: &str,
        value: &Value,
        signal_type: SignalType,
        origin: Option<&str>,
        state_lookup: F,
    ) -> Vec<PendingAction>
    where
        F: Fn(&str) -> Option<Value>,
    {
        // Skip if this change came from a rule (loop prevention)
        if let Some(origin) = origin {
            if origin.starts_with("rule:") {
                return vec![];
            }
        }

        let mut actions = Vec::new();
        let now = Instant::now();

        // Collect matching rule IDs first to avoid borrow issues
        let matching_ids: Vec<String> = self
            .rules
            .values()
            .filter(|rule| rule.enabled && rule.trigger.matches(address, signal_type))
            .map(|rule| rule.id.clone())
            .collect();

        for rule_id in matching_ids {
            // Loop detection
            if self.evaluating.contains(&rule_id) {
                continue;
            }

            let rule = match self.rules.get(&rule_id) {
                Some(r) => r,
                None => continue,
            };

            // Cooldown check
            if let Some(cooldown) = rule.cooldown {
                if let Some(last) = self.last_fired.get(&rule_id) {
                    if now.duration_since(*last) < cooldown {
                        continue;
                    }
                }
            }

            // Threshold check for OnThreshold triggers
            if let Trigger::OnThreshold { above, below, .. } = &rule.trigger {
                let f = match value {
                    Value::Float(f) => *f,
                    Value::Int(i) => *i as f64,
                    _ => continue,
                };

                let threshold_met = match (above, below) {
                    (Some(a), Some(b)) => f > *a || f < *b,
                    (Some(a), None) => f > *a,
                    (None, Some(b)) => f < *b,
                    (None, None) => true,
                };

                if !threshold_met {
                    continue;
                }
            }

            // Evaluate conditions
            let conditions_met = rule.conditions.iter().all(|cond| {
                if let Some(current) = state_lookup(&cond.address) {
                    cond.op.evaluate(&current, &cond.value)
                } else {
                    false
                }
            });

            if !conditions_met {
                continue;
            }

            // Mark as evaluating for loop detection
            self.evaluating.push(rule_id.clone());

            // Collect actions
            let rule_origin = format!("rule:{}", rule_id);
            for action in &rule.actions {
                let resolved_action = match action {
                    RuleAction::SetFromTrigger {
                        address: target,
                        transform,
                    } => RuleAction::Set {
                        address: target.clone(),
                        value: transform.apply(value),
                    },
                    other => other.clone(),
                };

                actions.push(PendingAction {
                    rule_id: rule_id.clone(),
                    action: resolved_action,
                    origin: rule_origin.clone(),
                });
            }

            // Update last fired time
            self.last_fired.insert(rule_id.clone(), now);

            // Clear evaluating
            self.evaluating.retain(|id| id != &rule_id);
        }

        actions
    }

    /// Evaluate an interval rule by ID.
    ///
    /// Unlike `evaluate()`, this doesn't match on address/signal_type — it fires
    /// the rule directly (checking enabled, cooldown, and conditions).
    /// Returns actions that should be executed by the router.
    pub fn evaluate_interval<F>(&mut self, rule_id: &str, state_lookup: F) -> Vec<PendingAction>
    where
        F: Fn(&str) -> Option<Value>,
    {
        let rule = match self.rules.get(rule_id) {
            Some(r) if r.enabled => r,
            _ => return vec![],
        };

        let now = Instant::now();

        // Cooldown check
        if let Some(cooldown) = rule.cooldown {
            if let Some(last) = self.last_fired.get(rule_id) {
                if now.duration_since(*last) < cooldown {
                    return vec![];
                }
            }
        }

        // Evaluate conditions
        let conditions_met = rule.conditions.iter().all(|cond| {
            if let Some(current) = state_lookup(&cond.address) {
                cond.op.evaluate(&current, &cond.value)
            } else {
                false
            }
        });

        if !conditions_met {
            return vec![];
        }

        let rule_origin = format!("interval:{}", rule_id);
        let actions: Vec<PendingAction> = rule
            .actions
            .iter()
            .map(|action| {
                let resolved_action = match action {
                    RuleAction::SetFromTrigger {
                        address: target,
                        transform,
                    } => {
                        // For interval triggers there's no trigger value, use Null
                        RuleAction::Set {
                            address: target.clone(),
                            value: transform.apply(&Value::Null),
                        }
                    }
                    other => other.clone(),
                };
                PendingAction {
                    rule_id: rule_id.to_string(),
                    action: resolved_action,
                    origin: rule_origin.clone(),
                }
            })
            .collect();

        self.last_fired.insert(rule_id.to_string(), now);
        actions
    }

    /// Get rule IDs that have interval triggers (for the router to schedule)
    pub fn interval_rules(&self) -> Vec<(String, u64)> {
        self.rules
            .values()
            .filter(|r| r.enabled)
            .filter_map(|r| {
                if let Trigger::OnInterval { seconds } = &r.trigger {
                    Some((r.id.clone(), *seconds))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::*;

    fn make_rule(id: &str, pattern: &str, target: &str, value: Value) -> Rule {
        Rule {
            id: id.to_string(),
            name: format!("Test rule {}", id),
            enabled: true,
            trigger: Trigger::OnChange {
                pattern: pattern.to_string(),
            },
            conditions: vec![],
            actions: vec![RuleAction::Set {
                address: target.to_string(),
                value,
            }],
            cooldown: None,
        }
    }

    #[test]
    fn test_basic_rule_evaluation() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(make_rule(
                "r1",
                "/sensor/motion",
                "/lights/room1",
                Value::Float(1.0),
            ))
            .unwrap();

        let actions = engine.evaluate(
            "/sensor/motion",
            &Value::Bool(true),
            SignalType::Param,
            None,
            |_| None,
        );

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].rule_id, "r1");
        assert!(matches!(
            &actions[0].action,
            RuleAction::Set { address, value } if address == "/lights/room1" && *value == Value::Float(1.0)
        ));
    }

    #[test]
    fn test_pattern_matching() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(make_rule("r1", "/sensor/**", "/output", Value::Bool(true)))
            .unwrap();

        // Should match
        let actions = engine.evaluate(
            "/sensor/motion/room1",
            &Value::Bool(true),
            SignalType::Param,
            None,
            |_| None,
        );
        assert_eq!(actions.len(), 1);

        // Should not match
        let actions = engine.evaluate(
            "/lights/room1",
            &Value::Bool(true),
            SignalType::Param,
            None,
            |_| None,
        );
        assert!(actions.is_empty());
    }

    #[test]
    fn test_disabled_rule() {
        let mut engine = RulesEngine::new();
        let mut rule = make_rule("r1", "/sensor/**", "/output", Value::Bool(true));
        rule.enabled = false;
        engine.add_rule(rule).unwrap();

        let actions = engine.evaluate(
            "/sensor/motion",
            &Value::Bool(true),
            SignalType::Param,
            None,
            |_| None,
        );
        assert!(actions.is_empty());
    }

    #[test]
    fn test_condition_check() {
        let mut engine = RulesEngine::new();
        let mut rule = make_rule("r1", "/sensor/motion", "/lights/room1", Value::Float(1.0));
        rule.conditions = vec![Condition {
            address: "/mode".to_string(),
            op: CompareOp::Eq,
            value: Value::String("auto".to_string()),
        }];
        engine.add_rule(rule).unwrap();

        // Condition met
        let actions = engine.evaluate(
            "/sensor/motion",
            &Value::Bool(true),
            SignalType::Param,
            None,
            |addr| {
                if addr == "/mode" {
                    Some(Value::String("auto".to_string()))
                } else {
                    None
                }
            },
        );
        assert_eq!(actions.len(), 1);

        // Condition not met
        let actions = engine.evaluate(
            "/sensor/motion",
            &Value::Bool(true),
            SignalType::Param,
            None,
            |addr| {
                if addr == "/mode" {
                    Some(Value::String("manual".to_string()))
                } else {
                    None
                }
            },
        );
        assert!(actions.is_empty());
    }

    #[test]
    fn test_threshold_trigger() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(Rule {
                id: "r1".to_string(),
                name: "High temp alert".to_string(),
                enabled: true,
                trigger: Trigger::OnThreshold {
                    address: "/sensor/temp".to_string(),
                    above: Some(30.0),
                    below: None,
                },
                conditions: vec![],
                actions: vec![RuleAction::Publish {
                    address: "/alerts/temp".to_string(),
                    signal: SignalType::Event,
                    value: Some(Value::String("high temperature".to_string())),
                }],
                cooldown: None,
            })
            .unwrap();

        // Below threshold -- no action
        let actions = engine.evaluate(
            "/sensor/temp",
            &Value::Float(25.0),
            SignalType::Param,
            None,
            |_| None,
        );
        assert!(actions.is_empty());

        // Above threshold -- fires
        let actions = engine.evaluate(
            "/sensor/temp",
            &Value::Float(35.0),
            SignalType::Param,
            None,
            |_| None,
        );
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_set_from_trigger_with_transform() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(Rule {
                id: "r1".to_string(),
                name: "Scale input".to_string(),
                enabled: true,
                trigger: Trigger::OnChange {
                    pattern: "/input/fader".to_string(),
                },
                conditions: vec![],
                actions: vec![RuleAction::SetFromTrigger {
                    address: "/output/brightness".to_string(),
                    transform: Transform::Scale {
                        scale: 255.0,
                        offset: 0.0,
                    },
                }],
                cooldown: None,
            })
            .unwrap();

        let actions = engine.evaluate(
            "/input/fader",
            &Value::Float(0.5),
            SignalType::Param,
            None,
            |_| None,
        );

        assert_eq!(actions.len(), 1);
        match &actions[0].action {
            RuleAction::Set { value, .. } => {
                assert_eq!(*value, Value::Float(127.5));
            }
            _ => panic!("expected Set action"),
        }
    }

    #[test]
    fn test_loop_prevention() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(make_rule("r1", "/sensor/**", "/output", Value::Bool(true)))
            .unwrap();

        // Origin from a rule should be skipped
        let actions = engine.evaluate(
            "/sensor/motion",
            &Value::Bool(true),
            SignalType::Param,
            Some("rule:r1"),
            |_| None,
        );
        assert!(actions.is_empty());
    }

    #[test]
    fn test_cooldown() {
        let mut engine = RulesEngine::new();
        let mut rule = make_rule("r1", "/sensor/**", "/output", Value::Bool(true));
        rule.cooldown = Some(std::time::Duration::from_secs(60));
        engine.add_rule(rule).unwrap();

        // First evaluation fires
        let actions = engine.evaluate(
            "/sensor/motion",
            &Value::Bool(true),
            SignalType::Param,
            None,
            |_| None,
        );
        assert_eq!(actions.len(), 1);

        // Second evaluation within cooldown does not fire
        let actions = engine.evaluate(
            "/sensor/motion",
            &Value::Bool(true),
            SignalType::Param,
            None,
            |_| None,
        );
        assert!(actions.is_empty());
    }

    #[test]
    fn test_remove_rule() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(make_rule("r1", "/a", "/b", Value::Null))
            .unwrap();
        assert_eq!(engine.len(), 1);

        engine.remove_rule("r1").unwrap();
        assert_eq!(engine.len(), 0);

        assert!(engine.remove_rule("nonexistent").is_err());
    }

    #[test]
    fn test_event_trigger() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(Rule {
                id: "r1".to_string(),
                name: "On button press".to_string(),
                enabled: true,
                trigger: Trigger::OnEvent {
                    pattern: "/buttons/**".to_string(),
                },
                conditions: vec![],
                actions: vec![RuleAction::Set {
                    address: "/lights/toggle".to_string(),
                    value: Value::Bool(true),
                }],
                cooldown: None,
            })
            .unwrap();

        // Event matches
        let actions = engine.evaluate(
            "/buttons/main",
            &Value::Null,
            SignalType::Event,
            None,
            |_| None,
        );
        assert_eq!(actions.len(), 1);

        // Param change does not match event trigger
        let actions = engine.evaluate(
            "/buttons/main",
            &Value::Null,
            SignalType::Param,
            None,
            |_| None,
        );
        assert!(actions.is_empty());
    }

    #[test]
    fn test_interval_rules() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(Rule {
                id: "heartbeat".to_string(),
                name: "Heartbeat".to_string(),
                enabled: true,
                trigger: Trigger::OnInterval { seconds: 30 },
                conditions: vec![],
                actions: vec![RuleAction::Publish {
                    address: "/system/heartbeat".to_string(),
                    signal: SignalType::Event,
                    value: None,
                }],
                cooldown: None,
            })
            .unwrap();

        let intervals = engine.interval_rules();
        assert_eq!(intervals.len(), 1);
        assert_eq!(intervals[0], ("heartbeat".to_string(), 30));
    }

    #[test]
    fn test_evaluate_interval() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(Rule {
                id: "heartbeat".to_string(),
                name: "Heartbeat".to_string(),
                enabled: true,
                trigger: Trigger::OnInterval { seconds: 30 },
                conditions: vec![],
                actions: vec![RuleAction::Publish {
                    address: "/system/heartbeat".to_string(),
                    signal: SignalType::Event,
                    value: None,
                }],
                cooldown: None,
            })
            .unwrap();

        let actions = engine.evaluate_interval("heartbeat", |_| None);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].rule_id, "heartbeat");
        assert!(actions[0].origin.starts_with("interval:"));
    }

    #[test]
    fn test_evaluate_interval_with_condition() {
        let mut engine = RulesEngine::new();
        engine
            .add_rule(Rule {
                id: "conditional_interval".to_string(),
                name: "Conditional interval".to_string(),
                enabled: true,
                trigger: Trigger::OnInterval { seconds: 10 },
                conditions: vec![Condition {
                    address: "/mode".to_string(),
                    op: CompareOp::Eq,
                    value: Value::String("active".to_string()),
                }],
                actions: vec![RuleAction::Set {
                    address: "/output".to_string(),
                    value: Value::Bool(true),
                }],
                cooldown: None,
            })
            .unwrap();

        // Condition not met
        let actions = engine.evaluate_interval("conditional_interval", |_| None);
        assert!(actions.is_empty());

        // Condition met
        let actions = engine.evaluate_interval("conditional_interval", |addr| {
            if addr == "/mode" {
                Some(Value::String("active".to_string()))
            } else {
                None
            }
        });
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_evaluate_interval_disabled() {
        let mut engine = RulesEngine::new();
        let rule = Rule {
            id: "disabled_interval".to_string(),
            name: "Disabled".to_string(),
            enabled: false,
            trigger: Trigger::OnInterval { seconds: 5 },
            conditions: vec![],
            actions: vec![RuleAction::Set {
                address: "/x".to_string(),
                value: Value::Null,
            }],
            cooldown: None,
        };
        engine.add_rule(rule).unwrap();

        let actions = engine.evaluate_interval("disabled_interval", |_| None);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_evaluate_interval_nonexistent() {
        let mut engine = RulesEngine::new();
        let actions = engine.evaluate_interval("nonexistent", |_| None);
        assert!(actions.is_empty());
    }
}
