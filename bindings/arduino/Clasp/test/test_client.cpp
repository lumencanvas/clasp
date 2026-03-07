/**
 * CLASP Client State Machine Tests (v3 binary protocol)
 *
 * Tests the ClaspClient with a mock TCP Client that simulates
 * server responses. Validates:
 * - Connection handshake (HELLO/WELCOME)
 * - Send operations (SET, EMIT, STREAM with all value types)
 * - Receive and dispatch (SET, PUBLISH through subscriptions)
 * - Wildcard subscription matching
 * - Partial frame delivery (incremental TCP reads)
 * - Oversized frame handling
 * - Reconnection state reset
 * - PING/PONG keepalive
 * - Subscription slot management
 *
 * Build and run on desktop with: make test
 */

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <vector>

#include "../src/ClaspCodec.h"
#include "../src/Clasp.h"

// Defined in Clasp.cpp for desktop builds
extern void _clasp_advance_millis(unsigned long ms);
extern void _clasp_set_millis_increment(unsigned long inc);
extern void _clasp_reset_millis();

static int tests_run = 0;
static int tests_passed = 0;
static int tests_failed = 0;

#define ASSERT(cond, msg) do { \
  tests_run++; \
  if (!(cond)) { \
    printf("  FAIL: %s (line %d)\n", msg, __LINE__); \
    tests_failed++; \
  } else { \
    tests_passed++; \
  } \
} while(0)

// ============================================================================
// Mock Client
// ============================================================================

class MockClient : public Client {
public:
  std::vector<uint8_t> txData;
  std::vector<uint8_t> rxData;
  size_t rxPos;
  bool isConnected;
  int maxBytesPerRead; // -1 = unlimited, >0 = limit bytes per read() call

  MockClient() : rxPos(0), isConnected(false), maxBytesPerRead(-1) {}

  int connect(const char* /* host */, uint16_t /* port */) override {
    isConnected = true;
    return 1;
  }

  size_t write(const uint8_t* buf, size_t size) override {
    txData.insert(txData.end(), buf, buf + size);
    return size;
  }

  int available() override {
    return (int)(rxData.size() - rxPos);
  }

  int read(uint8_t* buf, size_t size) override {
    int avail = available();
    if (avail <= 0) return 0;
    int toRead = (int)size < avail ? (int)size : avail;
    if (maxBytesPerRead > 0 && toRead > maxBytesPerRead) {
      toRead = maxBytesPerRead;
    }
    memcpy(buf, rxData.data() + rxPos, toRead);
    rxPos += toRead;
    return toRead;
  }

  uint8_t connected() override {
    return isConnected ? 1 : 0;
  }

  void stop() override {
    isConnected = false;
  }

  void reset() {
    txData.clear();
    rxData.clear();
    rxPos = 0;
    isConnected = false;
    maxBytesPerRead = -1;
  }

  // --- Frame injection helpers ---
  // All inject methods produce: [TCP prefix][CLASP frame header][payload]

  void injectWelcome() {
    // WELCOME payload: [0x02][version=3][features=0xE0][u64 time=0][u16+session][u16+name][u16+token=0]
    uint8_t payload[32];
    size_t pos = 0;
    payload[pos++] = CLASP_MSG_WELCOME;
    payload[pos++] = CLASP_PROTOCOL_VERSION; // version
    payload[pos++] = 0xE0; // features: param|event|stream
    // server time (u64 = 0)
    for (int i = 0; i < 8; i++) payload[pos++] = 0;
    // session string (empty)
    payload[pos++] = 0; payload[pos++] = 0;
    // name string (empty)
    payload[pos++] = 0; payload[pos++] = 0;
    // token (empty)
    payload[pos++] = 0; payload[pos++] = 0;

    injectFrame(payload, pos);
  }

  void injectSet(const char* address, float value) {
    uint8_t buf[256];
    size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), address, value);
    // buildSetFrame already includes TCP prefix, just inject the raw bytes
    rxData.insert(rxData.end(), buf, buf + len);
  }

  void injectSetInt(const char* address, int32_t value) {
    uint8_t buf[256];
    size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), address, value);
    rxData.insert(rxData.end(), buf, buf + len);
  }

  void injectSetString(const char* address, const char* value) {
    uint8_t buf[256];
    size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), address, value);
    rxData.insert(rxData.end(), buf, buf + len);
  }

  void injectSetBool(const char* address, bool value) {
    uint8_t buf[256];
    size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), address, value);
    rxData.insert(rxData.end(), buf, buf + len);
  }

  void injectEmit(const char* address) {
    uint8_t buf[256];
    size_t len = ClaspCodec::buildEmitFrame(buf, sizeof(buf), address);
    rxData.insert(rxData.end(), buf, buf + len);
  }

  void injectEmit(const char* address, float value) {
    uint8_t buf[256];
    size_t len = ClaspCodec::buildEmitFrame(buf, sizeof(buf), address, value);
    rxData.insert(rxData.end(), buf, buf + len);
  }

  void injectStream(const char* address, float value) {
    uint8_t buf[256];
    size_t len = ClaspCodec::buildStreamFrame(buf, sizeof(buf), address, value);
    rxData.insert(rxData.end(), buf, buf + len);
  }

  void injectPing() {
    // PING payload: [0x41]
    uint8_t payload[1] = { CLASP_MSG_PING };
    injectFrame(payload, 1);
  }

  // Inject a raw TCP-framed CLASP frame
  void injectFrame(const uint8_t* payload, size_t payloadLen) {
    uint8_t header[8]; // TCP prefix + frame header
    uint32_t frameLen = CLASP_FRAME_HEADER_SIZE + (uint32_t)payloadLen;
    ClaspCodec::writeTcpPrefix(header, frameLen);
    ClaspCodec::writeFrameHeader(header + CLASP_TCP_PREFIX_SIZE, (uint16_t)payloadLen);
    rxData.insert(rxData.end(), header, header + 8);
    rxData.insert(rxData.end(), payload, payload + payloadLen);
  }

  // Inject raw bytes (for oversized frame tests)
  void injectRaw(const uint8_t* data, size_t len) {
    rxData.insert(rxData.end(), data, data + len);
  }

  // Check if transmitted data contains a specific message type
  // TX format: [TCP prefix][frame header][payload where payload[0] = msg_type]
  bool txContainsMsgType(uint8_t msgType) {
    // Walk through txData looking for TCP prefix + frame header + msg_type
    size_t i = 0;
    while (i + CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE < txData.size()) {
      uint32_t frameLen = ClaspCodec::readTcpPrefix(txData.data() + i);
      if (frameLen >= 1 && i + CLASP_TCP_PREFIX_SIZE + frameLen <= txData.size()) {
        // Check magic
        if (txData[i + CLASP_TCP_PREFIX_SIZE] == CLASP_MAGIC) {
          // payload starts after TCP prefix + frame header
          uint8_t mt = txData[i + CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE];
          if (mt == msgType) return true;
        }
        i += CLASP_TCP_PREFIX_SIZE + frameLen;
      } else {
        i++;
      }
    }
    return false;
  }
};

// ============================================================================
// Callback tracking
// ============================================================================

static int callbackCount = 0;
static char lastAddress[128] = "";
static ClaspValue lastValue;
static int multiCount = 0;
static char multiAddresses[16][128];
static ClaspValue multiValues[16];

void resetCallbacks() {
  callbackCount = 0;
  lastAddress[0] = '\0';
  lastValue = ClaspValue();
  multiCount = 0;
}

void testCallback(const char* address, ClaspValue value) {
  callbackCount++;
  strncpy(lastAddress, address, sizeof(lastAddress) - 1);
  lastAddress[sizeof(lastAddress) - 1] = '\0';
  lastValue = value;
  if (multiCount < 16) {
    strncpy(multiAddresses[multiCount], address, 127);
    multiAddresses[multiCount][127] = '\0';
    multiValues[multiCount] = value;
    multiCount++;
  }
}

static int callback2Count = 0;
void testCallback2(const char* /* address */, ClaspValue /* value */) {
  callback2Count++;
}

// ============================================================================
// Connection tests
// ============================================================================

void test_connect_sends_hello() {
  printf("test_connect_sends_hello\n");
  MockClient mock;
  ClaspClient clasp(mock);

  mock.injectWelcome();
  bool ok = clasp.connect("localhost", 7330, "TestClient");
  ASSERT(ok, "connect succeeds");
  ASSERT(clasp.connected(), "reports connected");
  ASSERT(mock.txContainsMsgType(CLASP_MSG_HELLO), "sent HELLO");
}

void test_connect_default_name() {
  printf("test_connect_default_name\n");
  MockClient mock;
  ClaspClient clasp(mock);

  mock.injectWelcome();
  bool ok = clasp.connect("localhost", 7330);
  ASSERT(ok, "connect with default name succeeds");
  ASSERT(mock.txContainsMsgType(CLASP_MSG_HELLO), "sent HELLO");
}

void test_connect_timeout_without_welcome() {
  printf("test_connect_timeout_without_welcome\n");
  MockClient mock;
  ClaspClient clasp(mock);

  _clasp_reset_millis();
  _clasp_set_millis_increment(1000);
  bool ok = clasp.connect("localhost", 7330, "TimeoutTest");
  _clasp_set_millis_increment(0);
  _clasp_reset_millis();
  ASSERT(!ok, "connect fails without WELCOME");
  ASSERT(!clasp.connected(), "not connected after timeout");
}

void test_connect_tcp_failure() {
  printf("test_connect_tcp_failure\n");
  class FailClient : public Client {
  public:
    int connect(const char*, uint16_t) override { return 0; }
    size_t write(const uint8_t*, size_t) override { return 0; }
    int available() override { return 0; }
    int read(uint8_t*, size_t) override { return 0; }
    uint8_t connected() override { return 0; }
    void stop() override {}
  };

  FailClient fail;
  ClaspClient clasp(fail);
  ASSERT(!clasp.connect("localhost", 7330), "connect fails on TCP failure");
}

void test_disconnect() {
  printf("test_disconnect\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  ASSERT(clasp.connected(), "connected");

  clasp.disconnect();
  ASSERT(!clasp.connected(), "disconnected");
  ASSERT(!mock.isConnected, "TCP stopped");
}

void test_reconnection_resets_state() {
  printf("test_reconnection_resets_state\n");
  MockClient mock;
  ClaspClient clasp(mock);

  mock.injectWelcome();
  clasp.connect("localhost", 7330, "First");
  ASSERT(clasp.connected(), "first connect ok");

  resetCallbacks();
  clasp.subscribe("/test", testCallback);
  mock.injectSet("/test", 1.0f);
  clasp.loop();
  ASSERT(callbackCount == 1, "callback on first connection");

  clasp.disconnect();
  mock.reset();
  mock.injectWelcome();
  clasp.connect("localhost", 7330, "Second");
  ASSERT(clasp.connected(), "reconnect ok");

  resetCallbacks();
  mock.injectSet("/test", 2.0f);
  clasp.loop();
  ASSERT(callbackCount == 1, "subscription survives reconnect");
}

// ============================================================================
// Send operation tests
// ============================================================================

void test_set_float() {
  printf("test_set_float\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  mock.txData.clear();

  ASSERT(clasp.set("/test/value", 3.14f), "set float returns true");
  ASSERT(mock.txContainsMsgType(CLASP_MSG_SET), "sent SET");
}

void test_set_int() {
  printf("test_set_int\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  mock.txData.clear();

  ASSERT(clasp.set("/counter", (int32_t)42), "set int returns true");
  ASSERT(mock.txContainsMsgType(CLASP_MSG_SET), "sent SET");
}

void test_set_string() {
  printf("test_set_string\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  mock.txData.clear();

  ASSERT(clasp.set("/status", "online"), "set string returns true");
  ASSERT(mock.txContainsMsgType(CLASP_MSG_SET), "sent SET");
}

void test_set_bool() {
  printf("test_set_bool\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  mock.txData.clear();

  ASSERT(clasp.set("/flag", true), "set bool returns true");
  ASSERT(mock.txContainsMsgType(CLASP_MSG_SET), "sent SET");
}

void test_emit_no_value() {
  printf("test_emit_no_value\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  mock.txData.clear();

  ASSERT(clasp.emit("/cue/go"), "emit returns true");
  ASSERT(mock.txContainsMsgType(CLASP_MSG_PUBLISH), "sent PUBLISH");
}

void test_emit_with_value() {
  printf("test_emit_with_value\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  mock.txData.clear();

  ASSERT(clasp.emit("/level", 0.5f), "emit with value returns true");
  ASSERT(mock.txContainsMsgType(CLASP_MSG_PUBLISH), "sent PUBLISH");
}

void test_stream() {
  printf("test_stream\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  mock.txData.clear();

  ASSERT(clasp.stream("/audio/level", 0.42f), "stream returns true");
  ASSERT(mock.txContainsMsgType(CLASP_MSG_PUBLISH), "sent PUBLISH");
}

// ============================================================================
// Receive and dispatch tests
// ============================================================================

void test_dispatch_set_float() {
  printf("test_dispatch_set_float\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/sensors/temp", testCallback);

  mock.injectSet("/sensors/temp", 22.5f);
  clasp.loop();

  ASSERT(callbackCount == 1, "callback fired once");
  ASSERT(strcmp(lastAddress, "/sensors/temp") == 0, "address matches");
  ASSERT(lastValue.type == ClaspValue::Float, "type is Float");
  ASSERT(fabsf(lastValue.f - 22.5f) < 0.01f, "float value matches");
}

void test_dispatch_set_int() {
  printf("test_dispatch_set_int\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/counter", testCallback);

  mock.injectSetInt("/counter", -99);
  clasp.loop();

  ASSERT(callbackCount == 1, "callback fired");
  ASSERT(lastValue.type == ClaspValue::Int, "type is Int");
  ASSERT(lastValue.i == -99, "int value matches");
}

void test_dispatch_set_string() {
  printf("test_dispatch_set_string\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/status", testCallback);

  mock.injectSetString("/status", "ready");
  clasp.loop();

  ASSERT(callbackCount == 1, "callback fired");
  ASSERT(lastValue.type == ClaspValue::String, "type is String");
  ASSERT(lastValue.len == 5, "string length 5");
  ASSERT(memcmp(lastValue.str, "ready", 5) == 0, "string value matches");
}

void test_dispatch_set_bool() {
  printf("test_dispatch_set_bool\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/flag", testCallback);

  mock.injectSetBool("/flag", true);
  clasp.loop();

  ASSERT(callbackCount == 1, "callback fired");
  ASSERT(lastValue.type == ClaspValue::Bool, "type is Bool");
  ASSERT(lastValue.b == true, "bool value true");
}

void test_dispatch_emit() {
  printf("test_dispatch_emit\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/cues/**", testCallback);

  mock.injectEmit("/cues/go");
  clasp.loop();

  ASSERT(callbackCount == 1, "EMIT dispatched");
  ASSERT(strcmp(lastAddress, "/cues/go") == 0, "address matches");
  ASSERT(lastValue.type == ClaspValue::Null, "no-arg emit is Null");
}

void test_dispatch_emit_with_value() {
  printf("test_dispatch_emit_with_value\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/alerts/*", testCallback);

  mock.injectEmit("/alerts/fire", 1.0f);
  clasp.loop();

  ASSERT(callbackCount == 1, "EMIT with value dispatched");
  ASSERT(lastValue.type == ClaspValue::Float, "value type is Float");
  ASSERT(fabsf(lastValue.f - 1.0f) < 0.01f, "value matches");
}

void test_dispatch_stream() {
  printf("test_dispatch_stream\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/audio/**", testCallback);

  mock.injectStream("/audio/level", 0.42f);
  clasp.loop();

  ASSERT(callbackCount == 1, "STREAM dispatched");
  ASSERT(strcmp(lastAddress, "/audio/level") == 0, "address matches");
  ASSERT(fabsf(lastValue.f - 0.42f) < 0.01f, "value matches");
}

// ============================================================================
// Wildcard subscription tests
// ============================================================================

void test_wildcard_globstar() {
  printf("test_wildcard_globstar\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/sensors/**", testCallback);

  mock.injectSet("/sensors/room1/temp", 20.0f);
  clasp.loop();
  ASSERT(callbackCount == 1, "/** matches nested");

  mock.injectSet("/sensors/room2/humidity", 65.0f);
  clasp.loop();
  ASSERT(callbackCount == 2, "/** matches different nested");

  mock.injectSet("/lights/brightness", 0.5f);
  clasp.loop();
  ASSERT(callbackCount == 2, "/** doesn't match unrelated");
}

void test_wildcard_single_level() {
  printf("test_wildcard_single_level\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/sensors/*/temp", testCallback);

  mock.injectSet("/sensors/room1/temp", 20.0f);
  clasp.loop();
  ASSERT(callbackCount == 1, "/* matches single segment");

  mock.injectSet("/sensors/room1/humidity", 50.0f);
  clasp.loop();
  ASSERT(callbackCount == 1, "/* doesn't match wrong tail");

  mock.injectSet("/sensors/room1/sub/temp", 21.0f);
  clasp.loop();
  ASSERT(callbackCount == 1, "/* doesn't match multiple segments");
}

void test_no_match_no_dispatch() {
  printf("test_no_match_no_dispatch\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/sensors/temp", testCallback);

  mock.injectSet("/lights/brightness", 0.5f);
  clasp.loop();
  ASSERT(callbackCount == 0, "no callback for non-matching address");
}

// ============================================================================
// Subscription management tests
// ============================================================================

void test_unsubscribe() {
  printf("test_unsubscribe\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/test/val", testCallback);

  mock.injectSet("/test/val", 1.0f);
  clasp.loop();
  ASSERT(callbackCount == 1, "callback before unsubscribe");

  clasp.unsubscribe("/test/val");

  mock.injectSet("/test/val", 2.0f);
  clasp.loop();
  ASSERT(callbackCount == 1, "callback NOT fired after unsubscribe");
}

void test_unsubscribe_frees_slot() {
  printf("test_unsubscribe_frees_slot\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  char patterns[CLASP_MAX_SUBSCRIPTIONS][32];
  for (int i = 0; i < CLASP_MAX_SUBSCRIPTIONS; i++) {
    snprintf(patterns[i], sizeof(patterns[i]), "/slot/%d", i);
    ASSERT(clasp.subscribe(patterns[i], testCallback), "subscribe fills slot");
  }

  ASSERT(!clasp.subscribe("/overflow", testCallback), "subscribe fails when full");

  clasp.unsubscribe(patterns[0]);
  ASSERT(clasp.subscribe("/new", testCallback), "subscribe succeeds after free");
}

void test_max_subscriptions() {
  printf("test_max_subscriptions\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  for (int i = 0; i < CLASP_MAX_SUBSCRIPTIONS; i++) {
    char pattern[32];
    snprintf(pattern, sizeof(pattern), "/slot/%d", i);
    ASSERT(clasp.subscribe(pattern, testCallback), "subscribe slot ok");
  }

  ASSERT(!clasp.subscribe("/overflow", testCallback), "all slots full");
}

void test_multiple_subscriptions_same_message() {
  printf("test_multiple_subscriptions_same_message\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  callback2Count = 0;

  clasp.subscribe("/sensors/**", testCallback);
  clasp.subscribe("/sensors/temp", testCallback2);

  mock.injectSet("/sensors/temp", 25.0f);
  clasp.loop();

  ASSERT(callbackCount == 1, "first callback fired");
  ASSERT(callback2Count == 1, "second callback also fired");
}

// ============================================================================
// PING/PONG tests
// ============================================================================

void test_ping_sends_pong() {
  printf("test_ping_sends_pong\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  mock.txData.clear();

  mock.injectPing();
  clasp.loop();
  ASSERT(mock.txContainsMsgType(CLASP_MSG_PONG), "responded with PONG");
}

// ============================================================================
// Operations when disconnected
// ============================================================================

void test_operations_when_disconnected() {
  printf("test_operations_when_disconnected\n");
  MockClient mock;
  ClaspClient clasp(mock);

  ASSERT(!clasp.set("/test", 1.0f), "set float fails disconnected");
  ASSERT(!clasp.set("/test", (int32_t)1), "set int fails disconnected");
  ASSERT(!clasp.set("/test", "x"), "set string fails disconnected");
  ASSERT(!clasp.set("/test", true), "set bool fails disconnected");
  ASSERT(!clasp.emit("/test"), "emit fails disconnected");
  ASSERT(!clasp.emit("/test", 1.0f), "emit with value fails disconnected");
  ASSERT(!clasp.stream("/test", 1.0f), "stream fails disconnected");
  ASSERT(!clasp.subscribe("/test", testCallback), "subscribe fails disconnected");
  ASSERT(!clasp.unsubscribe("/test"), "unsubscribe fails disconnected");
}

void test_loop_detects_tcp_disconnect() {
  printf("test_loop_detects_tcp_disconnect\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);
  ASSERT(clasp.connected(), "starts connected");

  mock.isConnected = false;
  bool loopResult = clasp.loop();
  ASSERT(!loopResult, "loop returns false on TCP drop");
  ASSERT(!clasp.connected(), "disconnected after TCP drop");
}

// ============================================================================
// Multiple frames in one loop
// ============================================================================

void test_multiple_frames_in_one_loop() {
  printf("test_multiple_frames_in_one_loop\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/multi/**", testCallback);

  mock.injectSet("/multi/a", 1.0f);
  mock.injectSet("/multi/b", 2.0f);
  mock.injectSet("/multi/c", 3.0f);

  clasp.loop();
  ASSERT(callbackCount == 3, "all three callbacks in one loop");
  ASSERT(strcmp(multiAddresses[0], "/multi/a") == 0, "first addr");
  ASSERT(strcmp(multiAddresses[1], "/multi/b") == 0, "second addr");
  ASSERT(strcmp(multiAddresses[2], "/multi/c") == 0, "third addr");
  ASSERT(fabsf(multiValues[0].f - 1.0f) < 0.01f, "first value");
  ASSERT(fabsf(multiValues[1].f - 2.0f) < 0.01f, "second value");
  ASSERT(fabsf(multiValues[2].f - 3.0f) < 0.01f, "third value");
}

// ============================================================================
// Partial frame delivery (TCP fragmentation)
// ============================================================================

void test_partial_frame_delivery() {
  printf("test_partial_frame_delivery\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/partial", testCallback);

  mock.injectSet("/partial", 99.0f);
  mock.maxBytesPerRead = 1;

  // Need many loop() calls to reassemble 1 byte at a time
  for (int i = 0; i < 40; i++) {
    clasp.loop();
  }

  ASSERT(callbackCount == 1, "callback after incremental reassembly");
  ASSERT(strcmp(lastAddress, "/partial") == 0, "address correct");
  ASSERT(fabsf(lastValue.f - 99.0f) < 0.01f, "value correct");
}

void test_partial_then_complete() {
  printf("test_partial_then_complete\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/**", testCallback);

  // First frame: 2 bytes at a time
  mock.injectSet("/first", 1.0f);
  mock.maxBytesPerRead = 2;
  for (int i = 0; i < 30; i++) clasp.loop();
  ASSERT(callbackCount == 1, "first frame from 2-byte chunks");

  // Second frame: all at once
  mock.maxBytesPerRead = -1;
  mock.injectSet("/second", 2.0f);
  clasp.loop();
  ASSERT(callbackCount == 2, "second frame normal after partial");
}

// ============================================================================
// Oversized frame handling
// ============================================================================

void test_oversized_frame_skipped() {
  printf("test_oversized_frame_skipped\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/**", testCallback);

  // Build an oversized TCP-framed message
  // TCP prefix claiming frame of 300 bytes (exceeds CLASP_MAX_PACKET_SIZE - TCP_PREFIX_SIZE)
  uint8_t tcpPrefix[4];
  ClaspCodec::writeTcpPrefix(tcpPrefix, 300);
  mock.injectRaw(tcpPrefix, 4);

  // Inject 300 bytes of garbage (the "frame")
  uint8_t garbage[300];
  memset(garbage, 0xAA, 300);
  mock.injectRaw(garbage, 300);

  // Then inject a normal valid frame after it
  mock.injectSet("/after-oversize", 42.0f);

  for (int i = 0; i < 60; i++) clasp.loop();

  ASSERT(callbackCount == 1, "normal frame after oversized received");
  ASSERT(strcmp(lastAddress, "/after-oversize") == 0, "correct address after skip");
  ASSERT(fabsf(lastValue.f - 42.0f) < 0.01f, "correct value after skip");
}

// ============================================================================
// Mixed message type dispatch
// ============================================================================

void test_mixed_messages_in_sequence() {
  printf("test_mixed_messages_in_sequence\n");
  MockClient mock;
  ClaspClient clasp(mock);
  mock.injectWelcome();
  clasp.connect("localhost", 7330);

  resetCallbacks();
  clasp.subscribe("/**", testCallback);

  mock.injectSet("/a", 1.0f);
  mock.injectEmit("/b");
  mock.injectStream("/c", 3.0f);
  mock.injectEmit("/d", 4.0f);
  mock.injectSetInt("/e", 5);

  clasp.loop();
  ASSERT(callbackCount == 5, "all 5 messages dispatched");
  ASSERT(strcmp(multiAddresses[0], "/a") == 0, "SET address");
  ASSERT(strcmp(multiAddresses[1], "/b") == 0, "EMIT address");
  ASSERT(strcmp(multiAddresses[2], "/c") == 0, "STREAM address");
  ASSERT(strcmp(multiAddresses[3], "/d") == 0, "EMIT-with-value address");
  ASSERT(strcmp(multiAddresses[4], "/e") == 0, "SET-int address");
  ASSERT(multiValues[0].type == ClaspValue::Float, "SET is float");
  ASSERT(multiValues[1].type == ClaspValue::Null, "EMIT no-arg is Null");
  ASSERT(multiValues[2].type == ClaspValue::Float, "STREAM is float");
  ASSERT(multiValues[3].type == ClaspValue::Float, "EMIT with value is float");
  ASSERT(multiValues[4].type == ClaspValue::Int, "SET int is Int");
}

int main() {
  printf("=== CLASP Client Tests (v3 binary) ===\n\n");
  _clasp_reset_millis();

  // Connection
  test_connect_sends_hello();
  test_connect_default_name();
  test_connect_timeout_without_welcome();
  test_connect_tcp_failure();
  test_disconnect();
  test_reconnection_resets_state();

  // Send operations
  test_set_float();
  test_set_int();
  test_set_string();
  test_set_bool();
  test_emit_no_value();
  test_emit_with_value();
  test_stream();

  // Receive and dispatch
  test_dispatch_set_float();
  test_dispatch_set_int();
  test_dispatch_set_string();
  test_dispatch_set_bool();
  test_dispatch_emit();
  test_dispatch_emit_with_value();
  test_dispatch_stream();

  // Wildcards
  test_wildcard_globstar();
  test_wildcard_single_level();
  test_no_match_no_dispatch();

  // Subscription management
  test_unsubscribe();
  test_unsubscribe_frees_slot();
  test_max_subscriptions();
  test_multiple_subscriptions_same_message();

  // PING/PONG
  test_ping_sends_pong();

  // Disconnected operations
  test_operations_when_disconnected();
  test_loop_detects_tcp_disconnect();

  // Multiple frames
  test_multiple_frames_in_one_loop();

  // Partial delivery (TCP fragmentation)
  test_partial_frame_delivery();
  test_partial_then_complete();

  // Oversized frames
  test_oversized_frame_skipped();

  // Mixed messages
  test_mixed_messages_in_sequence();

  printf("\n%d/%d tests passed", tests_passed, tests_run);
  if (tests_failed > 0) printf(" (%d FAILED)", tests_failed);
  printf("\n");
  return tests_failed == 0 ? 0 : 1;
}
