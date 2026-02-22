---
title: App Config
description: Declarative scopes, write rules, and visibility without writing code
order: 1
---

# App Config

App config lets you define application-specific auth scopes, write validation, snapshot transforms, and visibility rules in a single JSON file -- no Rust code required. Instead of writing custom server logic, you describe your authorization and data-access policies declaratively, and the relay enforces them at runtime.

## Enable

Load an app config by passing the file path to the relay:

```bash
clasp-relay --app-config config/myapp.json
```

If `--app-config` is not specified, the relay attempts auto-detection. It checks `/etc/clasp/*.json` and then `./config/*.json`. If exactly one JSON file is found across those locations, it is loaded automatically. If multiple files are found, none is loaded and you must specify the path explicitly.

## Structure

An app config file has four top-level sections:

```json
{
  "scopes": [],
  "write_rules": [],
  "snapshot_transforms": [],
  "snapshot_visibility": [],
  "rate_limits": {}
}
```

| Section | Purpose |
|---------|---------|
| `scopes` | Auth scope templates expanded per user on login |
| `write_rules` | Server-side write validation checks |
| `snapshot_transforms` | Redact fields from state before delivery |
| `snapshot_visibility` | Control which state paths clients can see |
| `rate_limits` | Throttle login and registration attempts |

All sections are optional. You can start with just scopes and add rules as your application grows.

## Scope Templates

The `scopes` array contains scope template strings in `action:pattern` format. When a user registers or logs in, every template is expanded by replacing `{userId}` with the authenticated user's ID. The expanded scopes are attached to the user's session token.

```json
{
  "scopes": [
    "read:/app/**",
    "write:/app/user/{userId}/**",
    "read:/app/user/{userId}/**",
    "write:/app/room/*/members/{userId}",
    "emit:/app/events/{userId}/**"
  ]
}
```

When user `alice` logs in, these expand to:

```
read:/app/**
write:/app/user/alice/**
read:/app/user/alice/**
write:/app/room/*/members/alice
emit:/app/events/alice/**
```

The `read` action controls subscriptions and snapshot access. The `write` action controls param sets. The `emit` action controls event publishing. Wildcards `*` (single segment) and `**` (multi-segment) work in patterns.

## Write Rules

Write rules add server-side validation beyond scope checks. Each rule targets a path pattern and applies one or more checks before allowing the write.

```json
{
  "write_rules": [
    {
      "path": "/app/room/{roomId}/messages/{msgId}",
      "checks": [
        {"check": "value_field_equals_session", "field": "fromId"},
        {"check": "require_value_field", "field": "content"}
      ],
      "pre_checks": [
        {"check": "state_not_null", "lookup": "/app/room/{roomId}/meta"}
      ],
      "mode": "all",
      "allow_null_write": false
    }
  ]
}
```

### Path Matching

The `path` field is a pattern with named captures in braces. When a write arrives at `/app/room/general/messages/msg-42`, the captures `{roomId}` = `general` and `{msgId}` = `msg-42` are extracted and available to checks.

### Mode

- `"all"` (default) -- every check in `checks` must pass.
- `"any"` -- at least one check must pass.

`pre_checks` always run regardless of mode. If any pre-check fails, the write is rejected before evaluating `checks`.

### allow_null_write

When `allow_null_write` is `true`, writes with a null value (deletions) skip the main `checks` array. Pre-checks still apply. This is useful when you want to let users delete their own data without satisfying all creation-time validations.

### Check Types

There are 7 check types. Each can optionally include `"allow_if_missing": true` to pass when the referenced state does not exist (useful for first-write scenarios).

#### state_field_equals_session

Looks up existing state at a path and verifies that a specific field equals the session's userId. Use this to ensure only the creator of a resource can modify it.

```json
{
  "check": "state_field_equals_session",
  "lookup": "/app/room/{roomId}/meta",
  "field": "creatorId"
}
```

The relay reads the current state at `/app/room/{roomId}/meta`, extracts the `creatorId` field, and compares it to the writing session's userId.

#### state_not_null

Verifies that state exists at a lookup path. Use this to enforce that a prerequisite resource has been created.

```json
{
  "check": "state_not_null",
  "lookup": "/app/room/{roomId}/presence/{session}"
}
```

The `{session}` placeholder is replaced with the writing session's userId. This check ensures the user has a presence entry in the room before allowing the write.

#### value_field_equals_session

Checks that a field in the value being written matches the session's userId. Use this to prevent users from impersonating others.

```json
{
  "check": "value_field_equals_session",
  "field": "fromId"
}
```

If the written value is `{"fromId": "alice", "content": "hello"}` and the session belongs to `alice`, the check passes. If `bob` tries to write with `"fromId": "alice"`, it fails.

#### segment_equals_session

Checks that a named path segment from the write address matches the session's userId.

```json
{
  "check": "segment_equals_session",
  "segment": "userId"
}
```

For a write to `/app/user/{userId}/profile`, this verifies that the `{userId}` segment equals the writing session's userId. Users can only write to their own path.

#### either_state_not_null

Passes if state exists at either of two lookup paths. Use this for bidirectional relationships like friend connections.

```json
{
  "check": "either_state_not_null",
  "lookup_a": "/app/friends/{session}/{targetId}",
  "lookup_b": "/app/friends/{targetId}/{session}"
}
```

The write is allowed if a friendship record exists in either direction. `{session}` is replaced with the writing session's userId.

#### require_value_field

Ensures the written value contains a specific field. Use this for schema enforcement.

```json
{
  "check": "require_value_field",
  "field": "content"
}
```

The write is rejected if the value does not include a `content` field.

#### reject_unless_path_matches

Rejects the write unless the address matches an additional sub-pattern. Use this to constrain writes within an already-matched rule.

```json
{
  "check": "reject_unless_path_matches",
  "pattern": "/app/requests/{targetId}/{fromId}"
}
```

## Snapshot Transforms

Snapshot transforms redact sensitive fields from state before it is delivered to clients. The relay applies transforms during snapshot delivery and subscription updates.

```json
{
  "snapshot_transforms": [
    {
      "path": "/app/user/*/account",
      "redact_fields": ["passwordHash", "email", "authToken"]
    },
    {
      "path": "/app/room/*/meta",
      "redact_fields": ["inviteSecret"]
    }
  ]
}
```

Transforms match by path pattern. When state at a matching path is sent to any client, the listed fields are removed from the value. The stored state is unaffected -- redaction happens only on delivery.

## Snapshot Visibility

Visibility rules control which state paths a client can see in snapshots. Rules are evaluated in order; the first match wins. If no rule matches, the default is visible.

Three visibility modes are available:

### Boolean

Allow or deny visibility unconditionally.

```json
{
  "path_contains": "/internal/",
  "visible": false
}
```

Any state path containing `/internal/` is hidden from all clients.

```json
{
  "path_contains": "/public/",
  "visible": true
}
```

### Owner

Only the owning user can see the state, with an optional public sub-path.

```json
{
  "path": "/app/user/{userId}/**",
  "visible": "owner",
  "owner_segment": "userId",
  "public_sub": "profile"
}
```

State under `/app/user/alice/` is visible only to alice -- except paths under `/app/user/alice/profile/`, which are visible to everyone. The `owner_segment` names which path capture identifies the owner.

### require_state_not_null

State is visible only if a related piece of state exists.

```json
{
  "path": "/app/room/{roomId}/**",
  "visible": "require_state_not_null",
  "lookup": "/app/room/{roomId}/presence/{session}"
}
```

Room state is visible only to clients who have a presence entry in that room. `{session}` is replaced with the requesting session's userId.

## Rate Limits

Throttle authentication endpoints to prevent brute-force attacks.

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

| Field | Default | Description |
|-------|---------|-------------|
| `login_max_attempts` | 5 | Max login attempts per IP per window |
| `login_window_secs` | 60 | Window duration in seconds |
| `register_max_attempts` | 10 | Max registration attempts per IP per window |
| `register_window_secs` | 60 | Window duration in seconds |

When the limit is exceeded, the endpoint returns HTTP 429.

## Complete Example

A minimal app config for a chat application:

```json
{
  "scopes": [
    "read:/chat/**",
    "write:/chat/user/{userId}/**",
    "write:/chat/room/*/messages/*",
    "emit:/chat/room/*/typing"
  ],
  "write_rules": [
    {
      "path": "/chat/room/{roomId}/messages/{msgId}",
      "checks": [
        {"check": "value_field_equals_session", "field": "fromId"},
        {"check": "require_value_field", "field": "content"}
      ],
      "pre_checks": [
        {"check": "state_not_null", "lookup": "/chat/room/{roomId}/meta"}
      ]
    },
    {
      "path": "/chat/user/{userId}/profile",
      "checks": [
        {"check": "segment_equals_session", "segment": "userId"}
      ]
    }
  ],
  "snapshot_transforms": [
    {
      "path": "/chat/user/*/account",
      "redact_fields": ["passwordHash"]
    }
  ],
  "snapshot_visibility": [
    {
      "path": "/chat/user/{userId}/**",
      "visible": "owner",
      "owner_segment": "userId",
      "public_sub": "profile"
    },
    {
      "path": "/chat/room/{roomId}/**",
      "visible": "require_state_not_null",
      "lookup": "/chat/room/{roomId}/presence/{session}"
    }
  ],
  "rate_limits": {
    "login_max_attempts": 5,
    "login_window_secs": 60,
    "register_max_attempts": 10,
    "register_window_secs": 60
  }
}
```

## Next Steps

- [Rules Engine](./rules.md) -- server-side reactive automation
- [CPSK Tokens](../auth/cpsk.md) -- authentication setup
- [Relay Server](../deployment/relay.md) -- relay configuration reference
