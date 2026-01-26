//! Fuzz target for decode_message
//!
//! Tests that the message decoder handles arbitrary byte sequences without panicking.
//! This target uses a valid frame header prefix to increase coverage of the message parsing code.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Direct message decoding from raw bytes
    let _ = clasp_core::codec::decode_message(data);

    // If we have enough data, try with valid frame header prefix
    if !data.is_empty() {
        // Create a frame with valid header + fuzzed payload
        let payload_len = data.len().min(65535);
        let mut frame_data = Vec::with_capacity(4 + payload_len);

        // Valid frame header: magic (0x53), flags (0x01 for binary), length
        frame_data.push(0x53); // Magic byte 'S'
        frame_data.push(0x01); // Flags: binary encoding
        frame_data.extend_from_slice(&(payload_len as u16).to_be_bytes());
        frame_data.extend_from_slice(&data[..payload_len]);

        // Try to decode the constructed frame
        let _ = clasp_core::codec::decode(&frame_data);
    }

    // Also try decoding with various message type prefixes
    // to exercise different message parsing paths
    for msg_type in [
        0x01, // HELLO
        0x02, // WELCOME
        0x03, // ANNOUNCE
        0x10, // SUBSCRIBE
        0x11, // UNSUBSCRIBE
        0x20, // PUBLISH
        0x21, // SET
        0x22, // GET
        0x23, // SNAPSHOT
        0x30, // BUNDLE
        0x40, // SYNC
        0x41, // PING
        0x42, // PONG
        0x50, // ACK
        0x51, // ERROR
        0x60, // QUERY
        0x61, // RESULT
    ] {
        if !data.is_empty() {
            let mut msg_data = vec![msg_type];
            msg_data.extend_from_slice(data);
            let _ = clasp_core::codec::decode_message(&msg_data);
        }
    }
});
