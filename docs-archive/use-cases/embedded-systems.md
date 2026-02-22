---
title: "Embedded Systems"
description: "Deploy CLASP on microcontrollers and embedded devices."
section: use-cases
order: 2
---
# Embedded Systems

Deploy CLASP on microcontrollers and embedded devices.

## Supported Platforms

| Platform | RAM | Transport | Notes |
|----------|-----|-----------|-------|
| ESP32 | 520KB | WiFi, BLE | Most common |
| ESP8266 | 80KB | WiFi | Limited RAM |
| RP2040 | 264KB | USB, WiFi* | *With Pico W |
| STM32 | Varies | USB, Ethernet | Industrial |
| nRF52 | 256KB | BLE | Low power |

## Architecture Options

### HTTP POST (Simplest)

```
ESP32 → HTTP POST → CLASP HTTP Bridge → Router
```

Best for: Simple sensors, firewalled networks

### WebSocket (Bidirectional)

```
ESP32 ↔ WebSocket ↔ Router
```

Best for: Interactive devices, receiving commands

### UDP (Lowest Latency)

```
ESP32 ↔ UDP ↔ Router
```

Best for: LAN-only, high-rate sensors

### MQTT (IoT Standard)

```
ESP32 → MQTT Broker → CLASP MQTT Bridge → Router
```

Best for: Existing MQTT infrastructure

## ESP32 Example (HTTP POST)

```cpp
#include <WiFi.h>
#include <HTTPClient.h>

const char* ssid = "YOUR_WIFI";
const char* password = "YOUR_PASSWORD";
const char* claspUrl = "http://192.168.1.100:3000/api/set";

void setup() {
  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) delay(500);
}

void loop() {
  float temperature = readSensor();

  HTTPClient http;
  http.begin(claspUrl);
  http.addHeader("Content-Type", "application/json");

  String json = "{\"address\":\"/sensors/esp32/temp\",\"value\":" +
                String(temperature) + "}";
  http.POST(json);
  http.end();

  delay(5000);
}
```

## ESP32 Example (WebSocket)

```cpp
#include <WebSocketsClient.h>

WebSocketsClient webSocket;

void webSocketEvent(WStype_t type, uint8_t* payload, size_t length) {
  if (type == WStype_TEXT) {
    // Parse and handle incoming commands
    handleCommand((char*)payload);
  }
}

void setup() {
  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) delay(500);

  webSocket.begin("192.168.1.100", 7330, "/");
  webSocket.onEvent(webSocketEvent);
}

void loop() {
  webSocket.loop();

  static unsigned long lastSend = 0;
  if (millis() - lastSend > 1000) {
    String msg = "{\"type\":\"SET\",\"address\":\"/sensor/temp\",\"value\":" +
                 String(readTemp()) + "}";
    webSocket.sendTXT(msg);
    lastSend = millis();
  }
}
```

## Using clasp-embedded (Rust)

For Rust embedded projects:

```rust
#![no_std]
#![no_main]

use clasp_embedded::{Client, Value};

#[entry]
fn main() -> ! {
    let mut clasp = Client::new();

    loop {
        let temp = read_temperature();

        // Prepare frame (doesn't send)
        let frame = clasp.prepare_set(
            "/sensors/device1/temperature",
            Value::Float(temp as f64)
        );

        // Send via your transport
        transport.send(&frame);

        delay_ms(5000);
    }
}
```

**Memory usage:** ~3.6KB RAM

## Power Optimization

### Deep Sleep

```cpp
void loop() {
  // Read and send
  sendSensorData();

  // Deep sleep for 5 minutes
  esp_sleep_enable_timer_wakeup(5 * 60 * 1000000);
  esp_deep_sleep_start();
}
```

### Batching

Collect readings, send in batch:

```cpp
float readings[10];
int readingCount = 0;

void loop() {
  readings[readingCount++] = readSensor();

  if (readingCount >= 10) {
    sendBatch(readings, 10);
    readingCount = 0;
  }

  delay(1000);
}
```

## Multiple Devices

Use unique IDs per device:

```cpp
String deviceId = WiFi.macAddress();
deviceId.replace(":", "");

String address = "/sensors/" + deviceId + "/temperature";
```

Monitor all with wildcard:

```javascript
client.on('/sensors/*/temperature', (value, address) => {
  const deviceId = address.split('/')[2];
  console.log(`Device ${deviceId}: ${value}°C`);
});
```

## Receiving Commands

Handle incoming CLASP messages:

```cpp
void handleCommand(char* payload) {
  // Simple JSON parsing
  if (strstr(payload, "/led/brightness")) {
    float value = extractValue(payload);
    analogWrite(LED_PIN, (int)(value * 255));
  }
}
```

## Troubleshooting

### WiFi disconnects

- Use WiFi event callbacks to reconnect
- Increase WiFi power: `WiFi.setTxPower(WIFI_POWER_19_5dBm)`

### Memory issues

- Use static buffers, not String
- Minimize JSON nesting
- Consider binary encoding

### Latency

- Use UDP for LAN
- Reduce payload size
- Keep WiFi connected

## Next Steps

- [Embedded Sensor Tutorial](../tutorials/embedded-sensor-node.md)
- [clasp-embedded API](../reference/api/rust/clasp-embedded.md)
