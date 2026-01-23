# Sensor to Visualization Tutorial

Create a pipeline from IoT sensors to a visualization application.

**Time:** 20-30 minutes
**Prerequisites:** [First Connection](first-connection.md) tutorial, Python basics

## What You'll Build

A data pipeline that collects sensor data via MQTT and displays it in real-time:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Sensors   │────►│    MQTT     │────►│   Router    │────►│  Dashboard  │
│  (or mock)  │     │   Bridge    │     │ (port 7330) │     │  (browser)  │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

## Step 1: Start the Infrastructure

**Terminal 1 - Start an MQTT broker (if you don't have one):**
```bash
# Using Docker
docker run -p 1883:1883 eclipse-mosquitto

# Or install mosquitto locally
```

**Terminal 2 - CLASP Router:**
```bash
clasp server --port 7330
```

**Terminal 3 - MQTT Bridge:**
```bash
clasp mqtt --host localhost --port 1883 --topic "sensors/#"
```

This creates a bridge that:
- Connects to the MQTT broker
- Subscribes to `sensors/#` topics
- Maps MQTT topics to CLASP addresses: `sensors/room1/temp` → `/mqtt/sensors/room1/temp`

## Step 2: Create a Sensor Simulator

Create `sensors.py`:

```python
import asyncio
import json
import random
from datetime import datetime

# Using paho-mqtt for MQTT publishing
import paho.mqtt.client as mqtt

def main():
    # Connect to MQTT broker
    client = mqtt.Client()
    client.connect("localhost", 1883)
    client.loop_start()

    print("Sensor Simulator")
    print("Publishing to MQTT topics...")

    # Simulate sensors
    sensors = [
        {"id": "room1", "type": "temperature", "base": 22, "variance": 2},
        {"id": "room2", "type": "temperature", "base": 20, "variance": 3},
        {"id": "room1", "type": "humidity", "base": 45, "variance": 10},
        {"id": "room2", "type": "humidity", "base": 50, "variance": 8},
        {"id": "outdoor", "type": "temperature", "base": 15, "variance": 5},
        {"id": "outdoor", "type": "light", "base": 500, "variance": 200},
    ]

    try:
        while True:
            for sensor in sensors:
                value = sensor["base"] + random.uniform(-1, 1) * sensor["variance"]
                topic = f"sensors/{sensor['id']}/{sensor['type']}"

                payload = {
                    "value": round(value, 2),
                    "timestamp": datetime.now().isoformat(),
                    "unit": get_unit(sensor["type"])
                }

                client.publish(topic, json.dumps(payload))
                print(f"  {topic}: {payload['value']} {payload['unit']}")

            print()
            asyncio.get_event_loop().run_until_complete(asyncio.sleep(2))
    except KeyboardInterrupt:
        print("\nStopping...")
        client.disconnect()

def get_unit(sensor_type):
    units = {
        "temperature": "°C",
        "humidity": "%",
        "light": "lux",
        "pressure": "hPa"
    }
    return units.get(sensor_type, "")

if __name__ == "__main__":
    main()
```

Install paho-mqtt and run:
```bash
pip install paho-mqtt
python sensors.py
```

## Step 3: Create the Dashboard

Create `dashboard.html`:

```html
<!DOCTYPE html>
<html>
<head>
  <title>Sensor Dashboard</title>
  <style>
    * { box-sizing: border-box; }
    body {
      font-family: system-ui, sans-serif;
      background: #0f0f1a;
      color: #fff;
      margin: 0;
      padding: 20px;
    }
    h1 { color: #00d9ff; margin-bottom: 30px; }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
      gap: 20px;
    }
    .card {
      background: #1a1a2e;
      border-radius: 12px;
      padding: 20px;
      border: 1px solid #333;
    }
    .card h3 {
      margin: 0 0 10px 0;
      color: #888;
      font-size: 14px;
      text-transform: uppercase;
    }
    .value {
      font-size: 48px;
      font-weight: bold;
      color: #00d9ff;
    }
    .unit {
      font-size: 24px;
      color: #666;
    }
    .timestamp {
      font-size: 12px;
      color: #555;
      margin-top: 10px;
    }
    .status {
      position: fixed;
      top: 20px;
      right: 20px;
      padding: 8px 16px;
      border-radius: 20px;
      font-size: 12px;
    }
    .status.connected { background: #1b4332; }
    .status.disconnected { background: #7f1d1d; }
    .chart {
      height: 60px;
      margin-top: 10px;
      display: flex;
      align-items: flex-end;
      gap: 2px;
    }
    .chart-bar {
      flex: 1;
      background: #00d9ff44;
      min-height: 2px;
      transition: height 0.3s;
    }
  </style>
</head>
<body>
  <div id="status" class="status disconnected">Connecting...</div>
  <h1>Sensor Dashboard</h1>
  <div id="grid" class="grid"></div>

  <script type="module">
    import { ClaspBuilder } from 'https://unpkg.com/@clasp-to/core/dist/index.mjs';

    const grid = document.getElementById('grid');
    const status = document.getElementById('status');
    const sensors = new Map();

    // Connect to CLASP
    const client = await new ClaspBuilder('ws://localhost:7330')
      .withName('Sensor Dashboard')
      .connect();

    status.textContent = 'Connected';
    status.className = 'status connected';

    // Subscribe to all MQTT sensor data
    client.on('/mqtt/sensors/**', (data, address) => {
      updateSensor(address, data);
    });

    function updateSensor(address, data) {
      // Parse address: /mqtt/sensors/room1/temperature
      const parts = address.split('/');
      const location = parts[3];
      const type = parts[4];
      const id = `${location}-${type}`;

      // Parse data (could be JSON string or object)
      let value, unit;
      if (typeof data === 'string') {
        try {
          const parsed = JSON.parse(data);
          value = parsed.value;
          unit = parsed.unit || '';
        } catch {
          value = parseFloat(data);
          unit = '';
        }
      } else if (typeof data === 'object') {
        value = data.value;
        unit = data.unit || '';
      } else {
        value = data;
        unit = '';
      }

      // Get or create sensor card
      let sensor = sensors.get(id);
      if (!sensor) {
        sensor = createSensorCard(id, location, type);
        sensors.set(id, sensor);
      }

      // Update display
      sensor.valueEl.textContent = typeof value === 'number' ? value.toFixed(1) : value;
      sensor.unitEl.textContent = unit;
      sensor.timestampEl.textContent = new Date().toLocaleTimeString();

      // Update chart
      sensor.history.push(value);
      if (sensor.history.length > 20) sensor.history.shift();
      updateChart(sensor);
    }

    function createSensorCard(id, location, type) {
      const card = document.createElement('div');
      card.className = 'card';
      card.innerHTML = `
        <h3>${location} - ${type}</h3>
        <span class="value">--</span>
        <span class="unit"></span>
        <div class="chart"></div>
        <div class="timestamp">Waiting for data...</div>
      `;
      grid.appendChild(card);

      // Create chart bars
      const chart = card.querySelector('.chart');
      for (let i = 0; i < 20; i++) {
        const bar = document.createElement('div');
        bar.className = 'chart-bar';
        chart.appendChild(bar);
      }

      return {
        card,
        valueEl: card.querySelector('.value'),
        unitEl: card.querySelector('.unit'),
        timestampEl: card.querySelector('.timestamp'),
        chartEl: chart,
        history: []
      };
    }

    function updateChart(sensor) {
      const bars = sensor.chartEl.querySelectorAll('.chart-bar');
      const min = Math.min(...sensor.history);
      const max = Math.max(...sensor.history);
      const range = max - min || 1;

      bars.forEach((bar, i) => {
        const value = sensor.history[i];
        if (value !== undefined) {
          const height = ((value - min) / range) * 100;
          bar.style.height = `${Math.max(5, height)}%`;
        }
      });
    }
  </script>
</body>
</html>
```

Serve it:
```bash
python -m http.server 8080
```

Open http://localhost:8080

## Step 4: Add Data Processing

Create a processor that adds alerts for abnormal values:

Create `processor.py`:

```python
import asyncio
from clasp import ClaspBuilder

async def main():
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('Sensor Processor')
        .connect()
    )

    print('Sensor Processor running...')
    print('Monitoring for anomalies...')

    # Thresholds
    thresholds = {
        'temperature': {'min': 15, 'max': 28},
        'humidity': {'min': 30, 'max': 70},
        'light': {'min': 100, 'max': 1000}
    }

    @client.on('/mqtt/sensors/**')
    async def on_sensor(data, address):
        # Parse address
        parts = address.split('/')
        location = parts[3]
        sensor_type = parts[4]

        # Parse value
        if isinstance(data, str):
            import json
            data = json.loads(data)
        value = data.get('value', data) if isinstance(data, dict) else data

        # Check thresholds
        if sensor_type in thresholds:
            limits = thresholds[sensor_type]
            if value < limits['min'] or value > limits['max']:
                alert = {
                    'location': location,
                    'type': sensor_type,
                    'value': value,
                    'threshold': limits,
                    'status': 'low' if value < limits['min'] else 'high'
                }

                print(f'ALERT: {location} {sensor_type} is {alert["status"]} ({value})')

                # Publish alert to CLASP
                await client.emit('/alerts/sensor', alert)

    await asyncio.Event().wait()

asyncio.run(main())
```

Run it alongside the other components to see alerts.

## The Complete Pipeline

```
┌──────────────┐
│   Sensors    │ (real or simulated)
└──────┬───────┘
       │ MQTT
       ▼
┌──────────────┐
│ MQTT Broker  │ (mosquitto)
└──────┬───────┘
       │
       ▼
┌──────────────┐     ┌──────────────┐
│ MQTT Bridge  │────►│   Router     │
└──────────────┘     └──────┬───────┘
                            │
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
       ┌──────────┐  ┌──────────┐  ┌──────────┐
       │Dashboard │  │Processor │  │ Storage  │
       │(browser) │  │(alerts)  │  │ (future) │
       └──────────┘  └──────────┘  └──────────┘
```

## Without MQTT (Direct CLASP)

If you don't need MQTT, sensors can publish directly to CLASP:

```python
import asyncio
import random
from clasp import ClaspBuilder

async def main():
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('Direct Sensor')
        .connect()
    )

    while True:
        await client.set('/sensors/room1/temperature', 22 + random.random() * 2)
        await asyncio.sleep(2)

asyncio.run(main())
```

The dashboard subscribes to `/sensors/**` instead of `/mqtt/sensors/**`.

## Troubleshooting

### No data appearing
- Check MQTT broker is running
- Check MQTT bridge is connected to broker
- Verify topic patterns match (`sensors/#` in bridge, `sensors/...` in publisher)

### JSON parse errors
- Ensure sensor publishes valid JSON
- Check the MQTT bridge is translating correctly

### Dashboard not updating
- Check browser console for WebSocket errors
- Verify subscription patterns in dashboard match published addresses

## Next Steps

- [Home Automation Guide](../use-cases/home-automation.md) - More IoT patterns
- [MQTT Bridge Reference](../reference/bridges/mqtt.md) - Advanced MQTT options
- [Home Assistant Integration](../integrations/home-assistant.md) - Smart home integration
