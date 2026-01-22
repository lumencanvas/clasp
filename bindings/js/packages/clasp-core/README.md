# @clasp-to/core

JavaScript/TypeScript client for CLASP - Creative Low-Latency Application Streaming Protocol.

[![npm](https://img.shields.io/npm/v/@clasp-to/core)](https://www.npmjs.com/package/@clasp-to/core)
[![License](https://img.shields.io/npm/l/@clasp-to/core)](LICENSE)

## Installation

```bash
npm install @clasp-to/core
```

## Quick Start

```typescript
import { Clasp, ClaspBuilder } from '@clasp-to/core';

// Connect to a CLASP server
const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .connect();

// Subscribe to parameter changes
client.on('/lumen/layer/*/opacity', (value, address) => {
  console.log(`${address} = ${value}`);
});

// Set a parameter
await client.set('/lumen/layer/0/opacity', 0.75);

// Get a parameter
const opacity = await client.get('/lumen/layer/0/opacity');

// Emit an event
await client.emit('/cue/fire', { id: 'intro' });

// Stream high-rate data
client.stream('/fader/1', 0.5);

// Close when done
await client.close();
```

## API

### ClaspBuilder

```typescript
const client = await new ClaspBuilder(url)
  .withName('Client Name')        // Set client name
  .withFeatures(['param', 'event']) // Specify features
  .withReconnect(true, 5000)      // Auto-reconnect with interval
  .connect();
```

### Clasp Client

#### Reading

- `get(address)` - Get parameter value
- `on(pattern, callback)` - Subscribe to address pattern
- `cached(address)` - Get cached value (sync)

#### Writing

- `set(address, value)` - Set parameter (stateful)
- `emit(address, payload?)` - Emit event (ephemeral)
- `stream(address, value)` - Stream sample (high-rate)

#### Bundles

```typescript
// Atomic bundle
client.bundle([
  { set: ['/light/1', 1.0] },
  { set: ['/light/2', 0.0] }
]);

// Scheduled bundle
client.bundle([...], { at: client.time() + 100000 }); // 100ms later
```

#### Utilities

- `time()` - Get server-synced time (microseconds)
- `connected` - Check connection status
- `sessionId` - Get session ID
- `close()` - Close connection

### Address Patterns

CLASP supports wildcards in subscriptions:

| Pattern | Matches |
|---------|---------|
| `/lights/front` | Exact match |
| `/lights/*` | Single segment wildcard |
| `/lights/**` | Multi-segment wildcard |

## Browser Compatibility

`@clasp-to/core` works in both Node.js and browser environments.

### Browser Support

| Browser | Version | Notes |
|---------|---------|-------|
| Chrome | 68+ | Full support |
| Firefox | 63+ | Full support |
| Safari | 12+ | Full support |
| Edge | 79+ | Full support (Chromium) |
| IE | Not supported | Use Edge or polyfills |

### Bundle Size

- **ESM**: ~15KB minified
- **ESM + gzip**: ~5KB

### Browser Usage

```html
<script type="module">
import { Clasp } from 'https://unpkg.com/@clasp-to/core/dist/index.mjs';

const clasp = new Clasp('wss://your-server.com:7330');
await clasp.connect();

// Use normally
clasp.on('/sensor/*', (value, addr) => {
  document.getElementById('display').textContent = `${addr}: ${value}`;
});
</script>
```

### Build Tool Integration

Works with all modern bundlers:

```javascript
// Vite, Rollup, esbuild, webpack 5+
import { Clasp } from '@clasp-to/core';
```

### React Example

```jsx
import { useEffect, useState, useRef } from 'react';
import { Clasp } from '@clasp-to/core';

function useClasp(url) {
  const clientRef = useRef(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const client = new Clasp(url);
    clientRef.current = client;

    client.connect().then(() => setConnected(true));
    client.onDisconnect(() => setConnected(false));

    return () => client.close();
  }, [url]);

  return { client: clientRef.current, connected };
}

function Fader({ address }) {
  const { client, connected } = useClasp('wss://localhost:7330');
  const [value, setValue] = useState(0);

  useEffect(() => {
    if (!client || !connected) return;

    const unsub = client.on(address, (v) => setValue(v));
    return unsub;
  }, [client, connected, address]);

  const handleChange = (e) => {
    const v = parseFloat(e.target.value);
    setValue(v);
    client?.set(address, v);
  };

  return (
    <input
      type="range"
      min="0"
      max="1"
      step="0.01"
      value={value}
      onChange={handleChange}
      disabled={!connected}
    />
  );
}
```

### Known Limitations

- **No mDNS discovery**: Browsers cannot perform mDNS lookups. Provide explicit server URLs.
- **No raw UDP/TCP**: Only WebSocket transport is available in browsers.
- **CORS**: Server must allow cross-origin connections if client is served from different domain.

## Documentation

Visit **[clasp.to](https://clasp.to)** for full documentation.

## License

MIT

---

Maintained by [LumenCanvas](https://lumencanvas.studio) | 2026
