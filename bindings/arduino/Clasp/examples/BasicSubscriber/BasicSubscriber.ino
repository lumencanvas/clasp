/**
 * CLASP Basic Subscriber
 *
 * Subscribes to addresses and controls outputs based on received values.
 * Demonstrates subscribe() with wildcard patterns and ClaspValue handling.
 *
 * Hardware: Any board with WiFi + LED on pin 2 (ESP32 built-in)
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

// Callback for brightness control
void onBrightness(const char* address, ClaspValue value) {
  if (value.type == ClaspValue::Float) {
    int pwm = (int)(value.f * 255.0);
    analogWrite(LED_PIN, pwm);
    Serial.print("Brightness: ");
    Serial.println(value.f);
  }
}

// Callback for wildcard sensor data
void onSensor(const char* address, ClaspValue value) {
  Serial.print(address);
  Serial.print(" = ");
  switch (value.type) {
    case ClaspValue::Float:
      Serial.println(value.f);
      break;
    case ClaspValue::Int:
      Serial.println(value.i);
      break;
    case ClaspValue::Bool:
      Serial.println(value.b ? "true" : "false");
      break;
    case ClaspValue::String:
      Serial.write(value.str, value.len);
      Serial.println();
      break;
    default:
      Serial.println("(null)");
      break;
  }
}

void setup() {
  Serial.begin(115200);
  pinMode(LED_PIN, OUTPUT);

  WiFi.begin(WIFI_SSID, WIFI_PASS);
  Serial.print("WiFi");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println(" OK");

  if (clasp.connect(CLASP_HOST, CLASP_PORT, "ESP32 Subscriber")) {
    Serial.println("Connected to CLASP");

    // Subscribe to specific address
    clasp.subscribe("/lights/brightness", onBrightness);

    // Subscribe to wildcard pattern
    clasp.subscribe("/sensors/**", onSensor);

    Serial.println("Subscriptions active");
  } else {
    Serial.println("Connection failed");
  }
}

void loop() {
  if (!clasp.loop()) {
    Serial.println("Disconnected");
    delay(2000);
    if (clasp.connect(CLASP_HOST, CLASP_PORT, "ESP32 Subscriber")) {
      clasp.subscribe("/lights/brightness", onBrightness);
      clasp.subscribe("/sensors/**", onSensor);
    }
  }
}
