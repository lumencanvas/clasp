/**
 * CLASP Frame Round-Trip Tests (v3 binary protocol)
 *
 * Tests complete frames (TCP prefix + frame header + payload) encode/decode.
 * Every frame builder is verified by parsing back the message type, address, and value.
 * Build and run on desktop with: make test
 */

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include "../src/ClaspCodec.h"

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

// Helper: parse a SET frame built by our builders.
// All frames have: [4-byte TCP prefix][4-byte frame header][payload...]
// SET payload: [0x21][flags:vtype][u16+address][value_data]
struct ParsedSetFrame {
  uint8_t msgType;
  uint8_t vtype;
  char address[128];
  uint16_t addrLen;
  ClaspValue value;
  bool ok;
};

ParsedSetFrame parseSetFrame(const uint8_t* buf, size_t totalLen) {
  ParsedSetFrame f;
  memset(&f, 0, sizeof(f));
  f.ok = false;

  if (totalLen < CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + 2) return f;

  // Skip TCP prefix
  const uint8_t* frame = buf + CLASP_TCP_PREFIX_SIZE;
  uint16_t payloadLen;
  if (!ClaspCodec::parseFrameHeader(frame, totalLen - CLASP_TCP_PREFIX_SIZE, payloadLen)) return f;

  const uint8_t* payload = frame + CLASP_FRAME_HEADER_SIZE;
  if (payloadLen < 2) return f;

  f.msgType = payload[0];
  f.vtype = payload[1] & 0x0F;

  const uint8_t* p = payload + 2;
  size_t remaining = payloadLen - 2;

  const char* addr;
  size_t consumed = ClaspCodec::readString(p, remaining, addr, f.addrLen);
  if (consumed == 0) return f;

  if (f.addrLen < sizeof(f.address)) {
    memcpy(f.address, addr, f.addrLen);
    f.address[f.addrLen] = '\0';
  }

  p += consumed;
  remaining -= consumed;

  if (f.vtype != CLASP_VAL_NULL && remaining > 0) {
    ClaspCodec::readValueData(p, remaining, f.vtype, f.value);
  }
  f.ok = true;
  return f;
}

// Helper: parse a PUBLISH frame
struct ParsedPublishFrame {
  uint8_t sigType;
  char address[128];
  uint16_t addrLen;
  ClaspValue value;
  bool hasValue;
  bool ok;
};

ParsedPublishFrame parsePublishFrame(const uint8_t* buf, size_t totalLen) {
  ParsedPublishFrame f;
  memset(&f, 0, sizeof(f));
  f.ok = false;

  if (totalLen < CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + 2) return f;

  const uint8_t* frame = buf + CLASP_TCP_PREFIX_SIZE;
  uint16_t payloadLen;
  if (!ClaspCodec::parseFrameHeader(frame, totalLen - CLASP_TCP_PREFIX_SIZE, payloadLen)) return f;

  const uint8_t* payload = frame + CLASP_FRAME_HEADER_SIZE;
  if (payloadLen < 2) return f;

  // payload[0] should be CLASP_MSG_PUBLISH
  if (payload[0] != CLASP_MSG_PUBLISH) return f;
  uint8_t flags = payload[1];
  f.sigType = (flags >> 5) & 0x07;

  const uint8_t* p = payload + 2;
  size_t remaining = payloadLen - 2;

  const char* addr;
  size_t consumed = ClaspCodec::readString(p, remaining, addr, f.addrLen);
  if (consumed == 0) return f;

  if (f.addrLen < sizeof(f.address)) {
    memcpy(f.address, addr, f.addrLen);
    f.address[f.addrLen] = '\0';
  }

  p += consumed;
  remaining -= consumed;

  // Value indicator
  if (remaining >= 1) {
    uint8_t vi = *p++;
    remaining--;
    if (vi == 1 && remaining >= 1) {
      uint8_t vtype = *p++;
      remaining--;
      ClaspCodec::readValueData(p, remaining, vtype, f.value);
      f.hasValue = true;
    }
  }

  f.ok = true;
  return f;
}

// ============================================================================
// SET frame tests
// ============================================================================

void test_set_float_frame() {
  printf("test_set_float_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), "/test/val", 42.5f);
  ASSERT(len > 0, "buildSetFrame succeeds");

  // Verify TCP prefix
  uint32_t tcpLen = ClaspCodec::readTcpPrefix(buf);
  ASSERT(tcpLen == len - CLASP_TCP_PREFIX_SIZE, "TCP prefix length correct");

  ParsedSetFrame f = parseSetFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(f.msgType == CLASP_MSG_SET, "msg type is SET");
  ASSERT(f.vtype == CLASP_VAL_F32, "vtype is F32");
  ASSERT(strcmp(f.address, "/test/val") == 0, "address matches");
  ASSERT(f.value.type == ClaspValue::Float, "value type is Float");
  ASSERT(fabsf(f.value.f - 42.5f) < 0.001f, "float value matches");
}

void test_set_int_frame() {
  printf("test_set_int_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), "/counter", (int32_t)99);
  ASSERT(len > 0, "buildSetFrame int succeeds");

  ParsedSetFrame f = parseSetFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(f.msgType == CLASP_MSG_SET, "msg type is SET");
  ASSERT(f.vtype == CLASP_VAL_I32, "vtype is I32");
  ASSERT(strcmp(f.address, "/counter") == 0, "address matches");
  ASSERT(f.value.type == ClaspValue::Int, "value type is Int");
  ASSERT(f.value.i == 99, "int value matches");
}

void test_set_negative_int_frame() {
  printf("test_set_negative_int_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), "/neg", (int32_t)-42);
  ParsedSetFrame f = parseSetFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(f.value.type == ClaspValue::Int, "value type is Int");
  ASSERT(f.value.i == -42, "negative int value matches");
}

void test_set_string_frame() {
  printf("test_set_string_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), "/status", "online");
  ASSERT(len > 0, "buildSetFrame string succeeds");

  ParsedSetFrame f = parseSetFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(f.msgType == CLASP_MSG_SET, "msg type is SET");
  ASSERT(f.vtype == CLASP_VAL_STRING, "vtype is STRING");
  ASSERT(strcmp(f.address, "/status") == 0, "address matches");
  ASSERT(f.value.type == ClaspValue::String, "value type is String");
  ASSERT(f.value.len == 6, "string length is 6");
  ASSERT(memcmp(f.value.str, "online", 6) == 0, "string is 'online'");
}

void test_set_empty_string_frame() {
  printf("test_set_empty_string_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), "/empty", "");
  ASSERT(len > 0, "buildSetFrame empty string succeeds");

  ParsedSetFrame f = parseSetFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(f.value.type == ClaspValue::String, "value type is String");
  ASSERT(f.value.len == 0, "empty string length is 0");
}

void test_set_bool_frame() {
  printf("test_set_bool_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), "/flag", true);
  ASSERT(len > 0, "buildSetFrame bool-true succeeds");

  ParsedSetFrame f = parseSetFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(f.vtype == CLASP_VAL_BOOL, "vtype is BOOL");
  ASSERT(f.value.type == ClaspValue::Bool, "value type is Bool");
  ASSERT(f.value.b == true, "value is true");

  len = ClaspCodec::buildSetFrame(buf, sizeof(buf), "/flag", false);
  f = parseSetFrame(buf, len);
  ASSERT(f.value.b == false, "value is false");
}

// ============================================================================
// PUBLISH (EMIT/STREAM) frame tests
// ============================================================================

void test_emit_no_value_frame() {
  printf("test_emit_no_value_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildEmitFrame(buf, sizeof(buf), "/cues/go");
  ASSERT(len > 0, "buildEmitFrame succeeds");

  ParsedPublishFrame f = parsePublishFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(f.sigType == CLASP_SIG_EVENT, "sig type is EVENT");
  ASSERT(strcmp(f.address, "/cues/go") == 0, "address matches");
  ASSERT(!f.hasValue, "no value");
}

void test_emit_with_float_frame() {
  printf("test_emit_with_float_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildEmitFrame(buf, sizeof(buf), "/level", 0.75f);
  ASSERT(len > 0, "buildEmitFrame with value succeeds");

  ParsedPublishFrame f = parsePublishFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(f.sigType == CLASP_SIG_EVENT, "sig type is EVENT");
  ASSERT(strcmp(f.address, "/level") == 0, "address matches");
  ASSERT(f.hasValue, "has value");
  ASSERT(f.value.type == ClaspValue::Float, "value type is Float");
  ASSERT(fabsf(f.value.f - 0.75f) < 0.001f, "float value matches");
}

void test_stream_frame() {
  printf("test_stream_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildStreamFrame(buf, sizeof(buf), "/audio/level", 0.42f);
  ASSERT(len > 0, "buildStreamFrame succeeds");

  ParsedPublishFrame f = parsePublishFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(f.sigType == CLASP_SIG_STREAM, "sig type is STREAM");
  ASSERT(strcmp(f.address, "/audio/level") == 0, "address matches");
  ASSERT(f.hasValue, "has value");
  ASSERT(fabsf(f.value.f - 0.42f) < 0.001f, "float value matches");
}

// ============================================================================
// SUBSCRIBE / UNSUBSCRIBE frame tests
// ============================================================================

void test_subscribe_frame() {
  printf("test_subscribe_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildSubscribeFrame(buf, sizeof(buf), "/sensors/**", 7);
  ASSERT(len > 0, "buildSubscribeFrame succeeds");

  // Verify structure: TCP prefix + frame header + [0x10][u32 id][u16+pattern][type_mask][options]
  const uint8_t* payload = buf + CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE;
  ASSERT(payload[0] == CLASP_MSG_SUBSCRIBE, "msg type is SUBSCRIBE");

  // sub_id (u32 at offset 1)
  uint32_t subId = ((uint32_t)payload[1] << 24) | ((uint32_t)payload[2] << 16) |
                   ((uint32_t)payload[3] << 8) | payload[4];
  ASSERT(subId == 7, "sub ID is 7");

  const char* pattern;
  uint16_t patLen;
  ClaspCodec::readString(payload + 5, 32, pattern, patLen);
  ASSERT(patLen == 11, "pattern length correct");
  ASSERT(memcmp(pattern, "/sensors/**", 11) == 0, "pattern matches");
}

void test_unsubscribe_frame() {
  printf("test_unsubscribe_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildUnsubscribeFrame(buf, sizeof(buf), 42);
  ASSERT(len > 0, "buildUnsubscribeFrame succeeds");

  const uint8_t* payload = buf + CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE;
  ASSERT(payload[0] == CLASP_MSG_UNSUBSCRIBE, "msg type is UNSUBSCRIBE");

  uint32_t subId = ((uint32_t)payload[1] << 24) | ((uint32_t)payload[2] << 16) |
                   ((uint32_t)payload[3] << 8) | payload[4];
  ASSERT(subId == 42, "sub ID is 42");
}

// ============================================================================
// HELLO frame test
// ============================================================================

void test_hello_frame() {
  printf("test_hello_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildHelloFrame(buf, sizeof(buf), "Test Client");
  ASSERT(len > 0, "buildHelloFrame succeeds");

  // Verify TCP prefix
  uint32_t tcpLen = ClaspCodec::readTcpPrefix(buf);
  ASSERT(tcpLen == len - CLASP_TCP_PREFIX_SIZE, "TCP prefix correct");

  const uint8_t* payload = buf + CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE;
  ASSERT(payload[0] == CLASP_MSG_HELLO, "msg type is HELLO");
  ASSERT(payload[1] == CLASP_PROTOCOL_VERSION, "version is 3");
  // Features: param(0x80) + event(0x40) + stream(0x20) = 0xE0
  ASSERT(payload[2] == 0xE0, "features = param|event|stream");

  const char* name;
  uint16_t nameLen;
  ClaspCodec::readString(payload + 3, 64, name, nameLen);
  ASSERT(nameLen == 11, "name length correct");
  ASSERT(memcmp(name, "Test Client", 11) == 0, "name matches");

  // After name string: token (empty = u16 0)
  const uint8_t* tokenPos = payload + 3 + 2 + nameLen;
  ASSERT(tokenPos[0] == 0 && tokenPos[1] == 0, "empty token");
}

void test_hello_null_name_frame() {
  printf("test_hello_null_name_frame\n");
  uint8_t buf[256];

  size_t len = ClaspCodec::buildHelloFrame(buf, sizeof(buf), nullptr);
  ASSERT(len > 0, "buildHelloFrame with null name succeeds");

  const uint8_t* payload = buf + CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE;
  ASSERT(payload[0] == CLASP_MSG_HELLO, "msg type is HELLO");

  // Name should be empty string (u16 0)
  ASSERT(payload[3] == 0 && payload[4] == 0, "null name encodes as empty string");
}

// ============================================================================
// PONG frame test
// ============================================================================

void test_pong_frame() {
  printf("test_pong_frame\n");
  uint8_t buf[16];

  size_t len = ClaspCodec::buildPongFrame(buf, sizeof(buf));
  // TCP prefix(4) + frame header(4) + payload(1 byte for msg type)
  ASSERT(len == CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + 1, "pong is 9 bytes total");

  const uint8_t* payload = buf + CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE;
  ASSERT(payload[0] == CLASP_MSG_PONG, "msg type is PONG");

  uint16_t payloadLen;
  ClaspCodec::parseFrameHeader(buf + CLASP_TCP_PREFIX_SIZE, 4, payloadLen);
  ASSERT(payloadLen == 1, "payload is 1 byte (msg type only)");
}

// ============================================================================
// Buffer boundary tests
// ============================================================================

void test_buffer_too_small() {
  printf("test_buffer_too_small\n");
  uint8_t buf[8];

  ASSERT(ClaspCodec::buildSetFrame(buf, sizeof(buf), "/a/long/addr", 1.0f) == 0,
         "SET returns 0 when buffer too small");
  ASSERT(ClaspCodec::buildEmitFrame(buf, sizeof(buf), "/a/long/addr") == 0,
         "EMIT returns 0 when buffer too small");
  ASSERT(ClaspCodec::buildStreamFrame(buf, sizeof(buf), "/a/long/addr", 1.0f) == 0,
         "STREAM returns 0 when buffer too small");
  ASSERT(ClaspCodec::buildSubscribeFrame(buf, sizeof(buf), "/a/long/pattern/**") == 0,
         "SUBSCRIBE returns 0 when buffer too small");
  ASSERT(ClaspCodec::buildHelloFrame(buf, sizeof(buf), "A Very Long Name") == 0,
         "HELLO returns 0 when buffer too small");
}

void test_pong_buffer_too_small() {
  printf("test_pong_buffer_too_small\n");
  uint8_t buf[4];
  ASSERT(ClaspCodec::buildPongFrame(buf, sizeof(buf)) == 0,
         "PONG returns 0 when buffer < 9");
}

void test_exact_fit_buffer() {
  printf("test_exact_fit_buffer\n");
  // SET float for "/x": TCP(4) + frame(4) + msg(1) + flags(1) + addr(2+2) + f32(4) = 18
  uint8_t buf[18];
  size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), "/x", 1.0f);
  ASSERT(len == 18, "frame fits exactly");

  ParsedSetFrame f = parseSetFrame(buf, len);
  ASSERT(f.ok, "exact-fit frame parses");
  ASSERT(strcmp(f.address, "/x") == 0, "address correct");
}

void test_one_byte_short_buffer() {
  printf("test_one_byte_short_buffer\n");
  uint8_t buf[17];
  size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), "/x", 1.0f);
  ASSERT(len == 0, "one byte short returns 0");
}

void test_long_address() {
  printf("test_long_address\n");
  uint8_t buf[256];

  const char* longAddr = "/this/is/a/fairly/long/address/path";
  size_t len = ClaspCodec::buildSetFrame(buf, sizeof(buf), longAddr, 1.0f);
  ASSERT(len > 0, "long address succeeds");

  ParsedSetFrame f = parseSetFrame(buf, len);
  ASSERT(f.ok, "frame parses");
  ASSERT(strcmp(f.address, longAddr) == 0, "long address roundtrips");
}

int main() {
  printf("=== CLASP Frame Tests (v3 binary) ===\n\n");

  test_set_float_frame();
  test_set_int_frame();
  test_set_negative_int_frame();
  test_set_string_frame();
  test_set_empty_string_frame();
  test_set_bool_frame();
  test_emit_no_value_frame();
  test_emit_with_float_frame();
  test_stream_frame();
  test_subscribe_frame();
  test_unsubscribe_frame();
  test_hello_frame();
  test_hello_null_name_frame();
  test_pong_frame();
  test_buffer_too_small();
  test_pong_buffer_too_small();
  test_exact_fit_buffer();
  test_one_byte_short_buffer();
  test_long_address();

  printf("\n%d/%d tests passed", tests_passed, tests_run);
  if (tests_failed > 0) printf(" (%d FAILED)", tests_failed);
  printf("\n");
  return tests_failed == 0 ? 0 : 1;
}
