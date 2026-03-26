//! E2E tests for DefraDB-backed state synchronization.
//!
//! Requires two DefraDB nodes running and peered:
//!   cd tests/defra && bash setup.sh
//!
//! Run with:
//!   cargo run -p clasp-e2e --bin defra-sync-tests

use anyhow::Result;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("=== DefraDB Sync E2E Tests ===\n");

    let defra1 = std::env::var("DEFRA_URL_1").unwrap_or_else(|_| "http://localhost:9181".into());
    let defra2 = std::env::var("DEFRA_URL_2").unwrap_or_else(|_| "http://localhost:9182".into());

    // Check DefraDB availability
    let client = reqwest::Client::new();
    match client.get(format!("{}/health-check", defra1)).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("[OK] DefraDB node 1 at {}", defra1);
        }
        _ => {
            println!(
                "[SKIP] DefraDB not available at {}. Run: cd tests/defra && bash setup.sh",
                defra1
            );
            return Ok(());
        }
    }
    match client.get(format!("{}/health-check", defra2)).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("[OK] DefraDB node 2 at {}", defra2);
        }
        _ => {
            println!("[SKIP] DefraDB node 2 not available at {}", defra2);
            return Ok(());
        }
    }

    // Test 1: Journal entry sync
    test_journal_sync(&defra1, &defra2).await?;

    // Test 2: Router state sync via DefraStateStore
    test_state_store_sync(&defra1, &defra2).await?;

    // Test 3: Config sync
    test_config_sync(&defra1, &defra2).await?;

    println!("\n=== All DefraDB sync tests passed ===");
    Ok(())
}

async fn test_journal_sync(defra1: &str, defra2: &str) -> Result<()> {
    use clasp_core::Value;
    use clasp_journal::entry::JournalEntry;
    use clasp_journal::Journal;
    use clasp_journal_defra::DefraJournal;

    print!("  test_journal_sync ... ");

    let journal1 = DefraJournal::connect(defra1).await?;
    let journal2 = DefraJournal::connect(defra2).await?;

    // Write on node 1
    let unique_addr = format!("/e2e/journal-sync/{}", uuid::Uuid::new_v4());
    let entry = JournalEntry::from_set(
        unique_addr.clone(),
        Value::Float(42.0),
        1,
        "e2e-writer".into(),
        clasp_core::time::now(),
    );
    let seq = journal1.append(entry).await?;

    // Wait for DefraDB P2P sync
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Read from node 2
    let entries = journal2
        .query(&unique_addr, None, None, Some(10), &[])
        .await?;

    if entries.is_empty() {
        println!("FAILED (no entries synced to node 2)");
        anyhow::bail!("Journal sync failed: no entries on node 2");
    }

    let synced = &entries[0];
    assert_eq!(synced.address, unique_addr);
    if let Value::Float(v) = &synced.value {
        assert!((v - 42.0).abs() < f64::EPSILON);
    } else {
        anyhow::bail!(
            "Wrong value type synced: expected Float, got {:?}",
            synced.value
        );
    }

    println!("ok (seq={}, synced {} entries)", seq, entries.len());
    Ok(())
}

async fn test_state_store_sync(defra1: &str, defra2: &str) -> Result<()> {
    use clasp_core::Value;
    use clasp_state_defra::{DefraStateConfig, DefraStateStore};

    print!("  test_state_store_sync ... ");

    let config1 = DefraStateConfig {
        preload: false,
        ..Default::default()
    };

    let store1 = DefraStateStore::new(defra1, config1).await?;
    let _writer1 = store1.start_writer();

    let unique_addr = format!("/e2e/state-sync/{}", uuid::Uuid::new_v4());
    store1.set(
        &unique_addr,
        Value::String("synced-from-router-1".into()),
        "e2e-session",
        None,
        false,
        false,
        None,
    )?;

    // Flush writes to DefraDB
    tokio::time::sleep(Duration::from_secs(1)).await;
    store1.flush().await?;

    // Wait for P2P sync
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Load from node 2
    let config2 = DefraStateConfig {
        preload: true,
        ..Default::default()
    };
    let store2 = DefraStateStore::new(defra2, config2).await?;

    let val = store2.get(&unique_addr);
    match val {
        Some(Value::String(s)) if s == "synced-from-router-1" => {
            println!("ok");
        }
        Some(other) => {
            println!("FAILED (wrong value: {:?})", other);
            anyhow::bail!("State store sync returned wrong value");
        }
        None => {
            println!("FAILED (not synced to node 2)");
            anyhow::bail!("State store sync failed: value not on node 2");
        }
    }

    Ok(())
}

async fn test_config_sync(defra1: &str, defra2: &str) -> Result<()> {
    use clasp_config_defra::{DefraConfigStore, RouterConfig};

    print!("  test_config_sync ... ");

    let store1 = DefraConfigStore::new(defra1).await?;
    let store2 = DefraConfigStore::new(defra2).await?;

    let unique_id = format!(
        "e2e-router-{}",
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    );
    let config = RouterConfig::new(&unique_id, "E2E Sync Test Router", "e2e-owner");
    store1.save_router(&config).await?;

    // Wait for DefraDB P2P sync
    tokio::time::sleep(Duration::from_secs(3)).await;

    let loaded = store2.get_router(&unique_id).await?;
    match loaded {
        Some(r) if r.name == "E2E Sync Test Router" => {
            println!("ok");
        }
        Some(r) => {
            println!("FAILED (wrong name: {})", r.name);
            anyhow::bail!("Config sync returned wrong name");
        }
        None => {
            println!("FAILED (not synced to node 2)");
            anyhow::bail!("Config sync failed: router not on node 2");
        }
    }

    // Cleanup
    store1.delete_router(&unique_id).await?;
    Ok(())
}
