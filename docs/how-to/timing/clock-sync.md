# Clock Sync

Synchronize clocks between clients and router for coordinated timing.

## Automatic Sync

Clock sync happens automatically on connect:

```javascript
const client = await new ClaspBuilder(url).connect();

// Clock is already synced
const serverTime = client.time();
```

## Get Synchronized Time

```javascript
// JavaScript
const now = client.time();  // Microseconds since epoch

// Python
now = client.time()  # Microseconds

// Rust
let now = client.time();  // u64 microseconds
```

## Check Sync Quality

```javascript
const quality = client.clockQuality();
console.log('Offset:', quality.offset, 'µs');
console.log('RTT:', quality.rtt, 'µs');
console.log('Jitter:', quality.jitter, 'µs');
```

| Network | Typical Accuracy |
|---------|------------------|
| Wired LAN | ±1ms |
| WiFi | ±5-10ms |
| Internet | ±20-50ms |

## Manual Resync

Force a clock sync:

```javascript
await client.resync();
```

## Sync Interval

Clients resync every 30 seconds by default. Change with:

```javascript
const client = await new ClaspBuilder(url)
  .withSyncInterval(60000)  // 60 seconds
  .connect();
```

## Handling Clock Drift

For long-running sessions, enable continuous sync:

```javascript
const client = await new ClaspBuilder(url)
  .withContinuousSync(true)
  .connect();
```

## Next Steps

- [Scheduled Bundles](scheduled-bundles.md)
- [Timing Model Explanation](../../explanation/timing-model.md)
