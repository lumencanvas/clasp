//! Capability token types and operations
//!
//! Implements UCAN-inspired delegatable tokens where each token in a
//! delegation chain can only narrow (attenuate) scopes, never widen.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{CapError, Result};

/// A CLASP capability token.
///
/// Token format: `cap_<base64url(messagepack(CapabilityToken))>`
///
/// Tokens form delegation chains where each child can only narrow
/// the parent's scopes, never widen them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    /// Token version (currently 1)
    pub version: u8,
    /// Issuer's public key (Ed25519, 32 bytes)
    pub issuer: Vec<u8>,
    /// Audience public key (None = bearer token)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<u8>>,
    /// Scopes granted (same "action:pattern" format as existing CLASP scopes)
    pub scopes: Vec<String>,
    /// Expiration time (Unix timestamp, seconds)
    pub expires_at: u64,
    /// Unique nonce to prevent replay
    pub nonce: String,
    /// Proof chain: signatures of parent tokens in the delegation chain
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proofs: Vec<ProofLink>,
    /// Signature over the token payload (by issuer)
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

/// A link in the proof/delegation chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofLink {
    /// The parent token's issuer public key
    pub issuer: Vec<u8>,
    /// The parent token's scopes (for attenuation checking)
    pub scopes: Vec<String>,
    /// Signature of the parent token
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

/// Token prefix
pub const TOKEN_PREFIX: &str = "cap_";

impl CapabilityToken {
    /// Create and sign a new root capability token (no parent).
    pub fn create_root(
        signing_key: &SigningKey,
        scopes: Vec<String>,
        expires_at: u64,
        audience: Option<Vec<u8>>,
    ) -> Result<Self> {
        let issuer = signing_key.verifying_key().to_bytes().to_vec();
        let nonce = uuid::Uuid::new_v4().to_string();

        let mut token = Self {
            version: 1,
            issuer,
            audience,
            scopes,
            expires_at,
            nonce,
            proofs: vec![],
            signature: vec![],
        };

        // Sign the token
        let payload = token.signable_payload()?;
        let signature = signing_key.sign(&payload);
        token.signature = signature.to_bytes().to_vec();

        Ok(token)
    }

    /// Delegate this token to create a child with narrower scopes.
    ///
    /// The child token can only have scopes that are a subset of this token's scopes.
    pub fn delegate(
        &self,
        child_signing_key: &SigningKey,
        child_scopes: Vec<String>,
        expires_at: u64,
        audience: Option<Vec<u8>>,
    ) -> Result<Self> {
        // Verify attenuation: child scopes must be subset of parent scopes
        for child_scope in &child_scopes {
            if !self.scope_allows(child_scope) {
                return Err(CapError::AttenuationViolation(format!(
                    "child scope '{}' not allowed by parent scopes {:?}",
                    child_scope, self.scopes
                )));
            }
        }

        // Child expiration cannot exceed parent
        let child_expires = expires_at.min(self.expires_at);

        let child_issuer = child_signing_key.verifying_key().to_bytes().to_vec();
        let nonce = uuid::Uuid::new_v4().to_string();

        // Build proof chain: include all of parent's proofs + parent itself
        let mut proofs = self.proofs.clone();
        proofs.push(ProofLink {
            issuer: self.issuer.clone(),
            scopes: self.scopes.clone(),
            signature: self.signature.clone(),
        });

        let mut token = Self {
            version: 1,
            issuer: child_issuer,
            audience,
            scopes: child_scopes,
            expires_at: child_expires,
            nonce,
            proofs,
            signature: vec![],
        };

        let payload = token.signable_payload()?;
        let signature = child_signing_key.sign(&payload);
        token.signature = signature.to_bytes().to_vec();

        Ok(token)
    }

    /// Check if this token's scopes allow a given scope string.
    ///
    /// Uses the same matching logic as CLASP's `Scope::allows()`.
    fn scope_allows(&self, child_scope: &str) -> bool {
        // Parse child scope as "action:pattern"
        let Some((child_action, child_pattern)) = child_scope.split_once(':') else {
            return false;
        };

        for parent_scope in &self.scopes {
            let Some((parent_action, parent_pattern)) = parent_scope.split_once(':') else {
                continue;
            };

            // Check action attenuation
            let action_ok = match parent_action {
                "admin" => true,
                "write" => child_action == "write" || child_action == "read",
                "read" => child_action == "read",
                _ => parent_action == child_action,
            };

            if !action_ok {
                continue;
            }

            // Check pattern attenuation (child must be same or narrower)
            if pattern_is_subset(child_pattern, parent_pattern) {
                return true;
            }
        }

        false
    }

    /// Verify this token's signature
    pub fn verify_signature(&self) -> Result<()> {
        let verifying_key = VerifyingKey::from_bytes(
            self.issuer
                .as_slice()
                .try_into()
                .map_err(|_| CapError::KeyError("invalid issuer key length".to_string()))?,
        )
        .map_err(|e| CapError::KeyError(e.to_string()))?;

        let payload = self.signable_payload()?;
        let signature = Signature::from_bytes(
            self.signature
                .as_slice()
                .try_into()
                .map_err(|_| CapError::InvalidSignature)?,
        );

        verifying_key
            .verify(&payload, &signature)
            .map_err(|_| CapError::InvalidSignature)
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.expires_at
    }

    /// Get the delegation chain depth
    pub fn chain_depth(&self) -> usize {
        self.proofs.len()
    }

    /// Encode to the `cap_<base64>` wire format
    pub fn encode(&self) -> Result<String> {
        use base64::Engine;
        let bytes =
            rmp_serde::to_vec_named(self).map_err(|e| CapError::Encoding(e.to_string()))?;
        Ok(format!(
            "{}{}",
            TOKEN_PREFIX,
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
        ))
    }

    /// Decode from the `cap_<base64>` wire format
    pub fn decode(token: &str) -> Result<Self> {
        use base64::Engine;
        let encoded = token
            .strip_prefix(TOKEN_PREFIX)
            .ok_or_else(|| CapError::Encoding("missing cap_ prefix".to_string()))?;

        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|e| CapError::Encoding(e.to_string()))?;

        rmp_serde::from_slice(&bytes).map_err(|e| CapError::Encoding(e.to_string()))
    }

    /// Compute the signable payload (everything except the signature field)
    fn signable_payload(&self) -> Result<Vec<u8>> {
        // Create a copy without the signature for signing
        let signable = SignableToken {
            version: self.version,
            issuer: &self.issuer,
            audience: self.audience.as_deref(),
            scopes: &self.scopes,
            expires_at: self.expires_at,
            nonce: &self.nonce,
            proofs: &self.proofs,
        };

        rmp_serde::to_vec_named(&signable).map_err(|e| CapError::Encoding(e.to_string()))
    }
}

/// Signable portion of a token (excludes the signature itself)
#[derive(Serialize)]
struct SignableToken<'a> {
    version: u8,
    issuer: &'a [u8],
    audience: Option<&'a [u8]>,
    scopes: &'a [String],
    expires_at: u64,
    nonce: &'a str,
    proofs: &'a [ProofLink],
}

/// Check if `child` pattern is a subset of `parent` pattern.
///
/// A pattern is a subset if every address it matches is also matched by the parent.
/// Simple heuristic: check segment-by-segment prefix match with wildcard handling.
pub fn pattern_is_subset(child: &str, parent: &str) -> bool {
    // Exact match
    if child == parent {
        return true;
    }

    // Parent is "/**" or "**" -- matches everything
    if parent == "/**" || parent == "**" {
        return true;
    }

    let parent_parts: Vec<&str> = parent.split('/').filter(|s| !s.is_empty()).collect();
    let child_parts: Vec<&str> = child.split('/').filter(|s| !s.is_empty()).collect();

    // Walk through parent segments
    let mut pi = 0;
    let mut ci = 0;

    while pi < parent_parts.len() && ci < child_parts.len() {
        let pp = parent_parts[pi];
        let cp = child_parts[ci];

        if pp == "**" {
            // Parent ** matches any remaining child segments
            return true;
        }

        if pp == "*" {
            // Parent * matches one child segment
            pi += 1;
            ci += 1;
            continue;
        }

        if cp == "**" {
            // Child ** is wider than parent literal -- NOT a subset
            return false;
        }

        if cp == "*" {
            // Child * at position where parent has literal -- NOT a subset
            // (child could match things parent doesn't)
            return false;
        }

        // Both literal: must match
        if pp != cp {
            return false;
        }

        pi += 1;
        ci += 1;
    }

    // If parent has remaining ** at end, child is a subset
    if pi < parent_parts.len() && parent_parts[pi] == "**" {
        return true;
    }

    // Both must be exhausted for equal-length patterns
    pi >= parent_parts.len() && ci >= child_parts.len()
}


#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn test_key() -> SigningKey {
        SigningKey::from_bytes(&[1u8; 32])
    }

    fn future_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600
    }

    #[test]
    fn test_create_root_token() {
        let key = test_key();
        let token = CapabilityToken::create_root(
            &key,
            vec!["admin:/**".to_string()],
            future_timestamp(),
            None,
        )
        .unwrap();

        assert_eq!(token.version, 1);
        assert_eq!(token.scopes, vec!["admin:/**"]);
        assert!(token.proofs.is_empty());
        assert!(!token.signature.is_empty());
    }

    #[test]
    fn test_verify_signature() {
        let key = test_key();
        let token = CapabilityToken::create_root(
            &key,
            vec!["admin:/**".to_string()],
            future_timestamp(),
            None,
        )
        .unwrap();

        assert!(token.verify_signature().is_ok());
    }

    #[test]
    fn test_encode_decode() {
        let key = test_key();
        let token = CapabilityToken::create_root(
            &key,
            vec!["read:/**".to_string()],
            future_timestamp(),
            None,
        )
        .unwrap();

        let encoded = token.encode().unwrap();
        assert!(encoded.starts_with("cap_"));

        let decoded = CapabilityToken::decode(&encoded).unwrap();
        assert_eq!(decoded.scopes, token.scopes);
        assert_eq!(decoded.issuer, token.issuer);
    }

    #[test]
    fn test_delegation() {
        let root_key = test_key();
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

        assert_eq!(child.chain_depth(), 1);
        assert_eq!(child.scopes, vec!["write:/lights/**"]);
        assert!(child.verify_signature().is_ok());
    }

    #[test]
    fn test_attenuation_violation() {
        let root_key = test_key();
        let child_key = SigningKey::from_bytes(&[2u8; 32]);

        let root = CapabilityToken::create_root(
            &root_key,
            vec!["write:/lights/**".to_string()],
            future_timestamp(),
            None,
        )
        .unwrap();

        // Try to widen scope -- should fail
        let result = root.delegate(
            &child_key,
            vec!["write:/audio/**".to_string()],
            future_timestamp(),
            None,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CapError::AttenuationViolation(_)
        ));
    }

    #[test]
    fn test_expiration_clamped() {
        let root_key = test_key();
        let child_key = SigningKey::from_bytes(&[2u8; 32]);

        let root_expires = future_timestamp();
        let root = CapabilityToken::create_root(
            &root_key,
            vec!["admin:/**".to_string()],
            root_expires,
            None,
        )
        .unwrap();

        // Child tries to set expiration beyond parent's
        let child = root
            .delegate(
                &child_key,
                vec!["read:/**".to_string()],
                root_expires + 9999,
                None,
            )
            .unwrap();

        // Should be clamped to parent's expiration
        assert_eq!(child.expires_at, root_expires);
    }

    #[test]
    fn test_pattern_is_subset() {
        assert!(pattern_is_subset("/lights/room1/**", "/lights/**"));
        assert!(pattern_is_subset("/lights/room1", "/lights/**"));
        assert!(pattern_is_subset("/**", "/**"));
        assert!(!pattern_is_subset("/audio/**", "/lights/**"));
        assert!(!pattern_is_subset("/**", "/lights/**"));
        assert!(pattern_is_subset("/lights/1", "/lights/*"));
    }

    #[test]
    fn test_multi_hop_delegation() {
        let key_a = SigningKey::from_bytes(&[1u8; 32]);
        let key_b = SigningKey::from_bytes(&[2u8; 32]);
        let key_c = SigningKey::from_bytes(&[3u8; 32]);

        let root = CapabilityToken::create_root(
            &key_a,
            vec!["admin:/**".to_string()],
            future_timestamp(),
            None,
        )
        .unwrap();

        let child = root
            .delegate(
                &key_b,
                vec!["write:/lights/**".to_string()],
                future_timestamp(),
                None,
            )
            .unwrap();

        let grandchild = child
            .delegate(
                &key_c,
                vec!["write:/lights/room1/**".to_string()],
                future_timestamp(),
                None,
            )
            .unwrap();

        assert_eq!(grandchild.chain_depth(), 2);
        assert!(grandchild.verify_signature().is_ok());

        // Grandchild can't widen beyond child
        let result = child.delegate(
            &key_c,
            vec!["write:/audio/**".to_string()],
            future_timestamp(),
            None,
        );
        assert!(result.is_err());
    }
}
