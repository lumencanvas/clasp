---
title: Rules Schema
description: Complete JSON schema for CLASP rules engine
order: 6
---

# Rules Schema

Complete reference for the rules engine JSON file loaded via the `--rules` CLI flag or the `set_rules_engine()` router method. For a guided walkthrough of building rules, see the examples below.

## Top-Level Structure

A rules file is a JSON object with a single `rules` key containing an array of Rule objects.

```json
{
  "rules": [
    { ... },
    { ... }
  ]
}
```

## Rule Object

| Field        | Type           | Required | Default | Description                                         |
|--------------|----------------|----------|---------|-----------------------------------------------------|
| `id`         | string         | Yes      | --      | Unique identifier for the rule                      |
| `name`       | string         | Yes      | --      | Human-readable name                                 |
| `enabled`    | bool           | No       | `true`  | Whether the rule is active                          |
| `trigger`    | Trigger        | Yes      | --      | What causes the rule to evaluate                    |
| `conditions` | Condition[]    | No       | `[]`    | All conditions must be true for actions to execute  |
| `actions`    | RuleAction[]   | Yes      | --      | Actions to execute when triggered and conditions met|
| `cooldown`   | number         | No       | --      | Minimum seconds between firings                     |

## Triggers

A trigger defines what event causes the rule to evaluate. Exactly one trigger per rule. The `type` field determines the trigger kind.

### on_change

Fires when a parameter matching the glob pattern changes value.

```json
{
  "type": "on_change",
  "pattern": "/lights/*/intensity"
}
```

| Field     | Type   | Required | Description                          |
|-----------|--------|----------|--------------------------------------|
| `type`    | string | Yes      | Must be `"on_change"`                |
| `pattern` | string | Yes      | Glob pattern matching CLASP addresses. Supports `*` (single segment) and `**` (multiple segments). |

### on_threshold

Fires when a numeric parameter crosses a threshold boundary. The rule fires on the transition, not while the value remains above or below.

```json
{
  "type": "on_threshold",
  "address": "/sensors/temperature",
  "above": 30.0
}
```

| Field     | Type   | Required | Description                                   |
|-----------|--------|----------|-----------------------------------------------|
| `type`    | string | Yes      | Must be `"on_threshold"`                      |
| `address` | string | Yes      | Exact CLASP address to monitor                |
| `above`   | number | No*      | Fire when value crosses upward past this value |
| `below`   | number | No*      | Fire when value crosses downward past this value |

*At least one of `above` or `below` must be specified. Both may be specified to fire on either crossing.

### on_event

Fires when an event signal is emitted matching the glob pattern.

```json
{
  "type": "on_event",
  "pattern": "/events/**"
}
```

| Field     | Type   | Required | Description                          |
|-----------|--------|----------|--------------------------------------|
| `type`    | string | Yes      | Must be `"on_event"`                 |
| `pattern` | string | Yes      | Glob pattern matching event addresses |

### on_interval

Fires periodically on a timer.

```json
{
  "type": "on_interval",
  "seconds": 60
}
```

| Field     | Type   | Required | Description                        |
|-----------|--------|----------|------------------------------------|
| `type`    | string | Yes      | Must be `"on_interval"`            |
| `seconds` | number | Yes      | Interval in seconds between firings |

## Conditions

Conditions are optional guards. When present, all conditions must evaluate to true for the rule's actions to execute. Each condition reads a current parameter value and compares it using an operator.

```json
{
  "address": "/system/mode",
  "op": "eq",
  "value": "live"
}
```

| Field     | Type   | Required | Description                        |
|-----------|--------|----------|------------------------------------|
| `address` | string | Yes      | CLASP address to read              |
| `op`      | string | Yes      | Comparison operator (see below)    |
| `value`   | any    | Yes      | Value to compare against           |

### Comparison Operators

| Operator | Description              | Applicable Types          |
|----------|--------------------------|---------------------------|
| `eq`     | Equal to                 | All types                 |
| `ne`     | Not equal to             | All types                 |
| `gt`     | Greater than             | Numbers                   |
| `gte`    | Greater than or equal to | Numbers                   |
| `lt`     | Less than                | Numbers                   |
| `lte`    | Less than or equal to    | Numbers                   |

## Actions

Actions execute sequentially when a rule fires and all conditions pass. The `type` field determines the action kind.

### set

Sets a parameter to a fixed value.

```json
{
  "type": "set",
  "address": "/lights/main/intensity",
  "value": 1.0
}
```

| Field     | Type   | Required | Description             |
|-----------|--------|----------|-------------------------|
| `type`    | string | Yes      | Must be `"set"`         |
| `address` | string | Yes      | Target CLASP address    |
| `value`   | any    | Yes      | Value to set            |

### publish

Emits an event signal.

```json
{
  "type": "publish",
  "address": "/events/alerts/overheat",
  "value": { "source": "temperature-rule" }
}
```

| Field     | Type   | Required | Description              |
|-----------|--------|----------|--------------------------|
| `type`    | string | Yes      | Must be `"publish"`      |
| `address` | string | Yes      | Event address to emit on |
| `value`   | any    | Yes      | Event payload            |

### set_from_trigger

Copies the value that caused the trigger to fire, optionally applying a transform, and sets it on a target address. Only valid with `on_change` and `on_threshold` triggers.

```json
{
  "type": "set_from_trigger",
  "address": "/lights/mirror/intensity",
  "transform": { "type": "scale", "factor": 0.5, "offset": 0 }
}
```

| Field       | Type      | Required | Description                          |
|-------------|-----------|----------|--------------------------------------|
| `type`      | string    | Yes      | Must be `"set_from_trigger"`         |
| `address`   | string    | Yes      | Target CLASP address                 |
| `transform` | Transform | Yes      | Transform to apply to trigger value  |

### delay

Pauses execution for a specified duration before the next action in the sequence runs.

```json
{
  "type": "delay",
  "milliseconds": 1000
}
```

| Field          | Type   | Required | Description                |
|----------------|--------|----------|----------------------------|
| `type`         | string | Yes      | Must be `"delay"`          |
| `milliseconds` | number | Yes      | Pause duration in milliseconds |

## Transforms

Transforms modify a numeric value before it is written. Used by the `set_from_trigger` action.

### identity

Passes the value through unchanged.

```json
{ "type": "identity" }
```

Input `0.7` produces output `0.7`.

### scale

Applies a linear transformation: `output = value * factor + offset`.

```json
{ "type": "scale", "factor": 1.8, "offset": 32 }
```

| Field    | Type   | Required | Description        |
|----------|--------|----------|--------------------|
| `factor` | number | Yes      | Multiplication factor |
| `offset` | number | Yes      | Additive offset    |

Input `100` with `factor: 1.8, offset: 32` produces output `212` (Celsius to Fahrenheit).

### clamp

Constrains the value to a range.

```json
{ "type": "clamp", "min": 0, "max": 100 }
```

| Field | Type   | Required | Description    |
|-------|--------|----------|----------------|
| `min` | number | Yes      | Minimum bound  |
| `max` | number | Yes      | Maximum bound  |

Input `150` with `min: 0, max: 100` produces output `100`.

### threshold

Converts a numeric value to one of two binary outputs based on a cutoff.

```json
{ "type": "threshold", "value": 50, "above": true, "below": false }
```

| Field   | Type   | Required | Description                              |
|---------|--------|----------|------------------------------------------|
| `value` | number | Yes      | Threshold cutoff                         |
| `above` | any    | Yes      | Output when input >= threshold           |
| `below` | any    | Yes      | Output when input < threshold            |

Input `75` with `value: 50, above: true, below: false` produces output `true`.

### invert

Computes `1.0 - value`. Useful for inverting normalized (0.0--1.0) values.

```json
{ "type": "invert" }
```

Input `0.3` produces output `0.7`.

## Complete Example

A rules file that monitors temperature sensors and mirrors light values:

```json
{
  "rules": [
    {
      "id": "overheat-alert",
      "name": "Overheat Alert",
      "trigger": {
        "type": "on_threshold",
        "address": "/sensors/temperature",
        "above": 80.0
      },
      "conditions": [
        { "address": "/system/mode", "op": "eq", "value": "live" }
      ],
      "actions": [
        {
          "type": "publish",
          "address": "/events/alerts/overheat",
          "value": { "severity": "warning" }
        },
        {
          "type": "set",
          "address": "/lights/warning/intensity",
          "value": 1.0
        }
      ],
      "cooldown": 30
    },
    {
      "id": "mirror-intensity",
      "name": "Mirror Main to Secondary",
      "trigger": {
        "type": "on_change",
        "pattern": "/lights/main/intensity"
      },
      "actions": [
        {
          "type": "set_from_trigger",
          "address": "/lights/secondary/intensity",
          "transform": { "type": "scale", "factor": 0.5, "offset": 0 }
        }
      ]
    },
    {
      "id": "periodic-heartbeat",
      "name": "Heartbeat Event",
      "trigger": {
        "type": "on_interval",
        "seconds": 10
      },
      "actions": [
        {
          "type": "publish",
          "address": "/events/system/heartbeat",
          "value": { "status": "ok" }
        }
      ]
    }
  ]
}
```

## Next Steps

- [App Config Schema](app-config-schema.md) -- scopes, write rules, and rate limits for the relay server
- [Router Config](router-config.md) -- embedding a router in Rust with `RouterConfig`
