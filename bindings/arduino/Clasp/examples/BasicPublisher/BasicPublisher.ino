/**
 * CLASP Basic Publisher
 *
 * Publishes sensor readings to a CLASP router every second.
 * Demonstrates set() with different value types.
 *
 * Hardware: Any board with WiFi (ESP32, ESP8266, etc.)
 */

#include <WiFi.h>
#include <Clasp.h>

const char* WIFI_SSID = "YourNetwork";
const char* WIFI_PASS = "YourPassword";
const char* CLASP_HOST = "192.168.1.100";
const uint16_t CLASP_PORT = 7330;

WiFiClient tcp;
ClaspClient clasp(tcp);

void setup() {
  Serial.begin(115200);

  // Connect to WiFi
  WiFi.begin(WIFI_SSID, WIFI_PASS);
  Serial.print("Connecting to WiFi");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println(" OK");
  Serial.print("IP: ");
  Serial.println(WiFi.localIP());

  // Connect to CLASP router
  Serial.print("Connecting to CLASP...");
  if (clasp.connect(CLASP_HOST, CLASP_PORT, "ESP32 Publisher")) {
    Serial.println(" OK");
    clasp.set("/devices/esp32/status", "online");
  } else {
    Serial.println(" FAILED");
  }
}

void loop() {
  if (!clasp.loop()) {
    Serial.println("Disconnected, reconnecting...");
    delay(2000);
    clasp.connect(CLASP_HOST, CLASP_PORT, "ESP32 Publisher");
    return;
  }

  static unsigned long lastPublish = 0;
  if (millis() - lastPublish >= 1000) {
    lastPublish = millis();

    // Publish different value types
    float temperature = 20.0 + (analogRead(A0) / 4095.0) * 20.0;
    clasp.set("/sensors/esp32/temperature", temperature);

    int32_t uptime = (int32_t)(millis() / 1000);
    clasp.set("/sensors/esp32/uptime", uptime);

    bool buttonPressed = digitalRead(0) == LOW;
    clasp.set("/sensors/esp32/button", buttonPressed);

    clasp.set("/sensors/esp32/hostname", WiFi.getHostname());

    Serial.print("Published: temp=");
    Serial.print(temperature);
    Serial.print(" uptime=");
    Serial.println(uptime);
  }
}
