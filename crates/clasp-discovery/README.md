# clasp-discovery

Network discovery for CLASP (Creative Low-Latency Application Streaming Protocol) devices and servers.

## Features

- **mDNS/DNS-SD** - Zero-configuration discovery on local networks
- **UDP Broadcast** - Fallback discovery when mDNS is unavailable
- **Rendezvous Server** - WAN discovery via HTTP REST API
- **Cascade Discovery** - Automatically try mDNS → broadcast → rendezvous
- **Auto-Keepalive** - Automatic registration refresh with rendezvous server

## Feature Flags

| Feature | Description |
|---------|-------------|
| `mdns` | mDNS/DNS-SD discovery (default) |
| `broadcast` | UDP broadcast discovery (default) |
| `rendezvous` | WAN discovery via rendezvous server |

## Basic Usage

```rust
use clasp_discovery::Discovery;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut discovery = Discovery::new();

    // Start discovering devices
    let mut rx = discovery.start().await?;

    // Process discovery events
    while let Some(event) = rx.recv().await {
        match event {
            clasp_discovery::DiscoveryEvent::Found(device) => {
                println!("Found: {} at {:?}", device.name, device.endpoints);
            }
            clasp_discovery::DiscoveryEvent::Lost(id) => {
                println!("Lost device: {}", id);
            }
            clasp_discovery::DiscoveryEvent::Error(e) => {
                eprintln!("Discovery error: {}", e);
            }
        }
    }

    Ok(())
}
```

## WAN Discovery (Rendezvous)

For discovery across the internet, use the rendezvous feature:

```toml
[dependencies]
clasp-discovery = { version = "3.3", features = ["rendezvous"] }
```

```rust
use clasp_discovery::{Discovery, DiscoveryConfig, DeviceRegistration};
use std::time::Duration;

let config = DiscoveryConfig {
    // Use the public CLASP relay (includes rendezvous)
    rendezvous_url: Some("https://relay.clasp.to".into()),
    rendezvous_refresh_interval: Duration::from_secs(120),
    rendezvous_tag: Some("studio".into()),
    ..Default::default()
};

let mut discovery = Discovery::with_config(config);

// Register this device (starts automatic keepalive)
discovery.register_with_rendezvous(DeviceRegistration {
    name: "My Device".into(),
    endpoints: [("ws".into(), "wss://my-device.local:7330".into())].into(),
    tags: vec!["studio".into()],
    ..Default::default()
});

// Discover all devices (mDNS → broadcast → rendezvous)
let devices = discovery.discover_all().await?;

// Or discover WAN devices only
let wan_devices = discovery.discover_wan().await?;
```

## Rendezvous Server

The rendezvous server is **built into the CLASP relay server** by default. When you run `clasp-relay`, rendezvous is automatically available on port 7340.

```bash
# Start relay with rendezvous (default)
clasp-relay --ws-port 7330 --rendezvous-port 7340

# Disable rendezvous
clasp-relay --rendezvous-port 0
```

You can also run a standalone rendezvous server:

```rust
use clasp_discovery::rendezvous::{RendezvousServer, RendezvousConfig};

let server = RendezvousServer::new(RendezvousConfig {
    ttl: 300,           // 5 minute registration TTL
    cleanup_interval: 60,
    ..Default::default()
});

server.serve("0.0.0.0:7340").await?;
```

## mDNS Service Type

CLASP uses the service type `_clasp._tcp.local` for mDNS discovery.

## Documentation

Visit **[clasp.to](https://clasp.to)** for full documentation.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio) | 2026
