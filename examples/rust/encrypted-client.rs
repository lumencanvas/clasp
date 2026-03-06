// E2E Encrypted Client Example
//
// Demonstrates using clasp-crypto for end-to-end encryption.
// The router never holds keys or decrypts data.
//
// Usage:
//   cargo run --example encrypted-client
//
// Requires: clasp-crypto with "client" and "fs-store" features

use std::sync::Arc;
use std::time::Duration;

// This example shows the API but won't compile as a standalone file
// without the proper Cargo.toml setup. See the README for full instructions.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use MemoryKeyStore for this example. For production, use
    // FileSystemKeyStore (behind the "fs-store" feature) for persistent
    // keys across restarts:
    //
    //   let store = Arc::new(clasp_crypto::FileSystemKeyStore::new("~/.clasp-crypto"));
    let store = Arc::new(clasp_crypto::MemoryKeyStore::new());

    // Create a session directly (without CryptoClient wrapper)
    let mut session = clasp_crypto::E2ESession::new(clasp_crypto::E2ESessionConfig {
        identity_id: "rust-device-1".into(),
        base_path: "/myapp/signals".into(),
        store,
        on_key_change: Some(Arc::new(|peer, old_fp, new_fp| {
            eprintln!("WARNING: {peer} changed key!");
            eprintln!("  old: {old_fp}");
            eprintln!("  new: {new_fp}");
            // In production, prompt the user or check out-of-band
            false // reject by default
        })),
        password_hash: None,
        rotation_interval: Some(Duration::from_secs(3600)), // rotate every hour
        on_rotation: Some(Arc::new(|| {
            println!("Key rotated successfully");
        })),
        max_announcement_age: Some(Duration::from_secs(300)), // reject announcements >5min old
    });

    session.start().await?;
    session.enable_encryption().await?;

    println!("E2E encryption enabled on /myapp/signals");
    println!("  - Auto-rotation: every 1 hour");
    println!("  - Key store: in-memory (use FileSystemKeyStore for persistence)");
    println!("  - Announcement max age: 5 minutes");

    // Encrypt a value
    let envelope = session.encrypt(r#"{"fader": 0.75}"#)?;
    println!("\nEncrypted envelope: {:?}", envelope);

    // Decrypt it back
    let decrypted = session.decrypt(&envelope).await?;
    println!("Decrypted: {}", decrypted);

    // Clean up
    session.destroy();
    println!("\nSession destroyed, key material zeroed.");

    Ok(())
}
