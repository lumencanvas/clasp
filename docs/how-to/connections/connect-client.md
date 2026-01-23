# Connect a Client

Connect to a CLASP router from your application.

## Prerequisites

- A running CLASP router (see [Start a Router](start-router.md))

## JavaScript

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .connect();

console.log('Connected!', client.sessionId);
```

### With Options

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .withFeatures(['param', 'event', 'stream'])
  .withReconnect(true, 5000)  // Auto-reconnect every 5s
  .connect();
```

### Handle Disconnection

```javascript
client.onDisconnect(() => {
  console.log('Disconnected');
});

client.onReconnect(() => {
  console.log('Reconnected');
});
```

## Python

```python
import asyncio
from clasp import ClaspBuilder

async def main():
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('My App')
        .connect()
    )

    print(f'Connected! Session: {client.session_id}')

asyncio.run(main())
```

### With Reconnection

```python
client = await (
    ClaspBuilder('ws://localhost:7330')
    .with_name('My App')
    .with_reconnect(True, interval=5.0)
    .connect()
)
```

## Rust

```rust
use clasp_client::ClaspBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ClaspBuilder::new("ws://localhost:7330")
        .name("My App")
        .connect()
        .await?;

    println!("Connected! Session: {}", client.session_id());
    Ok(())
}
```

## Connection URLs

| URL | Transport |
|-----|-----------|
| `ws://host:port` | WebSocket (unencrypted) |
| `wss://host:port` | WebSocket (TLS) |
| `quic://host:port` | QUIC |
| `udp://host:port` | UDP |

```javascript
// Local development
new ClaspBuilder('ws://localhost:7330')

// Production with TLS
new ClaspBuilder('wss://clasp.example.com:7330')

// Native app with QUIC
new ClaspBuilder('quic://clasp.example.com:7331')
```

## Browser Considerations

Browsers only support WebSocket:

```javascript
// Must use ws:// or wss://
const client = await new ClaspBuilder('ws://localhost:7330').connect();
```

For production, use `wss://` (requires TLS certificate).

## Authentication

### With Token

```javascript
const client = await new ClaspBuilder('wss://server:7330')
  .withToken('eyJhbGciOiJIUzI1NiIs...')
  .connect();
```

### With Pairing Code

```javascript
const client = await new ClaspBuilder('ws://server:7330')
  .withPairingCode('847291')
  .connect();
```

## Error Handling

```javascript
try {
  const client = await new ClaspBuilder('ws://localhost:7330').connect();
} catch (error) {
  if (error.code === 'ECONNREFUSED') {
    console.log('Router not running');
  } else if (error.code === 'AUTH_FAILED') {
    console.log('Authentication failed');
  } else {
    console.log('Connection error:', error.message);
  }
}
```

## Connection Lifecycle

```
1. connect() called
2. WebSocket connection established
3. HELLO message sent
4. WELCOME message received
5. Clock sync performed
6. Ready for use
```

## Cleanup

Always close connections when done:

```javascript
await client.close();
```

Or use try/finally:

```javascript
const client = await new ClaspBuilder(url).connect();
try {
  // Use client
} finally {
  await client.close();
}
```

## Troubleshooting

### "Connection refused"

Router isn't running:
```bash
clasp server --port 7330
```

### "WebSocket connection failed"

- Check URL format (`ws://` not `http://`)
- Verify router is accessible from client's network
- Check firewall settings

### CORS errors (browser)

Router must allow cross-origin. The CLASP router allows all origins by default.

## Next Steps

- [Subscribe to Changes](../state/subscribe-to-changes.md)
- [Get and Set Values](../state/get-set-values.md)
