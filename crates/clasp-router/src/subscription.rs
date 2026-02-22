//! Subscription management — segment-level trie for O(k) pattern matching
//!
//! Replaces the previous prefix-based DashMap approach (O(n) per prefix bucket)
//! with a trie where each node represents a path segment. Wildcards (`*`) and
//! globstars (`**`) are dedicated child branches, giving O(k) lookup where
//! k = number of address segments (typically 3-5).

use clasp_core::{address::Pattern, SignalType, SubscribeOptions};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};

use crate::SessionId;

/// A subscription entry
#[derive(Debug, Clone)]
pub struct Subscription {
    /// Subscription ID (unique per session)
    pub id: u32,
    /// Session that owns this subscription
    pub session_id: SessionId,
    /// Pattern to match
    pub pattern: Pattern,
    /// Signal types to filter (empty = all)
    pub types: HashSet<SignalType>,
    /// Subscription options
    pub options: SubscribeOptions,
}

impl Subscription {
    pub fn new(
        id: u32,
        session_id: SessionId,
        pattern: &str,
        types: Vec<SignalType>,
        options: SubscribeOptions,
    ) -> Result<Self, clasp_core::Error> {
        let pattern = Pattern::compile(pattern)?;

        Ok(Self {
            id,
            session_id,
            pattern,
            types: types.into_iter().collect(),
            options,
        })
    }

    /// Check if this subscription matches an address
    pub fn matches(&self, address: &str, signal_type: Option<SignalType>) -> bool {
        // Check address pattern
        if !self.pattern.matches(address) {
            return false;
        }

        // Check signal type filter
        if !self.types.is_empty() {
            if let Some(st) = signal_type {
                if !self.types.contains(&st) {
                    return false;
                }
            }
        }

        true
    }
}

// ---------------------------------------------------------------------------
// Trie internals
// ---------------------------------------------------------------------------

/// Subscriber entry stored in trie leaf nodes
#[derive(Debug, Clone)]
struct SubscriberEntry {
    session_id: SessionId,
    sub_id: u32,
    types: HashSet<SignalType>,
    /// When set, this entry was placed in a wildcard/globstar bucket due to a
    /// partial wildcard segment (e.g. `zone5*`). The full pattern string is
    /// stored here for glob-match verification at query time.
    verify_pattern: Option<String>,
}

/// Segment-level trie node
#[derive(Debug, Default)]
struct TrieNode {
    /// Literal segment children
    children: HashMap<String, TrieNode>,
    /// Single-segment wildcard (`*`) child
    wildcard: Option<Box<TrieNode>>,
    /// Multi-segment globstar (`**`) child
    globstar: Option<Box<TrieNode>>,
    /// Subscriptions terminating at this node
    subscribers: Vec<SubscriberEntry>,
}

impl TrieNode {
    fn is_empty(&self) -> bool {
        self.subscribers.is_empty()
            && self.children.is_empty()
            && self.wildcard.is_none()
            && self.globstar.is_none()
    }

    /// Insert a subscriber entry at the path described by `segments`.
    fn insert(&mut self, segments: &[&str], entry: SubscriberEntry) {
        if segments.is_empty() {
            self.subscribers.push(entry);
            return;
        }

        let seg = segments[0];
        let rest = &segments[1..];

        if seg == "**" {
            self.globstar
                .get_or_insert_with(|| Box::new(TrieNode::default()))
                .insert(rest, entry);
        } else if seg == "*" || seg.contains('*') {
            // Pure `*` and partial wildcards (e.g. `zone5*`) both go into the
            // wildcard branch. Partial wildcards carry a verify_pattern for
            // post-match verification.
            self.wildcard
                .get_or_insert_with(|| Box::new(TrieNode::default()))
                .insert(rest, entry);
        } else {
            self.children
                .entry(seg.to_string())
                .or_default()
                .insert(rest, entry);
        }
    }

    /// Remove a specific subscriber entry by walking the segment path.
    fn remove(&mut self, segments: &[&str], session_id: &str, sub_id: u32) -> bool {
        if segments.is_empty() {
            let before = self.subscribers.len();
            self.subscribers
                .retain(|e| !(e.session_id == session_id && e.sub_id == sub_id));
            return self.subscribers.len() < before;
        }

        let seg = segments[0];
        let rest = &segments[1..];

        if seg == "**" {
            if let Some(ref mut gs) = self.globstar {
                let removed = gs.remove(rest, session_id, sub_id);
                if gs.is_empty() {
                    self.globstar = None;
                }
                return removed;
            }
            false
        } else if seg == "*" || seg.contains('*') {
            if let Some(ref mut wc) = self.wildcard {
                let removed = wc.remove(rest, session_id, sub_id);
                if wc.is_empty() {
                    self.wildcard = None;
                }
                return removed;
            }
            false
        } else {
            let key = seg.to_string();
            if let Some(child) = self.children.get_mut(&key) {
                let removed = child.remove(rest, session_id, sub_id);
                if child.is_empty() {
                    self.children.remove(&key);
                }
                removed
            } else {
                false
            }
        }
    }

    /// Remove all entries belonging to a session (full tree walk).
    fn remove_session(&mut self, session_id: &str) {
        self.subscribers.retain(|e| e.session_id != session_id);

        for child in self.children.values_mut() {
            child.remove_session(session_id);
        }
        self.children.retain(|_, c| !c.is_empty());

        if let Some(ref mut wc) = self.wildcard {
            wc.remove_session(session_id);
            if wc.is_empty() {
                self.wildcard = None;
            }
        }

        if let Some(ref mut gs) = self.globstar {
            gs.remove_session(session_id);
            if gs.is_empty() {
                self.globstar = None;
            }
        }
    }

    /// Collect all matching subscribers for the given address segments.
    fn find_matches(
        &self,
        segments: &[&str],
        idx: usize,
        signal_type: Option<SignalType>,
        address: &str,
        results: &mut HashSet<SessionId>,
    ) {
        // --- globstar child: `**` matches zero or more segments ---
        if let Some(ref gs) = self.globstar {
            for i in idx..=segments.len() {
                if i == segments.len() {
                    // `**` consumed all remaining segments
                    collect_filtered(&gs.subscribers, signal_type, address, results);
                    // Follow nested globstars (handles `/**/**` patterns)
                    collect_zero_remaining(gs, signal_type, address, results);
                } else {
                    // Try the globstar node's literal children at position i
                    if let Some(child) = gs.children.get(segments[i]) {
                        child.find_matches(segments, i + 1, signal_type, address, results);
                    }
                    // Try the globstar node's wildcard child at position i
                    if let Some(ref wc) = gs.wildcard {
                        wc.find_matches(segments, i + 1, signal_type, address, results);
                    }
                    // Try nested globstar (handles `/**/a/**/b` patterns)
                    if let Some(ref nested_gs) = gs.globstar {
                        // Only recurse into nested globstar for positions beyond idx
                        // to avoid re-checking the same start position
                        nested_gs.find_matches(segments, i, signal_type, address, results);
                    }
                }
            }
        }

        // --- base case: consumed all segments ---
        if idx >= segments.len() {
            collect_filtered(&self.subscribers, signal_type, address, results);
            return;
        }

        let seg = segments[idx];

        // --- literal match ---
        if let Some(child) = self.children.get(seg) {
            child.find_matches(segments, idx + 1, signal_type, address, results);
        }

        // --- single-segment wildcard (`*`) match ---
        if let Some(ref wc) = self.wildcard {
            wc.find_matches(segments, idx + 1, signal_type, address, results);
        }
    }
}

/// Collect subscribers from a node and all its nested globstar children,
/// representing `**` matching zero remaining segments.
fn collect_zero_remaining(
    node: &TrieNode,
    signal_type: Option<SignalType>,
    address: &str,
    results: &mut HashSet<SessionId>,
) {
    if let Some(ref gs) = node.globstar {
        collect_filtered(&gs.subscribers, signal_type, address, results);
        collect_zero_remaining(gs, signal_type, address, results);
    }
}

/// Add matching subscribers to the result set, applying signal-type and
/// optional glob-match verification filters.
fn collect_filtered(
    subscribers: &[SubscriberEntry],
    signal_type: Option<SignalType>,
    address: &str,
    results: &mut HashSet<SessionId>,
) {
    for entry in subscribers {
        // Verify partial wildcard entries against the full address
        if let Some(ref pat) = entry.verify_pattern {
            if !clasp_core::address::glob_match(pat, address) {
                continue;
            }
        }

        // Signal-type filter: empty types set means "match all"
        if entry.types.is_empty() || signal_type.is_none_or(|st| entry.types.contains(&st)) {
            results.insert(entry.session_id.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Inner state protected by RwLock
struct TrieInner {
    root: TrieNode,
    /// Full subscription data for `remove()` return value and `len()`
    subscriptions: HashMap<(SessionId, u32), Subscription>,
}

/// Manages all subscriptions using a segment-level trie.
pub struct SubscriptionManager {
    inner: RwLock<TrieInner>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(TrieInner {
                root: TrieNode::default(),
                subscriptions: HashMap::new(),
            }),
        }
    }

    /// Add a subscription
    pub fn add(&self, sub: Subscription) {
        let pattern_segments: Vec<String> = sub.pattern.address().segments().to_vec();
        let segments: Vec<&str> = pattern_segments.iter().map(|s| s.as_str()).collect();

        // Determine if any segment is a partial wildcard (contains `*` but
        // isn't exactly `*` or `**`)
        let has_partial_wildcard = pattern_segments
            .iter()
            .any(|s| s.contains('*') && s != "*" && s != "**");

        let entry = SubscriberEntry {
            session_id: sub.session_id.clone(),
            sub_id: sub.id,
            types: sub.types.clone(),
            verify_pattern: if has_partial_wildcard {
                Some(sub.pattern.address().as_str().to_string())
            } else {
                None
            },
        };

        let key = (sub.session_id.clone(), sub.id);
        let mut inner = self.inner.write();
        inner.root.insert(&segments, entry);
        inner.subscriptions.insert(key, sub);
    }

    /// Remove a subscription
    pub fn remove(&self, session_id: &SessionId, id: u32) -> Option<Subscription> {
        let mut inner = self.inner.write();
        let key = (session_id.clone(), id);
        if let Some(sub) = inner.subscriptions.remove(&key) {
            let pattern_segments: Vec<String> = sub.pattern.address().segments().to_vec();
            let segments: Vec<&str> = pattern_segments.iter().map(|s| s.as_str()).collect();
            inner.root.remove(&segments, session_id, id);
            Some(sub)
        } else {
            None
        }
    }

    /// Remove all subscriptions for a session
    pub fn remove_session(&self, session_id: &SessionId) {
        let mut inner = self.inner.write();
        inner.subscriptions.retain(|k, _| k.0 != *session_id);
        inner.root.remove_session(session_id);
    }

    /// Find all sessions subscribed to an address
    pub fn find_subscribers(
        &self,
        address: &str,
        signal_type: Option<SignalType>,
    ) -> Vec<SessionId> {
        // Split address into segments (strip leading '/')
        let segments: Vec<&str> = if address.len() > 1 {
            address[1..].split('/').collect()
        } else {
            // address is just "/"
            vec![""]
        };

        let mut results = HashSet::new();
        let inner = self.inner.read();
        inner
            .root
            .find_matches(&segments, 0, signal_type, address, &mut results);

        results.into_iter().collect()
    }

    /// Get subscription count
    pub fn len(&self) -> usize {
        self.inner.read().subscriptions.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().subscriptions.is_empty()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_matching() {
        let sub = Subscription::new(
            1,
            "session1".to_string(),
            "/lumen/scene/*/layer/*/opacity",
            vec![],
            SubscribeOptions::default(),
        )
        .unwrap();

        assert!(sub.matches("/lumen/scene/0/layer/3/opacity", None));
        assert!(!sub.matches("/lumen/scene/0/opacity", None));
    }

    #[test]
    fn test_manager() {
        let manager = SubscriptionManager::new();

        let sub = Subscription::new(
            1,
            "session1".to_string(),
            "/test/**",
            vec![],
            SubscribeOptions::default(),
        )
        .unwrap();

        manager.add(sub);

        let subscribers = manager.find_subscribers("/test/foo/bar", None);
        assert!(subscribers.contains(&"session1".to_string()));
    }

    #[test]
    fn test_root_globstar_subscription() {
        // Test that "/**" subscriptions match all addresses
        let manager = SubscriptionManager::new();

        let sub = Subscription::new(
            1,
            "session1".to_string(),
            "/**",
            vec![],
            SubscribeOptions::default(),
        )
        .unwrap();

        manager.add(sub);

        // Should match any address
        let subscribers = manager.find_subscribers("/a/b/c", None);
        assert!(
            subscribers.contains(&"session1".to_string()),
            "/** should match /a/b/c"
        );

        let subscribers = manager.find_subscribers("/foo", None);
        assert!(
            subscribers.contains(&"session1".to_string()),
            "/** should match /foo"
        );

        let subscribers = manager.find_subscribers("/deeply/nested/path/here", None);
        assert!(
            subscribers.contains(&"session1".to_string()),
            "/** should match deeply nested paths"
        );
    }

    #[test]
    fn test_multiple_globstar_patterns() {
        // Test multiple globstar patterns coexisting
        let manager = SubscriptionManager::new();

        // Root globstar
        manager.add(
            Subscription::new(
                1,
                "global".to_string(),
                "/**",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        // Specific prefix globstar
        manager.add(
            Subscription::new(
                2,
                "lumen".to_string(),
                "/lumen/**",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        // Non-matching prefix globstar
        manager.add(
            Subscription::new(
                3,
                "other".to_string(),
                "/other/**",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        // /lumen/scene/0 should match both "global" (/**) and "lumen" (/lumen/**)
        let subscribers = manager.find_subscribers("/lumen/scene/0", None);
        assert!(subscribers.contains(&"global".to_string()));
        assert!(subscribers.contains(&"lumen".to_string()));
        assert!(!subscribers.contains(&"other".to_string()));

        // /other/data should match "global" and "other"
        let subscribers = manager.find_subscribers("/other/data", None);
        assert!(subscribers.contains(&"global".to_string()));
        assert!(subscribers.contains(&"other".to_string()));
        assert!(!subscribers.contains(&"lumen".to_string()));
    }

    #[test]
    fn test_remove_cleans_up_by_prefix() {
        let manager = SubscriptionManager::new();

        // Add a subscription
        let sub = Subscription::new(
            1,
            "session1".to_string(),
            "/test/**",
            vec![],
            SubscribeOptions::default(),
        )
        .unwrap();

        manager.add(sub);
        assert_eq!(manager.len(), 1);

        // Remove the subscription
        let removed = manager.remove(&"session1".to_string(), 1);
        assert!(removed.is_some());
        assert_eq!(manager.len(), 0);

        // by_prefix should be cleaned up (empty entries removed)
        // We can verify this indirectly by checking that a new subscription
        // to the same prefix works correctly
        let sub2 = Subscription::new(
            2,
            "session2".to_string(),
            "/test/**",
            vec![],
            SubscribeOptions::default(),
        )
        .unwrap();

        manager.add(sub2);
        let subscribers = manager.find_subscribers("/test/foo", None);
        assert_eq!(subscribers.len(), 1);
        assert!(subscribers.contains(&"session2".to_string()));
    }

    #[test]
    fn test_remove_session_cleans_up_by_prefix() {
        let manager = SubscriptionManager::new();

        // Add multiple subscriptions for one session
        manager.add(
            Subscription::new(
                1,
                "session1".to_string(),
                "/test/**",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );
        manager.add(
            Subscription::new(
                2,
                "session1".to_string(),
                "/other/**",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        // Add subscription for different session
        manager.add(
            Subscription::new(
                1,
                "session2".to_string(),
                "/test/**",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        assert_eq!(manager.len(), 3);

        // Remove all subscriptions for session1
        manager.remove_session(&"session1".to_string());
        assert_eq!(manager.len(), 1);

        // Session2 should still get messages
        let subscribers = manager.find_subscribers("/test/foo", None);
        assert_eq!(subscribers.len(), 1);
        assert!(subscribers.contains(&"session2".to_string()));

        // /other/** should have no subscribers
        let subscribers = manager.find_subscribers("/other/foo", None);
        assert_eq!(subscribers.len(), 0);
    }

    // --- Additional trie-specific tests ---

    #[test]
    fn test_exact_address_match() {
        let manager = SubscriptionManager::new();
        manager.add(
            Subscription::new(
                1,
                "s1".to_string(),
                "/chat/room/abc/messages",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        assert_eq!(
            manager
                .find_subscribers("/chat/room/abc/messages", None)
                .len(),
            1
        );
        assert_eq!(
            manager
                .find_subscribers("/chat/room/xyz/messages", None)
                .len(),
            0
        );
        assert_eq!(manager.find_subscribers("/chat/room/abc", None).len(), 0);
    }

    #[test]
    fn test_single_wildcard() {
        let manager = SubscriptionManager::new();
        manager.add(
            Subscription::new(
                1,
                "s1".to_string(),
                "/chat/room/*/messages",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        assert_eq!(
            manager
                .find_subscribers("/chat/room/abc/messages", None)
                .len(),
            1
        );
        assert_eq!(
            manager
                .find_subscribers("/chat/room/xyz/messages", None)
                .len(),
            1
        );
        // * should not match multiple segments
        assert_eq!(
            manager
                .find_subscribers("/chat/room/a/b/messages", None)
                .len(),
            0
        );
    }

    #[test]
    fn test_globstar_matches_zero_segments() {
        let manager = SubscriptionManager::new();
        manager.add(
            Subscription::new(
                1,
                "s1".to_string(),
                "/chat/**",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        // ** matches zero segments (just /chat)
        assert_eq!(manager.find_subscribers("/chat", None).len(), 1);
        // ** matches one segment
        assert_eq!(manager.find_subscribers("/chat/room", None).len(), 1);
        // ** matches many segments
        assert_eq!(
            manager
                .find_subscribers("/chat/room/abc/messages", None)
                .len(),
            1
        );
    }

    #[test]
    fn test_signal_type_filtering() {
        let manager = SubscriptionManager::new();
        manager.add(
            Subscription::new(
                1,
                "s1".to_string(),
                "/data/**",
                vec![SignalType::Param],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );
        manager.add(
            Subscription::new(
                1,
                "s2".to_string(),
                "/data/**",
                vec![SignalType::Event],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );
        manager.add(
            Subscription::new(
                1,
                "s3".to_string(),
                "/data/**",
                vec![], // matches all types
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        let param_subs = manager.find_subscribers("/data/x", Some(SignalType::Param));
        assert!(param_subs.contains(&"s1".to_string()));
        assert!(!param_subs.contains(&"s2".to_string()));
        assert!(param_subs.contains(&"s3".to_string()));

        let event_subs = manager.find_subscribers("/data/x", Some(SignalType::Event));
        assert!(!event_subs.contains(&"s1".to_string()));
        assert!(event_subs.contains(&"s2".to_string()));
        assert!(event_subs.contains(&"s3".to_string()));

        // None signal_type matches all
        let all_subs = manager.find_subscribers("/data/x", None);
        assert_eq!(all_subs.len(), 3);
    }

    #[test]
    fn test_multiple_wildcards_in_pattern() {
        let manager = SubscriptionManager::new();
        manager.add(
            Subscription::new(
                1,
                "s1".to_string(),
                "/scene/*/layer/*/opacity",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        assert_eq!(
            manager
                .find_subscribers("/scene/0/layer/3/opacity", None)
                .len(),
            1
        );
        assert_eq!(
            manager
                .find_subscribers("/scene/main/layer/bg/opacity", None)
                .len(),
            1
        );
        assert_eq!(
            manager
                .find_subscribers("/scene/0/layer/3/color", None)
                .len(),
            0
        );
    }

    #[test]
    fn test_overlapping_patterns() {
        let manager = SubscriptionManager::new();

        // Exact match
        manager.add(
            Subscription::new(
                1,
                "exact".to_string(),
                "/chat/room/abc/messages",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        // Wildcard match
        manager.add(
            Subscription::new(
                1,
                "wild".to_string(),
                "/chat/room/*/messages",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        // Globstar match
        manager.add(
            Subscription::new(
                1,
                "glob".to_string(),
                "/chat/**",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        // Root globstar
        manager.add(
            Subscription::new(
                1,
                "root".to_string(),
                "/**",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        let subs = manager.find_subscribers("/chat/room/abc/messages", None);
        assert_eq!(subs.len(), 4, "All four patterns should match");
        assert!(subs.contains(&"exact".to_string()));
        assert!(subs.contains(&"wild".to_string()));
        assert!(subs.contains(&"glob".to_string()));
        assert!(subs.contains(&"root".to_string()));
    }

    #[test]
    fn test_trie_prunes_empty_nodes() {
        let manager = SubscriptionManager::new();

        manager.add(
            Subscription::new(
                1,
                "s1".to_string(),
                "/a/b/c",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );
        manager.add(
            Subscription::new(
                2,
                "s1".to_string(),
                "/a/b/d",
                vec![],
                SubscribeOptions::default(),
            )
            .unwrap(),
        );

        manager.remove(&"s1".to_string(), 1);
        assert_eq!(manager.len(), 1);

        // /a/b/d should still work
        assert_eq!(manager.find_subscribers("/a/b/d", None).len(), 1);
        // /a/b/c should no longer match
        assert_eq!(manager.find_subscribers("/a/b/c", None).len(), 0);

        manager.remove(&"s1".to_string(), 2);
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }
}
