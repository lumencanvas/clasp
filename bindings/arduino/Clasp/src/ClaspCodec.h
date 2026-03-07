#ifndef CLASP_CODEC_H
#define CLASP_CODEC_H

#include <stdint.h>
#include <stddef.h>
#include <string.h>

// CLASP v3 binary protocol
//
// TCP transport: 4-byte u32 big-endian length prefix, then CLASP frame
//
// Frame format:
//   Byte 0:    Magic 'S' (0x53)
//   Byte 1:    Flags [QoS:2][ts:1][enc:1][comp:1][version:3]
//              version=1 for binary encoding
//   Byte 2-3:  Payload length (big-endian uint16)
//   Byte 4+:   Payload (starts with message type byte)
//
// Message types:
//   0x01 = HELLO
//   0x02 = WELCOME
//   0x10 = SUBSCRIBE
//   0x11 = UNSUBSCRIBE
//   0x20 = PUBLISH (events, streams, gestures)
//   0x21 = SET
//   0x22 = GET
//   0x23 = SNAPSHOT
//   0x41 = PING
//   0x42 = PONG
//
// Value type codes (used in SET flags and PUBLISH):
//   0x00 = Null
//   0x01 = Bool (1 byte)
//   0x04 = Int32 (4 bytes BE)
//   0x06 = Float32 (4 bytes BE IEEE 754)
//   0x07 = Float64 (8 bytes BE IEEE 754)
//   0x05 = Int64 (8 bytes BE)
//   0x08 = String (u16 len + data)
//   0x09 = Bytes (u16 len + data)

#define CLASP_MAGIC 0x53
#define CLASP_FRAME_HEADER_SIZE 4
#define CLASP_TCP_PREFIX_SIZE 4

// Message type codes (first byte of payload)
#define CLASP_MSG_HELLO       0x01
#define CLASP_MSG_WELCOME     0x02
#define CLASP_MSG_SUBSCRIBE   0x10
#define CLASP_MSG_UNSUBSCRIBE 0x11
#define CLASP_MSG_PUBLISH     0x20
#define CLASP_MSG_SET         0x21
#define CLASP_MSG_GET         0x22
#define CLASP_MSG_SNAPSHOT    0x23
#define CLASP_MSG_PING        0x41
#define CLASP_MSG_PONG        0x42

// Value type codes
#define CLASP_VAL_NULL   0x00
#define CLASP_VAL_BOOL   0x01
#define CLASP_VAL_I32    0x04
#define CLASP_VAL_I64    0x05
#define CLASP_VAL_F32    0x06
#define CLASP_VAL_F64    0x07
#define CLASP_VAL_STRING 0x08
#define CLASP_VAL_BYTES  0x09

// Signal types for PUBLISH flags
#define CLASP_SIG_PARAM   0
#define CLASP_SIG_EVENT   1
#define CLASP_SIG_STREAM  2

// HELLO feature flags bitmask
#define CLASP_FEAT_PARAM     0x80
#define CLASP_FEAT_EVENT     0x40
#define CLASP_FEAT_STREAM    0x20
#define CLASP_FEAT_GESTURE   0x10
#define CLASP_FEAT_TIMELINE  0x08
#define CLASP_FEAT_FEDERATION 0x04

// Protocol version
#define CLASP_PROTOCOL_VERSION 1

// Frame flags: version=1 (binary encoding) in bits 0-2
#define CLASP_FLAGS_BINARY 0x01

// QoS levels in bits 7:6 of flags byte
#define CLASP_QOS_NONE    0x00  // bits 00
#define CLASP_QOS_CONFIRM 0x40  // bits 01
#define CLASP_QOS_GUARANTEED 0x80  // bits 10

// HELLO requires QoS::Confirm
#define CLASP_FLAGS_HELLO (CLASP_FLAGS_BINARY | CLASP_QOS_CONFIRM)  // 0x41

// Backward compat aliases for Clasp.h/Clasp.cpp
#define CLASP_OP_WELCOME  CLASP_MSG_WELCOME
#define CLASP_OP_SET      CLASP_MSG_SET
#define CLASP_OP_PUBLISH  CLASP_MSG_PUBLISH
#define CLASP_OP_SNAPSHOT CLASP_MSG_SNAPSHOT
#define CLASP_OP_PING     CLASP_MSG_PING

struct ClaspValue {
  enum Type : uint8_t { Null = 0, Bool, Int, Float, String, Bytes };
  Type type;
  union {
    bool b;
    int32_t i;
    float f;
  };
  const char* str;
  uint16_t len;

  ClaspValue() : type(Null), i(0), str(nullptr), len(0) {}
};

namespace ClaspCodec {

  // --- Low-level encoding helpers ---

  // Write a u32 big-endian TCP length prefix
  inline size_t writeTcpPrefix(uint8_t* buf, uint32_t frameLen) {
    buf[0] = (frameLen >> 24) & 0xFF;
    buf[1] = (frameLen >> 16) & 0xFF;
    buf[2] = (frameLen >> 8) & 0xFF;
    buf[3] = frameLen & 0xFF;
    return CLASP_TCP_PREFIX_SIZE;
  }

  // Write CLASP frame header (magic + flags + payload length).
  // flags parameter allows setting QoS bits etc.
  inline size_t writeFrameHeader(uint8_t* buf, uint16_t payloadLen,
                                 uint8_t flags) {
    buf[0] = CLASP_MAGIC;
    buf[1] = flags;
    buf[2] = (payloadLen >> 8) & 0xFF;
    buf[3] = payloadLen & 0xFF;
    return CLASP_FRAME_HEADER_SIZE;
  }

  // Write CLASP frame header with default flags (version=1, no QoS).
  inline size_t writeFrameHeader(uint8_t* buf, uint16_t payloadLen) {
    return writeFrameHeader(buf, payloadLen, CLASP_FLAGS_BINARY);
  }

  // Write a length-prefixed string (2-byte big-endian length + data).
  inline size_t writeString(uint8_t* buf, const char* str) {
    uint16_t slen = str ? (uint16_t)strlen(str) : 0;
    buf[0] = (slen >> 8) & 0xFF;
    buf[1] = slen & 0xFF;
    if (slen > 0) memcpy(buf + 2, str, slen);
    return 2 + slen;
  }

  // Write a u16 big-endian
  inline size_t writeU16(uint8_t* buf, uint16_t val) {
    buf[0] = (val >> 8) & 0xFF;
    buf[1] = val & 0xFF;
    return 2;
  }

  // Write a u32 big-endian
  inline size_t writeU32(uint8_t* buf, uint32_t val) {
    buf[0] = (val >> 24) & 0xFF;
    buf[1] = (val >> 16) & 0xFF;
    buf[2] = (val >> 8) & 0xFF;
    buf[3] = val & 0xFF;
    return 4;
  }

  // Write float32 value data (4 bytes BE, no type tag)
  inline size_t writeF32Data(uint8_t* buf, float val) {
    uint32_t bits;
    memcpy(&bits, &val, 4);
    buf[0] = (bits >> 24) & 0xFF;
    buf[1] = (bits >> 16) & 0xFF;
    buf[2] = (bits >> 8) & 0xFF;
    buf[3] = bits & 0xFF;
    return 4;
  }

  // Write int32 value data (4 bytes BE, no type tag)
  inline size_t writeI32Data(uint8_t* buf, int32_t val) {
    buf[0] = (val >> 24) & 0xFF;
    buf[1] = (val >> 16) & 0xFF;
    buf[2] = (val >> 8) & 0xFF;
    buf[3] = val & 0xFF;
    return 4;
  }

  // --- Decoding helpers ---

  // Parse a CLASP frame header. Returns true if valid.
  // Sets msgType to the first byte of payload (message type).
  // payloadLen is the payload length from the frame header.
  inline bool parseFrameHeader(const uint8_t* buf, size_t len,
                               uint16_t& payloadLen) {
    if (len < CLASP_FRAME_HEADER_SIZE) return false;
    if (buf[0] != CLASP_MAGIC) return false;
    // buf[1] is flags; we accept any flags
    payloadLen = ((uint16_t)buf[2] << 8) | buf[3];
    return true;
  }

  // Read a u32 big-endian TCP length prefix
  inline uint32_t readTcpPrefix(const uint8_t* buf) {
    return ((uint32_t)buf[0] << 24) | ((uint32_t)buf[1] << 16) |
           ((uint32_t)buf[2] << 8) | buf[3];
  }

  // Read a length-prefixed string. Sets str to point into buf (zero-copy).
  // Returns bytes consumed, or 0 on error.
  inline size_t readString(const uint8_t* buf, size_t available,
                           const char*& str, uint16_t& slen) {
    if (available < 2) return 0;
    slen = ((uint16_t)buf[0] << 8) | buf[1];
    if (available < 2u + slen) return 0;
    str = (const char*)(buf + 2);
    return 2 + slen;
  }

  // Read a typed value from payload data.
  // vtype is the value type code. Returns bytes consumed.
  inline size_t readValueData(const uint8_t* buf, size_t available,
                              uint8_t vtype, ClaspValue& val) {
    switch (vtype) {
      case CLASP_VAL_NULL:
        val.type = ClaspValue::Null;
        return 0; // no data bytes
      case CLASP_VAL_BOOL:
        if (available < 1) return 0;
        val.type = ClaspValue::Bool;
        val.b = buf[0] != 0;
        return 1;
      case CLASP_VAL_I32:
        if (available < 4) return 0;
        val.type = ClaspValue::Int;
        val.i = ((int32_t)buf[0] << 24) | ((int32_t)buf[1] << 16) |
                ((int32_t)buf[2] << 8) | buf[3];
        return 4;
      case CLASP_VAL_I64:
        // Truncate to int32 on Arduino
        if (available < 8) return 0;
        val.type = ClaspValue::Int;
        val.i = ((int32_t)buf[4] << 24) | ((int32_t)buf[5] << 16) |
                ((int32_t)buf[6] << 8) | buf[7];
        return 8;
      case CLASP_VAL_F32:
        if (available < 4) return 0;
        val.type = ClaspValue::Float;
        {
          uint32_t bits = ((uint32_t)buf[0] << 24) | ((uint32_t)buf[1] << 16) |
                          ((uint32_t)buf[2] << 8) | buf[3];
          memcpy(&val.f, &bits, 4);
        }
        return 4;
      case CLASP_VAL_F64:
        // Read 8-byte double, convert to float on Arduino
        if (available < 8) return 0;
        val.type = ClaspValue::Float;
        {
          // Read the f64 big-endian bytes
          uint64_t bits = 0;
          for (int k = 0; k < 8; k++) {
            bits = (bits << 8) | buf[k];
          }
          double d;
          memcpy(&d, &bits, 8);
          val.f = (float)d;
        }
        return 8;
      case CLASP_VAL_STRING:
        if (available < 2) return 0;
        {
          uint16_t slen = ((uint16_t)buf[0] << 8) | buf[1];
          if (available < 2u + slen) return 0;
          val.type = ClaspValue::String;
          val.str = (const char*)(buf + 2);
          val.len = slen;
          return 2 + slen;
        }
      case CLASP_VAL_BYTES:
        if (available < 2) return 0;
        {
          uint16_t blen = ((uint16_t)buf[0] << 8) | buf[1];
          if (available < 2u + blen) return 0;
          val.type = ClaspValue::Bytes;
          val.str = (const char*)(buf + 2);
          val.len = blen;
          return 2 + blen;
        }
      default:
        return 0;
    }
  }

  // --- Frame builders ---
  // All builders write: TCP prefix + frame header + payload
  // Returns total bytes written (TCP prefix + frame), or 0 if buffer too small.

  // Build a HELLO frame
  // Payload: [0x01][version:u8][features:u8][u16+name][u16+empty_token]
  inline size_t buildHelloFrame(uint8_t* buf, size_t bufSize,
                                const char* name) {
    uint16_t nameLen = name ? (uint16_t)strlen(name) : 0;
    // payload: msg_type(1) + version(1) + features(1) + name(2+len) + token(2)
    uint16_t payloadLen = 1 + 1 + 1 + 2 + nameLen + 2;
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    uint32_t frameLen = CLASP_FRAME_HEADER_SIZE + payloadLen;
    pos += writeTcpPrefix(buf + pos, frameLen);
    pos += writeFrameHeader(buf + pos, payloadLen, CLASP_FLAGS_HELLO);

    // Message type
    buf[pos++] = CLASP_MSG_HELLO;
    // Protocol version
    buf[pos++] = CLASP_PROTOCOL_VERSION;
    // Feature flags: param + event + stream
    buf[pos++] = CLASP_FEAT_PARAM | CLASP_FEAT_EVENT | CLASP_FEAT_STREAM;
    // Name
    pos += writeString(buf + pos, name);
    // Token (empty)
    buf[pos++] = 0;
    buf[pos++] = 0;

    return pos;
  }

  // Build a SET frame
  // Payload: [0x21][flags: 0|0|0|0|vtype_4bits][u16+address][value_data]
  inline size_t buildSetFrame(uint8_t* buf, size_t bufSize,
                              const char* address, float value) {
    uint16_t addrLen = (uint16_t)strlen(address);
    uint16_t payloadLen = 1 + 1 + 2 + addrLen + 4; // msg + flags + addr + f32
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    pos += writeTcpPrefix(buf + pos, CLASP_FRAME_HEADER_SIZE + payloadLen);
    pos += writeFrameHeader(buf + pos, payloadLen);
    buf[pos++] = CLASP_MSG_SET;
    buf[pos++] = CLASP_VAL_F32; // flags: vtype in lower 4 bits
    pos += writeString(buf + pos, address);
    pos += writeF32Data(buf + pos, value);
    return pos;
  }

  inline size_t buildSetFrame(uint8_t* buf, size_t bufSize,
                              const char* address, int32_t value) {
    uint16_t addrLen = (uint16_t)strlen(address);
    uint16_t payloadLen = 1 + 1 + 2 + addrLen + 4;
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    pos += writeTcpPrefix(buf + pos, CLASP_FRAME_HEADER_SIZE + payloadLen);
    pos += writeFrameHeader(buf + pos, payloadLen);
    buf[pos++] = CLASP_MSG_SET;
    buf[pos++] = CLASP_VAL_I32;
    pos += writeString(buf + pos, address);
    pos += writeI32Data(buf + pos, value);
    return pos;
  }

  inline size_t buildSetFrame(uint8_t* buf, size_t bufSize,
                              const char* address, bool value) {
    uint16_t addrLen = (uint16_t)strlen(address);
    uint16_t payloadLen = 1 + 1 + 2 + addrLen + 1;
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    pos += writeTcpPrefix(buf + pos, CLASP_FRAME_HEADER_SIZE + payloadLen);
    pos += writeFrameHeader(buf + pos, payloadLen);
    buf[pos++] = CLASP_MSG_SET;
    buf[pos++] = CLASP_VAL_BOOL;
    pos += writeString(buf + pos, address);
    buf[pos++] = value ? 1 : 0;
    return pos;
  }

  inline size_t buildSetFrame(uint8_t* buf, size_t bufSize,
                              const char* address, const char* value) {
    uint16_t addrLen = (uint16_t)strlen(address);
    uint16_t valLen = (uint16_t)strlen(value);
    uint16_t payloadLen = 1 + 1 + 2 + addrLen + 2 + valLen;
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    pos += writeTcpPrefix(buf + pos, CLASP_FRAME_HEADER_SIZE + payloadLen);
    pos += writeFrameHeader(buf + pos, payloadLen);
    buf[pos++] = CLASP_MSG_SET;
    buf[pos++] = CLASP_VAL_STRING;
    pos += writeString(buf + pos, address);
    pos += writeString(buf + pos, value);
    return pos;
  }

  // Build a PUBLISH frame (used for EMIT and STREAM)
  // Payload: [0x20][flags: sig_type(7:5)|0|0|phase(2:0)][u16+address][value_indicator][opt value]
  inline size_t buildPublishFrame(uint8_t* buf, size_t bufSize,
                                  const char* address, uint8_t sigType) {
    // No value
    uint16_t addrLen = (uint16_t)strlen(address);
    uint16_t payloadLen = 1 + 1 + 2 + addrLen + 1; // msg + flags + addr + value_indicator(0)
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    pos += writeTcpPrefix(buf + pos, CLASP_FRAME_HEADER_SIZE + payloadLen);
    pos += writeFrameHeader(buf + pos, payloadLen);
    buf[pos++] = CLASP_MSG_PUBLISH;
    buf[pos++] = (sigType & 0x07) << 5; // sig_type in upper 3 bits
    pos += writeString(buf + pos, address);
    buf[pos++] = 0; // no value
    return pos;
  }

  inline size_t buildPublishFrameFloat(uint8_t* buf, size_t bufSize,
                                       const char* address, uint8_t sigType,
                                       float value) {
    uint16_t addrLen = (uint16_t)strlen(address);
    // msg + flags + addr + value_indicator(1) + vtype + f32_data
    uint16_t payloadLen = 1 + 1 + 2 + addrLen + 1 + 1 + 4;
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    pos += writeTcpPrefix(buf + pos, CLASP_FRAME_HEADER_SIZE + payloadLen);
    pos += writeFrameHeader(buf + pos, payloadLen);
    buf[pos++] = CLASP_MSG_PUBLISH;
    buf[pos++] = (sigType & 0x07) << 5;
    pos += writeString(buf + pos, address);
    buf[pos++] = 1; // has value
    buf[pos++] = CLASP_VAL_F32;
    pos += writeF32Data(buf + pos, value);
    return pos;
  }

  // Convenience wrappers
  inline size_t buildEmitFrame(uint8_t* buf, size_t bufSize,
                               const char* address) {
    return buildPublishFrame(buf, bufSize, address, CLASP_SIG_EVENT);
  }

  inline size_t buildEmitFrame(uint8_t* buf, size_t bufSize,
                               const char* address, float value) {
    return buildPublishFrameFloat(buf, bufSize, address, CLASP_SIG_EVENT, value);
  }

  inline size_t buildStreamFrame(uint8_t* buf, size_t bufSize,
                                 const char* address, float value) {
    return buildPublishFrameFloat(buf, bufSize, address, CLASP_SIG_STREAM, value);
  }

  // Build a SUBSCRIBE frame
  // Payload: [0x10][u32 sub_id][u16+pattern][type_mask:u8][options:u8(0)]
  inline size_t buildSubscribeFrame(uint8_t* buf, size_t bufSize,
                                    const char* pattern, uint32_t subId = 0) {
    uint16_t patLen = (uint16_t)strlen(pattern);
    uint16_t payloadLen = 1 + 4 + 2 + patLen + 1 + 1; // msg + id + pattern + type_mask + options
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    pos += writeTcpPrefix(buf + pos, CLASP_FRAME_HEADER_SIZE + payloadLen);
    pos += writeFrameHeader(buf + pos, payloadLen);
    buf[pos++] = CLASP_MSG_SUBSCRIBE;
    pos += writeU32(buf + pos, subId);
    pos += writeString(buf + pos, pattern);
    buf[pos++] = 0xFF; // all signal types
    buf[pos++] = 0x00; // no options
    return pos;
  }

  // Build an UNSUBSCRIBE frame
  // Payload: [0x11][u32 sub_id]
  inline size_t buildUnsubscribeFrame(uint8_t* buf, size_t bufSize,
                                      uint32_t subId) {
    uint16_t payloadLen = 1 + 4;
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    pos += writeTcpPrefix(buf + pos, CLASP_FRAME_HEADER_SIZE + payloadLen);
    pos += writeFrameHeader(buf + pos, payloadLen);
    buf[pos++] = CLASP_MSG_UNSUBSCRIBE;
    pos += writeU32(buf + pos, subId);
    return pos;
  }

  // Build a PONG frame
  // Payload: [0x42]
  inline size_t buildPongFrame(uint8_t* buf, size_t bufSize) {
    uint16_t payloadLen = 1;
    size_t totalSize = CLASP_TCP_PREFIX_SIZE + CLASP_FRAME_HEADER_SIZE + payloadLen;
    if (totalSize > bufSize) return 0;

    size_t pos = 0;
    pos += writeTcpPrefix(buf + pos, CLASP_FRAME_HEADER_SIZE + payloadLen);
    pos += writeFrameHeader(buf + pos, payloadLen);
    buf[pos++] = CLASP_MSG_PONG;
    return pos;
  }

  // --- Wildcard pattern matching ---

  // Match an address against a pattern with * and ** wildcards.
  // '*' matches a single path segment, '**' matches any number of segments.
  bool matchPattern(const char* pattern, const char* address);

} // namespace ClaspCodec

#endif // CLASP_CODEC_H
