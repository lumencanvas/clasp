//! Declarative application configuration for the CLASP relay.
//!
//! A single JSON file (`--app-config <path>`) defines scopes, write rules, and
//! snapshot rules for any application. No plugins, no separate crates, no Rust
//! compilation to customize behavior.
//!
//! The protocol layer (clasp-core, clasp-router) stays app-agnostic. This module
//! provides a generic rule engine that replaces hardcoded chat validators.

use clasp_core::Value;
use clasp_router::{RouterState, Session, SnapshotFilter, WriteValidator};
use serde::Deserialize;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Config types
// ---------------------------------------------------------------------------

/// Top-level application config loaded from `--app-config`.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Scope templates with `{userId}` placeholder, e.g. `"read:/chat/user/{userId}/**"`.
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Write validation rules (first-match).
    #[serde(default)]
    pub write_rules: Vec<WriteRule>,

    /// Snapshot field transforms (all matching, not first-match).
    #[serde(default)]
    pub snapshot_transforms: Vec<SnapshotTransform>,

    /// Snapshot visibility rules (first-match).
    #[serde(default)]
    pub snapshot_visibility: Vec<VisibilityRule>,

    /// Rate limit overrides.
    #[serde(default)]
    pub rate_limits: Option<RateLimitConfig>,
}

/// A write validation rule.
#[derive(Debug, Clone, Deserialize)]
pub struct WriteRule {
    /// Path pattern with `{named}` captures, e.g. `/chat/room/{roomId}/meta`.
    pub path: String,

    /// If true, null writes skip `checks` (but `pre_checks` still run).
    #[serde(default)]
    pub allow_null_write: bool,

    /// `"all"` (default): every check must pass. `"any"`: at least one check must pass.
    #[serde(default = "default_check_mode")]
    pub mode: CheckMode,

    /// Checks that always run, even for null writes when `allow_null_write` is true.
    /// Uses `mode: "all"` unconditionally.
    #[serde(default)]
    pub pre_checks: Vec<WriteCheck>,

    /// The main checks to evaluate. Skipped for null writes when `allow_null_write` is true.
    pub checks: Vec<WriteCheck>,
}

/// How multiple checks within a rule combine.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CheckMode {
    All,
    Any,
}

fn default_check_mode() -> CheckMode {
    CheckMode::All
}

/// Individual check within a write rule.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum WriteCheck {
    /// Look up `lookup` in state, extract `field` from the Map, compare to session subject.
    #[serde(rename = "state_field_equals_session")]
    StateFieldEqualsSession {
        lookup: String,
        field: String,
        #[serde(default)]
        allow_if_missing: bool,
    },

    /// Look up `lookup` in state, pass if the value exists and is not null.
    #[serde(rename = "state_not_null")]
    StateNotNull { lookup: String },

    /// Extract `field` from the written value, compare to session subject.
    #[serde(rename = "value_field_equals_session")]
    ValueFieldEqualsSession { field: String },

    /// A named segment from the path pattern must equal the session subject.
    #[serde(rename = "segment_equals_session")]
    SegmentEqualsSession { segment: String },

    /// Pass if either `lookup_a` or `lookup_b` exists and is not null in state.
    #[serde(rename = "either_state_not_null")]
    EitherStateNotNull { lookup_a: String, lookup_b: String },

    /// Extract `field` from the written value, require it exists (non-null string).
    #[serde(rename = "require_value_field")]
    RequireValueField { field: String },

    /// Reject writes to addresses matching the rule path that DON'T match a
    /// more specific pattern. Used to reject malformed paths.
    #[serde(rename = "reject_unless_path_matches")]
    RejectUnlessPathMatches { pattern: String, message: String },
}

/// Snapshot field transform (redaction).
#[derive(Debug, Clone, Deserialize)]
pub struct SnapshotTransform {
    /// Path pattern to match.
    pub path: String,

    /// Fields to remove from Map values.
    #[serde(default)]
    pub redact_fields: Vec<String>,
}

/// Snapshot visibility rule (first-match).
#[derive(Debug, Clone, Deserialize)]
pub struct VisibilityRule {
    /// If set, match only addresses containing this substring.
    #[serde(default)]
    pub path_contains: Option<String>,

    /// If set, match only addresses matching this path pattern.
    #[serde(default)]
    pub path: Option<String>,

    /// Visibility: `true`, `false`, `"owner"`, or `"require_state_not_null"`.
    pub visible: VisibilityMode,

    /// For `"owner"` mode: which path segment must equal the session subject.
    #[serde(default)]
    pub owner_segment: Option<String>,

    /// For `"require_state_not_null"`: lookup template.
    #[serde(default)]
    pub lookup: Option<String>,

    /// For `"owner"` mode: sub-path that is publicly visible even for non-owners.
    #[serde(default)]
    pub public_sub: Option<String>,
}

/// How visibility is determined.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum VisibilityMode {
    /// Static: always visible or always hidden.
    Bool(bool),
    /// Dynamic: `"owner"` or `"require_state_not_null"`.
    Dynamic(String),
}

/// Rate limit configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_login_max")]
    pub login_max_attempts: u32,
    #[serde(default = "default_login_window")]
    pub login_window_secs: u64,
    #[serde(default = "default_register_max")]
    pub register_max_attempts: u32,
    #[serde(default = "default_register_window")]
    pub register_window_secs: u64,
}

fn default_login_max() -> u32 { 5 }
fn default_login_window() -> u64 { 60 }
fn default_register_max() -> u32 { 10 }
fn default_register_window() -> u64 { 60 }

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            login_max_attempts: default_login_max(),
            login_window_secs: default_login_window(),
            register_max_attempts: default_register_max(),
            register_window_secs: default_register_window(),
        }
    }
}

// ---------------------------------------------------------------------------
// Path pattern matching
// ---------------------------------------------------------------------------

/// Match a pattern like `/chat/room/{roomId}/admin/{targetId}` against an address.
///
/// Returns named captures if the pattern matches.
///
/// - `{name}` captures a single segment
/// - `*` matches any single segment (no capture)
/// - `**` matches everything remaining (must be last; if mid-pattern, remaining
///   segments are silently ignored)
/// - `{session}` is reserved — it's substituted before matching
pub fn match_address<'a>(pattern: &'a str, address: &'a str) -> Option<HashMap<&'a str, &'a str>> {
    let pat_segments: Vec<&str> = pattern.split('/').collect();
    let addr_segments: Vec<&str> = address.split('/').collect();

    let mut captures: HashMap<&str, &str> = HashMap::new();
    let mut pi = 0;
    let mut ai = 0;

    while pi < pat_segments.len() && ai < addr_segments.len() {
        let ps = pat_segments[pi];

        if ps == "**" {
            // ** matches everything remaining — collect the rest
            // Join remaining address segments
            // We need to return the rest as a single capture if needed
            return Some(captures);
        } else if ps.starts_with('{') && ps.ends_with('}') {
            let name = &ps[1..ps.len() - 1];
            captures.insert(name, addr_segments[ai]);
            pi += 1;
            ai += 1;
        } else if ps == "*" {
            // Single wildcard, matches any single segment (no capture)
            pi += 1;
            ai += 1;
        } else if ps == addr_segments[ai] {
            pi += 1;
            ai += 1;
        } else {
            return None;
        }
    }

    // Handle trailing ** in pattern
    if pi < pat_segments.len() && pat_segments[pi] == "**" {
        return Some(captures);
    }

    // Both must be exhausted for an exact match
    if pi == pat_segments.len() && ai == addr_segments.len() {
        Some(captures)
    } else {
        None
    }
}

/// Substitute captures and `{session}` into a template string.
///
/// Safety: `{session}` is replaced first, then named captures. This is safe because
/// `is_valid_user_id()` in auth.rs rejects user IDs containing `{` or `}`, preventing
/// double-substitution injection (e.g. a session_id of `{roomId}` expanding to a capture).
fn substitute(template: &str, captures: &HashMap<&str, &str>, session_id: &str) -> String {
    let mut result = template.to_string();
    result = result.replace("{session}", session_id);
    for (name, value) in captures {
        result = result.replace(&format!("{{{}}}", name), value);
    }
    result
}

// ---------------------------------------------------------------------------
// Rule-based write validator
// ---------------------------------------------------------------------------

/// Generic rule-based write validator. Implements `WriteValidator`.
pub struct RuleWriteValidator {
    rules: Vec<WriteRule>,
}

impl RuleWriteValidator {
    pub fn new(rules: Vec<WriteRule>) -> Self {
        Self { rules }
    }
}

impl WriteValidator for RuleWriteValidator {
    fn validate_write(
        &self,
        address: &str,
        value: &Value,
        session: &Session,
        state: &RouterState,
    ) -> Result<(), String> {
        let session_id = match session.subject.as_deref() {
            Some(id) => id,
            None => return Ok(()), // unauthenticated: let scope check handle it
        };

        for rule in &self.rules {
            if let Some(captures) = match_address(&rule.path, address) {
                // Pre-checks always run (even for null writes)
                if !rule.pre_checks.is_empty() {
                    evaluate_checks(
                        &rule.pre_checks,
                        CheckMode::All,
                        &captures,
                        session_id,
                        value,
                        state,
                        address,
                    )?;
                }
                // Skip main checks for null writes when allowed
                if rule.allow_null_write && matches!(value, Value::Null) {
                    return Ok(());
                }
                return evaluate_checks(
                    &rule.checks,
                    rule.mode,
                    &captures,
                    session_id,
                    value,
                    state,
                    address,
                );
            }
        }

        // No rule matched = pass through
        Ok(())
    }
}

/// Evaluate checks within a matched rule.
fn evaluate_checks(
    checks: &[WriteCheck],
    mode: CheckMode,
    captures: &HashMap<&str, &str>,
    session_id: &str,
    value: &Value,
    state: &RouterState,
    address: &str,
) -> Result<(), String> {
    if checks.is_empty() {
        return Ok(());
    }

    let mut results: Vec<Result<(), String>> = Vec::with_capacity(checks.len());

    for check in checks {
        let result = evaluate_single_check(check, captures, session_id, value, state, address);
        results.push(result);
    }

    match mode {
        CheckMode::All => {
            // All must pass
            for r in results {
                r?;
            }
            Ok(())
        }
        CheckMode::Any => {
            // At least one must pass
            if results.iter().any(|r| r.is_ok()) {
                Ok(())
            } else {
                // Return the first error
                results.into_iter().find(|r| r.is_err()).unwrap()
            }
        }
    }
}

/// Evaluate a single check.
fn evaluate_single_check(
    check: &WriteCheck,
    captures: &HashMap<&str, &str>,
    session_id: &str,
    value: &Value,
    state: &RouterState,
    address: &str,
) -> Result<(), String> {
    match check {
        WriteCheck::StateFieldEqualsSession {
            lookup,
            field,
            allow_if_missing,
        } => {
            let addr = substitute(lookup, captures, session_id);
            match state.get(&addr) {
                Some(val) => {
                    let field_val = match &val {
                        Value::Map(map) => map.get(field.as_str()).and_then(|v| v.as_str()),
                        _ => None,
                    };
                    if field_val == Some(session_id) {
                        Ok(())
                    } else {
                        Err(format!(
                            "Field '{}' at '{}' does not match session identity",
                            field, addr
                        ))
                    }
                }
                None => {
                    if *allow_if_missing {
                        Ok(()) // initial creation
                    } else {
                        Err(format!("State not found at '{}'", addr))
                    }
                }
            }
        }

        WriteCheck::StateNotNull { lookup } => {
            let addr = substitute(lookup, captures, session_id);
            match state.get(&addr) {
                Some(Value::Null) | None => {
                    Err(format!("State is null or missing at '{}'", addr))
                }
                Some(_) => Ok(()),
            }
        }

        WriteCheck::ValueFieldEqualsSession { field } => {
            let field_val = match value {
                Value::Map(map) => map.get(field.as_str()).and_then(|v| v.as_str()),
                _ => None,
            };
            match field_val {
                Some(v) if v == session_id => Ok(()),
                Some(v) => Err(format!(
                    "Value field '{}' is '{}', does not match session identity '{}'",
                    field, v, session_id
                )),
                None => Err(format!("Value must include a {} field", field)),
            }
        }

        WriteCheck::SegmentEqualsSession { segment } => {
            match captures.get(segment.as_str()) {
                Some(val) if *val == session_id => Ok(()),
                Some(val) => Err(format!(
                    "Path segment '{}' is '{}', does not match session identity '{}'",
                    segment, val, session_id
                )),
                None => Err(format!("Path segment '{}' not found in pattern", segment)),
            }
        }

        WriteCheck::EitherStateNotNull { lookup_a, lookup_b } => {
            let addr_a = substitute(lookup_a, captures, session_id);
            let addr_b = substitute(lookup_b, captures, session_id);
            let a_ok = matches!(state.get(&addr_a), Some(v) if !matches!(v, Value::Null));
            let b_ok = matches!(state.get(&addr_b), Some(v) if !matches!(v, Value::Null));
            if a_ok || b_ok {
                Ok(())
            } else {
                Err(format!(
                    "Neither '{}' nor '{}' exists in state",
                    addr_a, addr_b
                ))
            }
        }

        WriteCheck::RequireValueField { field } => {
            let field_val = match value {
                Value::Map(map) => map.get(field.as_str()).and_then(|v| v.as_str()),
                _ => None,
            };
            match field_val {
                Some(_) => Ok(()),
                None => Err(format!("Value must include a {} field", field)),
            }
        }

        WriteCheck::RejectUnlessPathMatches { pattern, message } => {
            if match_address(pattern, address).is_some() {
                Ok(())
            } else {
                Err(message.clone())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Rule-based snapshot filter
// ---------------------------------------------------------------------------

/// Generic rule-based snapshot filter. Implements `SnapshotFilter`.
pub struct RuleSnapshotFilter {
    transforms: Vec<SnapshotTransform>,
    visibility: Vec<VisibilityRule>,
}

impl RuleSnapshotFilter {
    pub fn new(transforms: Vec<SnapshotTransform>, visibility: Vec<VisibilityRule>) -> Self {
        Self {
            transforms,
            visibility,
        }
    }

    /// Apply matching transforms (redact fields) to a ParamValue.
    fn apply_transforms(&self, mut pv: clasp_core::ParamValue) -> clasp_core::ParamValue {
        for transform in &self.transforms {
            if match_address(&transform.path, &pv.address).is_some() {
                if let Value::Map(ref mut map) = pv.value {
                    for field in &transform.redact_fields {
                        map.remove(field.as_str());
                    }
                }
            }
        }
        pv
    }

    /// Check if a path is visible to the session (first-match rule wins).
    fn is_visible(&self, address: &str, session_id: &str, state: &RouterState) -> bool {
        for rule in &self.visibility {
            // Check if this rule matches the address
            if let Some(ref contains) = rule.path_contains {
                if !address.contains(contains.as_str()) {
                    continue;
                }
            } else if let Some(ref pattern) = rule.path {
                if match_address(pattern, address).is_none() {
                    continue;
                }
            }
            // else: no path/path_contains = catch-all, matches everything

            // This rule matched — evaluate visibility
            return match &rule.visible {
                VisibilityMode::Bool(b) => *b,
                VisibilityMode::Dynamic(mode) => match mode.as_str() {
                    "owner" => {
                        // Extract the owner segment from the address
                        if let Some(ref segment_name) = rule.owner_segment {
                            if let Some(ref pattern) = rule.path {
                                if let Some(captures) = match_address(pattern, address) {
                                    let owner = captures.get(segment_name.as_str()).copied().unwrap_or("");
                                    if owner == session_id {
                                        return true;
                                    }
                                    // Check if sub-path is public
                                    if let Some(ref public_sub) = rule.public_sub {
                                        // Use pattern structure to find the sub-path after
                                        // the owner segment (avoids fragile string searching)
                                        let pat_segments: Vec<&str> = pattern.split('/').collect();
                                        let addr_segments: Vec<&str> = address.split('/').collect();
                                        if let Some(owner_idx) = pat_segments.iter().position(|s| {
                                            s.starts_with('{')
                                                && s.ends_with('}')
                                                && &s[1..s.len() - 1] == segment_name.as_str()
                                        }) {
                                            if owner_idx + 1 < addr_segments.len() {
                                                let sub_path =
                                                    addr_segments[owner_idx + 1..].join("/");
                                                if sub_path == *public_sub
                                                    || sub_path
                                                        .starts_with(&format!("{}/", public_sub))
                                                {
                                                    return true;
                                                }
                                            }
                                        }
                                    }
                                    false
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    "require_state_not_null" => {
                        if let Some(ref lookup) = rule.lookup {
                            if let Some(ref pattern) = rule.path {
                                if let Some(captures) = match_address(pattern, address) {
                                    let addr = substitute(lookup, &captures, session_id);
                                    matches!(state.get(&addr), Some(v) if !matches!(v, Value::Null))
                                } else {
                                    false
                                }
                            } else {
                                // No path pattern, just substitute session
                                let addr = substitute(lookup, &HashMap::new(), session_id);
                                matches!(state.get(&addr), Some(v) if !matches!(v, Value::Null))
                            }
                        } else {
                            false
                        }
                    }
                    _ => {
                        tracing::warn!("Unknown visibility mode: '{}', defaulting to hidden", mode);
                        false
                    }
                },
            };
        }

        // No rule matched = visible
        true
    }
}

impl SnapshotFilter for RuleSnapshotFilter {
    fn filter_snapshot(
        &self,
        params: Vec<clasp_core::ParamValue>,
        session: &Session,
        state: &RouterState,
    ) -> Vec<clasp_core::ParamValue> {
        let session_id = session.subject.as_deref().unwrap_or("");

        params
            .into_iter()
            .filter_map(|pv| {
                // Check visibility first (first-match)
                if !self.is_visible(&pv.address, session_id, state) {
                    return None;
                }
                // Apply transforms (all matching)
                Some(self.apply_transforms(pv))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_exact() {
        let captures = match_address("/chat/room/abc/meta", "/chat/room/abc/meta");
        assert!(captures.is_some());
        assert!(captures.unwrap().is_empty());
    }

    #[test]
    fn test_match_named_capture() {
        let captures = match_address("/chat/room/{roomId}/meta", "/chat/room/abc123/meta");
        assert!(captures.is_some());
        let c = captures.unwrap();
        assert_eq!(c.get("roomId"), Some(&"abc123"));
    }

    #[test]
    fn test_match_multiple_captures() {
        let captures = match_address(
            "/chat/room/{roomId}/admin/{targetId}",
            "/chat/room/r1/admin/user1",
        );
        assert!(captures.is_some());
        let c = captures.unwrap();
        assert_eq!(c.get("roomId"), Some(&"r1"));
        assert_eq!(c.get("targetId"), Some(&"user1"));
    }

    #[test]
    fn test_match_wildcard_star() {
        assert!(match_address("/chat/room/*/meta", "/chat/room/abc/meta").is_some());
        assert!(match_address("/chat/room/*/meta", "/chat/room/xyz/meta").is_some());
    }

    #[test]
    fn test_match_double_star() {
        assert!(match_address("/chat/room/**", "/chat/room/r1/messages").is_some());
        assert!(match_address("/chat/room/**", "/chat/room/r1/admin/u1").is_some());
        assert!(match_address("/chat/**", "/chat/user/alice/profile").is_some());
    }

    #[test]
    fn test_no_match() {
        assert!(match_address("/chat/room/{roomId}/meta", "/chat/user/alice/profile").is_none());
        assert!(match_address("/chat/room/{roomId}/meta", "/chat/room/abc/admin/u1").is_none());
    }

    #[test]
    fn test_substitute() {
        let mut captures = HashMap::new();
        captures.insert("roomId", "r1");
        captures.insert("targetId", "bob");
        let result = substitute("/chat/room/{roomId}/admin/{session}", &captures, "alice");
        assert_eq!(result, "/chat/room/r1/admin/alice");
    }

    #[test]
    fn test_substitute_either() {
        let mut captures = HashMap::new();
        captures.insert("targetId", "bob");
        let result = substitute(
            "/chat/user/{session}/friends/{targetId}",
            &captures,
            "alice",
        );
        assert_eq!(result, "/chat/user/alice/friends/bob");
    }
}
