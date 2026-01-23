# Control Lights from Web Tutorial

Build a web interface that controls DMX lights through CLASP.

**Time:** 20-30 minutes
**Prerequisites:** [First Connection](first-connection.md) tutorial, basic HTML/JavaScript

## What You'll Build

A browser-based control panel that controls DMX lighting fixtures:

```
┌─────────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Web Browser   │────►│   Router    │────►│ Art-Net     │────►│   Lights    │
│  (index.html)   │     │ (port 7330) │     │ Bridge      │     │  (DMX)      │
└─────────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

## Step 1: Start the Infrastructure

Open three terminals:

**Terminal 1 - Router:**
```bash
clasp server --port 7330
```

**Terminal 2 - Art-Net Bridge:**
```bash
clasp artnet --bind 0.0.0.0:6454
```

This creates a bridge that:
- Listens for Art-Net on port 6454
- Connects to the CLASP router
- Maps `/dmx/{universe}/{channel}` addresses to DMX channels

## Step 2: Create the Web Interface

Create `index.html`:

```html
<!DOCTYPE html>
<html>
<head>
  <title>CLASP Light Controller</title>
  <style>
    body {
      font-family: system-ui, sans-serif;
      max-width: 600px;
      margin: 40px auto;
      padding: 20px;
      background: #1a1a2e;
      color: white;
    }
    h1 { color: #00d9ff; }
    .fixture {
      background: #16213e;
      padding: 20px;
      border-radius: 8px;
      margin: 20px 0;
    }
    .fixture h3 { margin-top: 0; }
    .channel {
      display: flex;
      align-items: center;
      margin: 10px 0;
    }
    .channel label {
      width: 100px;
    }
    .channel input[type="range"] {
      flex: 1;
      margin: 0 10px;
    }
    .channel .value {
      width: 50px;
      text-align: right;
    }
    .status {
      padding: 10px;
      border-radius: 4px;
      margin-bottom: 20px;
    }
    .status.connected { background: #1b4332; }
    .status.disconnected { background: #7f1d1d; }
    .color-preview {
      width: 100%;
      height: 60px;
      border-radius: 4px;
      margin-top: 10px;
    }
  </style>
</head>
<body>
  <h1>Light Controller</h1>

  <div id="status" class="status disconnected">Connecting...</div>

  <div class="fixture">
    <h3>RGB Par Light (DMX 1-4)</h3>

    <div class="channel">
      <label>Dimmer</label>
      <input type="range" id="dimmer" min="0" max="255" value="0">
      <span class="value" id="dimmer-value">0</span>
    </div>

    <div class="channel">
      <label>Red</label>
      <input type="range" id="red" min="0" max="255" value="0">
      <span class="value" id="red-value">0</span>
    </div>

    <div class="channel">
      <label>Green</label>
      <input type="range" id="green" min="0" max="255" value="0">
      <span class="value" id="green-value">0</span>
    </div>

    <div class="channel">
      <label>Blue</label>
      <input type="range" id="blue" min="0" max="255" value="0">
      <span class="value" id="blue-value">0</span>
    </div>

    <div id="color-preview" class="color-preview"></div>
  </div>

  <script type="module">
    import { ClaspBuilder } from 'https://unpkg.com/@clasp-to/core/dist/index.mjs';

    const status = document.getElementById('status');
    const preview = document.getElementById('color-preview');

    // Channel mapping (DMX universe 0)
    const channels = {
      dimmer: 1,
      red: 2,
      green: 3,
      blue: 4
    };

    // Connect to CLASP
    const client = await new ClaspBuilder('ws://localhost:7330')
      .withName('Web Light Controller')
      .connect();

    status.textContent = 'Connected';
    status.className = 'status connected';

    // Set up sliders
    for (const [name, channel] of Object.entries(channels)) {
      const slider = document.getElementById(name);
      const valueDisplay = document.getElementById(`${name}-value`);

      // Send value when slider changes
      slider.addEventListener('input', async () => {
        const value = parseInt(slider.value);
        valueDisplay.textContent = value;
        await client.set(`/dmx/0/${channel}`, value);
        updatePreview();
      });

      // Subscribe to changes from other sources
      client.on(`/dmx/0/${channel}`, (value) => {
        slider.value = value;
        valueDisplay.textContent = value;
        updatePreview();
      });

      // Get current value on load
      const current = await client.get(`/dmx/0/${channel}`);
      if (current !== null) {
        slider.value = current;
        valueDisplay.textContent = current;
      }
    }

    updatePreview();

    function updatePreview() {
      const r = document.getElementById('red').value;
      const g = document.getElementById('green').value;
      const b = document.getElementById('blue').value;
      const d = document.getElementById('dimmer').value / 255;

      preview.style.background = `rgb(${r * d}, ${g * d}, ${b * d})`;
    }
  </script>
</body>
</html>
```

## Step 3: Serve the Page

You need a local web server (browsers block WebSocket from file:// URLs):

```bash
# Python
python -m http.server 8080

# Node.js
npx serve .

# Or use any local server
```

Open http://localhost:8080 in your browser.

## Step 4: Test It

1. Move the sliders - you should see values update
2. If you have Art-Net nodes on your network, lights should respond
3. The color preview shows the mixed RGB color

## Simulating Without Hardware

If you don't have DMX hardware, create a simulator:

Create `simulator.py`:

```python
import asyncio
from clasp import ClaspBuilder

async def main():
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('Light Simulator')
        .connect()
    )

    print('Light Simulator')
    print('=' * 40)

    @client.on('/dmx/0/*')
    def on_dmx(value, address):
        channel = address.split('/')[-1]
        bar = '█' * (value // 10)
        print(f'CH {channel:>3}: {value:>3} {bar}')

    await asyncio.Event().wait()

asyncio.run(main())
```

Run it alongside your web interface to see DMX values.

## Adding Presets

Add preset buttons to your HTML:

```html
<div class="fixture">
  <h3>Presets</h3>
  <button onclick="setPreset('off')">Off</button>
  <button onclick="setPreset('warm')">Warm White</button>
  <button onclick="setPreset('cool')">Cool White</button>
  <button onclick="setPreset('red')">Red</button>
  <button onclick="setPreset('blue')">Blue</button>
</div>

<script>
async function setPreset(name) {
  const presets = {
    off:  { dimmer: 0,   red: 0,   green: 0,   blue: 0   },
    warm: { dimmer: 255, red: 255, green: 180, blue: 100 },
    cool: { dimmer: 255, red: 200, green: 220, blue: 255 },
    red:  { dimmer: 255, red: 255, green: 0,   blue: 0   },
    blue: { dimmer: 255, red: 0,   green: 0,   blue: 255 }
  };

  const preset = presets[name];
  if (!preset) return;

  // Use a bundle for atomic update
  await client.bundle([
    { set: ['/dmx/0/1', preset.dimmer] },
    { set: ['/dmx/0/2', preset.red] },
    { set: ['/dmx/0/3', preset.green] },
    { set: ['/dmx/0/4', preset.blue] }
  ]);
}
</script>
```

Bundles ensure all channels update atomically.

## Adding Multiple Fixtures

Scale up by adding more fixtures:

```javascript
const fixtures = [
  { name: 'Par Light 1', startChannel: 1 },
  { name: 'Par Light 2', startChannel: 5 },
  { name: 'Par Light 3', startChannel: 9 }
];

for (const fixture of fixtures) {
  createFixtureUI(fixture);
}
```

## Troubleshooting

### "WebSocket connection failed"
- Check router is running on port 7330
- Check browser console for CORS errors
- Make sure you're using http:// not file://

### Lights not responding
- Verify Art-Net bridge is running
- Check Art-Net nodes are on the same network
- Verify universe and channel numbers match your fixtures

### Sliders not syncing
- Check address patterns match exactly
- Ensure both clients are connected to the same router

## Architecture Notes

This tutorial demonstrates the bridge pattern:

1. **Web client** sends CLASP messages to addresses like `/dmx/0/1`
2. **Router** routes messages to all subscribers
3. **Art-Net bridge** subscribes to `/dmx/**` and translates to Art-Net
4. **Art-Net nodes** receive DMX data and control fixtures

The same pattern works for MIDI, OSC, MQTT, and other protocols.

## Next Steps

- [Add MIDI control](../how-to/connections/add-midi.md) - Control from hardware
- [Add OSC](../how-to/connections/add-osc.md) - Integrate with TouchOSC
- [TouchOSC Integration](../integrations/touchosc.md) - Mobile control
- [Resolume Integration](../integrations/resolume.md) - VJ software
