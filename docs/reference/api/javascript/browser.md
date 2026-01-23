# Browser Usage

Browser-specific notes for @clasp-to/core.

## Installation

### NPM + Bundler

```bash
npm install @clasp-to/core
```

Works with Webpack, Vite, Rollup, Parcel, etc.

### CDN

```html
<script type="module">
  import { Clasp } from 'https://cdn.jsdelivr.net/npm/@clasp-to/core/+esm';

  const client = await Clasp.connect('ws://localhost:7330');
</script>
```

### UMD Build

```html
<script src="https://cdn.jsdelivr.net/npm/@clasp-to/core/dist/clasp.umd.js"></script>
<script>
  const { Clasp } = window.ClaspTo;
  Clasp.connect('ws://localhost:7330').then(client => {
    // ...
  });
</script>
```

## Quick Start

```html
<!DOCTYPE html>
<html>
<head>
  <title>CLASP Demo</title>
</head>
<body>
  <input type="range" id="slider" min="0" max="100">
  <div id="value">0</div>

  <script type="module">
    import { Clasp } from '@clasp-to/core';

    const client = await Clasp.connect('ws://localhost:7330');

    // Send slider value
    document.getElementById('slider').addEventListener('input', (e) => {
      client.set('/control/slider', parseInt(e.target.value));
    });

    // Receive updates
    client.on('/control/slider', (value) => {
      document.getElementById('value').textContent = value;
    });
  </script>
</body>
</html>
```

## WebSocket Security

### Same-Origin Policy

Browsers enforce same-origin policy. For cross-origin WebSocket:

```javascript
// Router must allow cross-origin connections
// Or use a proxy
const client = await Clasp.connect('wss://router.example.com:7330');
```

### Secure Contexts

Use `wss://` (WebSocket Secure) for:
- Production deployments
- HTTPS pages (mixed content blocked otherwise)

```javascript
const client = await Clasp.connect('wss://router.example.com:7330');
```

## React Integration

```jsx
import { useState, useEffect, useCallback } from 'react';
import { Clasp } from '@clasp-to/core';

// Custom hook
function useClasp(url) {
  const [client, setClient] = useState(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    let clasp;

    async function connect() {
      clasp = await Clasp.connect(url);
      setClient(clasp);
      setConnected(true);

      clasp.on('disconnected', () => setConnected(false));
      clasp.on('connected', () => setConnected(true));
    }

    connect();

    return () => {
      if (clasp) clasp.disconnect();
    };
  }, [url]);

  return { client, connected };
}

// Value hook
function useClaspValue(client, address, initialValue) {
  const [value, setValue] = useState(initialValue);

  useEffect(() => {
    if (!client) return;

    const unsubscribe = client.on(address, setValue, { includeInitial: true });
    return unsubscribe;
  }, [client, address]);

  const set = useCallback((newValue) => {
    if (client) client.set(address, newValue);
  }, [client, address]);

  return [value, set];
}

// Usage
function LightControl() {
  const { client, connected } = useClasp('ws://localhost:7330');
  const [brightness, setBrightness] = useClaspValue(client, '/lights/brightness', 0);

  if (!connected) return <div>Connecting...</div>;

  return (
    <div>
      <input
        type="range"
        min="0"
        max="255"
        value={brightness}
        onChange={(e) => setBrightness(parseInt(e.target.value))}
      />
      <span>{brightness}</span>
    </div>
  );
}
```

## Vue Integration

```vue
<template>
  <div>
    <input
      type="range"
      min="0"
      max="255"
      :value="brightness"
      @input="setBrightness($event.target.value)"
    />
    <span>{{ brightness }}</span>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { Clasp } from '@clasp-to/core';

const client = ref(null);
const brightness = ref(0);

onMounted(async () => {
  client.value = await Clasp.connect('ws://localhost:7330');

  client.value.on('/lights/brightness', (value) => {
    brightness.value = value;
  }, { includeInitial: true });
});

onUnmounted(() => {
  if (client.value) client.value.disconnect();
});

function setBrightness(value) {
  client.value?.set('/lights/brightness', parseInt(value));
}
</script>
```

## Svelte Integration

```svelte
<script>
  import { onMount, onDestroy } from 'svelte';
  import { Clasp } from '@clasp-to/core';

  let client;
  let brightness = 0;

  onMount(async () => {
    client = await Clasp.connect('ws://localhost:7330');

    client.on('/lights/brightness', (value) => {
      brightness = value;
    }, { includeInitial: true });
  });

  onDestroy(() => {
    if (client) client.disconnect();
  });

  function handleInput(e) {
    const value = parseInt(e.target.value);
    client?.set('/lights/brightness', value);
  }
</script>

<input type="range" min="0" max="255" value={brightness} on:input={handleInput} />
<span>{brightness}</span>
```

## Performance Tips

### Throttle UI Updates

```javascript
client.on('/high-rate/data', updateUI, {
  maxRate: 60  // Max 60fps
});
```

### Use requestAnimationFrame

```javascript
let latestValue = null;

client.on('/animation/value', (value) => {
  latestValue = value;
});

function animate() {
  if (latestValue !== null) {
    updateCanvas(latestValue);
  }
  requestAnimationFrame(animate);
}

requestAnimationFrame(animate);
```

### Batch DOM Updates

```javascript
const updates = {};

client.on('/sensors/**', (value, address) => {
  updates[address] = value;
});

setInterval(() => {
  for (const [address, value] of Object.entries(updates)) {
    document.getElementById(address).textContent = value;
  }
}, 100);
```

## Offline Support

### Connection State UI

```javascript
const statusEl = document.getElementById('status');

client.on('connected', () => {
  statusEl.textContent = 'Connected';
  statusEl.className = 'online';
});

client.on('disconnected', () => {
  statusEl.textContent = 'Disconnected';
  statusEl.className = 'offline';
});

client.on('reconnecting', (attempt) => {
  statusEl.textContent = `Reconnecting (${attempt})...`;
  statusEl.className = 'connecting';
});
```

### Queue Offline Operations

```javascript
const offlineQueue = [];

async function safeSend(address, value) {
  if (client.isConnected()) {
    await client.set(address, value);
  } else {
    offlineQueue.push({ address, value });
  }
}

client.on('connected', async () => {
  for (const { address, value } of offlineQueue) {
    await client.set(address, value);
  }
  offlineQueue.length = 0;
});
```

## Security

### Token Storage

```javascript
// DON'T store tokens in localStorage for sensitive data
// DO use httpOnly cookies or short-lived tokens

const client = await Clasp.builder('wss://router.example.com:7330')
  .withToken(await fetchTokenFromServer())
  .withTokenRefresh(async () => {
    return await fetchNewToken();
  })
  .connect();
```

## Bundle Size

The browser bundle is ~15KB gzipped. To reduce further:

```javascript
// Import only what you need
import { connect, set, get, on } from '@clasp-to/core/minimal';
```

## See Also

- [@clasp-to/core API](clasp-core.md) - Full API reference
- [First Connection Tutorial](../../../tutorials/first-connection.md)
