/**
 * CLASP Ethernet Example
 *
 * Uses an Ethernet shield or built-in Ethernet (Teensy, WT32-ETH01).
 * Same API as WiFi -- just swap WiFiClient for EthernetClient.
 *
 * Hardware: Arduino + Ethernet Shield, Teensy 4.1, or WT32-ETH01
 */

#include <SPI.h>
#include <Ethernet.h>
#include <Clasp.h>

// MAC address for the Ethernet shield (use your own or a random one)
byte mac[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xED };

// CLASP router address
const char* CLASP_HOST = "192.168.1.100";
const uint16_t CLASP_PORT = 7330;

EthernetClient tcp;
ClaspClient clasp(tcp);

void onMessage(const char* address, ClaspValue value) {
  Serial.print(address);
  Serial.print(" = ");
  if (value.type == ClaspValue::Float) Serial.println(value.f);
  else if (value.type == ClaspValue::Int) Serial.println(value.i);
  else if (value.type == ClaspValue::String) {
    Serial.write(value.str, value.len);
    Serial.println();
  }
  else Serial.println("(other)");
}

void setup() {
  Serial.begin(115200);
  while (!Serial) delay(10); // Wait for USB serial on Teensy/Leonardo

  // Start Ethernet with DHCP
  Serial.print("Ethernet DHCP...");
  if (Ethernet.begin(mac) == 0) {
    Serial.println(" FAILED");
    // Try static IP as fallback
    IPAddress ip(192, 168, 1, 200);
    Ethernet.begin(mac, ip);
    Serial.print("Static IP: ");
  } else {
    Serial.print(" OK: ");
  }
  Serial.println(Ethernet.localIP());

  // Connect to CLASP
  if (clasp.connect(CLASP_HOST, CLASP_PORT, "Ethernet Node")) {
    Serial.println("CLASP connected");
    clasp.subscribe("/control/**", onMessage);
    clasp.set("/devices/ethernet/status", "online");
  } else {
    Serial.println("CLASP connection failed");
  }
}

void loop() {
  // Maintain DHCP lease
  Ethernet.maintain();

  if (!clasp.loop()) {
    Serial.println("CLASP disconnected, reconnecting...");
    delay(3000);
    clasp.connect(CLASP_HOST, CLASP_PORT, "Ethernet Node");
    clasp.subscribe("/control/**", onMessage);
    return;
  }

  static unsigned long lastPublish = 0;
  if (millis() - lastPublish >= 1000) {
    lastPublish = millis();

    float sensorValue = analogRead(A0) / 1023.0;
    clasp.set("/sensors/ethernet/analog0", sensorValue);

    int32_t uptime = (int32_t)(millis() / 1000);
    clasp.set("/sensors/ethernet/uptime", uptime);
  }
}
