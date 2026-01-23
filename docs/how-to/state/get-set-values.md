# Get and Set Values

Read and write parameter values.

## Set Values

### JavaScript

```javascript
// Set a single value
await client.set('/lights/brightness', 0.8);

// Set with different types
await client.set('/path/int', 42);
await client.set('/path/float', 3.14);
await client.set('/path/string', 'hello');
await client.set('/path/bool', true);
await client.set('/path/array', [1, 2, 3]);
await client.set('/path/object', { x: 1, y: 2 });
```

### Python

```python
await client.set('/lights/brightness', 0.8)
await client.set('/path/list', [1, 2, 3])
await client.set('/path/dict', {'x': 1, 'y': 2})
```

### Rust

```rust
use clasp_core::Value;

client.set("/lights/brightness", Value::Float(0.8)).await?;
client.set("/path/int", Value::Int(42)).await?;
client.set("/path/string", Value::String("hello".into())).await?;
```

## Get Values

### JavaScript

```javascript
// Get current value (async)
const brightness = await client.get('/lights/brightness');
console.log(brightness);  // 0.8

// Get with metadata
const result = await client.get('/lights/brightness', { meta: true });
console.log(result.value);     // 0.8
console.log(result.revision);  // 42
```

### Python

```python
brightness = await client.get('/lights/brightness')
print(brightness)  # 0.8
```

### Rust

```rust
let brightness = client.get("/lights/brightness").await?;
println!("{:?}", brightness);
```

## Cached Values

Read from local cache (instant, might be stale):

```javascript
// Must be subscribed to receive updates
client.on('/lights/**', () => {});

// Instant read from cache
const value = client.cached('/lights/brightness');
```

## Set Multiple Values

Use bundles for atomic updates:

```javascript
await client.bundle([
  { set: ['/lights/1/brightness', 1.0] },
  { set: ['/lights/2/brightness', 0.5] },
  { set: ['/lights/3/brightness', 0.0] }
]);
```

All values update atomically.

## Get Multiple Values

Query with pattern:

```javascript
const lights = await client.query('/lights/**');
// Returns array of { address, value, revision }

for (const param of lights) {
  console.log(`${param.address} = ${param.value}`);
}
```

## Events (Ephemeral)

For triggers that shouldn't be stored:

```javascript
// Events are not stored, just delivered to subscribers
await client.emit('/cue/fire', { id: 'intro' });

// Listen for events
client.on('/cue/*', (payload, address) => {
  console.log('Cue fired:', payload);
});
```

## Streams (High-Rate)

For continuous data where occasional loss is acceptable:

```javascript
// Fire-and-forget (no await needed)
client.stream('/sensor/position', { x: 0.5, y: 0.3 });

// Subscribe with rate limiting
client.on('/sensor/position', callback, { maxRate: 60 });
```

## Type Handling

CLASP preserves types:

```javascript
await client.set('/path', 42);           // Integer
await client.set('/path', 42.0);         // Float
await client.set('/path', '42');         // String
await client.set('/path', true);         // Boolean
await client.set('/path', null);         // Null
await client.set('/path', [1, 2, 3]);    // Array
await client.set('/path', { a: 1 });     // Map/Object
```

## Error Handling

```javascript
try {
  await client.set('/path', value);
} catch (error) {
  if (error.code === 'PERMISSION_DENIED') {
    console.log('Not allowed to write this address');
  } else if (error.code === 'CONFLICT') {
    console.log('Value was changed by someone else');
  }
}
```

## Next Steps

- [Handle Conflicts](handle-conflicts.md)
- [Use Locks](use-locks.md)
