---
title: Clock Sync & Timing
description: NTP-style clock synchronization, session time, and jitter buffering
order: 7
---

# Clock Sync & Timing

Synchronized timing is critical for live performance -- CLASP's primary use case. When multiple clients need to trigger lighting cues, play back audio, or coordinate effects at the exact same moment, they need a shared notion of "now."

CLASP provides three timing primitives:

- **ClockSync** -- NTP-style clock synchronization between client and server
- **SessionTime** -- elapsed time since session start
- **JitterBuffer** -- ordered playback buffer for smoothing high-rate streams

## ClockSync

`ClockSync` uses the NTP four-timestamp algorithm to estimate the offset between a client's local clock and the server's clock. It maintains a running estimate using exponential moving average (alpha = 0.3) so the offset converges smoothly rather than jumping on each sync round.

### How It Works

Each sync round exchanges four timestamps:

```
Client                          Server
  |                                |
  |--- t1 (client send) -------->|
  |                                |--- t2 (server receive)
  |                                |--- t3 (server send)
  |<------ t4 (client receive) ---|
```

From these four values, ClockSync computes:

- **Offset**: `((t2 - t1) + (t3 - t4)) / 2` -- how far the client clock is from the server
- **RTT**: `(t4 - t1) - (t3 - t2)` -- network round-trip time
- **Jitter**: standard deviation of recent RTT samples

### API (Rust)

```rust
use clasp_core::time::ClockSync;

let mut sync = ClockSync::new();

// Feed sync rounds as they complete
sync.process_sync(t1, t2, t3, t4);

// Query state
let offset = sync.offset();          // Estimated offset in microseconds (i64)
let rtt = sync.rtt();                // Round-trip time in microseconds
let jitter = sync.jitter();          // RTT jitter in microseconds
let quality = sync.quality();         // Sync quality score: 0.0 (poor) to 1.0 (excellent)
let needs = sync.needs_sync(30);      // True if >30 seconds since last sync

// Convert timestamps
let server_now = sync.server_time();           // Current server time estimate
let server_ts = sync.to_server_time(local_ts); // Local -> server
let local_ts = sync.to_local_time(server_ts);  // Server -> local
```

### API (JavaScript)

In the JavaScript SDK, clock sync happens automatically during the HELLO/WELCOME handshake. The `client.time()` method returns synchronized server time:

```javascript
const serverNow = client.time(); // microseconds, server-synchronized

// Schedule something 2 seconds from now in server time
client.bundle([
  { set: ['/lights/go', true] }
], { at: client.time() + 2_000_000 });
```

Additional sync rounds occur via SYNC messages. The client processes incoming SYNC responses to refine its offset estimate.

### Quality Score

`quality()` returns a value from 0.0 to 1.0 based on three factors:

| Factor | Weight | Scoring |
|--------|--------|---------|
| RTT | 40% | Lower is better (0 at 10ms+, 1.0 at 0) |
| Jitter | 40% | Lower is better (0 at 1ms+, 1.0 at 0) |
| Samples | 20% | More samples = higher confidence (maxes at 10) |

A quality score above 0.8 indicates reliable synchronization suitable for frame-accurate coordination.

## SessionTime

`SessionTime` tracks elapsed time since a session started, with conversions between session-relative and Unix timestamps.

```rust
use clasp_core::time::SessionTime;

let session = SessionTime::new();

// Time since session start (microseconds)
let elapsed = session.elapsed();

// When the session started (Unix timestamp, microseconds)
let start = session.start_time();

// Convert between session time and Unix time
let unix_ts = session.to_unix(elapsed);
let session_ts = session.from_unix(unix_ts);
```

Use `SessionTime` when you need relative timestamps (e.g., "5 seconds into the show") rather than absolute wall-clock time.

## JitterBuffer

The `JitterBuffer` provides ordered playback for high-rate streams where packets may arrive out of order. It holds samples in a time-sorted buffer and releases them in order when their timestamp has passed.

```rust
use clasp_core::time::JitterBuffer;

// Buffer up to 64 samples with a 50ms window
let mut buffer: JitterBuffer<f64> = JitterBuffer::new(64, 50);

// Push samples (may arrive out of order)
buffer.push(timestamp_a, 0.5);
buffer.push(timestamp_c, 0.7);  // arrived early
buffer.push(timestamp_b, 0.6);  // arrived late

// Pop the next sample ready for playback
if let Some(value) = buffer.pop(playback_time) {
    play(value);
}

// Or drain all ready samples at once
let ready = buffer.drain_ready(playback_time);
for value in ready {
    play(value);
}

// Buffer state
let depth = buffer.len();       // Number of buffered samples
let span = buffer.depth_us();   // Time span of buffer contents (microseconds)
let empty = buffer.is_empty();
```

### Constructor Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `capacity` | `usize` | Maximum number of buffered samples |
| `window_ms` | `u64` | Buffer window in milliseconds. Samples older than this are discarded. |

### When to Use a Jitter Buffer

Use `JitterBuffer` when receiving stream data over an unreliable transport (P2P unreliable channel, UDP) where:

- Packets may arrive out of order
- You need smooth, ordered playback
- Occasional packet loss is acceptable (the buffer skips gaps)

For reliable transports (WebSocket, P2P reliable channel), packets arrive in order and a jitter buffer is unnecessary.

## Example: Synchronized Lighting Cue

Two clients synchronize their clocks with the server, then schedule a lighting cue to trigger at the exact same server time:

```javascript
// Client A (venue 1)
const client = await new ClaspBuilder('ws://relay.clasp.to')
  .withName('Venue 1')
  .connect();

// client.time() is already synchronized via HELLO/WELCOME
const cueTime = client.time() + 5_000_000; // 5 seconds from now

// Schedule the cue
client.bundle([
  { set: ['/lights/venue1/brightness', 1.0] },
  { emit: ['/cues/go', { venue: 1 }] }
], { at: cueTime });

console.log(`Cue scheduled at server time ${cueTime}`);
```

```javascript
// Client B (venue 2) -- receives the same cue time
const client = await new ClaspBuilder('ws://relay.clasp.to')
  .withName('Venue 2')
  .connect();

// Subscribe for the cue time from a shared coordinator
client.on('/show/next-cue-time', (cueTime) => {
  // Schedule local actions at the same server time
  client.bundle([
    { set: ['/lights/venue2/brightness', 1.0] },
    { emit: ['/cues/go', { venue: 2 }] }
  ], { at: cueTime });
});
```

Both bundles execute at the same server time, regardless of network latency differences between the two venues.

## Next Steps

- [Bundles](./bundles.md) -- atomic and scheduled message groups
- [P2P & WebRTC](./p2p.md) -- direct peer connections where timing is critical
- [Signal Types](./signals.md) -- choosing the right signal type for your data
