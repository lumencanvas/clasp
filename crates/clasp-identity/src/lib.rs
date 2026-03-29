//! Unified identity for CLASP, DID, and libp2p.
//!
//! A single Ed25519 keypair produces three interoperable identifiers:
//! - CLASP `EntityId` (`clasp:<base58>`)
//! - DID (`did:key:z6Mk...`)
//! - libp2p PeerID (`12D3KooW...`)

pub mod did;
pub mod error;
pub mod identity;
pub mod peer_id;

#[cfg(feature = "secp256k1")]
pub mod defra_identity;

pub use did::{did_to_public_key, public_key_to_did};
pub use error::{IdentityError, Result};
pub use identity::Identity;
pub use peer_id::{peer_id_to_public_key, public_key_to_peer_id};

#[cfg(feature = "secp256k1")]
pub use defra_identity::DefraIdentity;
