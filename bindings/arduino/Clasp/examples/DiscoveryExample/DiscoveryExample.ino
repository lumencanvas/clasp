/**
 * CLASP Discovery Example
 *
 * Uses mDNS to discover CLASP routers on the local network,
 * then connects to the first one found.
 *
 * Hardware: ESP32 or ESP8266 with WiFi
 */

#include <WiFi.h>
#include <Clasp.h>
#include <ClaspDiscovery.h>

const char* WIFI_SSID = "YourNetwork";
const char* WIFI_PASS = "YourPassword";

WiFiClient tcp;
ClaspClient clasp(tcp);
ClaspDiscovery discovery;

void onMessage(const char* address, ClaspValue value) {
  Serial.print("[");
  Serial.print(address);
  Serial.print("] ");
  switch (value.type) {
    case ClaspValue::Float:  Serial.println(value.f); break;
    case ClaspValue::Int:    Serial.println(value.i); break;
    case ClaspValue::Bool:   Serial.println(value.b ? "true" : "false"); break;
    case ClaspValue::String:
      Serial.write(value.str, value.len);
      Serial.println();
      break;
    default: Serial.println("null"); break;
  }
}

void setup() {
  Serial.begin(115200);

  // Connect WiFi
  WiFi.begin(WIFI_SSID, WIFI_PASS);
  Serial.print("WiFi");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.print(" OK (");
  Serial.print(WiFi.localIP());
  Serial.println(")");

  // Discover CLASP routers
  Serial.println("Searching for CLASP routers...");
  ClaspDiscoveryResult result;
  if (discovery.find(result, 10000)) {
    Serial.print("Found: ");
    if (result.name[0]) {
      Serial.print(result.name);
      Serial.print(" at ");
    }
    Serial.print(result.host);
    Serial.print(":");
    Serial.println(result.port);

    // Connect to discovered router
    if (clasp.connect(result.host, result.port, "Discovery Client")) {
      Serial.println("Connected!");
      clasp.subscribe("/**", onMessage);
      clasp.set("/devices/discovery-client/status", "online");
    } else {
      Serial.println("Connection failed");
    }
  } else {
    Serial.println("No CLASP routers found");
    Serial.println("Falling back to localhost:7330");
    if (clasp.connect("192.168.1.100", 7330, "Discovery Client")) {
      Serial.println("Fallback connected");
      clasp.subscribe("/**", onMessage);
    }
  }
}

void loop() {
  if (!clasp.loop()) {
    Serial.println("Lost connection");
    delay(5000);

    // Re-discover
    ClaspDiscoveryResult result;
    if (discovery.find(result, 5000)) {
      clasp.connect(result.host, result.port, "Discovery Client");
      clasp.subscribe("/**", onMessage);
    }
  }
}
