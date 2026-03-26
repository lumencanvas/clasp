# clasp-journal-defra

DefraDB backend for the CLASP journal. Implements the `Journal` trait from `clasp-journal` using DefraDB's HTTP/GraphQL API, enabling:

- **P2P state sync**: Journal entries replicate automatically via DefraDB's Merkle CRDTs
- **GraphQL queries**: Query signal history with filters, time ranges, and pattern matching
- **Content-addressable audit**: Every entry gets a CID (content identifier) for tamper-proof verification
- **Schema evolution**: LensVM migrations when the CLASP protocol evolves

## Usage

```rust
use clasp_journal_defra::DefraJournal;
use clasp_journal::Journal;

let journal = DefraJournal::connect("http://localhost:9181").await?;

// Append entries (fire-and-forget from router hot path)
journal.append(entry).await?;

// Query history
let entries = journal.query("/lumen/**", Some(from_ts), None, Some(100), &[]).await?;

// Crash recovery
let entries = journal.since(last_known_seq, None).await?;
```

## Architecture

The journal stores two DefraDB collection types:

- `ClaspJournalEntry`: Individual state mutations (SET, PUBLISH) with sequence numbers
- `ClaspParamSnapshot`: Point-in-time parameter snapshots for fast recovery

Sequence numbers are router-local (AtomicU64). Cross-router sync uses DefraDB's native Merkle CRDT replication.

## DefraDB API

Communicates via HTTP/GraphQL at `{base_url}/api/v0/graphql`. Schemas are provisioned automatically on startup via `POST /api/v0/collections`.

Also exports `DefraClient` for reuse by other DefraDB integration crates.

## License

MIT OR Apache-2.0
