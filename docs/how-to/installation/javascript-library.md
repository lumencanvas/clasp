# JavaScript Library

Add CLASP to JavaScript or TypeScript projects.

## Install

```bash
npm install @clasp-to/core
```

Or with other package managers:

```bash
# Yarn
yarn add @clasp-to/core

# pnpm
pnpm add @clasp-to/core
```

## Usage

### ES Modules (Recommended)

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .connect();
```

### CommonJS

```javascript
const { ClaspBuilder } = require('@clasp-to/core');
```

### TypeScript

Full TypeScript support included:

```typescript
import { ClaspBuilder, Clasp, Value } from '@clasp-to/core';

const client: Clasp = await new ClaspBuilder('ws://localhost:7330')
  .connect();

await client.set('/path', 42 as Value);
```

### Browser

```html
<script type="module">
import { ClaspBuilder } from 'https://unpkg.com/@clasp-to/core/dist/index.mjs';

const client = await new ClaspBuilder('ws://localhost:7330').connect();
</script>
```

## Bundle Size

- **ESM:** ~15KB minified
- **ESM + gzip:** ~5KB

## Browser Compatibility

| Browser | Version |
|---------|---------|
| Chrome | 68+ |
| Firefox | 63+ |
| Safari | 12+ |
| Edge | 79+ |

## Framework Integration

### React

```jsx
import { useEffect, useState, useRef } from 'react';
import { ClaspBuilder } from '@clasp-to/core';

function useClasp(url) {
  const [client, setClient] = useState(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const builder = new ClaspBuilder(url);
    builder.connect().then(c => {
      setClient(c);
      setConnected(true);
    });

    return () => client?.close();
  }, [url]);

  return { client, connected };
}
```

### Vue

```vue
<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { ClaspBuilder } from '@clasp-to/core';

const client = ref(null);
const connected = ref(false);

onMounted(async () => {
  client.value = await new ClaspBuilder('ws://localhost:7330').connect();
  connected.value = true;
});

onUnmounted(() => {
  client.value?.close();
});
</script>
```

## Next Steps

- [Connect a Client](../connections/connect-client.md)
- [Subscribe to Changes](../state/subscribe-to-changes.md)
- [JavaScript API Reference](../../reference/api/javascript/clasp-core.md)
