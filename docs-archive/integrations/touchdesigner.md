---
title: "TouchDesigner Integration"
description: "Connect TouchDesigner to CLASP for interactive visual systems."
section: integrations
order: 7
---
# TouchDesigner Integration

Connect TouchDesigner to CLASP for interactive visual systems.

## Overview

TouchDesigner supports OSC, MIDI, and WebSocket communication. CLASP enables you to:

- Receive sensor data and control signals in TouchDesigner
- Send TouchDesigner parameters to other systems
- Bridge TouchDesigner with hardware (DMX, MIDI controllers)

## Setup Options

### Option 1: OSC (Most Common)

```bash
# Start CLASP OSC bridge
clasp osc --port 9000 --target 127.0.0.1:10000
```

In TouchDesigner:
1. Create an **OSC In CHOP** - set port to 9000
2. Create an **OSC Out CHOP** - set address to 127.0.0.1, port 10000

### Option 2: WebSocket (Direct)

TouchDesigner can connect directly to CLASP router:

```python
# TouchDesigner Python script
import json

def onReceive(dat, rowIndex, message, bytes, peer):
    data = json.loads(message)
    if data['type'] == 'SET':
        # Update TouchDesigner parameters
        op('constant1').par.value0 = data['value']
```

### Option 3: MIDI

```bash
clasp midi --device "TouchDesigner MIDI"
```

## OSC Communication

### Receiving in TouchDesigner

OSC In CHOP receives CLASP messages:

```
CLASP address: /control/fader1
→ OSC In CHOP channel: control:fader1 (or v1 in legacy mode)
```

Convert OSC to CHOP channels:
1. OSC In CHOP → Select CHOP (pick channels)
2. Or use **OSC In DAT** for message-based processing

### Sending from TouchDesigner

OSC Out CHOP sends to CLASP:

```python
# Script CHOP or Python
def sendToClasp(address, value):
    op('oscout1').sendOSC(address, [value])

# Example
sendToClasp('/td/output/level', 0.5)
```

## Example: Sensor-Driven Visuals

Route sensors through CLASP to TouchDesigner:

```javascript
// CLASP client (Node.js)
// Forward sensor data to TouchDesigner
client.on('/sensors/**', async (value, address) => {
  // Forward to TouchDesigner via OSC bridge
  await client.set('/osc/td' + address, value);
});

// Forward MIDI controller
client.on('/midi/controller/cc/*/*', async (value, address) => {
  const cc = address.split('/').pop();
  await client.set(`/osc/td/midi/cc/${cc}`, value / 127);
});
```

In TouchDesigner, map to TOP parameters:
1. OSC In CHOP receives `/td/sensors/distance`
2. Math CHOP normalizes values
3. CHOP Execute calls Python to update TOP

## Example: TouchDesigner to DMX

Control lights from TouchDesigner:

```python
# TouchDesigner Python
# In CHOP Execute DAT

def onValueChange(channel, sampleIndex, val, prev):
    # Send TouchDesigner values to CLASP
    if channel.name.startswith('light'):
        address = f'/osc/artnet/{channel.name}'
        op('oscout1').sendOSC(address, [val])
```

```javascript
// CLASP bridge (Node.js)
// Route TouchDesigner output to Art-Net
client.on('/osc/artnet/light*', async (value, address) => {
  const channel = parseInt(address.split('light')[1]);
  await client.set(`/artnet/0/0/0/${channel}`, Math.round(value * 255));
});
```

## Example: Audio-Reactive System

Audio analysis in TouchDesigner, output to other systems:

```python
# TouchDesigner Python
# In CHOP Execute on Audio Analysis CHOP

def onValueChange(channel, sampleIndex, val, prev):
    if channel.name == 'bass':
        op('oscout1').sendOSC('/td/audio/bass', [val])
    elif channel.name == 'mid':
        op('oscout1').sendOSC('/td/audio/mid', [val])
    elif channel.name == 'high':
        op('oscout1').sendOSC('/td/audio/high', [val])
```

```javascript
// CLASP routes to lighting
client.on('/osc/td/audio/bass', async (value) => {
  // Bass drives floor lights
  await client.set('/artnet/0/0/0/1', Math.round(value * 255));
});

client.on('/osc/td/audio/high', async (value) => {
  // Highs drive overhead
  await client.set('/artnet/0/0/0/5', Math.round(value * 255));
});
```

## Example: Multi-Screen Setup

Coordinate multiple TouchDesigner instances:

```javascript
// Central CLASP coordinator

// Receive from master TD instance
client.on('/osc/td/master/**', async (value, address) => {
  const param = address.replace('/osc/td/master/', '');

  // Broadcast to all slave instances
  await client.bundle([
    { set: [`/osc/td/slave1/${param}`, value] },
    { set: [`/osc/td/slave2/${param}`, value] },
    { set: [`/osc/td/slave3/${param}`, value] }
  ]);
});

// Sync playback position
client.on('/osc/td/master/timeline/position', async (frame) => {
  await client.bundle([
    { set: ['/osc/td/slave1/timeline/position', frame] },
    { set: ['/osc/td/slave2/timeline/position', frame] },
    { set: ['/osc/td/slave3/timeline/position', frame] }
  ]);
});
```

## Example: Interactive Installation

TouchDesigner as the visual engine:

```javascript
// CLASP aggregates all inputs for TouchDesigner

// Motion sensors
client.on('/mqtt/sensors/motion/*', async (value, address) => {
  const zone = address.split('/').pop();
  await client.set(`/osc/td/motion/${zone}`, value ? 1 : 0);
});

// Distance sensors (presence detection)
client.on('/http/sensors/distance/*', async (value, address) => {
  const sensor = address.split('/').pop();
  // Normalize to 0-1
  const normalized = Math.max(0, Math.min(1, value / 200));
  await client.set(`/osc/td/presence/${sensor}`, normalized);
});

// Environmental data
client.on('/mqtt/environment/#', async (value, address) => {
  const parts = address.split('/');
  const metric = parts[3];
  await client.set(`/osc/td/environment/${metric}`, value);
});
```

TouchDesigner network:
```
OSC In CHOP (motion zones)
    → Select CHOP
    → Trigger CHOP (edge detection)
    → CHOP Execute → Python → Trigger effects

OSC In CHOP (presence)
    → Math CHOP (smooth)
    → Lookup CHOP (response curve)
    → CHOP to SOP (geometry deformation)
```

## WebSocket Direct Connection

For lowest latency, connect TouchDesigner directly to CLASP router:

```python
# TouchDesigner WebSocket DAT callback

import json

def onReceive(dat, rowIndex, message, bytes, peer):
    try:
        data = json.loads(message)
        if data.get('type') == 'SET':
            address = data['address']
            value = data['value']

            # Route to appropriate operators
            if address.startswith('/control/'):
                param = address.split('/')[-1]
                op('control')[param].val = value

    except json.JSONDecodeError:
        pass

def sendSet(address, value):
    msg = json.dumps({
        'type': 'SET',
        'address': address,
        'value': value
    })
    op('websocket1').sendText(msg)
```

## Tips

### Optimize Performance

```python
# Rate limit OSC output
# Use a Timer CHOP to batch updates

import time

lastSend = {}
minInterval = 0.033  # 30fps max

def throttledSend(address, value):
    now = time.time()
    if address not in lastSend or (now - lastSend[address]) > minInterval:
        op('oscout1').sendOSC(address, [value])
        lastSend[address] = now
```

### Handle Arrays

```python
# Send array data
def sendArray(address, values):
    op('oscout1').sendOSC(address, values)

# Send matrix data
def sendMatrix(address, matrix):
    flat = [v for row in matrix for v in row]
    op('oscout1').sendOSC(address, flat)
```

### Debug Communication

```python
# Log all incoming OSC
def onReceiveOSC(address, args):
    print(f'OSC: {address} = {args}')
    # Also display in textport
    op('text_debug').text = f'{address}: {args}'
```

## Troubleshooting

### OSC Not Receiving

1. Check OSC In CHOP port matches CLASP bridge
2. Verify firewall allows UDP traffic
3. Use `Active` parameter in OSC In CHOP
4. Check Network column in OSC In CHOP

### High Latency

1. Use Cook on Demand for non-realtime components
2. Reduce OSC message rate
3. Use binary format for large data
4. Consider WebSocket for lower overhead

### Values Not Updating

1. Check CHOP channels are cooking
2. Verify value ranges match expectations
3. Add Math CHOP to debug values
4. Check for channel name conflicts

## Next Steps

- [Installation Art Guide](../use-cases/installation-art.md)
- [Add OSC Connection](../how-to/connections/add-osc.md)
- [Resolume Integration](resolume.md)
