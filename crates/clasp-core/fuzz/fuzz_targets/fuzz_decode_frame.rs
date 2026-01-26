//! Fuzz target for Frame::decode
//!
//! Tests that the frame decoder handles arbitrary byte sequences without panicking.

#![no_main]

use bytes::Bytes;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Attempt to decode random bytes as a frame
    // This should either succeed with a valid frame or return an error,
    // but should never panic or crash
    let bytes = Bytes::copy_from_slice(data);
    let _ = clasp_core::Frame::decode(bytes);
});
