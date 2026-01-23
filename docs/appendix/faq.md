# Frequently Asked Questions

Common questions about CLASP.

## General

### What is CLASP?

CLASP (Creative Low-Latency Application Streaming Protocol) is a universal protocol for connecting creative applications. It provides real-time, bidirectional communication between software, hardware, and services using a hierarchical address space.

### How is CLASP different from OSC?

CLASP builds on concepts from OSC but adds:
- **State management**: Values are stored and queryable
- **Subscriptions**: Clients receive updates automatically
- **Late joiner sync**: New clients get current state
- **Multiple transports**: WebSocket, QUIC, UDP, WebRTC
- **Binary encoding**: 55% smaller than JSON/text
- **Quality of service**: Fire, Confirm, Commit levels
- **Signal types**: Param, Event, Stream, Gesture, Timeline

### Is CLASP open source?

Yes, CLASP is open source. The protocol specification, router, and client libraries are available under permissive licenses.

### What languages are supported?

Official libraries exist for:
- **Rust**: Full-featured, high-performance
- **JavaScript/TypeScript**: Browser and Node.js
- **Python**: Async support

Community libraries may exist for other languages.

## Architecture

### Do I need a router?

Yes, CLASP uses a router-based architecture. The router:
- Routes messages between clients
- Manages state
- Handles subscriptions
- Provides late-joiner sync

For P2P scenarios, WebRTC can be used for direct client communication.

### Can I run multiple routers?

Yes. Common patterns:
- **Geographic distribution**: Routers in different locations
- **Redundancy**: Backup routers for failover
- **Scaling**: Multiple routers with cross-routing

### What's the maximum number of clients?

A single router can handle 10,000+ concurrent connections. Actual limits depend on:
- Hardware resources
- Message rate
- State size

## Performance

### What's the latency?

Typical latencies (LAN):
- WebSocket: 1-5ms
- QUIC: 0.5-3ms
- UDP: 0.1-1ms

### What's the maximum message rate?

Single router: 100,000+ messages/second
Single client: 50,000+ messages/second

Actual throughput depends on message size and hardware.

### Is CLASP suitable for audio?

CLASP is suitable for:
- Control data (MIDI, OSC-style)
- Meter data
- Transport sync

CLASP is NOT suitable for:
- Audio streaming
- Video streaming

Use dedicated protocols (DANTE, NDI) for media streaming.

## Security

### Is communication encrypted?

Yes, when using:
- **WSS**: TLS over WebSocket
- **QUIC**: Built-in TLS
- **WebRTC**: Built-in DTLS

UDP transport is not encrypted by default.

### How does authentication work?

CLASP uses JWT tokens with scoped permissions:
```json
{
  "clasp": {
    "read": ["/sensors/**"],
    "write": ["/control/*"]
  }
}
```

### Can I restrict access to addresses?

Yes, tokens specify:
- **read**: Addresses client can GET
- **write**: Addresses client can SET
- **emit**: Addresses client can EMIT events
- **subscribe**: Patterns client can subscribe to

## Bridges

### Which protocols are supported?

Built-in bridges:
- OSC
- MIDI
- Art-Net
- DMX
- MQTT
- sACN
- HTTP/REST

### Can I create custom bridges?

Yes, the bridge API allows creating bridges for any protocol. See [Custom Bridge](../how-to/advanced/custom-bridge.md).

### Do bridges run separately?

Bridges can run:
- As separate processes (CLI)
- Embedded in applications
- As part of the desktop app

## Integration

### Does CLASP work with TouchOSC?

Yes. Configure TouchOSC to send to the CLASP OSC bridge, and CLASP messages can be sent back to TouchOSC. See [TouchOSC Integration](../integrations/touchosc.md).

### Does CLASP work with Home Assistant?

Yes, via the MQTT bridge. CLASP can subscribe to Home Assistant MQTT topics and publish to control devices. See [Home Assistant Integration](../integrations/home-assistant.md).

### Can I use CLASP in a browser?

Yes. The JavaScript library works in browsers via WebSocket. WebRTC is also supported for P2P.

### Can I use CLASP on embedded devices?

Yes. The `clasp-embedded` crate supports no_std Rust for microcontrollers. HTTP POST can also be used for simple sensors.

## Troubleshooting

### Connection refused

1. Verify router is running
2. Check port and address
3. Check firewall settings

### No messages received

1. Verify subscription pattern
2. Check addresses match
3. Confirm sender is connected

### High latency

1. Check network path
2. Consider QUIC or UDP transport
3. Profile for bottlenecks

### Memory growing

1. Unsubscribe unused subscriptions
2. Clean up old state
3. Check for subscription loops

## Getting Help

### Where can I get support?

- GitHub Issues: Bug reports and feature requests
- Documentation: This site
- Examples: In the repository

### How do I report bugs?

Open an issue on GitHub with:
- CLASP version
- Steps to reproduce
- Expected vs actual behavior
- Logs if available

## See Also

- [Troubleshooting Guide](../how-to/troubleshooting.md)
- [Architecture](../explanation/architecture.md)
- [Protocol Overview](../reference/protocol/overview.md)
