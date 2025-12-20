#!/bin/bash
# Profiling script for Lemonade Tokio
# Supports flamegraph and perf profiling

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

PROFILE_TYPE=${1:-flamegraph}
BINARY=${2:-lemonade}
ARGS=${3:-"load-balancer --config config/load-balancer.yaml"}

echo -e "${GREEN}Starting profiling with $PROFILE_TYPE...${NC}"

case $PROFILE_TYPE in
    flamegraph)
        if ! command -v cargo-flamegraph &> /dev/null; then
            echo -e "${RED}Error: cargo-flamegraph is not installed${NC}"
            echo "Install with: cargo install flamegraph"
            exit 1
        fi
        
        echo -e "${YELLOW}Generating flamegraph...${NC}"
        sudo cargo flamegraph --bin "$BINARY" -- $ARGS
        echo -e "${GREEN}Flamegraph saved to: flamegraph.svg${NC}"
        ;;
    
    perf)
        if ! command -v perf &> /dev/null; then
            echo -e "${RED}Error: perf is not installed${NC}"
            echo "Install with: sudo apt-get install linux-perf (Debian/Ubuntu)"
            exit 1
        fi
        
        echo -e "${YELLOW}Profiling with perf...${NC}"
        perf record --call-graph dwarf -- cargo run --release --bin "$BINARY" -- $ARGS
        echo -e "${GREEN}Perf data saved. Generating report...${NC}"
        perf report
        ;;
    
    *)
        echo -e "${RED}Unknown profile type: $PROFILE_TYPE${NC}"
        echo "Usage: $0 [flamegraph|perf] [binary] [args]"
        exit 1
        ;;
esac

