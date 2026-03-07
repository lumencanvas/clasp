CXX = g++
CXXFLAGS = -std=c++17 -Wall -Wextra -g -I../src
SRC_DIR = ../src

CODEC_SRCS = $(SRC_DIR)/ClaspCodec.cpp
CLIENT_SRCS = $(SRC_DIR)/ClaspCodec.cpp $(SRC_DIR)/Clasp.cpp

.PHONY: all test test-integration clean

all: test_codec test_framing test_client

test_codec: test_codec.cpp $(CODEC_SRCS)
	$(CXX) $(CXXFLAGS) -o $@ $^

test_framing: test_framing.cpp $(CODEC_SRCS)
	$(CXX) $(CXXFLAGS) -o $@ $^

test_client: test_client.cpp $(CLIENT_SRCS)
	$(CXX) $(CXXFLAGS) -o $@ $^

test_integration: test_integration.cpp $(CLIENT_SRCS)
	$(CXX) $(CXXFLAGS) -o $@ $^

test: all
	@echo ""
	@echo "--- Codec Tests ---"
	@./test_codec
	@echo ""
	@echo "--- Framing Tests ---"
	@./test_framing
	@echo ""
	@echo "--- Client Tests ---"
	@./test_client
	@echo ""
	@echo "=== All unit tests complete ==="

test-integration: test_integration
	@echo ""
	@echo "--- Integration Tests ---"
	@./test_integration

test-all: test test-integration

clean:
	rm -f test_codec test_framing test_client test_integration
