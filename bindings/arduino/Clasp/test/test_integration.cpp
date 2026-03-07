/**
 * CLASP Integration Tests
 *
 * Tests the Arduino CLASP client against a real CLASP relay over TCP.
 * Requires a CLASP router running with TCP transport:
 *
 *   clasp-router --transport tcp --listen 127.0.0.1:17330
 *
 * Or set CLASP_TEST_PORT to use a different port.
 * If no server is available, all tests are skipped (not failed).
 *
 * Build:  make test_integration
 * Run:    ./test_integration
 */

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <fcntl.h>
#include <errno.h>
#include <signal.h>
#include <sys/wait.h>

#include "../src/ClaspCodec.h"
#include "../src/Clasp.h"

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
// Real TCP Client adapter (POSIX sockets -> Arduino Client interface)
// ============================================================================

class TcpClient : public Client {
public:
  TcpClient() : _fd(-1) {}
  ~TcpClient() { stop(); }

  int connect(const char* host, uint16_t port) override {
    _fd = socket(AF_INET, SOCK_STREAM, 0);
    if (_fd < 0) return 0;

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons(port);
    inet_pton(AF_INET, host, &addr.sin_addr);

    if (::connect(_fd, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
      close(_fd);
      _fd = -1;
      return 0;
    }

    // Set non-blocking for available() to work
    int flags = fcntl(_fd, F_GETFL, 0);
    fcntl(_fd, F_SETFL, flags | O_NONBLOCK);

    return 1;
  }

  size_t write(const uint8_t* buf, size_t size) override {
    if (_fd < 0) return 0;
    // Temporarily set blocking for writes
    int flags = fcntl(_fd, F_GETFL, 0);
    fcntl(_fd, F_SETFL, flags & ~O_NONBLOCK);
    ssize_t sent = ::write(_fd, buf, size);
    fcntl(_fd, F_SETFL, flags); // restore
    return sent > 0 ? (size_t)sent : 0;
  }

  int available() override {
    if (_fd < 0) return 0;
    uint8_t peek;
    ssize_t ret = recv(_fd, &peek, 1, MSG_PEEK | MSG_DONTWAIT);
    if (ret > 0) {
      uint8_t peekBuf[1024];
      ssize_t n = recv(_fd, peekBuf, sizeof(peekBuf), MSG_PEEK | MSG_DONTWAIT);
      return n > 0 ? (int)n : 0;
    }
    if (ret == 0) {
      // Connection closed
      _fd = -1;
      return 0;
    }
    // EAGAIN/EWOULDBLOCK = no data
    return 0;
  }

  int read(uint8_t* buf, size_t size) override {
    if (_fd < 0) return 0;
    ssize_t ret = recv(_fd, buf, size, MSG_DONTWAIT);
    if (ret > 0) return (int)ret;
    if (ret == 0) { _fd = -1; return 0; } // closed
    return 0; // EAGAIN
  }

  uint8_t connected() override {
    if (_fd < 0) return 0;
    // Check if connection is still alive
    uint8_t peek;
    ssize_t ret = recv(_fd, &peek, 1, MSG_PEEK | MSG_DONTWAIT);
    if (ret == 0) { _fd = -1; return 0; } // closed
    if (ret < 0 && errno != EAGAIN && errno != EWOULDBLOCK) { _fd = -1; return 0; }
    return 1;
  }

  void stop() override {
    if (_fd >= 0) {
      close(_fd);
      _fd = -1;
    }
  }

private:
  int _fd;
};

// ============================================================================
// Server process management
// ============================================================================

static pid_t serverPid = -1;
static uint16_t testPort = 0;

// Try to find the clasp-router binary
const char* findClaspBinary() {
  // Environment override
  const char* env = getenv("CLASP_BIN");
  if (env) return env;

  // Relative to test directory (typical build layout)
  const char* candidates[] = {
    "../../../../target/release/clasp-router",
    "../../../../target/debug/clasp-router",
    "/usr/local/bin/clasp-router",
    nullptr,
  };

  for (int i = 0; candidates[i]; i++) {
    if (access(candidates[i], X_OK) == 0) {
      return candidates[i];
    }
  }
  return nullptr;
}

bool startServer(uint16_t port) {
  const char* bin = findClaspBinary();
  if (!bin) {
    printf("  No clasp binary found. Set CLASP_BIN env var.\n");
    return false;
  }

  printf("  Starting clasp-router (TCP, port %d)...\n", port);

  serverPid = fork();
  if (serverPid < 0) return false;

  if (serverPid == 0) {
    // Child: exec the server
    char listenAddr[32];
    snprintf(listenAddr, sizeof(listenAddr), "127.0.0.1:%d", port);

    // Redirect stdout/stderr to /dev/null
    freopen("/dev/null", "w", stdout);
    freopen("/dev/null", "w", stderr);

    execlp(bin, bin, "--transport", "tcp", "--listen", listenAddr, nullptr);
    _exit(1);
  }

  // Parent: wait for server to be ready
  usleep(500000); // 500ms

  // Verify it's alive
  int status;
  pid_t result = waitpid(serverPid, &status, WNOHANG);
  if (result != 0) {
    printf("  Server process died immediately.\n");
    serverPid = -1;
    return false;
  }

  // Try connecting
  int fd = socket(AF_INET, SOCK_STREAM, 0);
  struct sockaddr_in addr;
  memset(&addr, 0, sizeof(addr));
  addr.sin_family = AF_INET;
  addr.sin_port = htons(port);
  inet_pton(AF_INET, "127.0.0.1", &addr.sin_addr);

  bool ready = false;
  for (int attempt = 0; attempt < 10; attempt++) {
    if (::connect(fd, (struct sockaddr*)&addr, sizeof(addr)) == 0) {
      ready = true;
      break;
    }
    usleep(200000); // 200ms
  }
  close(fd);

  if (!ready) {
    printf("  Server not accepting connections after 2s.\n");
    kill(serverPid, SIGTERM);
    waitpid(serverPid, nullptr, 0);
    serverPid = -1;
    return false;
  }

  printf("  Server ready (pid %d).\n", serverPid);
  return true;
}

void stopServer() {
  if (serverPid > 0) {
    kill(serverPid, SIGTERM);
    int status;
    waitpid(serverPid, &status, 0);
    printf("  Server stopped.\n");
    serverPid = -1;
  }
}

// Wait for data to arrive on the client (up to timeoutMs)
void waitForData(TcpClient& tcp, int timeoutMs) {
  for (int i = 0; i < timeoutMs / 10; i++) {
    if (tcp.available() > 0) return;
    usleep(10000); // 10ms
  }
}

// ============================================================================
// Integration tests
// ============================================================================

void test_integ_connect_and_disconnect() {
  printf("test_integ_connect_and_disconnect\n");
  _clasp_reset_millis();

  TcpClient tcp;
  ClaspClient clasp(tcp);

  bool ok = clasp.connect("127.0.0.1", testPort, "IntegTest");
  ASSERT(ok, "connect to real server succeeds");
  ASSERT(clasp.connected(), "reports connected");

  clasp.disconnect();
  ASSERT(!clasp.connected(), "disconnected");
}

void test_integ_set_and_get_roundtrip() {
  printf("test_integ_set_and_get_roundtrip\n");
  _clasp_reset_millis();

  // Publisher
  TcpClient tcp1;
  ClaspClient pub(tcp1);
  ASSERT(pub.connect("127.0.0.1", testPort, "Publisher"), "publisher connects");

  // Subscriber
  TcpClient tcp2;
  ClaspClient sub(tcp2);
  ASSERT(sub.connect("127.0.0.1", testPort, "Subscriber"), "subscriber connects");

  // Subscribe on the subscriber
  static int received = 0;
  static float receivedValue = 0;
  static char receivedAddr[128] = "";
  received = 0;

  sub.subscribe("/integ/temp", [](const char* address, ClaspValue value) {
    received++;
    receivedValue = value.f;
    strncpy(receivedAddr, address, sizeof(receivedAddr) - 1);
  });

  // Give the subscription time to register on the server
  usleep(100000); // 100ms

  // Publish a value
  ASSERT(pub.set("/integ/temp", 23.5f), "set float succeeds");

  // Poll the subscriber until we get the value or timeout
  for (int i = 0; i < 100 && received == 0; i++) {
    sub.loop();
    usleep(10000); // 10ms
  }

  ASSERT(received == 1, "subscriber received the value");
  ASSERT(fabsf(receivedValue - 23.5f) < 0.01f, "received value matches");
  ASSERT(strcmp(receivedAddr, "/integ/temp") == 0, "received address matches");

  pub.disconnect();
  sub.disconnect();
}

void test_integ_multiple_value_types() {
  printf("test_integ_multiple_value_types\n");
  _clasp_reset_millis();

  TcpClient tcp1, tcp2;
  ClaspClient pub(tcp1), sub(tcp2);
  pub.connect("127.0.0.1", testPort, "TypePub");
  sub.connect("127.0.0.1", testPort, "TypeSub");

  static int received = 0;
  static ClaspValue lastVal;
  received = 0;

  sub.subscribe("/integ/types/**", [](const char*, ClaspValue value) {
    received++;
    lastVal = value;
  });
  usleep(100000);

  // Float
  received = 0;
  pub.set("/integ/types/float", 3.14f);
  for (int i = 0; i < 100 && received == 0; i++) { sub.loop(); usleep(10000); }
  ASSERT(received >= 1, "received float");
  ASSERT(lastVal.type == ClaspValue::Float, "type is Float");
  ASSERT(fabsf(lastVal.f - 3.14f) < 0.01f, "float value correct");

  // Int
  received = 0;
  pub.set("/integ/types/int", (int32_t)-42);
  for (int i = 0; i < 100 && received == 0; i++) { sub.loop(); usleep(10000); }
  ASSERT(received >= 1, "received int");
  ASSERT(lastVal.type == ClaspValue::Int, "type is Int");
  ASSERT(lastVal.i == -42, "int value correct");

  // String
  received = 0;
  pub.set("/integ/types/str", "hello");
  for (int i = 0; i < 100 && received == 0; i++) { sub.loop(); usleep(10000); }
  ASSERT(received >= 1, "received string");
  ASSERT(lastVal.type == ClaspValue::String, "type is String");
  ASSERT(lastVal.len == 5, "string length correct");

  // Bool
  received = 0;
  pub.set("/integ/types/bool", true);
  for (int i = 0; i < 100 && received == 0; i++) { sub.loop(); usleep(10000); }
  ASSERT(received >= 1, "received bool");
  ASSERT(lastVal.type == ClaspValue::Bool, "type is Bool");
  ASSERT(lastVal.b == true, "bool value correct");

  pub.disconnect();
  sub.disconnect();
}

void test_integ_wildcard_subscription() {
  printf("test_integ_wildcard_subscription\n");
  _clasp_reset_millis();

  TcpClient tcp1, tcp2;
  ClaspClient pub(tcp1), sub(tcp2);
  pub.connect("127.0.0.1", testPort, "WildPub");
  sub.connect("127.0.0.1", testPort, "WildSub");

  static int received = 0;
  received = 0;

  sub.subscribe("/integ/wild/**", [](const char*, ClaspValue) {
    received++;
  });
  usleep(100000);

  pub.set("/integ/wild/a", 1.0f);
  pub.set("/integ/wild/b/c", 2.0f);
  pub.set("/integ/wild/d/e/f", 3.0f);
  pub.set("/integ/other/x", 4.0f); // should NOT match

  for (int i = 0; i < 100 && received < 3; i++) { sub.loop(); usleep(10000); }

  ASSERT(received == 3, "received exactly 3 matching values (not the non-matching one)");

  pub.disconnect();
  sub.disconnect();
}

void test_integ_emit_event() {
  printf("test_integ_emit_event\n");
  _clasp_reset_millis();

  TcpClient tcp1, tcp2;
  ClaspClient pub(tcp1), sub(tcp2);
  pub.connect("127.0.0.1", testPort, "EmitPub");
  sub.connect("127.0.0.1", testPort, "EmitSub");

  static int received = 0;
  static ClaspValue lastVal;
  received = 0;

  sub.subscribe("/integ/events/**", [](const char*, ClaspValue value) {
    received++;
    lastVal = value;
  });
  usleep(100000);

  pub.emit("/integ/events/go");
  for (int i = 0; i < 100 && received == 0; i++) { sub.loop(); usleep(10000); }

  ASSERT(received == 1, "received emit event");
  ASSERT(lastVal.type == ClaspValue::Null, "emit event has null value");

  received = 0;
  pub.emit("/integ/events/level", 0.75f);
  for (int i = 0; i < 100 && received == 0; i++) { sub.loop(); usleep(10000); }

  ASSERT(received == 1, "received emit with value");
  ASSERT(lastVal.type == ClaspValue::Float, "emit value is float");
  ASSERT(fabsf(lastVal.f - 0.75f) < 0.01f, "emit value correct");

  pub.disconnect();
  sub.disconnect();
}

void test_integ_stream() {
  printf("test_integ_stream\n");
  _clasp_reset_millis();

  TcpClient tcp1, tcp2;
  ClaspClient pub(tcp1), sub(tcp2);
  pub.connect("127.0.0.1", testPort, "StreamPub");
  sub.connect("127.0.0.1", testPort, "StreamSub");

  static int received = 0;
  received = 0;

  sub.subscribe("/integ/stream/**", [](const char*, ClaspValue) {
    received++;
  });
  usleep(100000);

  // Send 10 stream values rapidly
  for (int i = 0; i < 10; i++) {
    pub.stream("/integ/stream/accel", (float)i * 0.1f);
  }

  // Wait for them to arrive
  for (int i = 0; i < 200 && received < 10; i++) { sub.loop(); usleep(10000); }

  ASSERT(received >= 5, "received at least 5 of 10 stream values (some may be dropped)");

  pub.disconnect();
  sub.disconnect();
}

void test_integ_multiple_subscribers() {
  printf("test_integ_multiple_subscribers\n");
  _clasp_reset_millis();

  TcpClient tcp1, tcp2, tcp3;
  ClaspClient pub(tcp1), sub1(tcp2), sub2(tcp3);
  pub.connect("127.0.0.1", testPort, "MultiPub");
  sub1.connect("127.0.0.1", testPort, "MultiSub1");
  sub2.connect("127.0.0.1", testPort, "MultiSub2");

  static int count1 = 0, count2 = 0;
  count1 = 0; count2 = 0;

  sub1.subscribe("/integ/multi/val", [](const char*, ClaspValue) { count1++; });
  sub2.subscribe("/integ/multi/val", [](const char*, ClaspValue) { count2++; });
  usleep(100000);

  pub.set("/integ/multi/val", 42.0f);

  for (int i = 0; i < 100 && (count1 == 0 || count2 == 0); i++) {
    sub1.loop();
    sub2.loop();
    usleep(10000);
  }

  ASSERT(count1 == 1, "subscriber 1 received the value");
  ASSERT(count2 == 1, "subscriber 2 received the value");

  pub.disconnect();
  sub1.disconnect();
  sub2.disconnect();
}

void test_integ_unsubscribe() {
  printf("test_integ_unsubscribe\n");
  _clasp_reset_millis();

  TcpClient tcp1, tcp2;
  ClaspClient pub(tcp1), sub(tcp2);
  pub.connect("127.0.0.1", testPort, "UnsubPub");
  sub.connect("127.0.0.1", testPort, "UnsubSub");

  static int received = 0;
  received = 0;

  sub.subscribe("/integ/unsub/val", [](const char*, ClaspValue) { received++; });
  usleep(100000);

  pub.set("/integ/unsub/val", 1.0f);
  for (int i = 0; i < 100 && received == 0; i++) { sub.loop(); usleep(10000); }
  ASSERT(received == 1, "received before unsubscribe");

  sub.unsubscribe("/integ/unsub/val");
  usleep(100000);

  received = 0;
  pub.set("/integ/unsub/val", 2.0f);
  // Wait a bit to see if anything arrives (it shouldn't)
  for (int i = 0; i < 30; i++) { sub.loop(); usleep(10000); }
  ASSERT(received == 0, "nothing received after unsubscribe");

  pub.disconnect();
  sub.disconnect();
}

void test_integ_rapid_publish() {
  printf("test_integ_rapid_publish\n");
  _clasp_reset_millis();

  TcpClient tcp1, tcp2;
  ClaspClient pub(tcp1), sub(tcp2);
  pub.connect("127.0.0.1", testPort, "RapidPub");
  sub.connect("127.0.0.1", testPort, "RapidSub");

  static int received = 0;
  static float lastFloat = 0;
  received = 0;

  sub.subscribe("/integ/rapid", [](const char*, ClaspValue v) {
    received++;
    if (v.type == ClaspValue::Float) lastFloat = v.f;
  });
  usleep(100000);

  // Publish 50 values rapidly
  for (int i = 0; i < 50; i++) {
    pub.set("/integ/rapid", (float)i);
  }

  // Wait for delivery
  for (int i = 0; i < 200 && received < 50; i++) { sub.loop(); usleep(10000); }

  ASSERT(received >= 20, "received at least 20 of 50 rapid values");
  // The last value should be close to 49.0
  printf("  (received %d of 50, last value=%.1f)\n", received, lastFloat);

  pub.disconnect();
  sub.disconnect();
}

// ============================================================================
// Main
// ============================================================================

int main() {
  printf("=== CLASP Integration Tests ===\n\n");

  // Determine port
  const char* portEnv = getenv("CLASP_TEST_PORT");
  testPort = portEnv ? (uint16_t)atoi(portEnv) : 17330;

  bool serverManaged = false;

  // Check if server is already running on the port
  {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons(testPort);
    inet_pton(AF_INET, "127.0.0.1", &addr.sin_addr);

    if (::connect(fd, (struct sockaddr*)&addr, sizeof(addr)) == 0) {
      printf("Using existing server on port %d\n\n", testPort);
      close(fd);
    } else {
      close(fd);
      // Try to start our own
      if (startServer(testPort)) {
        serverManaged = true;
      } else {
        printf("\nNo CLASP server available. Skipping integration tests.\n");
        printf("To run: clasp-router --transport tcp --listen 127.0.0.1:%d\n", testPort);
        printf("Or set CLASP_BIN to the clasp-router binary path.\n\n");
        printf("0/0 tests passed (all skipped)\n");
        return 0;
      }
    }
    printf("\n");
  }

  test_integ_connect_and_disconnect();
  test_integ_set_and_get_roundtrip();
  test_integ_multiple_value_types();
  test_integ_wildcard_subscription();
  test_integ_emit_event();
  test_integ_stream();
  test_integ_multiple_subscribers();
  test_integ_unsubscribe();
  test_integ_rapid_publish();

  if (serverManaged) {
    printf("\n");
    stopServer();
  }

  printf("\n%d/%d tests passed", tests_passed, tests_run);
  if (tests_failed > 0) printf(" (%d FAILED)", tests_failed);
  printf("\n");
  return tests_failed == 0 ? 0 : 1;
}
