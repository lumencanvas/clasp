# clasp-rules

Server-side reactive automation engine for CLASP routers.

## Features

- **Reactive Triggers** - Fire on state change, threshold crossing, events, or intervals
- **Conditional Execution** - Guard rules with comparisons against live state
- **Transform Pipeline** - Scale, clamp, invert, or threshold values on the fly
- **Loop Prevention** - Automatic origin tagging prevents rule feedback loops
- **Cooldowns** - Per-rule minimum time between firings
- **JSON Rules** - Define rules in JSON for runtime loading

## Installation

```toml
[dependencies]
clasp-rules = "3.5"
```

## Usage

### Motion Sensor to Lights (OnChange)

```rust
use clasp_rules::{Rule, Trigger, RuleAction, RulesEngine};
use clasp_core::Value;
use std::time::Duration;

let rule = Rule {
    id: "motion-lights".to_string(),
    name: "Motion activates lights".to_string(),
    enabled: true,
    trigger: Trigger::OnChange {
        pattern: "/sensors/*/motion".to_string(),
    },
    conditions: vec![],
    actions: vec![RuleAction::Set {
        address: "/lights/hallway/brightness".to_string(),
        value: Value::Float(1.0),
    }],
    cooldown: Some(Duration::from_secs(5)),
};

let mut engine = RulesEngine::new();
engine.add_rule(rule)?;
```

### Threshold Alert

```rust
use clasp_rules::{Trigger, Condition, CompareOp, RuleAction};
use clasp_core::{Value, SignalType};

let rule = Rule {
    id: "temp-alert".to_string(),
    name: "High temperature alert".to_string(),
    enabled: true,
    trigger: Trigger::OnThreshold {
        address: "/sensors/room1/temperature".to_string(),
        above: Some(30.0),
        below: None,
    },
    conditions: vec![Condition {
        address: "/system/alerts/enabled".to_string(),
        op: CompareOp::Eq,
        value: Value::Bool(true),
    }],
    actions: vec![RuleAction::Publish {
        address: "/alerts/temperature".to_string(),
        signal: SignalType::Event,
        value: Some(Value::String("High temperature in room 1".into())),
    }],
    cooldown: Some(Duration::from_secs(60)),
};
```

### SetFromTrigger with Scale Transform

```rust
use clasp_rules::{RuleAction, Transform};

// Map a 0-1 slider to 0-255 DMX range
let rule = Rule {
    id: "slider-to-dmx".to_string(),
    name: "Scale slider to DMX".to_string(),
    enabled: true,
    trigger: Trigger::OnChange {
        pattern: "/controls/slider1".to_string(),
    },
    conditions: vec![],
    actions: vec![RuleAction::SetFromTrigger {
        address: "/dmx/1/channel/1".to_string(),
        transform: Transform::Scale {
            scale: 255.0,
            offset: 0.0,
        },
    }],
    cooldown: None,
};
```

### Periodic Heartbeat (OnInterval)

```rust
let rule = Rule {
    id: "heartbeat".to_string(),
    name: "Periodic heartbeat".to_string(),
    enabled: true,
    trigger: Trigger::OnInterval { seconds: 10 },
    conditions: vec![],
    actions: vec![RuleAction::Publish {
        address: "/system/heartbeat".to_string(),
        signal: SignalType::Event,
        value: None,
    }],
    cooldown: None,
};
```

### Evaluate Rules

```rust
let actions = engine.evaluate(
    "/sensors/room1/motion",    // address that changed
    &Value::Bool(true),         // new value
    SignalType::Param,          // signal type
    None,                       // origin (None = from client)
    |addr| {                    // state lookup function
        // Return current value for an address
        Some(Value::Bool(true))
    },
);

for action in actions {
    println!("Rule {} fires: {:?}", action.rule_id, action.action);
    // Execute action.action against the router
}

// For interval rules
let intervals = engine.interval_rules(); // Vec<(rule_id, seconds)>
let actions = engine.evaluate_interval("heartbeat", |addr| None);
```

### JSON Rule Definition

Rules are fully serializable for runtime configuration:

```json
{
  "id": "motion-lights",
  "name": "Motion activates lights",
  "enabled": true,
  "trigger": {
    "OnChange": {
      "pattern": "/sensors/*/motion"
    }
  },
  "conditions": [],
  "actions": [
    {
      "Set": {
        "address": "/lights/hallway/brightness",
        "value": 1.0
      }
    }
  ],
  "cooldown": {
    "secs": 5,
    "nanos": 0
  }
}
```

## Configuration Reference

### Rule

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | `String` | required | Unique rule identifier |
| `name` | `String` | required | Human-readable name |
| `enabled` | `bool` | required | Whether the rule is active |
| `trigger` | `Trigger` | required | What triggers the rule |
| `conditions` | `Vec<Condition>` | `[]` | Additional conditions (all must be true) |
| `actions` | `Vec<RuleAction>` | required | Actions to execute when the rule fires |
| `cooldown` | `Option<Duration>` | `None` | Minimum time between firings |

### Trigger Variants

| Variant | Fields | Description |
|---------|--------|-------------|
| `OnChange` | `pattern: String` | Fires when a param matching the pattern changes |
| `OnThreshold` | `address: String`, `above: Option<f64>`, `below: Option<f64>` | Fires when a value crosses a threshold |
| `OnEvent` | `pattern: String` | Fires when an event matching the pattern is published |
| `OnInterval` | `seconds: u64` | Fires periodically |

### Condition

| Field | Type | Description |
|-------|------|-------------|
| `address` | `String` | CLASP address to check |
| `op` | `CompareOp` | `Eq`, `Ne`, `Gt`, `Gte`, `Lt`, `Lte` |
| `value` | `Value` | Value to compare against |

### RuleAction Variants

| Variant | Fields | Description |
|---------|--------|-------------|
| `Set` | `address`, `value` | Set a parameter to a fixed value |
| `Publish` | `address`, `signal`, `value?` | Publish an event |
| `SetFromTrigger` | `address`, `transform` | Copy trigger's value with optional transform |
| `Delay` | `milliseconds` | Delay before the next action |

### Transform Variants

| Variant | Fields | Description |
|---------|--------|-------------|
| `Identity` | -- | Pass through unchanged |
| `Scale` | `scale`, `offset` | `output = input * scale + offset` |
| `Clamp` | `min`, `max` | Clamp to range `[min, max]` |
| `Threshold` | `value` | `true` if input > value, else `false` |
| `Invert` | `min`, `max` | `output = max - (input - min)` |

## Loop Prevention

Actions produced by rules carry an `origin` field (`"rule:{id}"` or `"interval:{id}"`). The engine skips evaluation when the origin starts with `"rule:"`, preventing infinite feedback loops.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio)
