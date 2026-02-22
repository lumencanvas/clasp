---
title: Discovery
description: Automatic device and router discovery
order: 5
---

# Discovery

CLASP supports automatic discovery so clients can find routers without hardcoded URLs. Three discovery methods cover both LAN and WAN scenarios, and all CLASP SDKs provide a unified `discover()` API that cascades through available methods.

## Discovery Methods

| Method | Scope | How It Works | Enable |
|--------|-------|--------------|--------|
| mDNS | LAN | Bonjour/Avahi service announcement | Automatic in SDKs |
| UDP Broadcast | LAN | Broadcast/response on port 7331 | Automatic in SDKs |
| Rendezvous | WAN | Central relay registry | `--rendezvous-port 7340` |

On a local network, mDNS and UDP broadcast work without any server-side configuration. For discovery across the internet, use the rendezvous server.

## mDNS

mDNS provides zero-configuration LAN discovery. The relay announces itself as a `_clasp._tcp.local` service, and clients find it through standard mDNS/DNS-SD queries.

Platform support:

| Platform | Implementation |
|----------|----------------|
| macOS | Bonjour (built-in) |
| Linux | Avahi (install `avahi-daemon` if not present) |
| Windows | Bonjour for Windows or built-in mDNS responder |

The relay automatically announces itself on startup. No flags are needed. The announcement includes the router's name, WebSocket port, and protocol version.

Clients using the CLASP SDKs discover mDNS services automatically:

```javascript
import { discover } from '@clasp-to/core';

const devices = await discover({ timeout: 3000 });
// [{ name: 'My Relay', url: 'ws://192.168.1.50:7330', source: 'mdns' }]
```

```python
from clasp import discover

devices = discover(timeout=3.0)
# [Device(name='My Relay', url='ws://192.168.1.50:7330', source='mdns')]
```

```rust
use clasp_discovery::discover;

let devices = discover(DiscoveryConfig {
    timeout: Duration::from_secs(3),
    ..Default::default()
}).await?;
```

## UDP Broadcast

UDP broadcast is a fallback for networks where mDNS is unavailable or blocked (some enterprise networks, certain Wi-Fi configurations). The client broadcasts a discovery packet on port 7331, and any relay on the network responds with its connection info.

The default broadcast port is 7331. To change it on the relay:

```bash
clasp-relay --broadcast-port 8331
```

On the client side, specify the matching port in the discovery config:

```javascript
const devices = await discover({
  broadcastPort: 8331,
  timeout: 3000
});
```

UDP broadcast only works within the same network broadcast domain. It does not cross routers or VLANs.

## Rendezvous Server

The rendezvous server enables discovery across the internet. Relays register with a central rendezvous service, and clients query the service to find available routers.

### Start the Rendezvous Service

The relay can run a rendezvous service on a dedicated port:

```bash
clasp-relay --rendezvous-port 7340 --rendezvous-ttl 300
```

| Flag | Default | Description |
|------|---------|-------------|
| `--rendezvous-port` | none | Port for the rendezvous HTTP service |
| `--rendezvous-ttl` | `300` | Seconds before a registration expires |

### Register a Relay

Other relays register with the rendezvous service:

```bash
clasp-relay \
  --rendezvous-url http://rendezvous.example.com:7340 \
  --rendezvous-refresh 120 \
  --rendezvous-tag "production"
```

| Flag | Default | Description |
|------|---------|-------------|
| `--rendezvous-url` | none | URL of the rendezvous service |
| `--rendezvous-refresh` | `120` | Seconds between registration renewals |
| `--rendezvous-tag` | none | Optional tag for filtering |

The relay re-registers at the refresh interval to keep its entry alive. If a relay shuts down or fails to renew, its entry expires after the TTL.

### Client Discovery via Rendezvous

Clients query the rendezvous service to find available routers:

```javascript
const devices = await discover({
  rendezvousUrl: 'http://rendezvous.example.com:7340',
  rendezvousTag: 'production',
  timeout: 5000
});
// [{ name: 'Prod Relay', url: 'ws://relay.example.com:7330', source: 'rendezvous' }]
```

## Client-Side Discovery

The SDK `discover()` function cascades through available discovery methods in order:

1. **mDNS** -- queries the local network for `_clasp._tcp.local` services.
2. **UDP Broadcast** -- broadcasts on the configured port and waits for responses.
3. **Rendezvous** -- queries the rendezvous URL if configured.

All discovered devices are returned in a single list with a `source` field indicating how each was found.

```javascript
import { discover } from '@clasp-to/core';

const devices = await discover({
  timeout: 5000,
  rendezvousUrl: 'http://rendezvous.example.com:7340'
});

for (const device of devices) {
  console.log(`${device.name} at ${device.url} (via ${device.source})`);
}
```

Discovery events are also available for real-time monitoring:

```javascript
import { DiscoveryListener } from '@clasp-to/core';

const listener = new DiscoveryListener();

listener.on('found', (device) => {
  console.log(`Found: ${device.name} at ${device.url}`);
});

listener.on('lost', (deviceId) => {
  console.log(`Lost: ${deviceId}`);
});

listener.start();
```

The listener emits three event types:

| Event | Description |
|-------|-------------|
| `found` | A new device was discovered |
| `lost` | A previously discovered device is no longer available |
| `error` | A discovery method encountered an error |

## Manual Connection

Discovery is optional. You can always connect directly by providing the relay URL:

```javascript
import { ClaspClient } from '@clasp-to/core';

const client = new ClaspClient('ws://192.168.1.50:7330');
await client.connect();
```

Manual connection is the right choice when:

- The relay address is known and stable (e.g., a cloud deployment with a DNS name).
- Discovery is blocked by network policy.
- You want deterministic connection behavior without scanning.

## Next Steps

- [Relay Server](../deployment/relay.md) -- relay deployment and configuration
- [Federation](./federation.md) -- multi-site state sync
- [Getting Started](../getting-started/README.md) -- first connection tutorial
