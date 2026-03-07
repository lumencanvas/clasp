#ifndef CLASP_DISCOVERY_H
#define CLASP_DISCOVERY_H

#include <stdint.h>

struct ClaspDiscoveryResult {
  char host[64];
  uint16_t port;
  char name[32];
};

#ifdef ARDUINO
#include <Arduino.h>
#include <WiFiUdp.h>

class ClaspDiscovery {
public:
  ClaspDiscovery();

  // Send an mDNS query for _clasp._tcp.local and wait for a response.
  // Returns true if a router was found within timeoutMs.
  bool find(ClaspDiscoveryResult& result, unsigned long timeoutMs = 5000);

private:
  WiFiUDP _udp;
  bool sendQuery();
  bool parseResponse(const uint8_t* buf, size_t len, ClaspDiscoveryResult& result);
};

#endif // ARDUINO

#endif // CLASP_DISCOVERY_H
