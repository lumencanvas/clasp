//! Address mapping between DefraDB document paths and CLASP signal addresses.

/// Parsed components of a `/defra/...` CLASP address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefraAddress {
    pub collection: String,
    pub doc_id: String,
    pub field: Option<String>,
}

/// Convert a DefraDB mutation into a CLASP signal address.
///
/// ```
/// use clasp_defra_bridge::defra_to_clasp_address;
///
/// assert_eq!(
///     defra_to_clasp_address("User", "bae-abc", Some("name")),
///     "/defra/User/bae-abc/name"
/// );
/// assert_eq!(
///     defra_to_clasp_address("User", "bae-abc", None),
///     "/defra/User/bae-abc"
/// );
/// ```
pub fn defra_to_clasp_address(collection: &str, doc_id: &str, field: Option<&str>) -> String {
    match field {
        Some(f) => format!("/defra/{collection}/{doc_id}/{f}"),
        None => format!("/defra/{collection}/{doc_id}"),
    }
}

/// Parse a CLASP address under `/defra/` into its components.
///
/// Returns `None` if the address does not start with `/defra/` or has
/// fewer than two path segments after the prefix.
///
/// ```
/// use clasp_defra_bridge::{parse_defra_address, DefraAddress};
///
/// let addr = parse_defra_address("/defra/User/bae-abc/name").unwrap();
/// assert_eq!(addr.collection, "User");
/// assert_eq!(addr.doc_id, "bae-abc");
/// assert_eq!(addr.field, Some("name".to_string()));
///
/// let addr = parse_defra_address("/defra/User/bae-abc").unwrap();
/// assert_eq!(addr.field, None);
///
/// assert!(parse_defra_address("/other/path").is_none());
/// ```
pub fn parse_defra_address(address: &str) -> Option<DefraAddress> {
    let rest = address.strip_prefix("/defra/")?;
    let mut parts = rest.splitn(3, '/');

    let collection = parts.next().filter(|s| !s.is_empty())?.to_string();
    let doc_id = parts.next().filter(|s| !s.is_empty())?.to_string();
    let field = parts.next().filter(|s| !s.is_empty()).map(String::from);

    Some(DefraAddress {
        collection,
        doc_id,
        field,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_defra_to_clasp() {
        assert_eq!(
            defra_to_clasp_address("User", "bae-123", Some("name")),
            "/defra/User/bae-123/name"
        );
        assert_eq!(
            defra_to_clasp_address("Post", "bae-456", None),
            "/defra/Post/bae-456"
        );
    }

    #[test]
    fn address_parse_full() {
        let addr = parse_defra_address("/defra/User/bae-abc/name").unwrap();
        assert_eq!(addr.collection, "User");
        assert_eq!(addr.doc_id, "bae-abc");
        assert_eq!(addr.field, Some("name".to_string()));
    }

    #[test]
    fn address_parse_no_field() {
        let addr = parse_defra_address("/defra/User/bae-abc").unwrap();
        assert_eq!(addr.collection, "User");
        assert_eq!(addr.doc_id, "bae-abc");
        assert_eq!(addr.field, None);
    }

    #[test]
    fn address_parse_invalid() {
        assert!(parse_defra_address("/other/path").is_none());
        assert!(parse_defra_address("/defra/").is_none());
        assert!(parse_defra_address("/defra/User").is_none());
        assert!(parse_defra_address("").is_none());
    }
}
