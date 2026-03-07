#include "ClaspDiscovery.h"

#ifdef ARDUINO

#include <IPAddress.h>

// mDNS multicast address and port
static const IPAddress MDNS_ADDR(224, 0, 0, 251);
static const uint16_t MDNS_PORT = 5353;

// Minimal mDNS query for _clasp._tcp.local
// This is a pre-built DNS query packet
static const uint8_t MDNS_QUERY[] = {
  0x00, 0x00, // Transaction ID
  0x00, 0x00, // Flags (standard query)
  0x00, 0x01, // Questions: 1
  0x00, 0x00, // Answers: 0
  0x00, 0x00, // Authority: 0
  0x00, 0x00, // Additional: 0
  // Question: _clasp._tcp.local
  0x06, '_', 'c', 'l', 'a', 's', 'p',
  0x04, '_', 't', 'c', 'p',
  0x05, 'l', 'o', 'c', 'a', 'l',
  0x00,       // Root label
  0x00, 0x0C, // Type: PTR
  0x00, 0x01, // Class: IN
};

ClaspDiscovery::ClaspDiscovery() {}

bool ClaspDiscovery::find(ClaspDiscoveryResult& result, unsigned long timeoutMs) {
  memset(&result, 0, sizeof(result));

  _udp.beginMulticast(MDNS_ADDR, MDNS_PORT);

  if (!sendQuery()) {
    _udp.stop();
    return false;
  }

  unsigned long start = millis();
  while (millis() - start < timeoutMs) {
    int packetSize = _udp.parsePacket();
    if (packetSize > 0) {
      uint8_t buf[512];
      int len = _udp.read(buf, sizeof(buf));
      if (len > 0 && parseResponse(buf, len, result)) {
        _udp.stop();
        return true;
      }
    }
    delay(10);
  }

  _udp.stop();
  return false;
}

bool ClaspDiscovery::sendQuery() {
  _udp.beginPacket(MDNS_ADDR, MDNS_PORT);
  _udp.write(MDNS_QUERY, sizeof(MDNS_QUERY));
  return _udp.endPacket() == 1;
}

bool ClaspDiscovery::parseResponse(const uint8_t* buf, size_t len,
                                   ClaspDiscoveryResult& result) {
  // Minimal mDNS response parsing
  // We look for an SRV record pointing to a host and port,
  // and an A record with the IP address.

  if (len < 12) return false;

  // Check it's a response (QR bit set)
  if (!(buf[2] & 0x80)) return false;

  uint16_t answers = ((uint16_t)buf[6] << 8) | buf[7];
  uint16_t additional = ((uint16_t)buf[10] << 8) | buf[11];
  uint16_t totalRecords = answers + additional;

  if (totalRecords == 0) return false;

  // Skip the question section
  size_t pos = 12;
  uint16_t qdcount = ((uint16_t)buf[4] << 8) | buf[5];
  for (uint16_t q = 0; q < qdcount && pos < len; q++) {
    // Skip name
    while (pos < len && buf[pos] != 0) {
      if ((buf[pos] & 0xC0) == 0xC0) { pos += 2; goto next_q; }
      pos += buf[pos] + 1;
      if (pos >= len) break;
    }
    if (pos < len) pos++; // null terminator
    next_q:
    pos += 4; // type + class
  }

  bool foundPort = false;
  bool foundIP = false;

  // Parse answer and additional records
  for (uint16_t r = 0; r < totalRecords && pos + 10 < len; r++) {
    // Skip name (may be compressed)
    while (pos < len && buf[pos] != 0) {
      if ((buf[pos] & 0xC0) == 0xC0) { pos += 2; goto parse_record; }
      pos += buf[pos] + 1;
      if (pos >= len) break;
    }
    if (pos < len) pos++; // null terminator

    parse_record:
    if (pos + 10 > len) break;

    uint16_t rtype = ((uint16_t)buf[pos] << 8) | buf[pos + 1];
    // uint16_t rclass = ((uint16_t)buf[pos + 2] << 8) | buf[pos + 3];
    // uint32_t ttl (4 bytes)
    uint16_t rdlength = ((uint16_t)buf[pos + 8] << 8) | buf[pos + 9];
    pos += 10;

    if (pos + rdlength > len) break;

    if (rtype == 0x0021 && rdlength >= 6) { // SRV record
      // Priority (2), Weight (2), Port (2), Target (rest)
      result.port = ((uint16_t)buf[pos + 4] << 8) | buf[pos + 5];
      foundPort = true;
    } else if (rtype == 0x0001 && rdlength == 4) { // A record
      snprintf(result.host, sizeof(result.host), "%d.%d.%d.%d",
               buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]);
      foundIP = true;
    } else if (rtype == 0x0010 && rdlength > 1) { // TXT record
      // Look for "name=..." in TXT record
      size_t txtPos = pos;
      size_t txtEnd = pos + rdlength;
      while (txtPos < txtEnd) {
        uint8_t txtLen = buf[txtPos++];
        if (txtPos + txtLen > txtEnd) break;
        if (txtLen > 5 && memcmp(buf + txtPos, "name=", 5) == 0) {
          uint8_t nameLen = txtLen - 5;
          if (nameLen >= sizeof(result.name)) nameLen = sizeof(result.name) - 1;
          memcpy(result.name, buf + txtPos + 5, nameLen);
          result.name[nameLen] = '\0';
        }
        txtPos += txtLen;
      }
    }

    pos += rdlength;
  }

  return foundPort && foundIP;
}

#endif // ARDUINO
