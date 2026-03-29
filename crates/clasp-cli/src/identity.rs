//! Identity subcommands: show all identity formats from an Ed25519 key file,
//! generate new identity keys.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use clasp_identity::Identity;

/// Display all identity formats derived from an Ed25519 signing key file.
pub fn handle_show(key_path: &Path) -> Result<()> {
    let signing_key = crate::load_signing_key(key_path)?;
    let identity = Identity::from_signing_key(signing_key);

    println!("{}: {}", "EntityId".cyan(), identity.entity_id());
    println!("{}: {}", "DID".cyan(), identity.did());
    println!("{}: {}", "PeerID".cyan(), identity.peer_id());
    println!(
        "{}: {}",
        "Public key".cyan(),
        crate::hex_encode(identity.public_key())
    );

    #[cfg(feature = "identity-defra")]
    {
        use clasp_identity::DefraIdentity;
        match DefraIdentity::derive_from(&identity) {
            Ok(defra) => {
                println!("{}: {}", "DefraDB key".cyan(), defra.to_hex());
            }
            Err(e) => {
                eprintln!(
                    "{} DefraDB derivation failed: {}",
                    "WARN".yellow().bold(),
                    e
                );
            }
        }
    }

    Ok(())
}

/// Generate a new Ed25519 identity key and display all derived formats.
pub fn handle_generate(out: Option<&Path>) -> Result<()> {
    let identity = Identity::generate();
    let hex_key = crate::hex_encode(&identity.export_secret());

    if let Some(path) = out {
        crate::write_secret_file(path, hex_key.as_bytes())
            .with_context(|| format!("Failed to write key file: {}", path.display()))?;

        eprintln!("{} Identity saved to: {}", "OK".green().bold(), path.display());
    } else {
        println!("{}", hex_key);
    }

    eprintln!("  {}: {}", "EntityId".cyan(), identity.entity_id());
    eprintln!("  {}: {}", "DID".cyan(), identity.did());
    eprintln!("  {}: {}", "PeerID".cyan(), identity.peer_id());
    eprintln!(
        "  {}: {}",
        "Public key".cyan(),
        crate::hex_encode(identity.public_key())
    );

    Ok(())
}
