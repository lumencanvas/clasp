//! Namespace management for federation
//!
//! Tracks which peer routers own which address patterns,
//! enabling intelligent message forwarding and loop prevention.

use std::collections::HashMap;

/// Manages namespace ownership across federated peers.
///
/// Each peer declares the address patterns it owns. When a message arrives,
/// the namespace manager determines which peers should receive it.
#[derive(Debug)]
pub struct NamespaceManager {
    /// Map of peer router ID -> owned patterns
    peer_namespaces: HashMap<String, Vec<String>>,
    /// Local router's owned patterns
    local_namespaces: Vec<String>,
}

impl NamespaceManager {
    /// Create a new namespace manager with local patterns
    pub fn new(local_namespaces: Vec<String>) -> Self {
        Self {
            peer_namespaces: HashMap::new(),
            local_namespaces,
        }
    }

    /// Register a peer's namespace patterns
    pub fn register_peer(&mut self, router_id: &str, patterns: Vec<String>) {
        self.peer_namespaces.insert(router_id.to_string(), patterns);
    }

    /// Remove a peer's namespace registrations
    pub fn remove_peer(&mut self, router_id: &str) {
        self.peer_namespaces.remove(router_id);
    }

    /// Get all peers that should receive a message for the given address.
    ///
    /// Returns peer router IDs whose namespace patterns match the address.
    /// Excludes the origin peer to prevent loops.
    pub fn peers_for_address(&self, address: &str, exclude_origin: Option<&str>) -> Vec<String> {
        self.peer_namespaces
            .iter()
            .filter(|(router_id, patterns)| {
                // Exclude origin to prevent loops
                if let Some(origin) = exclude_origin {
                    if router_id.as_str() == origin {
                        return false;
                    }
                }
                // Check if any of the peer's patterns match this address
                patterns
                    .iter()
                    .any(|p| clasp_core::address::glob_match(p, address))
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Check if an address belongs to the local router's namespace
    pub fn is_local(&self, address: &str) -> bool {
        self.local_namespaces
            .iter()
            .any(|p| clasp_core::address::glob_match(p, address))
    }

    /// Check if an address belongs to any peer's namespace
    pub fn is_remote(&self, address: &str) -> bool {
        self.peer_namespaces.values().any(|patterns| {
            patterns
                .iter()
                .any(|p| clasp_core::address::glob_match(p, address))
        })
    }

    /// Check for namespace conflicts between peers.
    ///
    /// Returns pairs of (pattern, peer_a, peer_b) for overlapping namespaces.
    pub fn find_conflicts(&self) -> Vec<(String, String, String)> {
        let mut conflicts = Vec::new();
        let peers: Vec<_> = self.peer_namespaces.iter().collect();

        for i in 0..peers.len() {
            for j in (i + 1)..peers.len() {
                let (id_a, patterns_a) = peers[i];
                let (id_b, patterns_b) = peers[j];

                for pa in patterns_a {
                    for pb in patterns_b {
                        if patterns_overlap(pa, pb) {
                            conflicts.push((
                                format!("{} <-> {}", pa, pb),
                                id_a.clone(),
                                id_b.clone(),
                            ));
                        }
                    }
                }
            }
        }

        conflicts
    }

    /// Get a peer's registered namespaces
    pub fn peer_patterns(&self, router_id: &str) -> Option<&Vec<String>> {
        self.peer_namespaces.get(router_id)
    }

    /// Get all registered peers
    pub fn peers(&self) -> Vec<String> {
        self.peer_namespaces.keys().cloned().collect()
    }

    /// Get local namespaces
    pub fn local_patterns(&self) -> &[String] {
        &self.local_namespaces
    }

    /// Number of registered peers
    pub fn peer_count(&self) -> usize {
        self.peer_namespaces.len()
    }
}

/// Check if two glob patterns can potentially match the same address.
///
/// This is a conservative check -- it may return true for patterns that
/// don't actually overlap, but will never return false for patterns that do.
fn patterns_overlap(a: &str, b: &str) -> bool {
    // If either pattern is "/**" or "**", they overlap with everything
    if a == "/**" || a == "**" || b == "/**" || b == "**" {
        return true;
    }

    // Check if one pattern could match a prefix of the other
    // Simple heuristic: split by '/' and compare non-wildcard segments
    let parts_a: Vec<&str> = a.split('/').filter(|s| !s.is_empty()).collect();
    let parts_b: Vec<&str> = b.split('/').filter(|s| !s.is_empty()).collect();

    let min_len = parts_a.len().min(parts_b.len());
    for i in 0..min_len {
        let pa = parts_a[i];
        let pb = parts_b[i];

        // If either segment is a wildcard, they could overlap
        if pa == "*" || pa == "**" || pb == "*" || pb == "**" {
            return true;
        }

        // If both are literal and different, no overlap
        if pa != pb {
            return false;
        }
    }

    // If we got here, all compared segments match.
    // They overlap if either has a ** or they're the same length.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_basics() {
        let mut ns = NamespaceManager::new(vec!["/local/**".to_string()]);

        ns.register_peer("peer-a", vec!["/site-a/**".to_string()]);
        ns.register_peer("peer-b", vec!["/site-b/**".to_string()]);

        assert!(ns.is_local("/local/foo"));
        assert!(!ns.is_local("/site-a/foo"));

        assert!(ns.is_remote("/site-a/foo"));
        assert!(ns.is_remote("/site-b/foo"));
        assert!(!ns.is_remote("/local/foo"));
    }

    #[test]
    fn test_peers_for_address() {
        let mut ns = NamespaceManager::new(vec!["/local/**".to_string()]);
        ns.register_peer("peer-a", vec!["/shared/**".to_string()]);
        ns.register_peer("peer-b", vec!["/shared/**".to_string()]);

        let peers = ns.peers_for_address("/shared/foo", None);
        assert_eq!(peers.len(), 2);

        // Exclude origin
        let peers = ns.peers_for_address("/shared/foo", Some("peer-a"));
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0], "peer-b");
    }

    #[test]
    fn test_remove_peer() {
        let mut ns = NamespaceManager::new(vec![]);
        ns.register_peer("peer-a", vec!["/a/**".to_string()]);
        assert_eq!(ns.peer_count(), 1);

        ns.remove_peer("peer-a");
        assert_eq!(ns.peer_count(), 0);
        assert!(!ns.is_remote("/a/foo"));
    }

    #[test]
    fn test_conflict_detection() {
        let mut ns = NamespaceManager::new(vec![]);
        ns.register_peer("peer-a", vec!["/shared/**".to_string()]);
        ns.register_peer("peer-b", vec!["/shared/**".to_string()]);

        let conflicts = ns.find_conflicts();
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_no_conflict_disjoint() {
        let mut ns = NamespaceManager::new(vec![]);
        ns.register_peer("peer-a", vec!["/site-a/**".to_string()]);
        ns.register_peer("peer-b", vec!["/site-b/**".to_string()]);

        let conflicts = ns.find_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_patterns_overlap() {
        assert!(patterns_overlap("/**", "/anything"));
        assert!(patterns_overlap("/a/**", "/a/b/c"));
        assert!(!patterns_overlap("/a/**", "/b/**"));
        assert!(patterns_overlap("/shared/**", "/shared/**"));
        assert!(!patterns_overlap("/site-a/data", "/site-b/data"));
    }
}
