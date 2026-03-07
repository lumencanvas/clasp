---
title: Arduino SDK
description: Use CLASP on Arduino, ESP32, and ESP8266
order: 5
---

# Arduino SDK

The CLASP Arduino library connects microcontrollers to a CLASP router over TCP. It works with any board that provides an Arduino `Client` interface -- ESP32, ESP8266, Ethernet shields, and more.

## Installation

### Arduino Library Manager

Search for **Clasp** in the Arduino Library Manager and click Install.

### Manual

Download or clone `bindings/arduino/Clasp/` into your Arduino libraries folder:

```
~/Arduino/libraries/Clasp/
```

## Wiring

CLASP uses raw TCP to connect to a CLASP router. Start the router with TCP transport enabled:

```bash
clasp-router --transport tcp --listen 0.0.0.0:7330
```

```
┌──────────┐       TCP        ┌──────────────┐
│  ESP32   │ ───────────────> │ CLASP Router │
│  (WiFi)  │   port 7330     │  (--transport │
└──────────┘                  │    tcp)       │
                              └──────────────┘
```

No special wiring is needed beyond your board's network connection (WiFi or Ethernet).

## Quick Start

```cpp
#include <WiFi.h>
#include <Clasp.h>

WiFiClient tcp;
ClaspClient clasp(tcp);

void onMessage(const char* address, ClaspValue value) {
  if (value.type == ClaspValue::Float) {
    analogWrite(LED_PIN, value.f * 255);
  }
}

void setup() {
  Serial.begin(115200);
  WiFi.begin("MyNetwork", "password");
  while (WiFi.status() != WL_CONNECTED) delay(500);

  if (clasp.connect("192.168.1.100", 7330, "ESP32 Sensor")) {
    clasp.subscribe("/lights/brightness", onMessage);
    clasp.set("/sensors/esp32/status", "online");
  }
}

void loop() {
  clasp.loop();

  static unsigned long last = 0;
  if (millis() - last > 1000) {
    last = millis();
    float temp = analogRead(A0) * 0.1;
    clasp.set("/sensors/esp32/temperature", temp);
  }
}
```

## API Reference

### ClaspClient

```cpp
ClaspClient(Client& client);
```

Accepts any Arduino `Client` -- `WiFiClient`, `EthernetClient`, `WiFiClientSecure`, etc.

#### Connection

| Method | Description |
|--------|-------------|
| `bool connect(host, port, name)` | Connect to a CLASP router. Port defaults to 7330. |
| `bool connected()` | Returns true if connected. |
| `void disconnect()` | Close the connection. |

#### Publishing

| Method | Description |
|--------|-------------|
| `bool set(address, value)` | Set a persistent param. Overloads for `float`, `int32_t`, `const char*`, `bool`. |
| `bool emit(address)` | Fire a valueless event. |
| `bool emit(address, value)` | Fire an event with a float payload. |
| `bool stream(address, value)` | Send high-rate data (float). |

#### Subscribing

```cpp
typedef void (*MessageCallback)(const char* address, ClaspValue value);

bool subscribe(const char* pattern, MessageCallback callback);
bool unsubscribe(const char* pattern);
```

Supports `*` (single-level) and `**` (multi-level) wildcards. Max subscriptions: `CLASP_MAX_SUBSCRIPTIONS` (default 4).

#### Loop

```cpp
bool loop();
```

Must be called in `loop()`. Reads available TCP bytes, reassembles frames, and dispatches callbacks. Returns `false` if disconnected.

### ClaspValue

```cpp
struct ClaspValue {
  enum Type { Null, Bool, Int, Float, String, Bytes };
  Type type;
  union { bool b; int32_t i; float f; };
  const char* str;  // Points into receive buffer (zero-copy)
  uint16_t len;     // String/bytes length
};
```

String values point into the receive buffer and are valid until the next `loop()` call. Copy them if you need to keep them.

### ClaspDiscovery (optional)

```cpp
#include <ClaspDiscovery.h>

ClaspDiscovery discovery;
ClaspDiscoveryResult result;

if (discovery.find(result, 5000)) {
  Serial.print("Found router: ");
  Serial.println(result.host);
  clasp.connect(result.host, result.port);
}
```

Sends mDNS queries for `_clasp._tcp.local` and returns the first result.

## Configuration

Define these before `#include <Clasp.h>` to override defaults:

| Define | Default | Description |
|--------|---------|-------------|
| `CLASP_MAX_PACKET_SIZE` | 256 | Max frame size in bytes |
| `CLASP_MAX_SUBSCRIPTIONS` | 4 | Max concurrent subscriptions |
| `CLASP_KEEPALIVE_MS` | 15000 | Keepalive interval (0 to disable) |

```cpp
#define CLASP_MAX_PACKET_SIZE 512
#define CLASP_MAX_SUBSCRIPTIONS 8
#include <Clasp.h>
```

## Memory Usage

The library uses no heap allocation. All buffers are stack-allocated or statically sized.

| Component | RAM | Flash |
|-----------|-----|-------|
| ClaspClient | ~300 B + `CLASP_MAX_PACKET_SIZE` | ~4 KB |
| Each subscription | ~40 B | -- |
| ClaspDiscovery | ~200 B | ~2 KB |

Typical total: **~800 bytes RAM** with default settings.

## Examples

| Example | Description |
|---------|-------------|
| `BasicPublisher` | Publish sensor readings every second |
| `BasicSubscriber` | Subscribe to addresses and control outputs |
| `WiFiExample` | Full WiFi setup with reconnection |
| `EthernetExample` | Ethernet shield usage |
| `DiscoveryExample` | Find routers with mDNS |

## Limitations

- **TCP only** -- no WebSocket or QUIC on Arduino (raw TCP is a valid CLASP transport)
- **No E2E encryption** -- constrained devices lack the crypto primitives; use network-level security (TLS via `WiFiClientSecure`) or keep on a trusted LAN
- **No auth tokens** -- CPSK tokens require Ed25519 which is too heavy for most MCUs
- **Fixed buffers** -- messages larger than `CLASP_MAX_PACKET_SIZE` are dropped
- **No JSON values** -- only scalar types (float, int, string, bool, null, bytes)
- **Single connection** -- one router connection per `ClaspClient` instance

## Platform Compatibility

The library works with any board that provides an Arduino `Client` interface (the standard abstract class for TCP connections). This includes:

| Board | Network | Notes |
|-------|---------|-------|
| ESP32 | WiFi, Ethernet | Most common target. `WiFiClient` or `EthernetClient`. |
| ESP8266 | WiFi | `WiFiClient`. Limited RAM -- consider reducing `CLASP_MAX_PACKET_SIZE`. |
| Arduino Uno + Ethernet Shield | Ethernet | 2 KB RAM -- set `CLASP_MAX_PACKET_SIZE` to 128 and `CLASP_MAX_SUBSCRIPTIONS` to 2. |
| Arduino Mega + Ethernet Shield | Ethernet | 8 KB RAM -- default settings work. |
| Teensy 4.x | Ethernet (built-in on 4.1) | Fast, plenty of RAM. |
| RP2040 (Pico W) | WiFi | Via the Arduino-Pico core's `WiFiClient`. |
| STM32 + W5500 | Ethernet | Via the STM32Ethernet library. |

Any board not listed above should work as long as it has a `Client`-compatible TCP library and enough RAM for the buffers (~800 bytes with defaults).

## Next Steps

- [JavaScript SDK](javascript.md) -- build CLASP clients in JS/TS
- [Easy Client SDK](easy-client.md) -- high-level SDK for JS/TS
- [Core Concepts](../concepts/architecture.md) -- CLASP architecture and signal types
