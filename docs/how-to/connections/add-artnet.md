# Add Art-Net

Connect Art-Net lighting nodes to CLASP for DMX over Ethernet.

## Prerequisites

- Running CLASP router
- Art-Net nodes on the network

## Start Art-Net Bridge

### CLI

```bash
clasp artnet --bind 0.0.0.0:6454
```

### Desktop App

1. Click **Add Protocol**
2. Select **Art-Net**
3. Configure bind address (default: 0.0.0.0:6454)
4. Click **Start**

## Address Format

Art-Net addresses include net, subnet, and universe:

```
/artnet/{net}/{subnet}/{universe}/{channel}
```

For simple setups (universe 0):
```
/artnet/0/0/0/{channel}
```

## Control Lights

```javascript
// Set channel 1 in universe 0 to full
await client.set('/artnet/0/0/0/1', 255);

// Set multiple channels
await client.bundle([
  { set: ['/artnet/0/0/0/1', 255] },   // Dimmer
  { set: ['/artnet/0/0/0/2', 255] },   // Red
  { set: ['/artnet/0/0/0/3', 0] },     // Green
  { set: ['/artnet/0/0/0/4', 128] }    // Blue
]);
```

## Receive from Art-Net

Art-Net is bidirectional. Receive data from nodes:

```javascript
client.on('/artnet/**', (value, address) => {
  console.log(address, value);
});
```

## Network Configuration

Art-Net typically uses:
- Port: 6454 (UDP)
- Network: 2.x.x.x or 10.x.x.x (traditional) or any subnet

```bash
# Bind to specific interface
clasp artnet --bind 2.0.0.1:6454

# Or on 10.x.x.x network
clasp artnet --bind 10.0.0.1:6454
```

## Art-Net Sync

For synchronized output, use sync mode:

```bash
clasp artnet --bind 0.0.0.0:6454 --sync
```

With sync enabled, data is buffered until a sync packet triggers output.

## Multiple Universes

Art-Net supports up to 32,768 universes:

```javascript
// Universe 0
await client.set('/artnet/0/0/0/1', 255);

// Universe 1
await client.set('/artnet/0/0/1/1', 255);

// Universe 16 (different subnet)
await client.set('/artnet/0/1/0/1', 255);
```

## Polling

Enable Art-Net polling to discover nodes:

```bash
clasp artnet --bind 0.0.0.0:6454 --poll
```

Discovered nodes appear as announcements in CLASP.

## Troubleshooting

### Lights not responding

1. Check network connectivity:
   ```bash
   ping 2.0.0.1  # Art-Net node IP
   ```

2. Verify Art-Net port:
   ```bash
   sudo tcpdump -i eth0 port 6454
   ```

3. Check universe addressing matches fixtures

### Broadcast issues

Art-Net uses broadcast by default. If on different subnets:

```bash
# Unicast to specific node
clasp artnet --bind 0.0.0.0:6454 --unicast 2.0.0.100
```

### High latency

- Use wired Ethernet (not WiFi)
- Reduce number of universes
- Enable sync mode for large installations

## Example: Complete Lighting Setup

```javascript
// Define fixtures
const fixtures = [
  { name: 'wash1', universe: 0, start: 1, channels: 8 },
  { name: 'wash2', universe: 0, start: 9, channels: 8 },
  { name: 'spot1', universe: 1, start: 1, channels: 16 },
];

// Set fixture channel
async function setChannel(fixture, channel, value) {
  const addr = `/artnet/0/0/${fixture.universe}/${fixture.start + channel}`;
  await client.set(addr, value);
}

// Set wash1 to blue
await setChannel(fixtures[0], 0, 255);  // Dimmer
await setChannel(fixtures[0], 3, 255);  // Blue
```

## Art-Net vs DMX

| Feature | Art-Net | USB DMX |
|---------|---------|---------|
| Connectivity | Ethernet | USB |
| Distance | 100m+ | 5m (USB) |
| Universes | 32,768 | 1 per interface |
| Latency | ~5-10ms | ~1-2ms |
| Cost | Higher | Lower |

Use Art-Net for:
- Large installations
- Long cable runs
- Multiple universes
- Network integration

## Next Steps

- [Add DMX](add-dmx.md) — USB DMX alternative
- [Art-Net Bridge Reference](../../reference/bridges/artnet.md)
- [Resolume Integration](../../integrations/resolume.md)
