---
title: Rules Engine
description: Server-side reactive automation with triggers, conditions, and actions
order: 2
---

# Rules Engine

The rules engine lets you define server-side automation: when a value changes or crosses a threshold, automatically set other values, emit events, or transform data. Rules run on the relay, so they execute even when no client is connected and respond faster than a round-trip to an external service.

## Enable

The rules engine requires the `rules` feature flag at build time and a rules file at runtime:

```bash
clasp-relay --rules ./rules.json
```

If you build with `--features full`, rules support is included.

## Rule Structure

A rules file contains a JSON object with a `rules` array. Each rule has the following fields:

```json
{
  "rules": [
    {
      "id": "unique-rule-id",
      "name": "Human-readable name",
      "enabled": true,
      "trigger": {},
      "conditions": [],
      "actions": [],
      "cooldown": 0
    }
  ]
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier for the rule |
| `name` | No | Human-readable label |
| `enabled` | No | Whether the rule is active (default `true`) |
| `trigger` | Yes | What causes the rule to evaluate |
| `conditions` | No | Additional state checks that must pass |
| `actions` | Yes | What happens when the rule fires |
| `cooldown` | No | Minimum seconds between firings (default `0`) |

## Triggers

Each rule has exactly one trigger. The trigger determines when the rule is evaluated.

### on_change

Fires when any param matching the pattern is set to a new value.

```json
{
  "type": "on_change",
  "pattern": "/sensors/**"
}
```

Wildcards `*` (single segment) and `**` (multi-segment) are supported. This trigger fires on every write to any path under `/sensors/`.

### on_threshold

Fires when a specific param's value crosses a numeric threshold. The rule fires once on the crossing, not on every update above the threshold.

```json
{
  "type": "on_threshold",
  "address": "/sensors/temp",
  "above": 30.0
}
```

Use `above` to fire when the value rises past the threshold, or `below` to fire when it drops:

```json
{
  "type": "on_threshold",
  "address": "/sensors/humidity",
  "below": 20.0
}
```

### on_event

Fires when an event matching the pattern is published.

```json
{
  "type": "on_event",
  "pattern": "/alerts/**"
}
```

Unlike `on_change`, this trigger responds to events (fire-and-forget signals) rather than param state changes.

### on_interval

Fires periodically on a timer.

```json
{
  "type": "on_interval",
  "seconds": 60
}
```

Useful for periodic state checks, heartbeat signals, or cleanup tasks.

## Conditions

Conditions are an optional array of state checks. All conditions must be true for the rule to fire. If any condition fails, the trigger is consumed but no actions execute.

```json
{
  "conditions": [
    {"address": "/system/alerts-enabled", "op": "eq", "value": true},
    {"address": "/system/mode", "op": "ne", "value": "maintenance"}
  ]
}
```

Each condition reads the current state at `address` and compares it to `value` using the operator `op`.

| Operator | Description |
|----------|-------------|
| `eq` | Equal |
| `ne` | Not equal |
| `gt` | Greater than |
| `gte` | Greater than or equal |
| `lt` | Less than |
| `lte` | Less than or equal |

Conditions are evaluated at the moment the trigger fires. If the state at the condition's address does not exist, the condition fails (unless the check is `eq` against `null`).

## Actions

Actions execute in order when a rule fires. A rule must have at least one action.

### set

Sets a param to a fixed value.

```json
{
  "type": "set",
  "address": "/hvac/fan",
  "value": true
}
```

### publish

Emits an event. The event is delivered to all subscribers but is not stored.

```json
{
  "type": "publish",
  "address": "/alerts/high-temp",
  "value": {"source": "temp-alert", "threshold": 30.0}
}
```

### set_from_trigger

Copies the trigger's value to another address, optionally applying a transform. This is the most powerful action type -- it lets you derive values from incoming data.

```json
{
  "type": "set_from_trigger",
  "address": "/display/temp-f",
  "transform": {
    "type": "scale",
    "factor": 1.8,
    "offset": 32
  }
}
```

If the trigger value is `25.0` (Celsius), this writes `77.0` (Fahrenheit) to `/display/temp-f`.

When no transform is specified, the trigger value is copied as-is (identity transform).

### delay

Pauses execution for the specified duration before continuing to the next action. Does not affect the trigger value.

```json
{
  "type": "delay",
  "milliseconds": 1000
}
```

Use delays to sequence actions with timing gaps, for example turning on a warning light before sounding an alarm.

## Transforms

Transforms modify the trigger value before it is written by a `set_from_trigger` action. Five transform types are available.

### identity

Passes the value through unchanged. This is the default when no transform is specified.

```json
{"type": "identity"}
```

### scale

Multiplies the value by `factor` and adds `offset`. Useful for unit conversions.

```json
{"type": "scale", "factor": 1.8, "offset": 32}
```

Formula: `output = (input * factor) + offset`

### clamp

Constrains the value to a range.

```json
{"type": "clamp", "min": 0.0, "max": 100.0}
```

Values below `min` become `min`; values above `max` become `max`.

### threshold

Outputs one of two values depending on whether the input is above or below a threshold.

```json
{"type": "threshold", "value": 50.0, "above": 1.0, "below": 0.0}
```

If input >= `value`, output is `above`. Otherwise, output is `below`.

### invert

Subtracts the value from 1.0. Useful for inverting normalized (0-1) ranges.

```json
{"type": "invert"}
```

Formula: `output = 1.0 - input`

## Loop Prevention

Rule-generated actions are tagged with `origin: "rule:{ruleId}"`. When the rules engine evaluates triggers caused by rule actions, it skips any rule whose ID matches the origin tag. This prevents infinite loops where a rule's output triggers itself.

For example, if rule `temp-alert` sets `/hvac/fan` to `true`, and another rule triggers on changes to `/hvac/**`, the second rule fires normally. But if `temp-alert` itself also triggers on `/hvac/**`, it will not re-fire from its own output.

Cross-rule chains (A triggers B triggers C) are allowed and execute in order. Only self-loops are prevented.

## Complete Example

A rules file that monitors temperature, activates a fan, converts to Fahrenheit for display, and sends an alert:

```json
{
  "rules": [
    {
      "id": "temp-alert",
      "name": "High Temperature Alert",
      "enabled": true,
      "trigger": {
        "type": "on_threshold",
        "address": "/sensors/temp",
        "above": 30.0
      },
      "conditions": [
        {"address": "/system/alerts-enabled", "op": "eq", "value": true}
      ],
      "actions": [
        {"type": "set", "address": "/hvac/fan", "value": true},
        {
          "type": "publish",
          "address": "/alerts/high-temp",
          "value": {"source": "temp-alert", "threshold": 30.0}
        },
        {
          "type": "set_from_trigger",
          "address": "/display/temp-f",
          "transform": {"type": "scale", "factor": 1.8, "offset": 32}
        }
      ],
      "cooldown": 60
    },
    {
      "id": "temp-normal",
      "name": "Temperature Normal",
      "enabled": true,
      "trigger": {
        "type": "on_threshold",
        "address": "/sensors/temp",
        "below": 25.0
      },
      "actions": [
        {"type": "set", "address": "/hvac/fan", "value": false}
      ],
      "cooldown": 60
    },
    {
      "id": "heartbeat",
      "name": "System Heartbeat",
      "enabled": true,
      "trigger": {
        "type": "on_interval",
        "seconds": 30
      },
      "actions": [
        {
          "type": "publish",
          "address": "/system/heartbeat",
          "value": {"status": "ok"}
        }
      ]
    }
  ]
}
```

## Next Steps

- [App Config](./app-config.md) -- declarative scopes and write rules
- [Persistence](./persistence.md) -- state snapshots and journal
- [Federation](./federation.md) -- multi-site state sync
