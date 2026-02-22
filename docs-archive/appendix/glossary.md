---
title: "Glossary"
description: "Key terms and concepts used in CLASP."
section: appendix
order: 3
---
# Glossary

Key terms and concepts used in CLASP.

## A

### Address
A hierarchical path identifying a signal in CLASP, similar to a file path. Example: `/sensors/temperature/room1`. Addresses support wildcards (`*` for single segment, `**` for multiple).

### Art-Net
A protocol for transmitting DMX512 data over IP networks, commonly used in professional lighting. CLASP provides an Art-Net bridge.

## B

### Bridge
A component that translates between CLASP and external protocols (OSC, MIDI, Art-Net, MQTT, etc.). Bridges run as separate processes or embedded in applications.

### Bundle
A collection of CLASP operations executed atomically. All operations in a bundle either succeed together or fail together, ensuring consistent state.

## C

### Client
An application that connects to a CLASP router to send and receive signals. Clients can be implemented in JavaScript, Python, Rust, or any language with WebSocket support.

### Codec
The binary encoding/decoding layer for CLASP messages. The CLASP codec is ~55% smaller than JSON.

### Commit (QoS)
The highest quality of service level. Ensures ordered, exactly-once delivery with full acknowledgment.

### Confirm (QoS)
Quality of service level providing acknowledged delivery. The sender receives confirmation when the message is processed.

## D

### DMX
Digital Multiplex (DMX512), a standard for controlling stage lighting and effects. Each universe contains 512 channels.

## E

### Emit
Send an ephemeral event that is not stored in state. Events are delivered to current subscribers but not retained for late joiners.

### Event
A signal type for one-time, ephemeral messages. Events are not stored in router state and only delivered to active subscribers.

## F

### Fire (QoS)
The default quality of service level. Best-effort delivery without acknowledgment, similar to UDP.

### Frame
The binary packet structure for CLASP messages, consisting of a header and payload.

## G

### GET
A CLASP message type to retrieve the current value at an address.

### Gesture
A signal type for phased interactions with begin, update, and end phases. Used for drag operations, drawing strokes, etc.

## H

### HELLO
The initial message sent by clients to establish a connection with a router, containing client identification and capabilities.

## L

### Late Joiner
A client that connects after state has been established. CLASP supports late joiner synchronization to bring new clients up to date.

### Lock
A mechanism for exclusive access to an address. Only the lock holder can modify the value until the lock is released.

### LWW (Last-Write-Wins)
The default conflict resolution strategy where the most recent write takes precedence based on timestamp.

## M

### mDNS
Multicast DNS, used for automatic router discovery on local networks. Routers advertise as `_clasp._tcp.local`.

### Message
A single CLASP protocol unit (SET, GET, SUBSCRIBE, etc.) transmitted between clients and routers.

### MQTT
Message Queuing Telemetry Transport, an IoT messaging protocol. CLASP provides an MQTT bridge for integration with IoT systems.

## O

### OSC
Open Sound Control, a protocol for communication between multimedia devices. CLASP provides an OSC bridge for integration with creative software.

## P

### Param
The default signal type for stateful values. Param values are retained in router state and delivered to late joiners.

### Pattern
An address with wildcards used for subscriptions. `*` matches a single segment, `**` matches multiple segments.

## Q

### QoS (Quality of Service)
The delivery guarantee level for messages: Fire (best-effort), Confirm (acknowledged), or Commit (exactly-once).

### QUIC
A UDP-based transport protocol providing lower latency than WebSocket with built-in encryption.

## R

### Router
The central message hub that clients connect to. The router handles message routing, state management, and subscription matching.

## S

### sACN
Streaming ACN (E1.31), a protocol for DMX over IP used in professional lighting. CLASP provides an sACN bridge.

### SET
A CLASP message type to store a value at an address.

### Signal
A value at an address with associated metadata (type, timestamp). Signals can be Param, Event, Stream, Gesture, or Timeline.

### Signal Type
The semantic category of a signal: Param (stateful), Event (ephemeral), Stream (high-rate), Gesture (phased), or Timeline (automation).

### State
The collection of all current values stored in the router. State is indexed by address and supports queries and subscriptions.

### Stream
A signal type for high-rate continuous data. Optimized for frequent updates like sensor readings or audio levels.

### SUBSCRIBE
A CLASP message type to register interest in addresses matching a pattern.

## T

### Timeline
A signal type for time-indexed automation data, containing keyframes with values and timestamps.

### Token
A JWT (JSON Web Token) used for authentication and authorization. Tokens contain permissions specifying allowed addresses and operations.

### Transport
The underlying communication mechanism (WebSocket, QUIC, UDP, WebRTC, etc.) used to transmit CLASP messages.

## U

### Universe
In DMX/Art-Net/sACN, a group of 512 channels. Multiple universes can be used for larger installations.

### UNSUBSCRIBE
A CLASP message type to cancel a previous subscription.

## V

### Value
The data stored at an address. CLASP supports primitives (int, float, bool, string), blobs, arrays, and maps.

## W

### Wildcard
Pattern characters for matching multiple addresses: `*` (single segment) and `**` (multiple segments).

### WebSocket
The default transport protocol for CLASP, providing reliable bidirectional communication over TCP.

### WebRTC
A peer-to-peer protocol used for direct client-to-client communication, bypassing the router.

## See Also

- [Protocol Overview](../reference/protocol/overview.md)
- [Signal Types](../reference/protocol/signal-types.md)
- [Addressing](../reference/protocol/addressing.md)
