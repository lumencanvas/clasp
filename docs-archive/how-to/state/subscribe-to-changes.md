---
title: "Subscribe to Changes"
description: "Listen for value updates using address patterns with wildcards."
section: how-to
order: 4
---
# Subscribe to Changes

Listen for value updates using address patterns with wildcards.

## Basic Subscription

### JavaScript

```javascript
// Subscribe to exact address
client.on('/lights/main/brightness', (value, address) => {
  console.log(`Brightness: ${value}`);
});

// Subscribe returns unsubscribe function
const unsubscribe = client.on('/lights/**', callback);

// Later: stop listening
unsubscribe();
```

### Python

```python
# Using decorator
@client.on('/lights/main/brightness')
def on_brightness(value, address):
    print(f'Brightness: {value}')

# Using subscribe method
def callback(value, address):
    print(f'{address} = {value}')

unsub = client.subscribe('/lights/**', callback)

# Later: stop listening
unsub()
```

### Rust

```rust
let unsub = client.subscribe("/lights/**", |value, addr| {
    println!("{} = {:?}", addr, value);
}).await?;

// Later: stop listening
unsub();
```

## Wildcard Patterns

| Pattern | Matches |
|---------|---------|
| `/lights/main/brightness` | Exact match only |
| `/lights/*/brightness` | `/lights/front/brightness`, `/lights/back/brightness` |
| `/lights/**` | All addresses starting with `/lights/` |

```javascript
// Single segment wildcard
client.on('/lights/*/brightness', (value, address) => {
  // Matches /lights/front/brightness, /lights/back/brightness
  // Not /lights/front/rgb/brightness
});

// Multi-segment wildcard
client.on('/lights/**', (value, address) => {
  // Matches everything under /lights/
});

// Combined
client.on('/app/*/layer/**/opacity', (value, address) => {
  // Matches /app/scene/layer/0/opacity
  // Matches /app/scene/layer/group/0/opacity
});
```

## Initial State (Snapshot)

When you subscribe, you receive the current values immediately:

```javascript
client.on('/lights/**', (value, address) => {
  // Called immediately with all current /lights/** values
  // Then called for each future update
});
```

Skip the initial snapshot:

```javascript
client.on('/lights/**', callback, { skipInitial: true });
```

## Subscription Options

```javascript
client.on('/sensors/**', callback, {
  maxRate: 30,      // Max 30 updates per second
  epsilon: 0.01,    // Only when change > 1%
  skipInitial: true // Don't send current values
});
```

### Rate Limiting

For high-frequency data:

```javascript
// Limit to 30 updates/second
client.on('/sensor/motion', callback, { maxRate: 30 });
```

### Epsilon (Change Threshold)

Only receive when value changes significantly:

```javascript
// Only when brightness changes by more than 1%
client.on('/lights/*/brightness', callback, { epsilon: 0.01 });
```

## Callback Parameters

```javascript
client.on('/path', (value, address, meta) => {
  console.log('Value:', value);
  console.log('Address:', address);
  console.log('Revision:', meta.revision);
  console.log('Writer:', meta.writer);
  console.log('Timestamp:', meta.timestamp);
});
```

## Multiple Subscriptions

```javascript
// Subscribe to multiple patterns
client.on('/lights/**', handleLights);
client.on('/audio/**', handleAudio);
client.on('/sensors/**', handleSensors);
```

## Async Callbacks

In JavaScript, callbacks can be async:

```javascript
client.on('/commands/**', async (value, address) => {
  await processCommand(value);
});
```

## Error Handling

```javascript
client.on('/path', (value, address) => {
  try {
    processValue(value);
  } catch (error) {
    console.error('Error processing:', error);
  }
});
```

## Cleanup

Always unsubscribe when done:

```javascript
// React useEffect pattern
useEffect(() => {
  const unsub = client.on('/path', callback);
  return () => unsub();
}, []);
```

```python
# Python context manager
async with client.subscription('/path', callback):
    # Subscribed while in this block
    pass
# Automatically unsubscribed
```

## Next Steps

- [Get and Set Values](get-set-values.md)
- [Addressing Reference](../../reference/protocol/addressing.md)
