/**
 * CLASP WiFi Example
 *
 * Full WiFi setup with reconnection handling for both WiFi and CLASP.
 * Publishes sensor data and subscribes to control signals.
 *
 * Hardware: ESP32 or ESP8266 with WiFi
 */

#include <WiFi.h>
#include <Clasp.h>

const char* WIFI_SSID = "YourNetwork";
const char* WIFI_PASS = "YourPassword";
const char* CLASP_HOST = "192.168.1.100";
const uint16_t CLASP_PORT = 7330;
const int LED_PIN = 2;

WiFiClient tcp;
ClaspClient clasp(tcp);

unsigned long lastReconnect = 0;
unsigned long lastPublish = 0;

void onControl(const char* address, ClaspValue value) {
  if (value.type == ClaspValue::Float) {
    int pwm = (int)(value.f * 255.0);
    analogWrite(LED_PIN, pwm);
  } else if (value.type == ClaspValue::Bool) {
    digitalWrite(LED_PIN, value.b ? HIGH : LOW);
  }
}

bool ensureWiFi() {
  if (WiFi.status() == WL_CONNECTED) return true;

  Serial.print("WiFi connecting...");
  WiFi.begin(WIFI_SSID, WIFI_PASS);

  unsigned long start = millis();
  while (WiFi.status() != WL_CONNECTED && millis() - start < 10000) {
    delay(250);
    Serial.print(".");
  }

  if (WiFi.status() == WL_CONNECTED) {
    Serial.print(" OK (");
    Serial.print(WiFi.localIP());
    Serial.println(")");
    return true;
  }

  Serial.println(" FAILED");
  return false;
}

bool ensureClasp() {
  if (clasp.connected()) return true;

  // Throttle reconnection attempts
  if (millis() - lastReconnect < 5000) return false;
  lastReconnect = millis();

  Serial.print("CLASP connecting...");
  if (clasp.connect(CLASP_HOST, CLASP_PORT, "ESP32 WiFi")) {
    Serial.println(" OK");
    clasp.subscribe("/control/esp32/**", onControl);
    clasp.set("/devices/esp32/status", "online");
    return true;
  }

  Serial.println(" FAILED");
  return false;
}

void setup() {
  Serial.begin(115200);
  pinMode(LED_PIN, OUTPUT);

  ensureWiFi();
  ensureClasp();
}

void loop() {
  if (!ensureWiFi()) return;
  if (!ensureClasp()) return;

  clasp.loop();

  if (millis() - lastPublish >= 1000) {
    lastPublish = millis();

    float temp = 20.0 + (analogRead(A0) / 4095.0) * 20.0;
    clasp.set("/sensors/esp32/temperature", temp);

    int32_t rssi = (int32_t)WiFi.RSSI();
    clasp.set("/sensors/esp32/rssi", rssi);

    int32_t freeHeap = (int32_t)(ESP.getFreeHeap() / 1024);
    clasp.set("/sensors/esp32/free_heap_kb", freeHeap);
  }
}
