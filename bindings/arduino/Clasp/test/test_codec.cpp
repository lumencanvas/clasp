/**
 * CLASP Codec Tests (v3 binary protocol)
 *
 * Tests for the binary codec encoding/decoding and pattern matching.
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

// ============================================================================
// TCP prefix and frame header
// ============================================================================

void test_tcp_prefix_roundtrip() {
  printf("test_tcp_prefix_roundtrip\n");
  uint8_t buf[4];
  ClaspCodec::writeTcpPrefix(buf, 42);
  ASSERT(ClaspCodec::readTcpPrefix(buf) == 42, "tcp prefix 42 roundtrips");

  ClaspCodec::writeTcpPrefix(buf, 0);
  ASSERT(ClaspCodec::readTcpPrefix(buf) == 0, "tcp prefix 0 roundtrips");

  ClaspCodec::writeTcpPrefix(buf, 65535);
  ASSERT(ClaspCodec::readTcpPrefix(buf) == 65535, "tcp prefix 65535 roundtrips");

  ClaspCodec::writeTcpPrefix(buf, 0x00010000);
  ASSERT(ClaspCodec::readTcpPrefix(buf) == 0x00010000, "tcp prefix > 16 bits roundtrips");
}

void test_frame_header_roundtrip() {
  printf("test_frame_header_roundtrip\n");
  uint8_t buf[4];
  ClaspCodec::writeFrameHeader(buf, 100);

  ASSERT(buf[0] == CLASP_MAGIC, "magic byte is 0x53");
  ASSERT(buf[1] == CLASP_FLAGS_BINARY, "flags byte has version=1");

  uint16_t payloadLen;
  ASSERT(ClaspCodec::parseFrameHeader(buf, 4, payloadLen), "parseFrameHeader returns true");
  ASSERT(payloadLen == 100, "payload length matches");
}

void test_frame_header_max_payload() {
  printf("test_frame_header_max_payload\n");
  uint8_t buf[4];
  ClaspCodec::writeFrameHeader(buf, 65535);
  uint16_t payloadLen;
  ClaspCodec::parseFrameHeader(buf, 4, payloadLen);
  ASSERT(payloadLen == 65535, "max uint16 payload length roundtrips");

  ClaspCodec::writeFrameHeader(buf, 0);
  ClaspCodec::parseFrameHeader(buf, 4, payloadLen);
  ASSERT(payloadLen == 0, "zero payload length roundtrips");
}

void test_frame_header_magic_check() {
  printf("test_frame_header_magic_check\n");
  uint8_t buf[4] = { 0x00, CLASP_FLAGS_BINARY, 0x00, 0x00 };
  uint16_t payloadLen;
  ASSERT(!ClaspCodec::parseFrameHeader(buf, 4, payloadLen), "rejects bad magic 0x00");
  buf[0] = 0x52;
  ASSERT(!ClaspCodec::parseFrameHeader(buf, 4, payloadLen), "rejects bad magic 0x52");
  buf[0] = 0xFF;
  ASSERT(!ClaspCodec::parseFrameHeader(buf, 4, payloadLen), "rejects bad magic 0xFF");
}

void test_frame_header_too_short() {
  printf("test_frame_header_too_short\n");
  uint8_t buf[4] = { CLASP_MAGIC, CLASP_FLAGS_BINARY, 0x00, 0x05 };
  uint16_t payloadLen;
  ASSERT(!ClaspCodec::parseFrameHeader(buf, 0, payloadLen), "rejects 0 bytes");
  ASSERT(!ClaspCodec::parseFrameHeader(buf, 1, payloadLen), "rejects 1 byte");
  ASSERT(!ClaspCodec::parseFrameHeader(buf, 3, payloadLen), "rejects 3 bytes");
  ASSERT(ClaspCodec::parseFrameHeader(buf, 4, payloadLen), "accepts 4 bytes");
}

// ============================================================================
// Value encode/decode (readValueData — no type tag prefix, type is separate)
// ============================================================================

void test_value_f32() {
  printf("test_value_f32\n");
  uint8_t buf[16];
  float original = 3.14f;
  size_t written = ClaspCodec::writeF32Data(buf, original);
  ASSERT(written == 4, "f32 writes 4 bytes");

  ClaspValue val;
  size_t rd = ClaspCodec::readValueData(buf, written, CLASP_VAL_F32, val);
  ASSERT(rd == 4, "f32 reads 4 bytes");
  ASSERT(val.type == ClaspValue::Float, "type is Float");
  ASSERT(fabsf(val.f - original) < 0.0001f, "f32 value matches");
}

void test_value_f32_special() {
  printf("test_value_f32_special\n");
  uint8_t buf[16];
  ClaspValue val;

  ClaspCodec::writeF32Data(buf, 0.0f);
  ClaspCodec::readValueData(buf, 4, CLASP_VAL_F32, val);
  ASSERT(val.f == 0.0f, "zero float");

  ClaspCodec::writeF32Data(buf, -1.5f);
  ClaspCodec::readValueData(buf, 4, CLASP_VAL_F32, val);
  ASSERT(fabsf(val.f - (-1.5f)) < 0.0001f, "negative float");

  ClaspCodec::writeF32Data(buf, 1.0f);
  ClaspCodec::readValueData(buf, 4, CLASP_VAL_F32, val);
  ASSERT(val.f == 1.0f, "1.0 exact");
}

void test_value_f64_decode() {
  printf("test_value_f64_decode\n");
  // Manually encode a f64 big-endian (1.0 as double = 0x3FF0000000000000)
  uint8_t buf[8] = { 0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 };
  ClaspValue val;
  size_t rd = ClaspCodec::readValueData(buf, 8, CLASP_VAL_F64, val);
  ASSERT(rd == 8, "f64 reads 8 bytes");
  ASSERT(val.type == ClaspValue::Float, "f64 decoded as Float");
  ASSERT(fabsf(val.f - 1.0f) < 0.0001f, "f64 1.0 decoded correctly");
}

void test_value_i32() {
  printf("test_value_i32\n");
  uint8_t buf[16];
  int32_t original = -12345;
  size_t written = ClaspCodec::writeI32Data(buf, original);
  ASSERT(written == 4, "i32 writes 4 bytes");

  ClaspValue val;
  size_t rd = ClaspCodec::readValueData(buf, written, CLASP_VAL_I32, val);
  ASSERT(rd == 4, "i32 reads 4 bytes");
  ASSERT(val.type == ClaspValue::Int, "type is Int");
  ASSERT(val.i == original, "i32 value matches");
}

void test_value_i32_edge_cases() {
  printf("test_value_i32_edge_cases\n");
  uint8_t buf[16];
  ClaspValue val;

  int32_t cases[] = { 0, 1, -1, 127, -128, 255, 32767, -32768,
                      2147483647, -2147483647 - 1 };
  for (int32_t c : cases) {
    ClaspCodec::writeI32Data(buf, c);
    ClaspCodec::readValueData(buf, 4, CLASP_VAL_I32, val);
    ASSERT(val.i == c, "i32 edge case roundtrips");
  }
}

void test_value_i64_decode() {
  printf("test_value_i64_decode\n");
  // i64 42 big-endian
  uint8_t buf[8] = { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 42 };
  ClaspValue val;
  size_t rd = ClaspCodec::readValueData(buf, 8, CLASP_VAL_I64, val);
  ASSERT(rd == 8, "i64 reads 8 bytes");
  ASSERT(val.type == ClaspValue::Int, "i64 decoded as Int");
  ASSERT(val.i == 42, "i64 42 decoded correctly (truncated to i32)");
}

void test_value_bool() {
  printf("test_value_bool\n");
  ClaspValue val;

  uint8_t buf_true[1] = { 1 };
  size_t rd = ClaspCodec::readValueData(buf_true, 1, CLASP_VAL_BOOL, val);
  ASSERT(rd == 1, "bool reads 1 byte");
  ASSERT(val.type == ClaspValue::Bool, "type is Bool");
  ASSERT(val.b == true, "bool true");

  uint8_t buf_false[1] = { 0 };
  ClaspCodec::readValueData(buf_false, 1, CLASP_VAL_BOOL, val);
  ASSERT(val.b == false, "bool false");
}

void test_value_string() {
  printf("test_value_string\n");
  // String value data: u16 len + data (no type tag)
  const char* original = "hello";
  uint8_t buf[16];
  buf[0] = 0x00;
  buf[1] = 5;
  memcpy(buf + 2, original, 5);

  ClaspValue val;
  size_t rd = ClaspCodec::readValueData(buf, 7, CLASP_VAL_STRING, val);
  ASSERT(rd == 7, "string reads 7 bytes");
  ASSERT(val.type == ClaspValue::String, "type is String");
  ASSERT(val.len == 5, "string len 5");
  ASSERT(memcmp(val.str, "hello", 5) == 0, "string content matches");
}

void test_value_null() {
  printf("test_value_null\n");
  ClaspValue val;
  val.type = ClaspValue::Float;
  size_t rd = ClaspCodec::readValueData(nullptr, 0, CLASP_VAL_NULL, val);
  ASSERT(rd == 0, "null reads 0 bytes");
  ASSERT(val.type == ClaspValue::Null, "type is Null");
}

void test_string_roundtrip() {
  printf("test_string_roundtrip\n");
  uint8_t buf[128];
  const char* original = "/sensors/room/temperature";
  size_t written = ClaspCodec::writeString(buf, original);
  ASSERT(written == 2 + strlen(original), "string length correct");

  const char* str;
  uint16_t slen;
  size_t rd = ClaspCodec::readString(buf, written, str, slen);
  ASSERT(rd == written, "read same bytes");
  ASSERT(slen == strlen(original), "string length matches");
  ASSERT(memcmp(str, original, slen) == 0, "string content matches");
}

// ============================================================================
// Truncation / malformed data
// ============================================================================

void test_read_truncated_f32() {
  printf("test_read_truncated_f32\n");
  ClaspValue val;
  uint8_t buf[2] = { 0x41, 0x00 };
  ASSERT(ClaspCodec::readValueData(buf, 2, CLASP_VAL_F32, val) == 0, "truncated f32 returns 0");
}

void test_read_truncated_i32() {
  printf("test_read_truncated_i32\n");
  ClaspValue val;
  uint8_t buf[2] = { 0x00, 0x01 };
  ASSERT(ClaspCodec::readValueData(buf, 2, CLASP_VAL_I32, val) == 0, "truncated i32 returns 0");
}

void test_read_truncated_bool() {
  printf("test_read_truncated_bool\n");
  ClaspValue val;
  ASSERT(ClaspCodec::readValueData(nullptr, 0, CLASP_VAL_BOOL, val) == 0, "truncated bool returns 0");
}

void test_read_truncated_string() {
  printf("test_read_truncated_string\n");
  ClaspValue val;
  uint8_t buf[4] = { 0x00, 0x0A, 'h', 'e' };
  ASSERT(ClaspCodec::readValueData(buf, 4, CLASP_VAL_STRING, val) == 0, "truncated string returns 0");
}

void test_read_truncated_f64() {
  printf("test_read_truncated_f64\n");
  ClaspValue val;
  uint8_t buf[4] = { 0x3F, 0xF0, 0x00, 0x00 };
  ASSERT(ClaspCodec::readValueData(buf, 4, CLASP_VAL_F64, val) == 0, "truncated f64 returns 0");
}

void test_read_truncated_i64() {
  printf("test_read_truncated_i64\n");
  ClaspValue val;
  uint8_t buf[4] = { 0x00, 0x00, 0x00, 0x00 };
  ASSERT(ClaspCodec::readValueData(buf, 4, CLASP_VAL_I64, val) == 0, "truncated i64 returns 0");
}

void test_read_unknown_vtype() {
  printf("test_read_unknown_vtype\n");
  ClaspValue val;
  uint8_t buf[1] = { 0x00 };
  ASSERT(ClaspCodec::readValueData(buf, 1, 0xFF, val) == 0, "unknown vtype 0xFF returns 0");
  ASSERT(ClaspCodec::readValueData(buf, 1, 0x0A, val) == 0, "unknown vtype 0x0A returns 0");
  ASSERT(ClaspCodec::readValueData(buf, 1, 0x0B, val) == 0, "unknown vtype 0x0B returns 0");
}

void test_read_zero_available() {
  printf("test_read_zero_available\n");
  ClaspValue val;
  uint8_t buf[1] = { 0x00 };
  // readString with 0 or 1 available
  const char* str;
  uint16_t slen;
  ASSERT(ClaspCodec::readString(buf, 0, str, slen) == 0, "readString 0 available");
  ASSERT(ClaspCodec::readString(buf, 1, str, slen) == 0, "readString 1 available");
}

// ============================================================================
// Pattern matching (matchPattern)
// ============================================================================

void test_match_exact() {
  printf("test_match_exact\n");
  ASSERT(ClaspCodec::matchPattern("/a/b/c", "/a/b/c"), "exact match");
  ASSERT(ClaspCodec::matchPattern("/sensors/temp", "/sensors/temp"), "exact match sensors/temp");
  ASSERT(ClaspCodec::matchPattern("/", "/"), "root match");
}

void test_match_exact_nonmatch() {
  printf("test_match_exact_nonmatch\n");
  ASSERT(!ClaspCodec::matchPattern("/a/b/c", "/a/b/d"), "different last segment");
  ASSERT(!ClaspCodec::matchPattern("/a/b/c", "/a/b"), "pattern longer");
  ASSERT(!ClaspCodec::matchPattern("/a/b", "/a/b/c"), "address longer");
  ASSERT(!ClaspCodec::matchPattern("/a", "/b"), "completely different");
}

void test_match_single_wildcard() {
  printf("test_match_single_wildcard\n");
  ASSERT(ClaspCodec::matchPattern("/a/*/c", "/a/b/c"), "* matches single segment");
  ASSERT(ClaspCodec::matchPattern("/a/*/c", "/a/xyz/c"), "* matches multi-char segment");
  ASSERT(ClaspCodec::matchPattern("/*", "/anything"), "/* matches any single segment");
  ASSERT(ClaspCodec::matchPattern("/a/*", "/a/b"), "trailing * matches");
}

void test_match_single_wildcard_nonmatch() {
  printf("test_match_single_wildcard_nonmatch\n");
  ASSERT(!ClaspCodec::matchPattern("/a/*/c", "/a/b/d/c"), "* does not cross segments");
  ASSERT(!ClaspCodec::matchPattern("/a/*/c", "/a/c"), "* requires one segment");
  ASSERT(!ClaspCodec::matchPattern("/*", "/a/b"), "/* doesn't match two segments");
}

void test_match_globstar_trailing() {
  printf("test_match_globstar_trailing\n");
  ASSERT(ClaspCodec::matchPattern("/a/**", "/a/b"), "/** one level");
  ASSERT(ClaspCodec::matchPattern("/a/**", "/a/b/c"), "/** two levels");
  ASSERT(ClaspCodec::matchPattern("/a/**", "/a/b/c/d/e"), "/** many levels");
}

void test_match_globstar_zero() {
  printf("test_match_globstar_zero\n");
  ASSERT(ClaspCodec::matchPattern("/a/**", "/a"), "/** zero additional segments");
}

void test_match_globstar_mid() {
  printf("test_match_globstar_mid\n");
  ASSERT(ClaspCodec::matchPattern("/a/**/c", "/a/c"), "/**/c zero between");
  ASSERT(ClaspCodec::matchPattern("/a/**/c", "/a/b/c"), "/**/c one between");
  ASSERT(ClaspCodec::matchPattern("/a/**/c", "/a/b/d/c"), "/**/c two between");
  ASSERT(ClaspCodec::matchPattern("/a/**/c", "/a/x/y/z/c"), "/**/c many between");
}

void test_match_globstar_mid_nonmatch() {
  printf("test_match_globstar_mid_nonmatch\n");
  ASSERT(!ClaspCodec::matchPattern("/a/**/c", "/a/b/d"), "/**/c requires c at end");
  ASSERT(!ClaspCodec::matchPattern("/a/**/c", "/b/c"), "/**/c requires a at start");
}

void test_match_root_globstar() {
  printf("test_match_root_globstar\n");
  ASSERT(ClaspCodec::matchPattern("/**", "/a"), "/** any single");
  ASSERT(ClaspCodec::matchPattern("/**", "/a/b/c"), "/** any deep");
  ASSERT(ClaspCodec::matchPattern("/**", "/"), "/** matches root");
}

void test_match_multiple_wildcards() {
  printf("test_match_multiple_wildcards\n");
  ASSERT(ClaspCodec::matchPattern("/*/b/*", "/a/b/c"), "two * wildcards");
  ASSERT(!ClaspCodec::matchPattern("/*/b/*", "/a/x/c"), "two *, middle mismatch");
  ASSERT(ClaspCodec::matchPattern("/*/*", "/a/b"), "consecutive *");
  ASSERT(!ClaspCodec::matchPattern("/*/*", "/a/b/c"), "two *, too many segments");
}

void test_match_empty_strings() {
  printf("test_match_empty_strings\n");
  ASSERT(ClaspCodec::matchPattern("", ""), "both empty");
  ASSERT(!ClaspCodec::matchPattern("", "/a"), "empty pattern non-empty addr");
  ASSERT(!ClaspCodec::matchPattern("/a", ""), "non-empty pattern empty addr");
}

int main() {
  printf("=== CLASP Codec Tests (v3 binary) ===\n\n");

  // TCP prefix and frame header
  test_tcp_prefix_roundtrip();
  test_frame_header_roundtrip();
  test_frame_header_max_payload();
  test_frame_header_magic_check();
  test_frame_header_too_short();

  // Value encode/decode
  test_value_f32();
  test_value_f32_special();
  test_value_f64_decode();
  test_value_i32();
  test_value_i32_edge_cases();
  test_value_i64_decode();
  test_value_bool();
  test_value_string();
  test_value_null();
  test_string_roundtrip();

  // Truncation / malformed data
  test_read_truncated_f32();
  test_read_truncated_i32();
  test_read_truncated_bool();
  test_read_truncated_string();
  test_read_truncated_f64();
  test_read_truncated_i64();
  test_read_unknown_vtype();
  test_read_zero_available();

  // Pattern matching
  test_match_exact();
  test_match_exact_nonmatch();
  test_match_single_wildcard();
  test_match_single_wildcard_nonmatch();
  test_match_globstar_trailing();
  test_match_globstar_zero();
  test_match_globstar_mid();
  test_match_globstar_mid_nonmatch();
  test_match_root_globstar();
  test_match_multiple_wildcards();
  test_match_empty_strings();

  printf("\n%d/%d tests passed", tests_passed, tests_run);
  if (tests_failed > 0) printf(" (%d FAILED)", tests_failed);
  printf("\n");
  return tests_failed == 0 ? 0 : 1;
}
