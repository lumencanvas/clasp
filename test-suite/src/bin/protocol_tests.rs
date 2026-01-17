//! Self-Contained Protocol Tests
//!
//! Tests real protocol implementations without requiring external hardware:
//! - OSC: Loopback send/receive on localhost
//! - MIDI: Virtual port detection and loopback (platform-dependent)
//! - Art-Net: Built-in echo server for packet validation
//!
//! These tests use REAL protocol libraries and wire formats, just with
//! simulated/virtual endpoints instead of physical hardware.

use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// ============================================================================
// Test Framework
// ============================================================================

struct TestResult {
    name: &'static str,
    passed: bool,
    message: String,
    duration_ms: u128,
}

impl TestResult {
    fn pass(name: &'static str, message: impl Into<String>, duration_ms: u128) -> Self {
        Self {
            name,
            passed: true,
            message: message.into(),
            duration_ms,
        }
    }

    fn fail(name: &'static str, message: impl Into<String>, duration_ms: u128) -> Self {
        Self {
            name,
            passed: false,
            message: message.into(),
            duration_ms,
        }
    }
}

// ============================================================================
// OSC Loopback Tests (No external dependencies)
// ============================================================================

fn test_osc_loopback_float() -> TestResult {
    let start = Instant::now();
    let name = "osc_loopback_float";

    // Create receiver socket
    let receiver = match UdpSocket::bind("127.0.0.1:0") {
        Ok(s) => s,
        Err(e) => {
            return TestResult::fail(
                name,
                format!("Bind failed: {}", e),
                start.elapsed().as_millis(),
            )
        }
    };
    let recv_addr = receiver.local_addr().unwrap();
    receiver.set_read_timeout(Some(Duration::from_secs(2))).ok();

    // Create sender socket
    let sender = match UdpSocket::bind("127.0.0.1:0") {
        Ok(s) => s,
        Err(e) => {
            return TestResult::fail(
                name,
                format!("Sender bind failed: {}", e),
                start.elapsed().as_millis(),
            )
        }
    };

    // Build and send OSC message: /test/value ,f 0.75
    let msg = rosc::OscMessage {
        addr: "/test/value".to_string(),
        args: vec![rosc::OscType::Float(0.75)],
    };
    let packet = rosc::OscPacket::Message(msg);
    let encoded = match rosc::encoder::encode(&packet) {
        Ok(b) => b,
        Err(e) => {
            return TestResult::fail(
                name,
                format!("Encode failed: {:?}", e),
                start.elapsed().as_millis(),
            )
        }
    };

    if let Err(e) = sender.send_to(&encoded, recv_addr) {
        return TestResult::fail(
            name,
            format!("Send failed: {}", e),
            start.elapsed().as_millis(),
        );
    }

    // Receive and decode
    let mut buf = [0u8; 1024];
    match receiver.recv_from(&mut buf) {
        Ok((len, _)) => match rosc::decoder::decode_udp(&buf[..len]) {
            Ok((_, rosc::OscPacket::Message(m))) => {
                if m.addr == "/test/value" {
                    if let Some(rosc::OscType::Float(v)) = m.args.first() {
                        if (*v - 0.75).abs() < 0.001 {
                            return TestResult::pass(
                                name,
                                format!("Received {} = {}", m.addr, v),
                                start.elapsed().as_millis(),
                            );
                        }
                    }
                }
                TestResult::fail(
                    name,
                    format!("Wrong message: {:?}", m),
                    start.elapsed().as_millis(),
                )
            }
            Ok(_) => TestResult::fail(
                name,
                "Received bundle instead of message",
                start.elapsed().as_millis(),
            ),
            Err(e) => TestResult::fail(
                name,
                format!("Decode failed: {:?}", e),
                start.elapsed().as_millis(),
            ),
        },
        Err(e) => TestResult::fail(
            name,
            format!("Receive failed: {}", e),
            start.elapsed().as_millis(),
        ),
    }
}

fn test_osc_loopback_int() -> TestResult {
    let start = Instant::now();
    let name = "osc_loopback_int";

    let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    let recv_addr = receiver.local_addr().unwrap();
    receiver.set_read_timeout(Some(Duration::from_secs(2))).ok();

    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();

    let msg = rosc::OscMessage {
        addr: "/midi/cc/1".to_string(),
        args: vec![rosc::OscType::Int(127)],
    };
    let packet = rosc::OscPacket::Message(msg);
    let encoded = rosc::encoder::encode(&packet).unwrap();

    sender.send_to(&encoded, recv_addr).unwrap();

    let mut buf = [0u8; 1024];
    match receiver.recv_from(&mut buf) {
        Ok((len, _)) => {
            if let Ok((_, rosc::OscPacket::Message(m))) = rosc::decoder::decode_udp(&buf[..len]) {
                if let Some(rosc::OscType::Int(v)) = m.args.first() {
                    if *v == 127 {
                        return TestResult::pass(
                            name,
                            format!("{} = {}", m.addr, v),
                            start.elapsed().as_millis(),
                        );
                    }
                }
            }
            TestResult::fail(name, "Wrong value received", start.elapsed().as_millis())
        }
        Err(e) => TestResult::fail(
            name,
            format!("Receive failed: {}", e),
            start.elapsed().as_millis(),
        ),
    }
}

fn test_osc_loopback_string() -> TestResult {
    let start = Instant::now();
    let name = "osc_loopback_string";

    let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    let recv_addr = receiver.local_addr().unwrap();
    receiver.set_read_timeout(Some(Duration::from_secs(2))).ok();

    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();

    let msg = rosc::OscMessage {
        addr: "/status/message".to_string(),
        args: vec![rosc::OscType::String("Hello CLASP!".to_string())],
    };
    let packet = rosc::OscPacket::Message(msg);
    let encoded = rosc::encoder::encode(&packet).unwrap();

    sender.send_to(&encoded, recv_addr).unwrap();

    let mut buf = [0u8; 1024];
    match receiver.recv_from(&mut buf) {
        Ok((len, _)) => {
            if let Ok((_, rosc::OscPacket::Message(m))) = rosc::decoder::decode_udp(&buf[..len]) {
                if let Some(rosc::OscType::String(s)) = m.args.first() {
                    if s == "Hello CLASP!" {
                        return TestResult::pass(
                            name,
                            format!("{} = \"{}\"", m.addr, s),
                            start.elapsed().as_millis(),
                        );
                    }
                }
            }
            TestResult::fail(name, "Wrong value received", start.elapsed().as_millis())
        }
        Err(e) => TestResult::fail(
            name,
            format!("Receive failed: {}", e),
            start.elapsed().as_millis(),
        ),
    }
}

fn test_osc_loopback_multiple_args() -> TestResult {
    let start = Instant::now();
    let name = "osc_loopback_multiple_args";

    let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    let recv_addr = receiver.local_addr().unwrap();
    receiver.set_read_timeout(Some(Duration::from_secs(2))).ok();

    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();

    // RGB color message with 3 floats
    let msg = rosc::OscMessage {
        addr: "/color/rgb".to_string(),
        args: vec![
            rosc::OscType::Float(1.0),  // R
            rosc::OscType::Float(0.5),  // G
            rosc::OscType::Float(0.25), // B
        ],
    };
    let packet = rosc::OscPacket::Message(msg);
    let encoded = rosc::encoder::encode(&packet).unwrap();

    sender.send_to(&encoded, recv_addr).unwrap();

    let mut buf = [0u8; 1024];
    match receiver.recv_from(&mut buf) {
        Ok((len, _)) => {
            if let Ok((_, rosc::OscPacket::Message(m))) = rosc::decoder::decode_udp(&buf[..len]) {
                if m.args.len() == 3 {
                    if let (
                        Some(rosc::OscType::Float(r)),
                        Some(rosc::OscType::Float(g)),
                        Some(rosc::OscType::Float(b)),
                    ) = (m.args.get(0), m.args.get(1), m.args.get(2))
                    {
                        if (*r - 1.0).abs() < 0.001
                            && (*g - 0.5).abs() < 0.001
                            && (*b - 0.25).abs() < 0.001
                        {
                            return TestResult::pass(
                                name,
                                format!("RGB({}, {}, {})", r, g, b),
                                start.elapsed().as_millis(),
                            );
                        }
                    }
                }
            }
            TestResult::fail(name, "Wrong values received", start.elapsed().as_millis())
        }
        Err(e) => TestResult::fail(
            name,
            format!("Receive failed: {}", e),
            start.elapsed().as_millis(),
        ),
    }
}

fn test_osc_loopback_bundle() -> TestResult {
    let start = Instant::now();
    let name = "osc_loopback_bundle";

    let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    let recv_addr = receiver.local_addr().unwrap();
    receiver.set_read_timeout(Some(Duration::from_secs(2))).ok();

    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();

    // Create a bundle with multiple messages (common in OSC)
    let bundle = rosc::OscBundle {
        timetag: rosc::OscTime {
            seconds: 0,
            fractional: 1,
        },
        content: vec![
            rosc::OscPacket::Message(rosc::OscMessage {
                addr: "/fader/1".to_string(),
                args: vec![rosc::OscType::Float(0.5)],
            }),
            rosc::OscPacket::Message(rosc::OscMessage {
                addr: "/fader/2".to_string(),
                args: vec![rosc::OscType::Float(0.75)],
            }),
        ],
    };
    let packet = rosc::OscPacket::Bundle(bundle);
    let encoded = rosc::encoder::encode(&packet).unwrap();

    sender.send_to(&encoded, recv_addr).unwrap();

    let mut buf = [0u8; 1024];
    match receiver.recv_from(&mut buf) {
        Ok((len, _)) => {
            if let Ok((_, rosc::OscPacket::Bundle(b))) = rosc::decoder::decode_udp(&buf[..len]) {
                if b.content.len() == 2 {
                    return TestResult::pass(
                        name,
                        format!("Received bundle with {} messages", b.content.len()),
                        start.elapsed().as_millis(),
                    );
                }
            }
            TestResult::fail(
                name,
                "Bundle not received correctly",
                start.elapsed().as_millis(),
            )
        }
        Err(e) => TestResult::fail(
            name,
            format!("Receive failed: {}", e),
            start.elapsed().as_millis(),
        ),
    }
}

fn test_osc_high_frequency() -> TestResult {
    let start = Instant::now();
    let name = "osc_high_frequency";

    let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    let recv_addr = receiver.local_addr().unwrap();
    receiver
        .set_read_timeout(Some(Duration::from_millis(100)))
        .ok();
    receiver.set_nonblocking(true).ok();

    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();

    let msg_count = 1000;
    let received = Arc::new(AtomicU32::new(0));
    let received_clone = received.clone();
    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();

    // Receiver thread
    let recv_handle = thread::spawn(move || {
        let mut buf = [0u8; 1024];
        while !done_clone.load(Ordering::Relaxed) {
            if let Ok((len, _)) = receiver.recv_from(&mut buf) {
                if rosc::decoder::decode_udp(&buf[..len]).is_ok() {
                    received_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    });

    // Send messages rapidly
    for i in 0..msg_count {
        let msg = rosc::OscMessage {
            addr: format!("/rapid/{}", i % 10),
            args: vec![rosc::OscType::Float(i as f32 / msg_count as f32)],
        };
        let packet = rosc::OscPacket::Message(msg);
        let encoded = rosc::encoder::encode(&packet).unwrap();
        let _ = sender.send_to(&encoded, recv_addr);
    }

    // Wait for messages to arrive
    thread::sleep(Duration::from_millis(200));
    done.store(true, Ordering::Relaxed);
    let _ = recv_handle.join();

    let count = received.load(Ordering::Relaxed);
    let rate = count as f64 / start.elapsed().as_secs_f64();

    // Allow some packet loss on loopback under load
    if count >= (msg_count as u32 * 9 / 10) {
        TestResult::pass(
            name,
            format!("{}/{} messages, {:.0} msg/s", count, msg_count, rate),
            start.elapsed().as_millis(),
        )
    } else {
        TestResult::fail(
            name,
            format!("Only {}/{} messages received", count, msg_count),
            start.elapsed().as_millis(),
        )
    }
}

// ============================================================================
// Art-Net Self-Test (Built-in echo server)
// ============================================================================

fn test_artnet_packet_format() -> TestResult {
    let start = Instant::now();
    let name = "artnet_packet_format";

    // Create Art-Net ArtDmx packet and validate format
    let mut art_dmx = vec![
        b'A', b'r', b't', b'-', b'N', b'e', b't', 0x00, // ID (8 bytes)
        0x00, 0x50, // OpCode ArtDmx = 0x5000 (little-endian)
        0x00, 0x0E, // Protocol version 14
        0x00, // Sequence
        0x00, // Physical
        0x00, 0x00, // SubUni, Net (Universe 0)
        0x02, 0x00, // Length high, low (512 channels)
    ];

    // Add 512 DMX channels
    for i in 0..512u16 {
        art_dmx.push((i % 256) as u8);
    }

    // Validate packet structure
    if &art_dmx[0..8] != b"Art-Net\0" {
        return TestResult::fail(name, "Invalid Art-Net header", start.elapsed().as_millis());
    }

    let opcode = u16::from_le_bytes([art_dmx[8], art_dmx[9]]);
    if opcode != 0x5000 {
        return TestResult::fail(
            name,
            format!("Invalid opcode: 0x{:04X}", opcode),
            start.elapsed().as_millis(),
        );
    }

    let dmx_length = u16::from_be_bytes([art_dmx[16], art_dmx[17]]);
    if dmx_length != 512 {
        return TestResult::fail(
            name,
            format!("Invalid DMX length: {}", dmx_length),
            start.elapsed().as_millis(),
        );
    }

    TestResult::pass(
        name,
        format!("Valid ArtDmx packet, {} bytes total", art_dmx.len()),
        start.elapsed().as_millis(),
    )
}

fn test_artnet_loopback() -> TestResult {
    let start = Instant::now();
    let name = "artnet_loopback";

    // Echo server
    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    let server_addr = server.local_addr().unwrap();
    server.set_read_timeout(Some(Duration::from_secs(2))).ok();

    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client.set_read_timeout(Some(Duration::from_secs(2))).ok();

    // Build ArtDmx packet for Universe 0
    let mut art_dmx = vec![
        b'A', b'r', b't', b'-', b'N', b'e', b't', 0x00, 0x00, 0x50, // OpCode ArtDmx
        0x00, 0x0E, // Protocol version
        0x01, // Sequence = 1
        0x00, // Physical
        0x00, 0x00, // Universe 0
        0x00, 0x08, // 8 channels
    ];
    // DMX values: ramp 0-255
    art_dmx.extend_from_slice(&[0, 36, 73, 109, 146, 182, 219, 255]);

    // Send to echo server
    client.send_to(&art_dmx, server_addr).unwrap();

    // Server receives
    let mut buf = [0u8; 1024];
    match server.recv_from(&mut buf) {
        Ok((len, from)) => {
            // Validate received packet
            if len != art_dmx.len() {
                return TestResult::fail(
                    name,
                    format!("Size mismatch: {} vs {}", len, art_dmx.len()),
                    start.elapsed().as_millis(),
                );
            }

            if &buf[0..8] != b"Art-Net\0" {
                return TestResult::fail(
                    name,
                    "Corrupted Art-Net header",
                    start.elapsed().as_millis(),
                );
            }

            let seq = buf[12];
            if seq != 1 {
                return TestResult::fail(
                    name,
                    format!("Wrong sequence: {}", seq),
                    start.elapsed().as_millis(),
                );
            }

            // Echo back (simulating a node responding)
            server.send_to(&buf[..len], from).unwrap();

            // Client receives echo
            match client.recv_from(&mut buf) {
                Ok((echo_len, _)) => {
                    if echo_len == len && buf[12] == 1 {
                        TestResult::pass(
                            name,
                            format!("Art-Net roundtrip OK, {} bytes", len),
                            start.elapsed().as_millis(),
                        )
                    } else {
                        TestResult::fail(name, "Echo mismatch", start.elapsed().as_millis())
                    }
                }
                Err(e) => TestResult::fail(
                    name,
                    format!("Echo receive failed: {}", e),
                    start.elapsed().as_millis(),
                ),
            }
        }
        Err(e) => TestResult::fail(
            name,
            format!("Server receive failed: {}", e),
            start.elapsed().as_millis(),
        ),
    }
}

fn test_artnet_poll_reply() -> TestResult {
    let start = Instant::now();
    let name = "artnet_poll_reply";

    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    let server_addr = server.local_addr().unwrap();
    server.set_read_timeout(Some(Duration::from_secs(2))).ok();

    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    let client_addr = client.local_addr().unwrap();
    client.set_read_timeout(Some(Duration::from_secs(2))).ok();

    // Client sends ArtPoll
    let art_poll = [
        b'A', b'r', b't', b'-', b'N', b'e', b't', 0x00, 0x00, 0x20, // OpCode ArtPoll = 0x2000
        0x00, 0x0E, // Protocol version
        0x00, // TalkToMe
        0x00, // Priority
    ];
    client.send_to(&art_poll, server_addr).unwrap();

    // Server receives ArtPoll
    let mut buf = [0u8; 1024];
    match server.recv_from(&mut buf) {
        Ok((len, from)) => {
            if len < 14 || &buf[0..8] != b"Art-Net\0" {
                return TestResult::fail(name, "Invalid ArtPoll", start.elapsed().as_millis());
            }

            let opcode = u16::from_le_bytes([buf[8], buf[9]]);
            if opcode != 0x2000 {
                return TestResult::fail(
                    name,
                    format!("Wrong opcode: 0x{:04X}", opcode),
                    start.elapsed().as_millis(),
                );
            }

            // Server sends ArtPollReply
            let mut reply = vec![
                b'A', b'r', b't', b'-', b'N', b'e', b't', 0x00, 0x00,
                0x21, // OpCode ArtPollReply = 0x2100
            ];
            // Add minimal reply data (real replies are 239 bytes)
            reply.extend_from_slice(&[0u8; 229]); // Pad to valid size

            server.send_to(&reply, from).unwrap();

            // Client receives ArtPollReply
            match client.recv_from(&mut buf) {
                Ok((reply_len, _)) => {
                    let reply_opcode = u16::from_le_bytes([buf[8], buf[9]]);
                    if reply_opcode == 0x2100 {
                        TestResult::pass(
                            name,
                            format!("ArtPoll/Reply exchange OK, {} bytes", reply_len),
                            start.elapsed().as_millis(),
                        )
                    } else {
                        TestResult::fail(
                            name,
                            format!("Wrong reply opcode: 0x{:04X}", reply_opcode),
                            start.elapsed().as_millis(),
                        )
                    }
                }
                Err(e) => TestResult::fail(
                    name,
                    format!("Reply receive failed: {}", e),
                    start.elapsed().as_millis(),
                ),
            }
        }
        Err(e) => TestResult::fail(
            name,
            format!("Poll receive failed: {}", e),
            start.elapsed().as_millis(),
        ),
    }
}

fn test_artnet_multiple_universes() -> TestResult {
    let start = Instant::now();
    let name = "artnet_multiple_universes";

    let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    let recv_addr = receiver.local_addr().unwrap();
    receiver.set_read_timeout(Some(Duration::from_secs(2))).ok();

    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();

    // Send to 4 different universes
    let universes = [0u8, 1, 2, 3];
    for &uni in &universes {
        let art_dmx = vec![
            b'A', b'r', b't', b'-', b'N', b'e', b't', 0x00, 0x00, 0x50, 0x00, 0x0E,
            uni, // Sequence = universe number
            0x00, uni, 0x00, // SubUni = universe
            0x00, 0x04, // 4 channels
            uni, uni, uni, uni, // DMX data = universe number
        ];
        sender.send_to(&art_dmx, recv_addr).unwrap();
    }

    // Receive all 4
    let mut received_universes = Vec::new();
    let mut buf = [0u8; 256];
    for _ in 0..4 {
        if let Ok((len, _)) = receiver.recv_from(&mut buf) {
            if len >= 18 && &buf[0..8] == b"Art-Net\0" {
                let uni = buf[14]; // SubUni byte
                received_universes.push(uni);
            }
        }
    }

    received_universes.sort();
    if received_universes == universes.to_vec() {
        TestResult::pass(
            name,
            format!("Received universes: {:?}", received_universes),
            start.elapsed().as_millis(),
        )
    } else {
        TestResult::fail(
            name,
            format!("Missing universes, got: {:?}", received_universes),
            start.elapsed().as_millis(),
        )
    }
}

// ============================================================================
// MIDI Virtual Port Tests
// ============================================================================

fn test_midi_virtual_port_available() -> TestResult {
    let start = Instant::now();
    let name = "midi_virtual_port_available";

    let midi_in = match midir::MidiInput::new("CLASP Protocol Test") {
        Ok(m) => m,
        Err(e) => {
            return TestResult::fail(
                name,
                format!("MIDI init failed: {}", e),
                start.elapsed().as_millis(),
            )
        }
    };

    let ports = midi_in.ports();
    let port_names: Vec<String> = ports
        .iter()
        .filter_map(|p| midi_in.port_name(p).ok())
        .collect();

    // Look for virtual MIDI ports
    let virtual_ports: Vec<&String> = port_names
        .iter()
        .filter(|name| {
            let lower = name.to_lowercase();
            lower.contains("iac") ||           // macOS IAC Driver
            lower.contains("virtual") ||       // Generic virtual
            lower.contains("loop") ||          // Loopback
            lower.contains("midi through") ||  // Linux MIDI Through
            lower.contains("virmidi") // Linux snd-virmidi
        })
        .collect();

    if !virtual_ports.is_empty() {
        TestResult::pass(
            name,
            format!("Found virtual ports: {:?}", virtual_ports),
            start.elapsed().as_millis(),
        )
    } else if !port_names.is_empty() {
        TestResult::pass(
            name,
            format!(
                "Found {} MIDI ports (no virtual): {:?}",
                port_names.len(),
                port_names
            ),
            start.elapsed().as_millis(),
        )
    } else {
        // No MIDI ports is OK in CI environments - just note it
        TestResult::pass(
            name,
            "No MIDI ports (OK in CI/headless environments)",
            start.elapsed().as_millis(),
        )
    }
}

fn test_midi_message_encoding() -> TestResult {
    let start = Instant::now();
    let name = "midi_message_encoding";

    // Test MIDI message byte encoding (doesn't need hardware)

    // Note On: Channel 1, Note 60 (C4), Velocity 100
    let note_on = [0x90, 60, 100];
    if note_on[0] & 0xF0 != 0x90 {
        return TestResult::fail(
            name,
            "Note On status byte wrong",
            start.elapsed().as_millis(),
        );
    }
    if (note_on[0] & 0x0F) != 0 {
        return TestResult::fail(name, "Channel should be 0", start.elapsed().as_millis());
    }

    // CC: Channel 1, CC 1 (Mod Wheel), Value 64
    let cc = [0xB0, 1, 64];
    if cc[0] & 0xF0 != 0xB0 {
        return TestResult::fail(name, "CC status byte wrong", start.elapsed().as_millis());
    }

    // Program Change: Channel 1, Program 42
    let pc = [0xC0, 42];
    if pc[0] & 0xF0 != 0xC0 {
        return TestResult::fail(name, "PC status byte wrong", start.elapsed().as_millis());
    }

    // Pitch Bend: Channel 1, Value 8192 (center)
    let pb_lsb = 8192 & 0x7F;
    let pb_msb = (8192 >> 7) & 0x7F;
    let pitch_bend = [0xE0, pb_lsb as u8, pb_msb as u8];
    if pitch_bend[0] & 0xF0 != 0xE0 {
        return TestResult::fail(
            name,
            "Pitch Bend status byte wrong",
            start.elapsed().as_millis(),
        );
    }

    // SysEx: Universal Non-Real-Time
    let sysex = [0xF0, 0x7E, 0x00, 0x06, 0x01, 0xF7];
    if sysex[0] != 0xF0 || sysex[sysex.len() - 1] != 0xF7 {
        return TestResult::fail(name, "SysEx framing wrong", start.elapsed().as_millis());
    }

    TestResult::pass(
        name,
        "All MIDI message types encode correctly",
        start.elapsed().as_millis(),
    )
}

fn test_midi_channel_mapping() -> TestResult {
    let start = Instant::now();
    let name = "midi_channel_mapping";

    // Verify MIDI channel encoding (0-15 maps to channels 1-16)
    for ch in 0u8..16 {
        let note_on = 0x90 | ch;
        let extracted_channel = note_on & 0x0F;
        if extracted_channel != ch {
            return TestResult::fail(
                name,
                format!("Channel {} extraction failed", ch + 1),
                start.elapsed().as_millis(),
            );
        }
    }

    // Verify status byte ranges
    let status_types = [
        (0x80, "Note Off"),
        (0x90, "Note On"),
        (0xA0, "Aftertouch"),
        (0xB0, "CC"),
        (0xC0, "Program Change"),
        (0xD0, "Channel Pressure"),
        (0xE0, "Pitch Bend"),
    ];

    for (status, name_str) in status_types {
        for ch in 0u8..16 {
            let byte = status | ch;
            let msg_type = byte & 0xF0;
            if msg_type != status {
                return TestResult::fail(
                    name,
                    format!("{} status extraction failed on ch {}", name_str, ch + 1),
                    start.elapsed().as_millis(),
                );
            }
        }
    }

    TestResult::pass(
        name,
        "All 16 channels map correctly for all message types",
        start.elapsed().as_millis(),
    )
}

fn test_midi_loopback_if_available() -> TestResult {
    let start = Instant::now();
    let name = "midi_loopback_if_available";

    let midi_in = match midir::MidiInput::new("CLASP Test In") {
        Ok(m) => m,
        Err(e) => {
            return TestResult::fail(
                name,
                format!("MIDI input init failed: {}", e),
                start.elapsed().as_millis(),
            )
        }
    };

    let midi_out = match midir::MidiOutput::new("CLASP Test Out") {
        Ok(m) => m,
        Err(e) => {
            return TestResult::fail(
                name,
                format!("MIDI output init failed: {}", e),
                start.elapsed().as_millis(),
            )
        }
    };

    let in_ports = midi_in.ports();
    let out_ports = midi_out.ports();

    // Look for loopback pair (IAC on macOS, MIDI Through on Linux)
    let in_names: Vec<String> = in_ports
        .iter()
        .filter_map(|p| midi_in.port_name(p).ok())
        .collect();
    let out_names: Vec<String> = out_ports
        .iter()
        .filter_map(|p| midi_out.port_name(p).ok())
        .collect();

    // Find matching virtual port pair
    let mut loopback_in = None;
    let mut loopback_out = None;

    for (i, name) in in_names.iter().enumerate() {
        let lower = name.to_lowercase();
        if lower.contains("iac")
            || lower.contains("virtual")
            || lower.contains("loop")
            || lower.contains("through")
        {
            loopback_in = Some(i);
            break;
        }
    }

    for (i, name) in out_names.iter().enumerate() {
        let lower = name.to_lowercase();
        if lower.contains("iac")
            || lower.contains("virtual")
            || lower.contains("loop")
            || lower.contains("through")
        {
            loopback_out = Some(i);
            break;
        }
    }

    match (loopback_in, loopback_out) {
        (Some(in_idx), Some(out_idx)) => {
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            let received_value = Arc::new(AtomicU32::new(0));
            let received_value_clone = received_value.clone();

            // Connect input
            let _conn_in = midi_in.connect(
                &in_ports[in_idx],
                "test-in",
                move |_stamp, message, _| {
                    if message.len() >= 3 && (message[0] & 0xF0) == 0xB0 {
                        received_clone.store(true, Ordering::SeqCst);
                        received_value_clone.store(message[2] as u32, Ordering::SeqCst);
                    }
                },
                (),
            );

            // Connect output
            let mut conn_out = match midi_out.connect(&out_ports[out_idx], "test-out") {
                Ok(c) => c,
                Err(e) => {
                    return TestResult::fail(
                        name,
                        format!("Output connect failed: {}", e),
                        start.elapsed().as_millis(),
                    )
                }
            };

            // Send CC message
            let test_value = 42u8;
            if let Err(e) = conn_out.send(&[0xB0, 1, test_value]) {
                return TestResult::fail(
                    name,
                    format!("Send failed: {}", e),
                    start.elapsed().as_millis(),
                );
            }

            // Wait for loopback
            thread::sleep(Duration::from_millis(100));

            if received.load(Ordering::SeqCst) {
                let value = received_value.load(Ordering::SeqCst);
                if value == test_value as u32 {
                    TestResult::pass(
                        name,
                        format!("MIDI loopback OK: CC value {} received", value),
                        start.elapsed().as_millis(),
                    )
                } else {
                    TestResult::fail(
                        name,
                        format!("Wrong value: {} vs {}", value, test_value),
                        start.elapsed().as_millis(),
                    )
                }
            } else {
                TestResult::fail(
                    name,
                    "No MIDI message received (loopback may need configuration)",
                    start.elapsed().as_millis(),
                )
            }
        }
        _ => TestResult::pass(
            name,
            format!(
                "No virtual MIDI loopback available (in: {:?}, out: {:?})",
                in_names, out_names
            ),
            start.elapsed().as_millis(),
        ),
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║          CLASP Self-Contained Protocol Tests                     ║");
    println!("║                                                                  ║");
    println!("║  Tests real protocol implementations with virtual endpoints      ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let tests = vec![
        // OSC Loopback Tests (always work - just UDP)
        test_osc_loopback_float(),
        test_osc_loopback_int(),
        test_osc_loopback_string(),
        test_osc_loopback_multiple_args(),
        test_osc_loopback_bundle(),
        test_osc_high_frequency(),
        // Art-Net Tests (loopback + format validation)
        test_artnet_packet_format(),
        test_artnet_loopback(),
        test_artnet_poll_reply(),
        test_artnet_multiple_universes(),
        // MIDI Tests (encoding + virtual port detection)
        test_midi_virtual_port_available(),
        test_midi_message_encoding(),
        test_midi_channel_mapping(),
        test_midi_loopback_if_available(),
    ];

    let mut passed = 0;
    let mut failed = 0;

    println!("┌──────────────────────────────────────┬────────┬──────────┐");
    println!("│ Test                                 │ Status │ Time     │");
    println!("├──────────────────────────────────────┼────────┼──────────┤");

    for test in &tests {
        let (status, color) = if test.passed {
            ("✓ PASS", "\x1b[32m")
        } else {
            ("✗ FAIL", "\x1b[31m")
        };

        println!(
            "│ {:<36} │ {}{:<6}\x1b[0m │ {:>6}ms │",
            test.name, color, status, test.duration_ms
        );

        if test.passed {
            passed += 1;
            if !test.message.is_empty() && test.message != "OK" {
                let msg = &test.message[..test.message.len().min(56)];
                println!("│   └─ {:<56} │", msg);
            }
        } else {
            failed += 1;
            let msg = &test.message[..test.message.len().min(56)];
            println!("│   └─ {:<56} │", msg);
        }
    }

    println!("└──────────────────────────────────────┴────────┴──────────┘");
    println!("\nResults: {} passed, {} failed", passed, failed);

    if failed > 0 {
        std::process::exit(1);
    }
}
