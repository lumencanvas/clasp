#include "Clasp.h"

#ifndef ARDUINO
// Desktop shims
static unsigned long _millis_counter = 0;
static unsigned long _millis_auto_increment = 0;
unsigned long millis() {
  unsigned long val = _millis_counter;
  _millis_counter += _millis_auto_increment;
  return val;
}
// Advance the clock by ms
void _clasp_advance_millis(unsigned long ms) { _millis_counter += ms; }
// Set auto-increment: each millis() call advances the clock by this amount
void _clasp_set_millis_increment(unsigned long inc) { _millis_auto_increment = inc; }
// Reset millis to zero with no auto-increment
void _clasp_reset_millis() { _millis_counter = 0; _millis_auto_increment = 0; }
#endif

ClaspClient::ClaspClient(Client& client)
  : _client(&client)
  , _connected(false)
  , _welcomed(false)
  , _rxPos(0)
  , _rxState(WAIT_TCP_PREFIX)
  , _rxFrameLen(0)
  , _nextSubId(1)
  , _lastActivity(0)
{
  memset(_subs, 0, sizeof(_subs));
}

bool ClaspClient::connect(const char* host, uint16_t port, const char* name) {
  if (!_client->connect(host, port)) {
    return false;
  }

  _connected = true;
  _welcomed = false;
  _rxPos = 0;
  _rxState = WAIT_TCP_PREFIX;
  _lastActivity = millis();

  // Send HELLO
  size_t frameLen = ClaspCodec::buildHelloFrame(
    _txBuf, sizeof(_txBuf), name ? name : "arduino"
  );
  if (frameLen == 0 || !sendFrame(_txBuf, frameLen)) {
    disconnect();
    return false;
  }

  // Wait for WELCOME (up to 5 seconds)
  unsigned long start = millis();
  while (!_welcomed && (millis() - start) < 5000) {
    if (!_client->connected()) {
      _connected = false;
      return false;
    }
    loop();
  }

  if (!_welcomed) {
    disconnect();
    return false;
  }

  return true;
}

bool ClaspClient::connected() {
  return _connected && _client->connected();
}

void ClaspClient::disconnect() {
  if (_client->connected()) {
    _client->stop();
  }
  _connected = false;
  _welcomed = false;
  _rxPos = 0;
  _rxState = WAIT_TCP_PREFIX;
}

bool ClaspClient::set(const char* address, float value) {
  if (!connected()) return false;
  size_t len = ClaspCodec::buildSetFrame(_txBuf, sizeof(_txBuf), address, value);
  return len > 0 && sendFrame(_txBuf, len);
}

bool ClaspClient::set(const char* address, int32_t value) {
  if (!connected()) return false;
  size_t len = ClaspCodec::buildSetFrame(_txBuf, sizeof(_txBuf), address, value);
  return len > 0 && sendFrame(_txBuf, len);
}

bool ClaspClient::set(const char* address, const char* value) {
  if (!connected()) return false;
  size_t len = ClaspCodec::buildSetFrame(_txBuf, sizeof(_txBuf), address, value);
  return len > 0 && sendFrame(_txBuf, len);
}

bool ClaspClient::set(const char* address, bool value) {
  if (!connected()) return false;
  size_t len = ClaspCodec::buildSetFrame(_txBuf, sizeof(_txBuf), address, value);
  return len > 0 && sendFrame(_txBuf, len);
}

bool ClaspClient::emit(const char* address) {
  if (!connected()) return false;
  size_t len = ClaspCodec::buildEmitFrame(_txBuf, sizeof(_txBuf), address);
  return len > 0 && sendFrame(_txBuf, len);
}

bool ClaspClient::emit(const char* address, float value) {
  if (!connected()) return false;
  size_t len = ClaspCodec::buildEmitFrame(_txBuf, sizeof(_txBuf), address, value);
  return len > 0 && sendFrame(_txBuf, len);
}

bool ClaspClient::stream(const char* address, float value) {
  if (!connected()) return false;
  size_t len = ClaspCodec::buildStreamFrame(_txBuf, sizeof(_txBuf), address, value);
  return len > 0 && sendFrame(_txBuf, len);
}

bool ClaspClient::subscribe(const char* pattern, MessageCallback callback) {
  if (!connected()) return false;

  // Find a free slot
  int slot = -1;
  for (int i = 0; i < CLASP_MAX_SUBSCRIPTIONS; i++) {
    if (!_subs[i].active) {
      slot = i;
      break;
    }
  }
  if (slot < 0) return false; // No free slots

  uint32_t subId = _nextSubId++;

  // Send SUBSCRIBE frame
  size_t len = ClaspCodec::buildSubscribeFrame(_txBuf, sizeof(_txBuf), pattern, subId);
  if (len == 0 || !sendFrame(_txBuf, len)) return false;

  // Store subscription
  strncpy(_subs[slot].pattern, pattern, sizeof(_subs[slot].pattern) - 1);
  _subs[slot].pattern[sizeof(_subs[slot].pattern) - 1] = '\0';
  _subs[slot].callback = callback;
  _subs[slot].subId = subId;
  _subs[slot].active = true;

  return true;
}

bool ClaspClient::unsubscribe(const char* pattern) {
  if (!connected()) return false;

  // Find and remove from local subscriptions
  for (int i = 0; i < CLASP_MAX_SUBSCRIPTIONS; i++) {
    if (_subs[i].active && strcmp(_subs[i].pattern, pattern) == 0) {
      // Send UNSUBSCRIBE frame with the subscription ID
      size_t len = ClaspCodec::buildUnsubscribeFrame(_txBuf, sizeof(_txBuf), _subs[i].subId);
      if (len == 0 || !sendFrame(_txBuf, len)) return false;
      _subs[i].active = false;
      break;
    }
  }

  return true;
}

bool ClaspClient::loop() {
  if (!_client->connected()) {
    _connected = false;
    return false;
  }

  // Read available bytes using TCP length-prefixed framing
  while (_client->available() > 0) {
    if (_rxState == WAIT_TCP_PREFIX) {
      // Need 4 bytes for TCP length prefix
      int toRead = CLASP_TCP_PREFIX_SIZE - _rxPos;
      if (toRead > _client->available()) toRead = _client->available();
      int got = _client->read(_rxBuf + _rxPos, toRead);
      if (got <= 0) break;
      _rxPos += got;

      if (_rxPos >= CLASP_TCP_PREFIX_SIZE) {
        _rxFrameLen = ClaspCodec::readTcpPrefix(_rxBuf);

        if (_rxFrameLen == 0 || _rxFrameLen > sizeof(_rxBuf) - CLASP_TCP_PREFIX_SIZE) {
          // Invalid or too large: skip by discarding bytes
          uint32_t remaining = _rxFrameLen;
          while (remaining > 0 && _client->available() > 0) {
            uint8_t discard[64];
            int chunk = remaining > 64 ? 64 : (int)remaining;
            if (chunk > _client->available()) chunk = _client->available();
            int got2 = _client->read(discard, chunk);
            if (got2 <= 0) break;
            remaining -= got2;
          }
          _rxPos = 0;
          continue;
        }

        // We'll reuse _rxBuf starting at offset 0 for the frame data
        _rxPos = 0;
        _rxState = WAIT_FRAME;
      }
    }

    if (_rxState == WAIT_FRAME) {
      int toRead = (int)_rxFrameLen - _rxPos;
      if (toRead > _client->available()) toRead = _client->available();
      if (toRead > 0) {
        int got = _client->read(_rxBuf + _rxPos, toRead);
        if (got <= 0) break;
        _rxPos += got;
      }

      if (_rxPos >= _rxFrameLen) {
        // We have a complete frame: [magic][flags][u16 payload_len][payload...]
        uint16_t payloadLen = 0;
        if (ClaspCodec::parseFrameHeader(_rxBuf, _rxPos, payloadLen)) {
          if (payloadLen > 0 && CLASP_FRAME_HEADER_SIZE + payloadLen <= _rxPos) {
            processMessage(_rxBuf + CLASP_FRAME_HEADER_SIZE, payloadLen);
          } else if (payloadLen == 0) {
            // Empty payload frame (shouldn't happen in v3 but handle gracefully)
          }
        }
        _rxPos = 0;
        _rxState = WAIT_TCP_PREFIX;
      }
    }
  }

  // Keepalive
#if CLASP_KEEPALIVE_MS > 0
  if (_welcomed && (millis() - _lastActivity) >= CLASP_KEEPALIVE_MS) {
    size_t len = ClaspCodec::buildPongFrame(_txBuf, sizeof(_txBuf));
    if (len > 0) sendFrame(_txBuf, len);
    _lastActivity = millis();
  }
#endif

  return _connected;
}

bool ClaspClient::sendFrame(const uint8_t* buf, size_t len) {
  if (!_client->connected()) return false;
  // buf already contains TCP prefix + frame, write it all
  size_t written = _client->write(buf, len);
  if (written == len) {
    _lastActivity = millis();
    return true;
  }
  return false;
}

void ClaspClient::processMessage(const uint8_t* payload, uint16_t len) {
  _lastActivity = millis();

  if (len < 1) return;
  uint8_t msgType = payload[0];

  switch (msgType) {
    case CLASP_MSG_WELCOME:
      _welcomed = true;
      break;

    case CLASP_MSG_PING: {
      size_t pongLen = ClaspCodec::buildPongFrame(_txBuf, sizeof(_txBuf));
      if (pongLen > 0) sendFrame(_txBuf, pongLen);
      break;
    }

    case CLASP_MSG_SET: {
      // SET: [0x21][flags][u16+address][value_data]
      if (len < 4) break; // msg_type + flags + at least 2 for addr len
      uint8_t flags = payload[1];
      uint8_t vtype = flags & 0x0F;

      const uint8_t* p = payload + 2;
      size_t remaining = len - 2;

      const char* addr = nullptr;
      uint16_t addrLen = 0;
      size_t consumed = ClaspCodec::readString(p, remaining, addr, addrLen);
      if (consumed == 0) break;
      p += consumed;
      remaining -= consumed;

      ClaspValue val;
      if (vtype != CLASP_VAL_NULL) {
        ClaspCodec::readValueData(p, remaining, vtype, val);
      }

      dispatchMessage(msgType, addr, addrLen, val);
      break;
    }

    case CLASP_MSG_PUBLISH: {
      // PUBLISH: [0x20][flags][u16+address][value_indicator][opt vtype + value_data]
      if (len < 4) break;
      // uint8_t flags = payload[1]; // sig_type, has_ts, has_id, phase

      const uint8_t* p = payload + 2;
      size_t remaining = len - 2;

      const char* addr = nullptr;
      uint16_t addrLen = 0;
      size_t consumed = ClaspCodec::readString(p, remaining, addr, addrLen);
      if (consumed == 0) break;
      p += consumed;
      remaining -= consumed;

      ClaspValue val;
      if (remaining >= 1) {
        uint8_t valueIndicator = *p++;
        remaining--;
        if (valueIndicator == 1 && remaining >= 1) {
          uint8_t vtype = *p++;
          remaining--;
          ClaspCodec::readValueData(p, remaining, vtype, val);
        }
      }

      dispatchMessage(msgType, addr, addrLen, val);
      break;
    }

    case CLASP_MSG_SNAPSHOT: {
      // SNAPSHOT: [0x23][u16 count][entries...]
      // Each entry: [u16+address][vtype][value_data][u64 rev][opt_flags][...]
      if (len < 3) break;
      uint16_t count = ((uint16_t)payload[1] << 8) | payload[2];
      const uint8_t* p = payload + 3;
      size_t remaining = len - 3;

      for (uint16_t i = 0; i < count && remaining > 0; i++) {
        const char* addr = nullptr;
        uint16_t addrLen = 0;
        size_t consumed = ClaspCodec::readString(p, remaining, addr, addrLen);
        if (consumed == 0) break;
        p += consumed;
        remaining -= consumed;

        if (remaining < 1) break;
        uint8_t vtype = *p++;
        remaining--;

        ClaspValue val;
        size_t valConsumed = ClaspCodec::readValueData(p, remaining, vtype, val);
        p += valConsumed;
        remaining -= valConsumed;

        // Skip revision (u64) + opt_flags + optional fields
        if (remaining < 9) break; // u64 rev + u8 opt_flags
        p += 8; // skip revision
        remaining -= 8;
        uint8_t optFlags = *p++;
        remaining--;

        // Skip optional writer string
        if (optFlags & 0x01) {
          if (remaining < 2) break;
          uint16_t wlen = ((uint16_t)p[0] << 8) | p[1];
          size_t skip = 2 + wlen;
          if (remaining < skip) break;
          p += skip;
          remaining -= skip;
        }
        // Skip optional timestamp
        if (optFlags & 0x02) {
          if (remaining < 8) break;
          p += 8;
          remaining -= 8;
        }

        dispatchMessage(CLASP_MSG_SET, addr, addrLen, val);
      }
      break;
    }

    default:
      break;
  }
}

void ClaspClient::dispatchMessage(uint8_t /* msgType */, const char* address,
                                  uint16_t addrLen, const ClaspValue& value) {
  // Null-terminate the address in-place (it's in our rx buffer).
  // Safe: callers always have at least one byte after the address
  // (SET has value data, PUBLISH has value indicator, SNAPSHOT has vtype).
  char* addrMut = const_cast<char*>(address);
  char saved = addrMut[addrLen];
  addrMut[addrLen] = '\0';

  for (int i = 0; i < CLASP_MAX_SUBSCRIPTIONS; i++) {
    if (_subs[i].active && _subs[i].callback) {
      if (ClaspCodec::matchPattern(_subs[i].pattern, addrMut)) {
        _subs[i].callback(addrMut, value);
      }
    }
  }

  addrMut[addrLen] = saved;
}
