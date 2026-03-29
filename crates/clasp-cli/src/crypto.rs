//! Crypto subcommands: generate ECDH keypairs, show fingerprints.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

/// Generate an ECDH P-256 keypair. Saves private key to file (or prints to
/// stdout), prints public key hex to stderr.
pub fn handle_keygen(out: Option<&Path>) -> Result<()> {
    let kp = clasp_crypto::generate_ecdh_key_pair();
    let priv_hex = crate::hex_encode(&kp.private_key);
    let pub_hex = crate::hex_encode(&kp.public_key);

    if let Some(path) = out {
        crate::write_secret_file(path, priv_hex.as_bytes())
            .with_context(|| format!("Failed to write key file: {}", path.display()))?;

        eprintln!(
            "{} ECDH private key saved to: {}",
            "OK".green().bold(),
            path.display()
        );
    } else {
        println!("{}", priv_hex);
    }

    eprintln!(
        "  {}: {}",
        "Public key (SEC1 hex)".cyan(),
        pub_hex,
    );
    eprintln!(
        "  {}: {}",
        "Fingerprint".cyan(),
        clasp_crypto::fingerprint(&kp.public_key),
    );

    Ok(())
}

/// Compute SHA-256 fingerprint of a public key. Accepts either a hex string
/// or a file path containing hex bytes.
pub fn handle_fingerprint(key: &str) -> Result<()> {
    let bytes = if Path::new(key).is_file() {
        let hex_str = std::fs::read_to_string(key)
            .with_context(|| format!("Failed to read key file: {}", key))?;
        crate::hex_decode(hex_str.trim())?
    } else {
        crate::hex_decode(key).context("Invalid hex string for public key")?
    };

    let fp = clasp_crypto::fingerprint(&bytes);
    println!("{}", fp);

    Ok(())
}
