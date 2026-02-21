//! Capability token validator for CLASP router integration
//!
//! Implements `TokenValidator` so capability tokens can be validated
//! alongside existing CPSK tokens via `ValidatorChain`.

use clasp_core::security::{Action, Scope, TokenInfo, TokenValidator, ValidationResult};
use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};

use crate::error::CapError;
use crate::token::{CapabilityToken, TOKEN_PREFIX};

/// Validates CLASP capability tokens.
///
/// Integrates with `ValidatorChain` to support `cap_` prefixed tokens
/// alongside existing `cpsk_` and `ext_` tokens.
pub struct CapabilityValidator {
    /// Trusted root issuer public keys
    trust_anchors: Vec<Vec<u8>>,
    /// Maximum delegation chain depth
    max_depth: usize,
}

impl CapabilityValidator {
    /// Create a new capability validator.
    ///
    /// `trust_anchors` are the Ed25519 public keys of trusted root issuers.
    /// Only tokens whose delegation chain ultimately leads to a trust anchor
    /// will be accepted.
    pub fn new(trust_anchors: Vec<Vec<u8>>, max_depth: usize) -> Self {
        Self {
            trust_anchors,
            max_depth,
        }
    }

    /// Add a trust anchor (root issuer public key)
    pub fn add_trust_anchor(&mut self, public_key: Vec<u8>) {
        self.trust_anchors.push(public_key);
    }

    /// Validate a capability token and return the result
    fn validate_token(&self, token_str: &str) -> std::result::Result<CapabilityToken, CapError> {
        // Decode the token
        let token = CapabilityToken::decode(token_str)?;

        // Check expiration
        if token.is_expired() {
            return Err(CapError::Expired);
        }

        // Check chain depth
        if token.chain_depth() > self.max_depth {
            return Err(CapError::ChainTooDeep {
                depth: token.chain_depth(),
                max: self.max_depth,
            });
        }

        // Verify signature
        token.verify_signature()?;

        // Verify the delegation chain root leads to a trust anchor
        let root_issuer = if token.proofs.is_empty() {
            &token.issuer
        } else {
            &token.proofs[0].issuer
        };

        if !self.trust_anchors.iter().any(|anchor| anchor == root_issuer) {
            return Err(CapError::UntrustedIssuer(hex::encode(root_issuer)));
        }

        // Verify scope attenuation through the chain
        if !token.proofs.is_empty() {
            // Check each link: child scopes must be subset of parent scopes
            for i in 1..token.proofs.len() {
                let parent = &token.proofs[i - 1];
                let child = &token.proofs[i];
                for scope in &child.scopes {
                    if !scope_within_parent(scope, &parent.scopes) {
                        return Err(CapError::AttenuationViolation(format!(
                            "scope '{}' at depth {} exceeds parent",
                            scope, i
                        )));
                    }
                }
            }

            // Check final token's scopes against last proof
            let last_proof = token.proofs.last().unwrap();
            for scope in &token.scopes {
                if !scope_within_parent(scope, &last_proof.scopes) {
                    return Err(CapError::AttenuationViolation(format!(
                        "token scope '{}' exceeds last delegation",
                        scope
                    )));
                }
            }
        }

        Ok(token)
    }
}

/// Check if a scope string is allowed by any of the parent scopes
fn scope_within_parent(scope: &str, parent_scopes: &[String]) -> bool {
    let Some((child_action, child_pattern)) = scope.split_once(':') else {
        return false;
    };

    for parent in parent_scopes {
        let Some((parent_action, parent_pattern)) = parent.split_once(':') else {
            continue;
        };

        let action_ok = match parent_action {
            "admin" => true,
            "write" => child_action == "write" || child_action == "read",
            "read" => child_action == "read",
            _ => parent_action == child_action,
        };

        if action_ok && crate::token::pattern_is_subset(child_pattern, parent_pattern) {
            return true;
        }
    }

    false
}

/// Hex encoding helper (minimal, avoids a dependency)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl TokenValidator for CapabilityValidator {
    fn validate(&self, token: &str) -> ValidationResult {
        // Only handle cap_ tokens
        if !token.starts_with(TOKEN_PREFIX) {
            return ValidationResult::NotMyToken;
        }

        match self.validate_token(token) {
            Ok(cap_token) => {
                // Convert capability scopes to CLASP Scope objects
                let scopes: Vec<Scope> = cap_token
                    .scopes
                    .iter()
                    .filter_map(|s| {
                        let (action_str, pattern) = s.split_once(':')?;
                        let action = match action_str {
                            "admin" => Action::Admin,
                            "write" => Action::Write,
                            "read" => Action::Read,
                            _ => return None,
                        };
                        Scope::new(action, pattern).ok()
                    })
                    .collect();

                let expires_at = if cap_token.expires_at > 0 {
                    Some(UNIX_EPOCH + Duration::from_secs(cap_token.expires_at))
                } else {
                    None
                };

                let mut metadata = HashMap::new();
                metadata.insert(
                    "chain_depth".to_string(),
                    cap_token.chain_depth().to_string(),
                );

                let info = TokenInfo {
                    token_id: cap_token.nonce.clone(),
                    subject: None, // Capability tokens are bearer tokens
                    scopes,
                    expires_at,
                    metadata,
                };

                ValidationResult::Valid(info)
            }
            Err(CapError::Expired) => ValidationResult::Expired,
            Err(e) => ValidationResult::Invalid(e.to_string()),
        }
    }

    fn name(&self) -> &str {
        "Capability"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::CapabilityToken;
    use ed25519_dalek::SigningKey;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn future_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600
    }

    fn root_key() -> SigningKey {
        SigningKey::from_bytes(&[1u8; 32])
    }

    fn make_validator() -> CapabilityValidator {
        let key = root_key();
        let pub_key = key.verifying_key().to_bytes().to_vec();
        CapabilityValidator::new(vec![pub_key], 5)
    }

    #[test]
    fn test_validate_root_token() {
        let validator = make_validator();
        let key = root_key();

        let token = CapabilityToken::create_root(
            &key,
            vec!["admin:/**".to_string()],
            future_timestamp(),
            None,
        )
        .unwrap();

        let encoded = token.encode().unwrap();
        match validator.validate(&encoded) {
            ValidationResult::Valid(info) => {
                assert!(!info.scopes.is_empty());
                assert!(info.has_scope(Action::Admin, "/anything"));
            }
            other => panic!("expected Valid, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_delegated_token() {
        let validator = make_validator();
        let root_key = root_key();
        let child_key = SigningKey::from_bytes(&[2u8; 32]);

        let root = CapabilityToken::create_root(
            &root_key,
            vec!["admin:/**".to_string()],
            future_timestamp(),
            None,
        )
        .unwrap();

        let child = root
            .delegate(
                &child_key,
                vec!["write:/lights/**".to_string()],
                future_timestamp(),
                None,
            )
            .unwrap();

        let encoded = child.encode().unwrap();
        match validator.validate(&encoded) {
            ValidationResult::Valid(info) => {
                assert!(info.has_scope(Action::Write, "/lights/room1"));
                assert!(!info.has_scope(Action::Write, "/audio/channel1"));
            }
            other => panic!("expected Valid, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_untrusted_issuer() {
        let validator = make_validator();
        let untrusted_key = SigningKey::from_bytes(&[99u8; 32]);

        let token = CapabilityToken::create_root(
            &untrusted_key,
            vec!["admin:/**".to_string()],
            future_timestamp(),
            None,
        )
        .unwrap();

        let encoded = token.encode().unwrap();
        match validator.validate(&encoded) {
            ValidationResult::Invalid(msg) => {
                assert!(msg.contains("untrusted"));
            }
            other => panic!("expected Invalid, got {:?}", other),
        }
    }

    #[test]
    fn test_not_my_token() {
        let validator = make_validator();
        match validator.validate("cpsk_something") {
            ValidationResult::NotMyToken => {}
            other => panic!("expected NotMyToken, got {:?}", other),
        }
    }

    #[test]
    fn test_expired_token() {
        let validator = make_validator();
        let key = root_key();

        // Create a token that's already expired
        let token = CapabilityToken::create_root(
            &key,
            vec!["admin:/**".to_string()],
            0, // Expired at epoch
            None,
        )
        .unwrap();

        let encoded = token.encode().unwrap();
        match validator.validate(&encoded) {
            ValidationResult::Expired => {}
            other => panic!("expected Expired, got {:?}", other),
        }
    }
}
