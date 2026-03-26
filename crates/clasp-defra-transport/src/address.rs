//! CLASP address namespace for DefraDB tunnel traffic.

/// Base namespace for DefraDB sync traffic.
pub const DEFRA_SYNC_NS: &str = "/defra/sync";

/// Parsed components of a sync address.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncAddress {
    pub peer_id: String,
    pub collection: Option<String>,
    pub is_block_channel: bool,
}

/// Build address for a specific peer's sync channel.
///
/// Returns `/defra/sync/{peer_id}`.
pub fn peer_channel(peer_id: &str) -> String {
    format!("{DEFRA_SYNC_NS}/{peer_id}")
}

/// Build address for collection-level sync.
///
/// Returns `/defra/sync/{peer_id}/{collection}`.
pub fn collection_channel(peer_id: &str, collection: &str) -> String {
    format!("{DEFRA_SYNC_NS}/{peer_id}/{collection}")
}

/// Build address for block transfer.
///
/// Returns `/defra/sync/{peer_id}/blocks`.
pub fn block_channel(peer_id: &str) -> String {
    format!("{DEFRA_SYNC_NS}/{peer_id}/blocks")
}

/// Parse a sync address into its components.
///
/// Expects addresses of the form:
/// - `/defra/sync/{peer_id}`
/// - `/defra/sync/{peer_id}/blocks`
/// - `/defra/sync/{peer_id}/{collection}`
///
/// Returns `None` if the address does not start with `/defra/sync/` or has
/// fewer than three path segments after the leading slash.
pub fn parse_sync_address(address: &str) -> Option<SyncAddress> {
    let stripped = address.strip_prefix("/defra/sync/")?;
    let parts: Vec<&str> = stripped.splitn(2, '/').collect();

    let peer_id = parts[0];
    if peer_id.is_empty() {
        return None;
    }

    match parts.get(1) {
        None => Some(SyncAddress {
            peer_id: peer_id.to_string(),
            collection: None,
            is_block_channel: false,
        }),
        Some(&"blocks") => Some(SyncAddress {
            peer_id: peer_id.to_string(),
            collection: None,
            is_block_channel: true,
        }),
        Some(collection) if !collection.is_empty() => Some(SyncAddress {
            peer_id: peer_id.to_string(),
            collection: Some(collection.to_string()),
            is_block_channel: false,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_peer_channel() {
        assert_eq!(peer_channel("peer-abc"), "/defra/sync/peer-abc");
    }

    #[test]
    fn address_collection_channel() {
        assert_eq!(
            collection_channel("peer-abc", "users"),
            "/defra/sync/peer-abc/users"
        );
    }

    #[test]
    fn address_block_channel() {
        assert_eq!(block_channel("peer-abc"), "/defra/sync/peer-abc/blocks");
    }

    #[test]
    fn address_parse() {
        // Peer-only address
        let addr = parse_sync_address("/defra/sync/peer-abc").unwrap();
        assert_eq!(addr.peer_id, "peer-abc");
        assert_eq!(addr.collection, None);
        assert!(!addr.is_block_channel);

        // Collection address
        let addr = parse_sync_address("/defra/sync/peer-abc/users").unwrap();
        assert_eq!(addr.peer_id, "peer-abc");
        assert_eq!(addr.collection, Some("users".into()));
        assert!(!addr.is_block_channel);

        // Block channel
        let addr = parse_sync_address("/defra/sync/peer-abc/blocks").unwrap();
        assert_eq!(addr.peer_id, "peer-abc");
        assert_eq!(addr.collection, None);
        assert!(addr.is_block_channel);

        // Invalid addresses
        assert!(parse_sync_address("/defra/other/peer").is_none());
        assert!(parse_sync_address("/not/defra").is_none());
        assert!(parse_sync_address("").is_none());
        assert!(parse_sync_address("/defra/sync/").is_none());
    }
}
