# clasp-transport

Transport layer implementations for CLASP (Creative Low-Latency Application Streaming Protocol).

## Supported Transports

| Transport | Feature Flag | Description |
|-----------|--------------|-------------|
| **WebSocket** | `websocket` (default) | Primary transport for browser and server communication |
| **QUIC** | `quic` | Low-latency UDP-based transport with TLS 1.3 |
| **TCP** | `tcp` | Reliable streaming transport |
| **UDP** | `udp` | Lightweight datagram transport for LAN |
| **WebRTC** | `webrtc` | P2P data channels with NAT traversal |
| **BLE** | `ble` | Bluetooth Low Energy GATT service |
| **Serial** | `serial` | Hardware serial port communication |

## Usage

```rust
use clasp_transport::WebSocketTransport;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = WebSocketTransport::connect("ws://localhost:7330").await?;

    // Send messages
    transport.send(message).await?;

    // Receive messages
    while let Some(msg) = transport.recv().await {
        println!("Received: {:?}", msg);
    }

    Ok(())
}
```

## Features

- Async/await with Tokio
- Automatic frame encoding/decoding
- Connection health monitoring
- TLS support

## Documentation

Visit **[clasp.to](https://clasp.to)** for full documentation.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio) | 2026
