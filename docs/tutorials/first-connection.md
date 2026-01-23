# First Connection Tutorial

Connect two applications and send your first CLASP messages.

**Time:** 5-10 minutes
**Prerequisites:** Node.js 18+ or Python 3.9+

## What You'll Build

A simple system where one application publishes sensor data and another receives it:

```
┌─────────────────┐     ┌─────────────┐     ┌─────────────────┐
│    Publisher    │────►│   Router    │────►│   Subscriber    │
│  (sensor.js)    │     │ (port 7330) │     │  (display.py)   │
└─────────────────┘     └─────────────┘     └─────────────────┘
```

## Step 1: Install CLASP

Choose your platform:

**CLI (required for router):**
```bash
cargo install clasp-cli
```

**JavaScript:**
```bash
npm install @clasp-to/core
```

**Python:**
```bash
pip install clasp-to
```

## Step 2: Start the Router

Open a terminal and start the CLASP router:

```bash
clasp server --port 7330
```

You should see:
```
CLASP router started on ws://0.0.0.0:7330
```

Keep this running in its own terminal.

## Step 3: Create the Publisher

Create a file called `sensor.js`:

```javascript
import { ClaspBuilder } from '@clasp-to/core';

async function main() {
  // Connect to the router
  const client = await new ClaspBuilder('ws://localhost:7330')
    .withName('Temperature Sensor')
    .connect();

  console.log('Connected! Publishing temperature data...');

  // Publish temperature every second
  let temp = 20.0;
  setInterval(async () => {
    // Simulate temperature fluctuation
    temp += (Math.random() - 0.5) * 0.5;

    await client.set('/sensors/room1/temperature', temp);
    console.log(`Published: ${temp.toFixed(1)}°C`);
  }, 1000);
}

main().catch(console.error);
```

Run it:
```bash
node sensor.js
```

You should see:
```
Connected! Publishing temperature data...
Published: 20.2°C
Published: 20.4°C
...
```

## Step 4: Create the Subscriber

Create a file called `display.py`:

```python
import asyncio
from clasp import ClaspBuilder

async def main():
    # Connect to the router
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('Temperature Display')
        .connect()
    )

    print('Connected! Waiting for temperature data...')

    # Subscribe to all temperature sensors
    @client.on('/sensors/*/temperature')
    def on_temperature(value, address):
        print(f'{address}: {value:.1f}°C')

    # Keep running (blocks until disconnected)
    await asyncio.Event().wait()

asyncio.run(main())
```

Run it in a new terminal:
```bash
python display.py
```

You should see:
```
Connected! Waiting for temperature data...
/sensors/room1/temperature: 20.2°C
/sensors/room1/temperature: 20.4°C
...
```

## What Just Happened?

1. **Router** acts as the central hub, routing messages between clients
2. **Publisher** connects and uses `set()` to update parameter values
3. **Subscriber** connects and uses `on()` to listen for changes
4. The wildcard `*` in `/sensors/*/temperature` matches any sensor name

## Try These Experiments

### 1. Add More Sensors

Modify `sensor.js` to publish multiple sensors:

```javascript
setInterval(async () => {
  await client.set('/sensors/room1/temperature', 20 + Math.random());
  await client.set('/sensors/room2/temperature', 22 + Math.random());
  await client.set('/sensors/outside/temperature', 15 + Math.random());
}, 1000);
```

The subscriber will receive all of them because of the `*` wildcard.

### 2. Get Current Value

Add to the subscriber before the `@client.on` decorator:

```python
# Get current value (might be None if not set yet)
current = await client.get('/sensors/room1/temperature')
if current is not None:
    print(f'Current temperature: {current:.1f}°C')
```

### 3. Use Events Instead

Events are for triggers that don't have persistent state:

**Publisher (JavaScript):**
```javascript
// Emit an event (not stored, just triggers subscribers)
await client.emit('/alerts/high-temp', {
  sensor: 'room1',
  temperature: 30.5
});
```

**Subscriber (Python):**
```python
@client.on('/alerts/*')
def on_alert(payload, address):
    print(f'Alert! {address}: {payload}')
```

## Understanding Addresses

CLASP uses path-style addresses:

| Address | Description |
|---------|-------------|
| `/sensors/room1/temperature` | Specific sensor |
| `/sensors/*/temperature` | All sensors (single wildcard) |
| `/sensors/**` | All sensor data (multi wildcard) |

## Troubleshooting

### "Connection refused"
Make sure the router is running (`clasp server --port 7330`).

### "Module not found"
Make sure you installed the library:
- JavaScript: `npm install @clasp-to/core`
- Python: `pip install clasp-to`

### Values not updating
Check that addresses match exactly (case-sensitive).

## Next Steps

- [Control Lights from Web](control-lights-from-web.md) - Build a real UI
- [How to Start a Router](../how-to/connections/start-router.md) - Router options
- [Addressing Reference](../reference/protocol/addressing.md) - Wildcard patterns
