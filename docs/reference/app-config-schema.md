---
title: App Config Schema
description: Complete JSON schema for CLASP app config
order: 5
---

# App Config Schema

Complete reference for the app config JSON file loaded via `--app-config`. The app config provides a declarative way to define scopes, write validation rules, snapshot transforms, snapshot visibility, and rate limits -- without writing Rust code or compiling plugins.

The protocol layer (`clasp-core`, `clasp-router`) stays application-agnostic. This config drives a generic rule engine in the relay that replaces hardcoded validators.

## Top-Level Structure

```json
{
  "scopes": [],
  "write_rules": [],
  "snapshot_transforms": [],
  "snapshot_visibility": [],
  "rate_limits": {}
}
```

All fields are optional. An empty JSON object `{}` is a valid config.

## scopes

Array of scope template strings. Each scope defines what actions a session is allowed to perform on which address patterns.

**Format:** `"action:pattern"`

**Actions:**

| Action | Description |
|--------|-------------|
| `read` | Subscribe to and receive values at matching addresses |
| `write` | Set/publish values at matching addresses |
| `admin` | Full access (read + write + administrative operations) |

**Placeholders:**

| Placeholder | Expansion |
|-------------|-----------|
| `{userId}` | Replaced with the authenticated user's ID at session creation |

**Examples:**

```json
{
  "scopes": [
    "read:/app/**",
    "write:/app/user/{userId}/**",
    "write:/app/room/*/messages",
    "admin:/app/admin/**"
  ]
}
```

With these scopes, a user `alice` would receive:
- `read:/app/**` -- can read everything under `/app/`
- `write:/app/user/alice/**` -- can write to her own user subtree
- `write:/app/room/*/messages` -- can write messages in any room
- `admin:/app/admin/**` -- admin access under `/app/admin/`

## write_rules

Array of write validation rule objects. Rules are evaluated in order against the write address; the first matching rule determines validation. If no rule matches, the write is allowed (pass-through).

### WriteRule Object

```json
{
  "path": "/app/room/{roomId}/messages",
  "checks": [],
  "pre_checks": [],
  "allow_null_write": false,
  "mode": "all"
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `path` | string | yes | -- | Path pattern with `{named}` captures (e.g., `/app/room/{roomId}/meta`) |
| `checks` | array | yes | -- | Array of check objects to evaluate |
| `pre_checks` | array | no | `[]` | Checks that always run, even for null writes when `allow_null_write` is true. Always uses `mode: "all"`. |
| `allow_null_write` | bool | no | `false` | If true, null (delete) writes skip `checks` but `pre_checks` still run |
| `mode` | string | no | `"all"` | How checks combine: `"all"` (every check must pass) or `"any"` (at least one must pass) |

## Check Types

Each check is a JSON object with a `type` field that determines the check kind. Below is the complete reference for all 7 check types.

### state_field_equals_session

Look up a value in state by address, extract a named field from the map, and compare it to the session's authenticated subject (user ID).

```json
{
  "type": "state_field_equals_session",
  "lookup": "/app/room/{roomId}/meta",
  "field": "createdBy",
  "allow_if_missing": false
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `lookup` | string | yes | -- | State address to look up. Supports `{captures}` from the rule path and `{session}` for the current user ID. |
| `field` | string | yes | -- | Field name to extract from the map value at `lookup` |
| `allow_if_missing` | bool | no | `false` | If true, the check passes when the state at `lookup` does not exist (useful for initial creation) |

**Behavior:** Looks up `lookup` in router state. If found, extracts `field` from the map value. Passes if `field == session_subject`. Fails if the field does not match or is missing. If `allow_if_missing` is true and the state entry does not exist, the check passes (allowing initial creation).

### state_not_null

Look up a value in state and require that it exists and is not null.

```json
{
  "type": "state_not_null",
  "lookup": "/app/room/{roomId}/presence/{session}"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `lookup` | string | yes | State address to check. Supports `{captures}` and `{session}`. |

**Behavior:** Passes if the state at `lookup` exists and is not null. Fails otherwise.

### value_field_equals_session

Extract a named field from the value being written and compare it to the session's authenticated subject.

```json
{
  "type": "value_field_equals_session",
  "field": "userId"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field` | string | yes | Field name to extract from the written value (must be a map) |

**Behavior:** Extracts `field` from the written value. Passes if it equals the session subject. Fails if the field is missing or does not match. The written value must be a map.

### segment_equals_session

Require that a named path segment from the matched pattern equals the session's authenticated subject.

```json
{
  "type": "segment_equals_session",
  "segment": "userId"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `segment` | string | yes | Name of the path capture (from the rule's `path` pattern) to compare |

**Behavior:** Looks up the named capture `segment` from the matched path. Passes if it equals the session subject. Fails otherwise.

**Example:** With rule path `/app/user/{userId}/profile` and a write to `/app/user/alice/profile`, the capture `userId` is `"alice"`. If the session subject is `"alice"`, the check passes.

### either_state_not_null

Pass if either of two state lookups exists and is not null.

```json
{
  "type": "either_state_not_null",
  "lookup_a": "/app/user/{session}/friends/{targetId}",
  "lookup_b": "/app/user/{targetId}/friends/{session}"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `lookup_a` | string | yes | First state address to check. Supports `{captures}` and `{session}`. |
| `lookup_b` | string | yes | Second state address to check. Supports `{captures}` and `{session}`. |

**Behavior:** Passes if either `lookup_a` or `lookup_b` exists and is not null in state. Fails if both are null or missing.

### require_value_field

Require that the written value contains a specific non-null string field.

```json
{
  "type": "require_value_field",
  "field": "content"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field` | string | yes | Field name that must exist as a non-null string in the written value |

**Behavior:** Extracts `field` from the written value (must be a map). Passes if the field exists and is a non-null string. Fails otherwise.

### reject_unless_path_matches

Reject writes to addresses matching the rule path unless they also match a more specific pattern. Used to reject malformed or unexpected paths.

```json
{
  "type": "reject_unless_path_matches",
  "pattern": "/app/room/{roomId}/admin/{targetId}",
  "message": "Invalid admin path format"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `pattern` | string | yes | More specific path pattern that the address must match |
| `message` | string | yes | Error message returned if the address does not match |

**Behavior:** Passes if the write address matches `pattern`. Fails with `message` if it does not.

## snapshot_transforms

Array of transform rules applied to snapshot values before delivery. All matching transforms are applied (not first-match). Used for field redaction.

```json
{
  "snapshot_transforms": [
    {
      "path": "/app/user/{id}/auth",
      "redact_fields": ["passwordHash", "passwordSalt", "totpSecret"]
    },
    {
      "path": "/app/*/meta",
      "redact_fields": ["internalId"]
    }
  ]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | yes | Path pattern to match against parameter addresses |
| `redact_fields` | array | yes | Field names to remove from map values before delivery |

**Behavior:** For each parameter in the snapshot, if the address matches `path`, the listed fields are removed from the value (if it is a map). Multiple transforms can match the same parameter.

## snapshot_visibility

Array of visibility rules controlling which parameters are included in snapshots. Rules are evaluated in order; the first matching rule wins. If no rule matches, the parameter is visible (default: visible).

Three visibility modes exist:

### Boolean Visibility

Static visibility: always include or always exclude.

```json
{
  "path_contains": "/__auth",
  "visible": false
}
```

```json
{
  "path": "/app/*/profile",
  "visible": true
}
```

### Owner Visibility

Only the owner (determined by a path segment matching the session subject) can see the parameter. An optional `public_sub` allows a specific sub-path to be publicly visible.

```json
{
  "path": "/app/user/{userId}/**",
  "visible": "owner",
  "owner_segment": "userId",
  "public_sub": "profile"
}
```

With this rule:
- User `alice` can see everything under `/app/user/alice/**`
- Other users can only see parameters under `/app/user/alice/profile` and `/app/user/alice/profile/**`
- All other paths under `/app/user/alice/` are hidden from non-owners

### Require State Not Null

Only show the parameter if a related state entry exists and is not null (e.g., presence check).

```json
{
  "path": "/app/room/{roomId}/**",
  "visible": "require_state_not_null",
  "lookup": "/app/room/{roomId}/presence/{session}"
}
```

### VisibilityRule Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path_contains` | string | no | Match addresses containing this substring |
| `path` | string | no | Match addresses against this path pattern |
| `visible` | bool or string | yes | `true`, `false`, `"owner"`, or `"require_state_not_null"` |
| `owner_segment` | string | for owner | Path capture name that must equal the session subject |
| `lookup` | string | for require_state_not_null | State address template to check |
| `public_sub` | string | no | For owner mode: sub-path visible to non-owners |

If neither `path_contains` nor `path` is set, the rule matches all addresses (catch-all).

## rate_limits

Rate limit configuration for the auth subsystem.

```json
{
  "rate_limits": {
    "login_max_attempts": 5,
    "login_window_secs": 60,
    "register_max_attempts": 10,
    "register_window_secs": 60
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `login_max_attempts` | integer | `5` | Maximum login attempts per window |
| `login_window_secs` | integer | `60` | Login rate limit window in seconds |
| `register_max_attempts` | integer | `10` | Maximum registration attempts per window |
| `register_window_secs` | integer | `60` | Registration rate limit window in seconds |

All fields are optional. Missing fields use the shown defaults.

## Path Matching

Path patterns used in `write_rules`, `snapshot_transforms`, and `snapshot_visibility` support the following syntax:

| Pattern | Description |
|---------|-------------|
| `{name}` | Named capture: matches a single path segment and captures its value |
| `*` | Wildcard: matches any single path segment (no capture) |
| `**` | Multi-wildcard: matches all remaining segments (must be last) |
| `/literal` | Exact segment match |

**Reserved captures:**

| Capture | Description |
|---------|-------------|
| `{session}` | Replaced with the session's authenticated subject (user ID) before matching |

**Substitution order:** `{session}` is substituted first, then named captures. The auth system rejects user IDs containing `{` or `}` to prevent double-substitution injection.

**Examples:**

| Pattern | Matches | Does Not Match |
|---------|---------|----------------|
| `/app/room/{roomId}/meta` | `/app/room/r1/meta` | `/app/room/r1/admin/u1` |
| `/app/room/*/meta` | `/app/room/xyz/meta` | `/app/room/xyz/admin` |
| `/app/room/**` | `/app/room/r1/messages`, `/app/room/r1/admin/u1` | `/app/user/alice` |
| `/app/user/{userId}/profile` | `/app/user/alice/profile` | `/app/user/alice/settings` |

## Complete Example

```json
{
  "scopes": [
    "read:/chat/**",
    "write:/chat/user/{userId}/**",
    "write:/chat/room/*/messages",
    "write:/chat/room/*/presence/{userId}",
    "write:/chat/room/*/typing/{userId}"
  ],

  "write_rules": [
    {
      "path": "/chat/user/{userId}/profile",
      "checks": [
        { "type": "segment_equals_session", "segment": "userId" },
        { "type": "require_value_field", "field": "displayName" }
      ]
    },
    {
      "path": "/chat/room/{roomId}/messages",
      "checks": [
        { "type": "state_not_null", "lookup": "/chat/room/{roomId}/presence/{session}" },
        { "type": "value_field_equals_session", "field": "userId" },
        { "type": "require_value_field", "field": "content" }
      ]
    },
    {
      "path": "/chat/room/{roomId}/meta",
      "checks": [
        {
          "type": "state_field_equals_session",
          "lookup": "/chat/room/{roomId}/meta",
          "field": "createdBy",
          "allow_if_missing": true
        }
      ]
    }
  ],

  "snapshot_transforms": [
    {
      "path": "/chat/user/{id}/auth",
      "redact_fields": ["passwordHash", "passwordSalt"]
    }
  ],

  "snapshot_visibility": [
    { "path_contains": "/__auth", "visible": false },
    {
      "path": "/chat/user/{userId}/**",
      "visible": "owner",
      "owner_segment": "userId",
      "public_sub": "profile"
    },
    { "path": "/chat/**", "visible": true }
  ],

  "rate_limits": {
    "login_max_attempts": 5,
    "login_window_secs": 60,
    "register_max_attempts": 10,
    "register_window_secs": 60
  }
}
```

## Auto-Detection

If `--app-config` is not specified, the relay auto-detects the config file by searching:

1. `/etc/clasp/` -- if the directory contains exactly one `.json` file, it is used
2. `./config/` -- if the directory contains exactly one `.json` file, it is used

If a directory contains multiple `.json` files, auto-detection is skipped and `--app-config` must be specified explicitly.

## Next Steps

- [Relay CLI Reference](relay-cli.md) -- `--app-config` flag and related options
- [Wire Protocol](protocol-spec.md) -- how Set/Publish messages trigger write validation
- [Security Model](../concepts/security-model.md) -- how scopes and write rules fit into the security architecture
