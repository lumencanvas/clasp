---
title: "Browser Usage"
description: "Browser-specific notes for @clasp-to/core."
section: reference
order: 1
---
# Browser Usage

Browser-specific notes for @clasp-to/core.

## Installation

### NPM + Bundler

```bash
npm install @clasp-to/core
```

Works with Webpack, Vite, Rollup, Parcel, etc.

### CDN (ESM)

```html
<script type="module">
  import { ClaspBuilder } from 'https://unpkg.com/@clasp-to/core/dist/index.mjs';

  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('Browser App')
    .connect();
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
    import { ClaspBuilder } from '@clasp-to/core';

    const client = await new ClaspBuilder('ws://localhost:7330')
      .withName('Slider Demo')
      .connect();

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
const client = await new ClaspBuilder('wss://router.example.com:7330')
  .withName('my-app')
  .connect();
```

### Secure Contexts

Use `wss://` (WebSocket Secure) for:
- Production deployments
- HTTPS pages (mixed content blocked otherwise)

## React Integration

```jsx
import { useState, useEffect, useCallback } from 'react';
import { ClaspBuilder } from '@clasp-to/core';

// Custom hook
function useClasp(url) {
  const [client, setClient] = useState(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    let clasp;

    async function connect() {
      clasp = await new ClaspBuilder(url)
        .withName('React App')
        .connect();

      setClient(clasp);
      setConnected(true);

      clasp.onDisconnect(() => setConnected(false));
      clasp.onReconnect(() => setConnected(true));
    }

    connect();

    return () => {
      if (clasp) clasp.close();
    };
  }, [url]);

  return { client, connected };
}

// Value hook
function useClaspValue(client, address, initialValue) {
  const [value, setValue] = useState(initialValue);

  useEffect(() => {
    if (!client) return;

    // Get initial value
    client.get(address).then((v) => {
      if (v !== undefined) setValue(v);
    });

    const unsubscribe = client.on(address, setValue);
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
import { ClaspBuilder } from '@clasp-to/core';

const client = ref(null);
const brightness = ref(0);

onMounted(async () => {
  client.value = await new ClaspBuilder('ws://localhost:7330')
    .withName('Vue App')
    .connect();

  client.value.on('/lights/brightness', (value) => {
    brightness.value = value;
  });
});

onUnmounted(() => {
  if (client.value) client.value.close();
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
  import { ClaspBuilder } from '@clasp-to/core';

  let client;
  let brightness = 0;

  onMount(async () => {
    client = await new ClaspBuilder('ws://localhost:7330')
      .withName('Svelte App')
      .connect();

    client.on('/lights/brightness', (value) => {
      brightness = value;
    });
  });

  onDestroy(() => {
    if (client) client.close();
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

## Connection State UI

```javascript
const statusEl = document.getElementById('status');

client.onConnect(() => {
  statusEl.textContent = 'Connected';
  statusEl.className = 'online';
});

client.onDisconnect(() => {
  statusEl.textContent = 'Disconnected';
  statusEl.className = 'offline';
});

client.onReconnect(() => {
  statusEl.textContent = 'Reconnected';
  statusEl.className = 'online';
});
```

### Queue Offline Operations

```javascript
const offlineQueue = [];

function safeSend(address, value) {
  if (client.connected) {
    client.set(address, value);
  } else {
    offlineQueue.push({ address, value });
  }
}

client.onConnect(() => {
  for (const { address, value } of offlineQueue) {
    client.set(address, value);
  }
  offlineQueue.length = 0;
});
```

## Security

### Token Storage

```javascript
// DON'T store tokens in localStorage for sensitive data
// DO use httpOnly cookies or short-lived tokens

const client = await new ClaspBuilder('wss://router.example.com:7330')
  .withToken(await fetchTokenFromServer())
  .connect();
```

## Bundle Size

The browser bundle is ~15KB gzipped (~5KB minified).

## See Also

- [@clasp-to/core API](clasp-core.md) - Full API reference
- [First Connection Tutorial](../../../tutorials/first-connection.md)
