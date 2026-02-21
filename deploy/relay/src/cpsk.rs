//! CPSK token generation, file management, and shared validator wrapper.
//!
//! Provides [`SharedValidator`] so the same `CpskValidator` instance can be
//! used by both the router (token validation) and the auth HTTP API (token
//! registration). Also includes [`write_secret_file`] for safe credential I/O.

use clasp_core::security::{CpskValidator, TokenValidator, ValidationResult};
use std::sync::Arc;

/// Write a file containing sensitive data with restrictive permissions.
///
/// On Unix, the file is created with mode 0o600 from the start (via OpenOptions),
/// avoiding the TOCTOU window where `write()` + `set_permissions()` could briefly
/// expose the file with default permissions.
/// See pentest ADM-06: Admin Token File Exposure
pub fn write_secret_file(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(data)?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, data)?;
        tracing::warn!(
            "Non-Unix platform: file {} may not have restrictive permissions",
            path.display()
        );
    }
    Ok(())
}

/// Wrapper to share a CpskValidator between the router and auth module.
/// Both hold Arc<CpskValidator> pointing to the same instance.
pub struct SharedValidator(pub Arc<CpskValidator>);

impl TokenValidator for SharedValidator {
    fn validate(&self, token: &str) -> ValidationResult {
        self.0.validate(token)
    }
    fn name(&self) -> &str {
        self.0.name()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
