#!/bin/bash
#
# CLASP Production Test Suite Runner
#
# Runs the complete suite of production-readiness tests:
# 1. Unit/integration tests (always run)
# 2. Hardware tests (if devices connected)
# 3. Broker tests (if Docker services running)
# 4. Network simulation tests
# 5. Soak tests (configurable duration)
#
# Usage:
#   ./run-production-tests.sh              # Quick run (5 min soak)
#   ./run-production-tests.sh --full       # Full run (60 min soak)
#   ./run-production-tests.sh --hardware   # Include hardware tests
#   ./run-production-tests.sh --brokers    # Include broker tests
#   ./run-production-tests.sh --all        # Everything

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SOAK_DURATION=5  # minutes
RUN_HARDWARE=false
RUN_BROKERS=false
RUN_SOAK=true
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --full)
            SOAK_DURATION=60
            shift
            ;;
        --hardware)
            RUN_HARDWARE=true
            shift
            ;;
        --brokers)
            RUN_BROKERS=true
            shift
            ;;
        --all)
            RUN_HARDWARE=true
            RUN_BROKERS=true
            SOAK_DURATION=60
            shift
            ;;
        --quick)
            SOAK_DURATION=1
            RUN_SOAK=true
            shift
            ;;
        --no-soak)
            RUN_SOAK=false
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "CLASP Production Test Suite"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --quick      Quick test run (1 min soak)"
            echo "  --full       Full test run (60 min soak)"
            echo "  --hardware   Include hardware tests (MIDI/Art-Net/OSC)"
            echo "  --brokers    Include broker tests (requires Docker)"
            echo "  --all        Run everything"
            echo "  --no-soak    Skip soak tests"
            echo "  --verbose    Show detailed output"
            echo "  --help       Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}"
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              CLASP Production Test Suite                         ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$PROJECT_ROOT"

# Track results
PASSED=0
FAILED=0
SKIPPED=0

run_test() {
    local name=$1
    local cmd=$2
    local required=${3:-true}

    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Running: $name${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    if $VERBOSE; then
        if eval "$cmd"; then
            echo -e "${GREEN}✓ $name PASSED${NC}"
            ((PASSED++))
        else
            if $required; then
                echo -e "${RED}✗ $name FAILED${NC}"
                ((FAILED++))
            else
                echo -e "${YELLOW}⊘ $name SKIPPED (optional)${NC}"
                ((SKIPPED++))
            fi
        fi
    else
        if eval "$cmd" > /tmp/clasp-test-output.log 2>&1; then
            echo -e "${GREEN}✓ $name PASSED${NC}"
            ((PASSED++))
        else
            if $required; then
                echo -e "${RED}✗ $name FAILED${NC}"
                echo "Last 20 lines of output:"
                tail -20 /tmp/clasp-test-output.log
                ((FAILED++))
            else
                echo -e "${YELLOW}⊘ $name SKIPPED (optional)${NC}"
                ((SKIPPED++))
            fi
        fi
    fi
}

# ============================================================================
# Phase 1: Core Tests (always run)
# ============================================================================

echo -e "\n${YELLOW}Phase 1: Core Tests${NC}"

run_test "Baseline Tests" "cargo run -p clasp-test-suite --bin run-all-tests"
run_test "Client Tests" "cargo run -p clasp-test-suite --bin client-tests"
run_test "Discovery Tests" "cargo run -p clasp-test-suite --bin discovery-tests"
run_test "Bridge Tests" "cargo run -p clasp-test-suite --bin bridge-tests"
run_test "Embedded Tests" "cargo run -p clasp-test-suite --bin embedded-tests"
run_test "UDP Tests" "cargo run -p clasp-test-suite --bin udp-tests"
run_test "Session Tests" "cargo run -p clasp-test-suite --bin session-tests"
run_test "E2E Protocol Tests" "cargo run -p clasp-test-suite --bin e2e-protocol-tests"

# ============================================================================
# Phase 2: Network Simulation Tests
# ============================================================================

echo -e "\n${YELLOW}Phase 2: Network Simulation Tests${NC}"

run_test "Network Tests" "cargo run -p clasp-test-suite --bin network-tests"

# ============================================================================
# Phase 3: Hardware Tests (optional)
# ============================================================================

if $RUN_HARDWARE; then
    echo -e "\n${YELLOW}Phase 3: Hardware Tests${NC}"

    export CLASP_TEST_MIDI=1
    export CLASP_TEST_ARTNET=1
    export CLASP_TEST_OSC=1

    run_test "Hardware Tests" "cargo run -p clasp-test-suite --bin hardware-tests" false
else
    echo -e "\n${YELLOW}Phase 3: Hardware Tests (skipped - use --hardware to enable)${NC}"
    ((SKIPPED++))
fi

# ============================================================================
# Phase 4: Broker Integration Tests (optional)
# ============================================================================

if $RUN_BROKERS; then
    echo -e "\n${YELLOW}Phase 4: Broker Integration Tests${NC}"

    # Check if Docker services are running
    if docker ps | grep -q "clasp-test-mqtt"; then
        export CLASP_TEST_BROKERS=1
        run_test "Broker Tests" "cargo run -p clasp-test-suite --bin broker-tests"
    else
        echo -e "${YELLOW}Starting Docker services...${NC}"
        cd "$PROJECT_ROOT/test-suite/docker"
        docker-compose up -d
        sleep 5
        cd "$PROJECT_ROOT"

        export CLASP_TEST_BROKERS=1
        run_test "Broker Tests" "cargo run -p clasp-test-suite --bin broker-tests"

        echo -e "${YELLOW}Stopping Docker services...${NC}"
        cd "$PROJECT_ROOT/test-suite/docker"
        docker-compose down
        cd "$PROJECT_ROOT"
    fi
else
    echo -e "\n${YELLOW}Phase 4: Broker Tests (skipped - use --brokers to enable)${NC}"
    ((SKIPPED++))
fi

# ============================================================================
# Phase 5: Soak Tests
# ============================================================================

if $RUN_SOAK; then
    echo -e "\n${YELLOW}Phase 5: Soak Tests ($SOAK_DURATION minutes)${NC}"
    run_test "Soak Test" "cargo run -p clasp-test-suite --bin soak-tests -- $SOAK_DURATION"
else
    echo -e "\n${YELLOW}Phase 5: Soak Tests (skipped - use --full to enable)${NC}"
    ((SKIPPED++))
fi

# ============================================================================
# Summary
# ============================================================================

echo -e "\n${BLUE}"
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║                    TEST SUITE SUMMARY                            ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo -e "║  ${GREEN}Passed:  $PASSED${BLUE}                                                      ║"
echo -e "║  ${RED}Failed:  $FAILED${BLUE}                                                      ║"
echo -e "║  ${YELLOW}Skipped: $SKIPPED${BLUE}                                                      ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}PRODUCTION TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}PRODUCTION TESTS PASSED${NC}"
    exit 0
fi
