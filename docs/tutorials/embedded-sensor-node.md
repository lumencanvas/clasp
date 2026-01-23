# Embedded Sensor Node Tutorial

Build a wireless sensor node using CLASP on a microcontroller.

**Time:** 30-45 minutes
**Prerequisites:** [First Connection](first-connection.md) tutorial, ESP32 or RP2040 board, basic embedded development experience

## What You'll Build

An ESP32 that reads sensor data and publishes it to CLASP:

```
┌─────────────────┐                      ┌─────────────┐     ┌─────────────┐
│     ESP32       │     WiFi + HTTP      │   Router    │     │  Dashboard  │
│  + DHT22 Sensor │─────────────────────►│ (port 7330) │────►│  (browser)  │
└─────────────────┘                      └─────────────┘     └─────────────┘
```

## Architecture Options

CLASP supports multiple transport options for embedded devices:

| Transport | Pros | Cons |
|-----------|------|------|
| HTTP POST | Simple, works through firewalls | Higher latency, no subscriptions |
| WebSocket | Bidirectional, low latency | More complex, memory overhead |
| UDP | Lowest latency, minimal overhead | LAN only, no reliability |
| MQTT | Standard IoT protocol, QoS | Requires MQTT broker |

This tutorial uses HTTP POST for simplicity. See the [Embedded Transport Guide](../how-to/advanced/embed-router.md) for other options.

## Step 1: Set Up the Router

On your computer/server:

```bash
# Start CLASP router
clasp server --port 7330

# Start HTTP bridge (accepts POST requests)
clasp http --bind 0.0.0.0:3000
```

The HTTP bridge provides an endpoint at `http://<your-ip>:3000/api/set` that accepts CLASP messages.

## Step 2: Test the HTTP Endpoint

Before writing embedded code, test with curl:

```bash
# Set a value via HTTP
curl -X POST http://localhost:3000/api/set \
  -H "Content-Type: application/json" \
  -d '{"address": "/sensors/test/temperature", "value": 23.5}'

# Should return: {"ok": true}
```

## Step 3: ESP32 Arduino Code

Create a new Arduino sketch:

```cpp
#include <WiFi.h>
#include <HTTPClient.h>
#include <DHT.h>

// WiFi credentials
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// CLASP HTTP endpoint
const char* claspUrl = "http://192.168.1.100:3000/api/set";  // Replace with your server IP

// DHT sensor
#define DHTPIN 4
#define DHTTYPE DHT22
DHT dht(DHTPIN, DHTTYPE);

// Device ID (unique per device)
const char* deviceId = "esp32-001";

// Timing
unsigned long lastPublish = 0;
const unsigned long publishInterval = 5000;  // 5 seconds

void setup() {
  Serial.begin(115200);

  // Initialize DHT sensor
  dht.begin();

  // Connect to WiFi
  Serial.printf("Connecting to %s...\n", ssid);
  WiFi.begin(ssid, password);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  Serial.println("\nConnected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());
}

void loop() {
  unsigned long now = millis();

  if (now - lastPublish >= publishInterval) {
    lastPublish = now;

    // Read sensor
    float temperature = dht.readTemperature();
    float humidity = dht.readHumidity();

    if (!isnan(temperature) && !isnan(humidity)) {
      // Publish to CLASP
      publishValue("/sensors/" + String(deviceId) + "/temperature", temperature);
      publishValue("/sensors/" + String(deviceId) + "/humidity", humidity);

      Serial.printf("Published: temp=%.1f°C, humidity=%.1f%%\n",
                    temperature, humidity);
    } else {
      Serial.println("Failed to read sensor");
    }
  }
}

void publishValue(String address, float value) {
  if (WiFi.status() != WL_CONNECTED) {
    Serial.println("WiFi disconnected, skipping publish");
    return;
  }

  HTTPClient http;
  http.begin(claspUrl);
  http.addHeader("Content-Type", "application/json");

  // Build JSON payload
  String payload = "{\"address\":\"" + address + "\",\"value\":" + String(value, 2) + "}";

  int httpCode = http.POST(payload);

  if (httpCode > 0) {
    if (httpCode != HTTP_CODE_OK) {
      Serial.printf("HTTP error: %d\n", httpCode);
    }
  } else {
    Serial.printf("HTTP request failed: %s\n", http.errorToString(httpCode).c_str());
  }

  http.end();
}
```

### Wiring

```
ESP32        DHT22
------       -----
3.3V    -->  VCC
GND     -->  GND
GPIO4   -->  DATA (with 10K pull-up to VCC)
```

## Step 4: Using PlatformIO (Alternative)

For a more robust setup, use PlatformIO:

Create `platformio.ini`:
```ini
[env:esp32]
platform = espressif32
board = esp32dev
framework = arduino
lib_deps =
    adafruit/DHT sensor library@^1.4.4
    adafruit/Adafruit Unified Sensor@^1.1.9
monitor_speed = 115200
```

## Step 5: WebSocket Transport (Bidirectional)

For receiving commands from CLASP, use WebSocket:

```cpp
#include <WebSocketsClient.h>

WebSocketsClient webSocket;

void webSocketEvent(WStype_t type, uint8_t* payload, size_t length) {
  switch (type) {
    case WStype_DISCONNECTED:
      Serial.println("WebSocket disconnected");
      break;

    case WStype_CONNECTED:
      Serial.println("WebSocket connected");
      // Send HELLO message
      sendHello();
      break;

    case WStype_TEXT:
      handleMessage((char*)payload);
      break;
  }
}

void setup() {
  // ... WiFi setup ...

  // Connect WebSocket
  webSocket.begin("192.168.1.100", 7330, "/");
  webSocket.onEvent(webSocketEvent);
  webSocket.setReconnectInterval(5000);
}

void loop() {
  webSocket.loop();

  // Publish periodically
  if (shouldPublish()) {
    publishSensor();
  }
}

void sendHello() {
  // Minimal HELLO message
  String hello = "{\"type\":\"HELLO\",\"version\":1,\"name\":\"ESP32-Sensor\"}";
  webSocket.sendTXT(hello);
}

void publishSensor() {
  float temp = dht.readTemperature();

  String msg = "{\"type\":\"SET\",\"address\":\"/sensors/";
  msg += deviceId;
  msg += "/temperature\",\"value\":";
  msg += String(temp, 2);
  msg += "}";

  webSocket.sendTXT(msg);
}

void handleMessage(char* payload) {
  // Parse incoming CLASP messages
  // Example: control an LED based on received value

  // Simple parsing (production code should use ArduinoJson)
  String msg = String(payload);

  if (msg.indexOf("/led/brightness") > 0) {
    int valueStart = msg.indexOf("\"value\":") + 8;
    int valueEnd = msg.indexOf("}", valueStart);
    float brightness = msg.substring(valueStart, valueEnd).toFloat();

    // Control LED
    analogWrite(LED_PIN, (int)(brightness * 255));
  }
}
```

## Step 6: Using clasp-embedded (Rust)

For Rust embedded projects, use `clasp-embedded`:

```rust
#![no_std]
#![no_main]

use clasp_embedded::{Client, Value};
use esp_hal::prelude::*;

#[entry]
fn main() -> ! {
    // Initialize hardware...

    let mut clasp = Client::new();

    loop {
        // Read sensor
        let temperature = read_temperature();

        // Prepare CLASP frame
        let frame = clasp.prepare_set(
            "/sensors/esp32/temperature",
            Value::Float(temperature as f64)
        );

        // Send via your transport
        wifi.send(&frame);

        delay_ms(5000);
    }
}
```

`clasp-embedded` is transport-agnostic and uses only ~3.6KB RAM.

## Step 7: View the Data

Create a simple dashboard to see your sensor data:

Use the dashboard from the [Sensor to Visualization](sensor-to-visualization.md) tutorial, modifying the subscription pattern:

```javascript
client.on('/sensors/esp32-*/**', (value, address) => {
  updateDisplay(address, value);
});
```

## Power Optimization

For battery-powered devices:

```cpp
// Deep sleep between readings
void loop() {
  // Read and publish
  publishSensor();

  // Deep sleep for 5 minutes
  esp_sleep_enable_timer_wakeup(5 * 60 * 1000000);  // microseconds
  esp_deep_sleep_start();
}
```

With deep sleep, an ESP32 with DHT22 can run for months on batteries.

## Multiple Devices

Scale by giving each device a unique ID:

```cpp
// Generate ID from MAC address
String getDeviceId() {
  uint8_t mac[6];
  WiFi.macAddress(mac);
  char id[13];
  sprintf(id, "%02x%02x%02x%02x%02x%02x",
          mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
  return String(id);
}
```

All devices publish to the same router and can be monitored with:
```javascript
client.on('/sensors/**', callback);  // All sensors
```

## Troubleshooting

### "Connection refused"
- Verify router IP address
- Check firewall settings
- Ensure ESP32 is on same network

### Sensor reading NaN
- Check wiring
- Verify pull-up resistor
- Try a different GPIO pin

### Messages not appearing
- Check HTTP response code
- Verify JSON format
- Monitor router logs: `RUST_LOG=debug clasp server`

## Next Steps

- [Embedded Systems Guide](../use-cases/embedded-systems.md) - More embedded patterns
- [clasp-embedded API](../reference/api/rust/clasp-embedded.md) - Rust embedded API
- [Performance Tuning](../how-to/advanced/performance-tuning.md) - Optimization tips
