//! Receives CLASP signals and writes them to DefraDB documents.

use clasp_core::Value;
use tracing::{debug, warn};

use clasp_journal_defra::DefraClient;

use crate::address::parse_defra_address;
use crate::convert::clasp_value_to_json;
use crate::error::{BridgeError, Result};

/// Handles incoming CLASP signals that target DefraDB documents and
/// translates them into GraphQL mutations.
pub struct DefraWriter {
    client: DefraClient,
}

impl DefraWriter {
    pub fn new(client: DefraClient) -> Self {
        Self { client }
    }

    /// Handle a CLASP signal targeting a DefraDB document.
    ///
    /// Parses the address, determines whether to update a single field or
    /// the entire document, and issues the appropriate GraphQL mutation.
    pub async fn handle_signal(&self, address: &str, value: Value) -> Result<()> {
        let addr = parse_defra_address(address)
            .ok_or_else(|| BridgeError::Address(format!("invalid defra address: {address}")))?;

        debug!(
            collection = %addr.collection,
            doc_id = %addr.doc_id,
            field = ?addr.field,
            "writing signal to DefraDB"
        );

        match addr.field {
            Some(field) => self.update_field(&addr.collection, &addr.doc_id, &field, value).await,
            None => self.update_document(&addr.collection, &addr.doc_id, value).await,
        }
    }

    /// Update a single field on a document.
    async fn update_field(
        &self,
        collection: &str,
        doc_id: &str,
        field: &str,
        value: Value,
    ) -> Result<()> {
        let json_value = clasp_value_to_json(&value);
        let value_str = serde_json::to_string(&json_value)
            .map_err(|e| BridgeError::Conversion(e.to_string()))?;

        let mutation = format!(
            r#"mutation {{ update_{collection}(docID: "{doc_id}", input: {{ {field}: {value_str} }}) {{ _docID }} }}"#
        );

        self.client
            .graphql(&mutation, None)
            .await
            .map_err(|e| BridgeError::Defra(e.to_string()))?;

        Ok(())
    }

    /// Update all fields on a document from a CLASP Map value.
    async fn update_document(
        &self,
        collection: &str,
        doc_id: &str,
        value: Value,
    ) -> Result<()> {
        let fields = match &value {
            Value::Map(map) => map,
            _ => {
                warn!(
                    collection = %collection,
                    doc_id = %doc_id,
                    "expected Map value for whole-document update, ignoring"
                );
                return Ok(());
            }
        };

        if fields.is_empty() {
            return Ok(());
        }

        let mut input_parts = Vec::new();
        for (k, v) in fields {
            let json_val = clasp_value_to_json(v);
            let val_str = serde_json::to_string(&json_val)
                .map_err(|e| BridgeError::Conversion(e.to_string()))?;
            input_parts.push(format!("{k}: {val_str}"));
        }
        let input = input_parts.join(", ");

        let mutation = format!(
            r#"mutation {{ update_{collection}(docID: "{doc_id}", input: {{ {input} }}) {{ _docID }} }}"#
        );

        self.client
            .graphql(&mutation, None)
            .await
            .map_err(|e| BridgeError::Defra(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_parses_field_address() {
        // Verify the writer can parse addresses correctly (the actual
        // GraphQL call requires a running DefraDB instance).
        let addr = parse_defra_address("/defra/User/bae-abc/name").unwrap();
        assert_eq!(addr.collection, "User");
        assert_eq!(addr.doc_id, "bae-abc");
        assert_eq!(addr.field, Some("name".to_string()));
    }

    #[test]
    fn writer_rejects_non_defra_address() {
        let writer = DefraWriter::new(DefraClient::new("http://localhost:19181"));
        // We cannot call handle_signal synchronously, but we can verify
        // that parse_defra_address returns None for bad addresses.
        assert!(parse_defra_address("/other/path").is_none());
        let _ = writer; // suppress unused warning
    }
}
