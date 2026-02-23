//! E2E encryption primitives.
//!
//! AES-256-GCM for symmetric encryption, ECDH P-256 for key exchange,
//! HKDF-SHA256 for key derivation, ECDSA P-256 for signing.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::{STANDARD as B64, URL_SAFE_NO_PAD as B64URL}, Engine};
use ecdsa::signature::{Signer, Verifier};
use hkdf::Hkdf;
use p256::{
    ecdsa::{SigningKey, VerifyingKey, Signature},
    elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint},
    PublicKey, SecretKey,
};
use rand::RngCore;
use sha2::Sha256;
use zeroize::Zeroize;

use crate::error::{CryptoError, Result};
use crate::types::{ECDHKeyPair, SigningKeyPair};

// --- AES-256-GCM ---

/// Generate a random 256-bit AES key.
pub fn generate_group_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Encrypt plaintext with AES-256-GCM. Returns (ciphertext, iv).
pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    if key.len() != 32 {
        return Err(CryptoError::InvalidKey("AES key must be 32 bytes".into()));
    }
    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);

    let mut iv = [0u8; 12];
    OsRng.fill_bytes(&mut iv);
    let nonce = Nonce::from_slice(&iv);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    Ok((ciphertext, iv.to_vec()))
}

/// Decrypt ciphertext with AES-256-GCM.
pub fn decrypt(key: &[u8], ciphertext: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(CryptoError::InvalidKey("AES key must be 32 bytes".into()));
    }
    if iv.len() != 12 {
        return Err(CryptoError::InvalidKey("IV must be 12 bytes".into()));
    }
    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);
    let nonce = Nonce::from_slice(iv);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
}

// --- ECDH P-256 ---

/// Generate an ECDH P-256 key pair.
/// Returns (SEC1-encoded public key, raw scalar private key).
pub fn generate_ecdh_key_pair() -> ECDHKeyPair {
    let secret = SecretKey::random(&mut OsRng);
    let public = secret.public_key();

    ECDHKeyPair {
        public_key: public.to_encoded_point(false).as_bytes().to_vec(),
        private_key: secret.to_bytes().to_vec(),
    }
}

/// Derive a shared AES-256 key from ECDH + HKDF-SHA256.
pub fn derive_shared_key(
    private_key: &[u8],
    peer_public_key: &[u8],
    info: Option<&str>,
) -> Result<Vec<u8>> {
    let info_str = info.unwrap_or("clasp-e2e-keyex-v1");

    let secret = SecretKey::from_bytes(private_key.into())
        .map_err(|e| CryptoError::InvalidKey(format!("invalid ECDH private key: {e}")))?;

    let peer_point = p256::EncodedPoint::from_bytes(peer_public_key)
        .map_err(|e| CryptoError::InvalidKey(format!("invalid peer public key: {e}")))?;
    let peer_pub: PublicKey = Option::from(PublicKey::from_encoded_point(&peer_point))
        .ok_or_else(|| CryptoError::InvalidKey("peer public key is not on curve".into()))?;

    let shared_secret = p256::ecdh::diffie_hellman(
        secret.to_nonzero_scalar(),
        peer_pub.as_affine(),
    );
    let raw_bytes = shared_secret.raw_secret_bytes();

    let hkdf = Hkdf::<Sha256>::new(Some(&[0u8; 32]), raw_bytes);
    let mut okm = [0u8; 32];
    hkdf.expand(info_str.as_bytes(), &mut okm)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let result = okm.to_vec();
    okm.zeroize();
    Ok(result)
}

// --- ECDSA P-256 ---

/// Generate an ECDSA P-256 signing key pair.
pub fn generate_signing_key_pair() -> SigningKeyPair {
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    SigningKeyPair {
        public_key: verifying_key.to_encoded_point(false).as_bytes().to_vec(),
        private_key: signing_key.to_bytes().to_vec(),
    }
}

/// Sign data with an ECDSA P-256 private key.
pub fn sign(private_key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let signing_key = SigningKey::from_bytes(private_key.into())
        .map_err(|e| CryptoError::InvalidKey(format!("invalid signing key: {e}")))?;
    let signature: Signature = signing_key.sign(data);
    Ok(signature.to_bytes().to_vec())
}

/// Verify an ECDSA P-256 signature.
pub fn verify(public_key: &[u8], data: &[u8], signature: &[u8]) -> Result<bool> {
    let point = p256::EncodedPoint::from_bytes(public_key)
        .map_err(|e| CryptoError::InvalidKey(format!("invalid verifying key: {e}")))?;
    let verifying_key = VerifyingKey::from_encoded_point(&point)
        .map_err(|e| CryptoError::InvalidKey(format!("invalid verifying key: {e}")))?;
    let sig = Signature::from_bytes(signature.into())
        .map_err(|e| CryptoError::VerificationFailed(format!("invalid signature: {e}")))?;

    Ok(verifying_key.verify(data, &sig).is_ok())
}

// --- JWK interop (matches Web Crypto API JWK format for JS compatibility) ---

/// Convert a P-256 SEC1 uncompressed public key to a JWK JSON value.
/// Matches the format produced by Web Crypto `exportKey('jwk', ecdhPublicKey)`.
pub fn public_key_to_jwk(sec1_public_key: &[u8]) -> Result<serde_json::Value> {
    // SEC1 uncompressed: 0x04 || x (32 bytes) || y (32 bytes)
    if sec1_public_key.len() != 65 || sec1_public_key[0] != 0x04 {
        return Err(CryptoError::InvalidKey("expected 65-byte SEC1 uncompressed point".into()));
    }
    let x = B64URL.encode(&sec1_public_key[1..33]);
    let y = B64URL.encode(&sec1_public_key[33..65]);
    Ok(serde_json::json!({
        "crv": "P-256",
        "ext": true,
        "key_ops": [],
        "kty": "EC",
        "x": x,
        "y": y
    }))
}

/// Convert a JWK JSON value (ECDH P-256 public key) to SEC1 uncompressed bytes.
pub fn jwk_to_public_key(jwk: &serde_json::Value) -> Result<Vec<u8>> {
    // Validate key type and curve
    let kty = jwk.get("kty").and_then(|v| v.as_str());
    let crv = jwk.get("crv").and_then(|v| v.as_str());
    if kty != Some("EC") || crv != Some("P-256") {
        return Err(CryptoError::InvalidKey("JWK must be EC/P-256 public key".into()));
    }
    let x_b64 = jwk.get("x").and_then(|v| v.as_str())
        .ok_or_else(|| CryptoError::InvalidKey("JWK missing 'x' field".into()))?;
    let y_b64 = jwk.get("y").and_then(|v| v.as_str())
        .ok_or_else(|| CryptoError::InvalidKey("JWK missing 'y' field".into()))?;
    let x = B64URL.decode(x_b64)
        .map_err(|e| CryptoError::InvalidKey(format!("invalid JWK x: {e}")))?;
    let y = B64URL.decode(y_b64)
        .map_err(|e| CryptoError::InvalidKey(format!("invalid JWK y: {e}")))?;
    if x.len() != 32 || y.len() != 32 {
        return Err(CryptoError::InvalidKey("JWK x/y must be 32 bytes each for P-256".into()));
    }
    let mut sec1 = Vec::with_capacity(65);
    sec1.push(0x04);
    sec1.extend_from_slice(&x);
    sec1.extend_from_slice(&y);
    Ok(sec1)
}

/// Convert a raw 32-byte AES-256 group key to a JWK JSON value.
/// Matches the format produced by Web Crypto `exportKey('jwk', aesKey)`.
pub fn group_key_to_jwk(raw_key: &[u8]) -> Result<serde_json::Value> {
    if raw_key.len() != 32 {
        return Err(CryptoError::InvalidKey("AES key must be 32 bytes".into()));
    }
    let k = B64URL.encode(raw_key);
    Ok(serde_json::json!({
        "alg": "A256GCM",
        "ext": true,
        "k": k,
        "key_ops": ["encrypt", "decrypt"],
        "kty": "oct"
    }))
}

/// Convert a JWK JSON value (AES-256-GCM key) to raw 32-byte key.
pub fn jwk_to_group_key(jwk: &serde_json::Value) -> Result<Vec<u8>> {
    // Validate key type
    let kty = jwk.get("kty").and_then(|v| v.as_str());
    if kty != Some("oct") {
        return Err(CryptoError::InvalidKey("JWK must be kty=oct for group key".into()));
    }
    let k_b64 = jwk.get("k").and_then(|v| v.as_str())
        .ok_or_else(|| CryptoError::InvalidKey("JWK missing 'k' field".into()))?;
    let bytes = B64URL.decode(k_b64)
        .map_err(|e| CryptoError::InvalidKey(format!("invalid JWK k: {e}")))?;
    if bytes.len() != 32 {
        return Err(CryptoError::InvalidKey(format!(
            "group key must be 32 bytes, got {}", bytes.len()
        )));
    }
    Ok(bytes)
}

// --- Legacy key serialization (internal/Rust-only) ---

/// Export a public key as base64-encoded SEC1 bytes.
pub fn export_public_key(key: &[u8]) -> String {
    B64.encode(key)
}

/// Import a base64-encoded SEC1 public key.
pub fn import_public_key(encoded: &str) -> Result<Vec<u8>> {
    B64.decode(encoded)
        .map_err(|e| CryptoError::InvalidKey(format!("invalid base64: {e}")))
}

/// Export a group key as base64.
pub fn export_group_key(key: &[u8]) -> String {
    B64.encode(key)
}

/// Import a base64-encoded group key.
pub fn import_group_key(encoded: &str) -> Result<Vec<u8>> {
    let bytes = B64.decode(encoded)
        .map_err(|e| CryptoError::InvalidKey(format!("invalid base64: {e}")))?;
    if bytes.len() != 32 {
        return Err(CryptoError::InvalidKey(format!(
            "group key must be 32 bytes, got {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

// --- Fingerprinting ---

/// Compute a SHA-256 fingerprint of a JWK public key value.
/// Normalizes to identity-relevant fields ({crv, kty, x, y} for EC keys) for
/// cross-platform interop with the JS implementation.
/// Returns hex string in groups of 4 for display.
pub fn fingerprint_jwk(jwk: &serde_json::Value) -> String {
    use sha2::Digest;
    // Normalize to identity-relevant fields for deterministic fingerprinting
    let normalized = match jwk.get("kty").and_then(|v| v.as_str()) {
        Some("EC") => serde_json::json!({
            "crv": jwk.get("crv").cloned().unwrap_or(serde_json::Value::Null),
            "kty": "EC",
            "x": jwk.get("x").cloned().unwrap_or(serde_json::Value::Null),
            "y": jwk.get("y").cloned().unwrap_or(serde_json::Value::Null),
        }),
        _ => jwk.clone(),
    };
    let canonical = canonical_json(&normalized);
    let hash = Sha256::digest(canonical.as_bytes());
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    hex.as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Constant-time byte comparison to prevent timing side-channels.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Compute a SHA-256 fingerprint of raw SEC1 public key bytes.
/// Note: This does NOT match the JS fingerprint (which uses JWK JSON).
/// Use `fingerprint_jwk` for cross-platform interop.
pub fn fingerprint(public_key: &[u8]) -> String {
    use sha2::Digest;
    let hash = Sha256::digest(public_key);
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    hex.as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Produce a canonical JSON string with sorted top-level keys.
/// Matches JavaScript's `JSON.stringify(obj, Object.keys(obj).sort())`.
fn canonical_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let entries: Vec<String> = keys.iter().map(|k| {
                // Use serde_json to properly escape key strings
                let escaped_key = serde_json::to_string(k).unwrap_or_else(|_| format!("\"{}\"", k));
                format!("{}:{}", escaped_key, canonical_json(&map[*k]))
            }).collect();
            format!("{{{}}}", entries.join(","))
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(canonical_json).collect();
            format!("[{}]", items.join(","))
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_round_trip() {
        let key = generate_group_key();
        let plaintext = b"Hello, world!";
        let (ciphertext, iv) = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &ciphertext, &iv).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes_different_iv_each_time() {
        let key = generate_group_key();
        let plaintext = b"Same message";
        let (ct1, iv1) = encrypt(&key, plaintext).unwrap();
        let (ct2, iv2) = encrypt(&key, plaintext).unwrap();
        assert_ne!(iv1, iv2);
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn aes_wrong_key_fails() {
        let key1 = generate_group_key();
        let key2 = generate_group_key();
        let (ciphertext, iv) = encrypt(&key1, b"secret").unwrap();
        assert!(decrypt(&key2, &ciphertext, &iv).is_err());
    }

    #[test]
    fn aes_empty_plaintext() {
        let key = generate_group_key();
        let (ciphertext, iv) = encrypt(&key, b"").unwrap();
        let decrypted = decrypt(&key, &ciphertext, &iv).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn ecdh_key_pair_generation() {
        let kp = generate_ecdh_key_pair();
        assert!(!kp.public_key.is_empty());
        assert!(!kp.private_key.is_empty());
        assert_eq!(kp.private_key.len(), 32);
    }

    #[test]
    fn ecdh_shared_secret_symmetric() {
        let kp_a = generate_ecdh_key_pair();
        let kp_b = generate_ecdh_key_pair();
        let shared_ab = derive_shared_key(&kp_a.private_key, &kp_b.public_key, None).unwrap();
        let shared_ba = derive_shared_key(&kp_b.private_key, &kp_a.public_key, None).unwrap();
        assert_eq!(shared_ab, shared_ba);
    }

    #[test]
    fn ecdh_cross_encrypt_decrypt() {
        let kp_a = generate_ecdh_key_pair();
        let kp_b = generate_ecdh_key_pair();
        let shared_ab = derive_shared_key(&kp_a.private_key, &kp_b.public_key, None).unwrap();
        let shared_ba = derive_shared_key(&kp_b.private_key, &kp_a.public_key, None).unwrap();

        let (ct, iv) = encrypt(&shared_ab, b"test secret").unwrap();
        let pt = decrypt(&shared_ba, &ct, &iv).unwrap();
        assert_eq!(pt, b"test secret");
    }

    #[test]
    fn ecdh_different_info_different_keys() {
        let kp_a = generate_ecdh_key_pair();
        let kp_b = generate_ecdh_key_pair();
        let key1 = derive_shared_key(&kp_a.private_key, &kp_b.public_key, Some("domain-1")).unwrap();
        let key2 = derive_shared_key(&kp_a.private_key, &kp_b.public_key, Some("domain-2")).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn ecdsa_sign_verify() {
        let kp = generate_signing_key_pair();
        let data = b"message to sign";
        let sig = sign(&kp.private_key, data).unwrap();
        assert!(verify(&kp.public_key, data, &sig).unwrap());
    }

    #[test]
    fn ecdsa_wrong_key_fails() {
        let kp1 = generate_signing_key_pair();
        let kp2 = generate_signing_key_pair();
        let sig = sign(&kp1.private_key, b"message").unwrap();
        assert!(!verify(&kp2.public_key, b"message", &sig).unwrap());
    }

    #[test]
    fn ecdsa_tampered_data_fails() {
        let kp = generate_signing_key_pair();
        let sig = sign(&kp.private_key, b"original").unwrap();
        assert!(!verify(&kp.public_key, b"tampered", &sig).unwrap());
    }

    #[test]
    fn key_export_import_round_trip() {
        let key = generate_group_key();
        let exported = export_group_key(&key);
        let imported = import_group_key(&exported).unwrap();
        assert_eq!(key, imported);
    }

    #[test]
    fn public_key_export_import_round_trip() {
        let kp = generate_ecdh_key_pair();
        let exported = export_public_key(&kp.public_key);
        let imported = import_public_key(&exported).unwrap();
        assert_eq!(kp.public_key, imported);
    }

    #[test]
    fn jwk_public_key_round_trip() {
        let kp = generate_ecdh_key_pair();
        let jwk = public_key_to_jwk(&kp.public_key).unwrap();
        let recovered = jwk_to_public_key(&jwk).unwrap();
        assert_eq!(kp.public_key, recovered);
    }

    #[test]
    fn jwk_group_key_round_trip() {
        let key = generate_group_key();
        let jwk = group_key_to_jwk(&key).unwrap();
        let recovered = jwk_to_group_key(&jwk).unwrap();
        assert_eq!(key, recovered);
    }

    #[test]
    fn fingerprint_jwk_consistent() {
        let kp = generate_ecdh_key_pair();
        let jwk = public_key_to_jwk(&kp.public_key).unwrap();
        let fp1 = fingerprint_jwk(&jwk);
        let fp2 = fingerprint_jwk(&jwk);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn fingerprint_jwk_different_keys() {
        let kp1 = generate_ecdh_key_pair();
        let kp2 = generate_ecdh_key_pair();
        let fp1 = fingerprint_jwk(&public_key_to_jwk(&kp1.public_key).unwrap());
        let fp2 = fingerprint_jwk(&public_key_to_jwk(&kp2.public_key).unwrap());
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn fingerprint_format() {
        let kp = generate_ecdh_key_pair();
        let fp = fingerprint(&kp.public_key);
        let groups: Vec<&str> = fp.split(' ').collect();
        assert_eq!(groups.len(), 16);
        for g in &groups {
            assert_eq!(g.len(), 4);
            assert!(g.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn canonical_json_sorts_keys() {
        let val = serde_json::json!({"z": 1, "a": 2, "m": 3});
        let result = canonical_json(&val);
        assert_eq!(result, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn canonical_json_escapes_special_chars() {
        let val = serde_json::json!({"key\"with\\special": "val"});
        let result = canonical_json(&val);
        assert_eq!(result, r#"{"key\"with\\special":"val"}"#);
    }
}
