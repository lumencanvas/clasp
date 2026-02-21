//! Delegatable capability tokens for CLASP
//!
//! Implements UCAN-inspired capability tokens where each token in a
//! delegation chain can only narrow (attenuate) scopes, never widen.
//!
//! Tokens use Ed25519 signatures and can be chained:
//!
//! ```text
//! Root token:   admin:/**
//!   -> Child:   write:/lights/**          (valid: admin allows write)
//!     -> Grand: write:/lights/room1/**    (valid: narrower pattern)
//!       -> Bad: write:/audio/**           (rejected: not subset of /lights/**)
//! ```
//!
//! # Token Format
//!
//! `cap_<base64url(messagepack(CapabilityToken))>`
//!
//! # Integration
//!
//! Add to `ValidatorChain` alongside existing CPSK tokens:
//!
//! ```no_run
//! use clasp_caps::{CapabilityValidator, CapabilityToken};
//! use ed25519_dalek::SigningKey;
//!
//! // Create validator with trusted root key
//! let root_key = SigningKey::from_bytes(&[1u8; 32]);
//! let pub_key = root_key.verifying_key().to_bytes().to_vec();
//! let validator = CapabilityValidator::new(vec![pub_key], 5);
//!
//! // Use with ValidatorChain
//! // chain.add(validator);
//! ```

pub mod error;
pub mod token;
pub mod validator;

pub use error::{CapError, Result};
pub use token::{CapabilityToken, ProofLink};
pub use validator::CapabilityValidator;
