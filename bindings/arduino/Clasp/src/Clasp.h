#ifndef CLASP_H
#define CLASP_H

// Configuration defaults (define before including to override)
#ifndef CLASP_MAX_PACKET_SIZE
#define CLASP_MAX_PACKET_SIZE 256
#endif

#ifndef CLASP_MAX_SUBSCRIPTIONS
#define CLASP_MAX_SUBSCRIPTIONS 4
#endif

#ifndef CLASP_KEEPALIVE_MS
#define CLASP_KEEPALIVE_MS 15000
#endif

#ifdef ARDUINO
#include <Arduino.h>
#include <Client.h>
#else
// Desktop build: provide a minimal Client interface for testing
#include <cstdint>
#include <cstddef>
class Client {
public:
  virtual int connect(const char* host, uint16_t port) = 0;
  virtual size_t write(const uint8_t* buf, size_t size) = 0;
  virtual int available() = 0;
  virtual int read(uint8_t* buf, size_t size) = 0;
  virtual uint8_t connected() = 0;
  virtual void stop() = 0;
  virtual ~Client() {}
};
#endif

#include "ClaspCodec.h"

class ClaspClient {
public:
  typedef void (*MessageCallback)(const char* address, ClaspValue value);

  explicit ClaspClient(Client& client);

  // Connection
  bool connect(const char* host, uint16_t port = 7330, const char* name = nullptr);
  bool connected();
  void disconnect();

  // Publishing
  bool set(const char* address, float value);
  bool set(const char* address, int32_t value);
  bool set(const char* address, const char* value);
  bool set(const char* address, bool value);
  bool emit(const char* address);
  bool emit(const char* address, float value);
  bool stream(const char* address, float value);

  // Subscriptions
  bool subscribe(const char* pattern, MessageCallback callback);
  bool unsubscribe(const char* pattern);

  // Must be called in loop()
  bool loop();

private:
  Client* _client;
  bool _connected;
  bool _welcomed;

  // Send/receive buffers
  uint8_t _txBuf[CLASP_MAX_PACKET_SIZE];
  uint8_t _rxBuf[CLASP_MAX_PACKET_SIZE];
  uint16_t _rxPos;

  // Receive state machine for TCP length-prefixed framing
  // TCP sends: [4-byte u32 BE frame_len][CLASP frame bytes]
  enum RxState : uint8_t { WAIT_TCP_PREFIX, WAIT_FRAME };
  RxState _rxState;
  uint32_t _rxFrameLen; // expected frame length from TCP prefix

  // Subscriptions
  struct Subscription {
    char pattern[64];
    MessageCallback callback;
    uint32_t subId;
    bool active;
  };
  Subscription _subs[CLASP_MAX_SUBSCRIPTIONS];
  uint32_t _nextSubId;

  // Keepalive
  unsigned long _lastActivity;

  // Internal
  bool sendFrame(const uint8_t* buf, size_t len);
  void processMessage(const uint8_t* payload, uint16_t len);
  void dispatchMessage(uint8_t msgType, const char* address, uint16_t addrLen,
                       const ClaspValue& value);
};

#endif // CLASP_H
