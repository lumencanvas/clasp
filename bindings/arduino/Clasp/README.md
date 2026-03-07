# CLASP Arduino Library

Connect Arduino, ESP32, and ESP8266 to a CLASP router over TCP.

## Features

- State sync (`set`/`get`), events, streams
- Wildcard subscriptions (`*`, `**`)
- Zero heap allocation -- fixed buffers only
- Works with any Arduino `Client` (WiFi, Ethernet, etc.)
- Optional mDNS discovery

## Quick Start

```cpp
#include <WiFi.h>
#include <Clasp.h>

WiFiClient tcp;
ClaspClient clasp(tcp);

void onMessage(const char* address, ClaspValue value) {
  Serial.print(address);
  Serial.print(" = ");
  if (value.type == ClaspValue::Float) Serial.println(value.f);
}

void setup() {
  Serial.begin(115200);
  WiFi.begin("SSID", "password");
  while (WiFi.status() != WL_CONNECTED) delay(500);

  if (clasp.connect("192.168.1.100", 7330, "Arduino")) {
    clasp.subscribe("/sensors/**", onMessage);
    clasp.set("/devices/arduino/status", "online");
  }
}

void loop() {
  clasp.loop();
}
```

## Installation

**Arduino Library Manager:** Search for "Clasp" and install.

**Manual:** Copy this folder to `~/Arduino/libraries/Clasp/`.

## Examples

| Example | Description |
|---------|-------------|
| BasicPublisher | Publish sensor readings |
| BasicSubscriber | Subscribe and control outputs |
| WiFiExample | Full WiFi with reconnection |
| EthernetExample | Ethernet shield usage |
| DiscoveryExample | mDNS router discovery |

## Configuration

Define before `#include <Clasp.h>`:

```cpp
#define CLASP_MAX_PACKET_SIZE 512    // Default: 256
#define CLASP_MAX_SUBSCRIPTIONS 8    // Default: 4
#define CLASP_KEEPALIVE_MS 30000     // Default: 15000
```

## Running Tests

Desktop tests (no Arduino hardware needed):

```bash
cd test
make test
```

## Documentation

Full docs at [clasp.to](https://clasp.to/docs/sdk/arduino).

## License

MIT / Apache-2.0
