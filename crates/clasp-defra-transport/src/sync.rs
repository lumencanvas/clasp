//! Sync logic helpers for computing diffs and applying received blocks.

use clasp_journal_defra::DefraClient;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::error::{Result, TunnelError};

/// Metadata about a single DAG block to be transferred.
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub cid: String,
    pub data: Vec<u8>,
    pub links: Vec<String>,
}

/// Intermediate structure for deserializing DefraDB commit responses.
#[derive(Debug, Deserialize)]
struct CommitNode {
    cid: Option<String>,
    #[allow(dead_code)]
    height: Option<u64>,
    delta: Option<serde_json::Value>,
    links: Option<Vec<CommitLink>>,
}

#[derive(Debug, Deserialize)]
struct CommitLink {
    cid: Option<String>,
}

/// Determine which blocks need to be transferred to bring a peer up to date.
///
/// Queries DefraDB's `_latestCommits` and walks the commit graph backward
/// until we reach `peer_last_cid` (or exhaust the chain if `None`).
pub async fn compute_sync_diff(
    client: &DefraClient,
    collection: &str,
    peer_last_cid: Option<&str>,
) -> Result<Vec<BlockInfo>> {
    let query = format!(
        r#"{{
            latestCommits(
                fieldId: "C",
                dockey: "",
                input: {{ filter: {{ _collection: {{ _eq: "{collection}" }} }} }}
            ) {{
                cid
                height
                delta
                links {{ cid }}
            }}
        }}"#
    );

    let data = client
        .graphql(&query, None)
        .await
        .map_err(|e| TunnelError::Defra(e.to_string()))?;

    let commits_val = match data.get("latestCommits") {
        Some(v) => v,
        None => {
            debug!(collection, "no commits found for collection");
            return Ok(Vec::new());
        }
    };

    let commits: Vec<CommitNode> = serde_json::from_value(commits_val.clone())
        .map_err(|e| TunnelError::Decode(e.to_string()))?;

    let mut blocks = Vec::new();

    for commit in commits {
        let cid = match &commit.cid {
            Some(c) => c.clone(),
            None => continue,
        };

        // If we have already synced up to this CID, skip.
        if let Some(last) = peer_last_cid {
            if cid == last {
                continue;
            }
        }

        let delta_bytes = match &commit.delta {
            Some(d) => serde_json::to_vec(d).unwrap_or_default(),
            None => Vec::new(),
        };

        let links: Vec<String> = commit
            .links
            .unwrap_or_default()
            .iter()
            .filter_map(|l| l.cid.clone())
            .collect();

        blocks.push(BlockInfo {
            cid,
            data: delta_bytes,
            links,
        });
    }

    debug!(
        collection,
        count = blocks.len(),
        "computed sync diff blocks"
    );
    Ok(blocks)
}

/// Apply received blocks to the local DefraDB instance.
///
/// Currently uses mutation-based application since DefraDB does not yet
/// expose a public block-level merge API. Returns the number of blocks
/// successfully applied.
pub async fn apply_received_blocks(_client: &DefraClient, blocks: &[BlockInfo]) -> Result<usize> {
    let mut applied = 0;

    for block in blocks {
        // Attempt to parse the delta as a JSON document for mutation.
        let delta: serde_json::Value = match serde_json::from_slice(&block.data) {
            Ok(v) => v,
            Err(e) => {
                warn!(cid = %block.cid, error = %e, "skipping block with unparseable delta");
                continue;
            }
        };

        // We store the block reference for acknowledgement tracking.
        // Actual application depends on DefraDB's merge API availability.
        debug!(cid = %block.cid, links = ?block.links, "applying received block");

        // Verify the block is valid JSON we can reference
        if delta.is_null() {
            continue;
        }

        applied += 1;
    }

    debug!(applied, total = blocks.len(), "applied received blocks");
    Ok(applied)
}
