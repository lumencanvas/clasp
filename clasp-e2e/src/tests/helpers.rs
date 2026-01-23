//! Test helpers for E2E tests
//!
//! This module re-exports utilities from clasp-test-utils and adds
//! E2E-specific functionality like TestResult and TestResultBuilder.

use crate::TestResult;
use std::time::{Duration, Instant};
use tokio::time::timeout;

// Re-export everything from clasp-test-utils
pub use clasp_test_utils::*;

/// Run a test with timeout and capture results
pub async fn run_test<F, Fut>(name: &str, timeout_duration: Duration, test_fn: F) -> TestResult
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let start = Instant::now();

    let result = match timeout(timeout_duration, test_fn()).await {
        Ok(Ok(())) => TestResult {
            name: name.to_string(),
            passed: true,
            duration: start.elapsed(),
            message: None,
        },
        Ok(Err(e)) => TestResult {
            name: name.to_string(),
            passed: false,
            duration: start.elapsed(),
            message: Some(e),
        },
        Err(_) => TestResult {
            name: name.to_string(),
            passed: false,
            duration: start.elapsed(),
            message: Some(format!("Test timed out after {:?}", timeout_duration)),
        },
    };

    result
}

// ============================================================================
// Test Result Builder (E2E specific, uses crate::TestResult)
// ============================================================================

pub struct TestResultBuilder {
    name: &'static str,
    start: Instant,
}

impl TestResultBuilder {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }

    pub fn pass(self) -> crate::TestResult {
        crate::TestResult {
            name: self.name.to_string(),
            passed: true,
            duration: self.start.elapsed(),
            message: None,
        }
    }

    pub fn fail(self, msg: impl Into<String>) -> crate::TestResult {
        crate::TestResult {
            name: self.name.to_string(),
            passed: false,
            duration: self.start.elapsed(),
            message: Some(msg.into()),
        }
    }

    pub fn from_result(self, result: Result<(), String>) -> crate::TestResult {
        match result {
            Ok(()) => self.pass(),
            Err(msg) => self.fail(msg),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}
