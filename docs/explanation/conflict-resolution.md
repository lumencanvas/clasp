# Conflict Resolution

When multiple clients write to the same parameter simultaneously, conflicts occur. This document explains how CLASP handles them.

## The Problem

```
t=0: Client A reads /volume = 0.5 (rev 10)
t=0: Client B reads /volume = 0.5 (rev 10)
t=1: Client A sets /volume = 0.8
t=1: Client B sets /volume = 0.3
t=2: Router receives both writes
t=3: ??? What should /volume be?
```

## Resolution Strategies

CLASP supports multiple strategies:

| Strategy | Behavior | Use Case |
|----------|----------|----------|
| `lww` | Last-Write-Wins (by timestamp) | Default, general purpose |
| `max` | Keep highest value | Meters, peak levels |
| `min` | Keep lowest value | Limits, thresholds |
| `lock` | First writer holds exclusive control | Faders, manual control |
| `merge` | Application-defined merge function | Complex objects |

## Last-Write-Wins (LWW)

The default strategy. The write with the latest timestamp wins.

```
t=100: Client A sets /volume = 0.8 (timestamp: 1704067200100)
t=100: Client B sets /volume = 0.3 (timestamp: 1704067200105)

Router receives B's write later, but B's timestamp is newer.
Result: /volume = 0.3 (B wins)
```

### Clock Synchronization

LWW requires synchronized clocks. CLASP's SYNC protocol keeps clients within ~1-5ms on LAN.

### Pros and Cons

**Pros:**
- Simple to understand
- No coordination needed
- Works for most cases

**Cons:**
- Earlier intention can be overwritten
- Requires clock sync
- Not suitable for collaborative editing

## Max / Min Strategies

Use the highest or lowest value.

### Max (Keep Highest)

```javascript
// Configure on router
{ address: "/audio/peak/*", strategy: "max" }
```

```
Client A: SET /audio/peak/1 = 0.8
Client B: SET /audio/peak/1 = 0.6
Result: /audio/peak/1 = 0.8 (max wins)
```

Use for:
- Peak meters
- High water marks
- Maximum readings

### Min (Keep Lowest)

```javascript
{ address: "/limits/*", strategy: "min" }
```

```
Client A: SET /limits/brightness = 0.8
Client B: SET /limits/brightness = 0.5
Result: /limits/brightness = 0.5 (min wins)
```

Use for:
- Safety limits
- Minimum thresholds
- Conservative values

## Lock Strategy

One client holds exclusive control until released.

### Acquiring a Lock

```javascript
// Request lock
await client.set('/mixer/fader/1', 0.5, { lock: true });
```

Router response if granted:
```javascript
{ type: "ACK", locked: true, holder: "session:abc" }
```

### Holding a Lock

While locked, only the holder can write:

```
Holder: SET /mixer/fader/1 = 0.6  → OK
Other:  SET /mixer/fader/1 = 0.4  → ERROR (lock held)
```

### Releasing a Lock

```javascript
// Release lock
await client.set('/mixer/fader/1', 0.5, { unlock: true });

// Or disconnect (auto-releases all locks)
```

### Lock Timeout

Locks expire after configurable timeout (default: 30 seconds of inactivity).

### Use Cases

- Manual fader control
- Exclusive editing
- Take/release ownership

## Optimistic Locking (Revisions)

Check-and-set with expected revision:

```javascript
// Read current value and revision
const { value, revision } = await client.get('/config/setting');
// revision = 10

// Attempt update, expecting revision 10
try {
  await client.set('/config/setting', newValue, { revision: 10 });
  // Success if still revision 10
} catch (e) {
  // Conflict: revision changed
  // Re-read and retry
}
```

### Compare-And-Swap Flow

```
1. Read value and revision
2. Compute new value
3. Write with expected revision
4. If conflict, go to 1
```

### Use Cases

- Configuration updates
- Transactional changes
- Avoiding lost updates

## Merge Strategy

For complex values, define a merge function:

```javascript
// Router configuration
{
  address: "/doc/content",
  strategy: "merge",
  mergeFunction: "text-crdt"  // Built-in CRDT
}
```

### Built-in Merge Functions

| Function | Description |
|----------|-------------|
| `text-crdt` | Collaborative text editing |
| `set-union` | Merge sets by union |
| `counter` | Add increments |

### Custom Merge (Advanced)

```javascript
// Application provides merge logic
router.registerMerge('/custom/**', (a, b, base) => {
  // Return merged value
  return { ...a, ...b };
});
```

## Per-Address Configuration

Configure strategy per address pattern:

```yaml
# Router config
conflict_resolution:
  default: lww

  rules:
    - pattern: "/audio/peak/*"
      strategy: max

    - pattern: "/mixer/fader/*"
      strategy: lock
      lock_timeout: 30s

    - pattern: "/config/**"
      strategy: optimistic
```

## Practical Recommendations

### Use LWW for Most Things

```javascript
// Default behavior, good for:
await client.set('/app/value', 123);
```

Simple, predictable, usually correct.

### Use Lock for Manual Control

```javascript
// Fader control
const unsub = await client.set('/mixer/fader/1', value, { lock: true });

// When user releases fader
await client.set('/mixer/fader/1', value, { unlock: true });
```

Prevents fighting over controls.

### Use Optimistic Locking for Config

```javascript
async function updateConfig(key, transform) {
  while (true) {
    const { value, revision } = await client.get(key);
    const newValue = transform(value);

    try {
      await client.set(key, newValue, { revision });
      return;
    } catch (e) {
      if (e.code === 'CONFLICT') continue;
      throw e;
    }
  }
}
```

Ensures atomic updates.

### Avoid Conflicts When Possible

Best conflict resolution: **avoid conflicts**.

```javascript
// Bad: Multiple clients writing same address
/volume = ?

// Better: Separate addresses per client
/client/alice/volume = 0.8
/client/bob/volume = 0.5
/volume = max(/client/*/volume)  // Derived
```

## Debugging Conflicts

### Enable Conflict Logging

```bash
RUST_LOG=clasp_router::conflict=debug clasp server
```

### Inspect Revision History

```javascript
const history = await client.query('/path?history=10');
// Returns last 10 revisions with writers and timestamps
```

### Check Lock Status

```javascript
const status = await client.get('/path?meta=true');
// { value: ..., revision: ..., locked: true, holder: "session:abc" }
```

## See Also

- [State Management](state-management.md) — How state works
- [Use Locks How-To](../how-to/state/use-locks.md) — Lock usage guide
- [Handle Conflicts How-To](../how-to/state/handle-conflicts.md) — Practical patterns
