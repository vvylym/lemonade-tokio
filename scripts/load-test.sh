#!/bin/bash
# Load testing script for Lemonade Load Balancer
# Uses wrk for HTTP load testing

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default parameters
THREADS=${THREADS:-10}
CONNECTIONS=${CONNECTIONS:-100000}
DURATION=${DURATION:-300s}
URL=${URL:-http://127.0.0.1:50501/work}
SCRIPT=${SCRIPT:-}

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo -e "${RED}Error: wrk is not installed${NC}"
    echo "Install with: sudo apt-get install wrk (Debian/Ubuntu)"
    echo "Or: brew install wrk (macOS)"
    exit 1
fi

# Check if load balancer is running
if ! curl -sf http://127.0.0.1:50501/health > /dev/null 2>&1; then
    echo -e "${YELLOW}Warning: Load balancer health check failed${NC}"
    echo "Make sure the load balancer is running on http://127.0.0.1:50501"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo -e "${GREEN}Starting load test...${NC}"
echo "Threads: $THREADS"
echo "Connections: $CONNECTIONS"
echo "Duration: $DURATION"
echo "URL: $URL"
echo ""

# Build wrk command
WRK_CMD="wrk -t$THREADS -c$CONNECTIONS -d$DURATION --latency --timeout 10s"

# Add script if provided
if [ -n "$SCRIPT" ]; then
    WRK_CMD="$WRK_CMD -s $SCRIPT"
fi

WRK_CMD="$WRK_CMD $URL"

# Run load test
echo -e "${GREEN}Running load test...${NC}"
$WRK_CMD

echo -e "${GREEN}Load test completed${NC}"

