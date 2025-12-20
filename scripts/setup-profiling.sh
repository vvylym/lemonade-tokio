#!/bin/bash
# Setup script for profiling tools
# Installs necessary tools for profiling and optimization

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up profiling tools...${NC}"

# Install cargo-flamegraph
if ! command -v cargo-flamegraph &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-flamegraph...${NC}"
    cargo install flamegraph
else
    echo -e "${GREEN}cargo-flamegraph already installed${NC}"
fi

# Install cargo-nextest (for faster tests)
if ! command -v cargo-nextest &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-nextest...${NC}"
    cargo install cargo-nextest
else
    echo -e "${GREEN}cargo-nextest already installed${NC}"
fi

# Install cargo-llvm-cov (for coverage)
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-llvm-cov...${NC}"
    cargo install cargo-llvm-cov
else
    echo -e "${GREEN}cargo-llvm-cov already installed${NC}"
fi

# Check for perf (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if ! command -v perf &> /dev/null; then
        echo -e "${YELLOW}perf is not installed. Install with:${NC}"
        echo "  sudo apt-get install linux-perf  # Debian/Ubuntu"
        echo "  sudo yum install perf            # RHEL/CentOS"
    else
        echo -e "${GREEN}perf is installed${NC}"
    fi
else
    echo -e "${YELLOW}perf is only available on Linux${NC}"
fi

# Check for wrk (for load testing)
if ! command -v wrk &> /dev/null; then
    echo -e "${YELLOW}wrk is not installed. Install with:${NC}"
    echo "  sudo apt-get install wrk  # Debian/Ubuntu"
    echo "  brew install wrk           # macOS"
else
    echo -e "${GREEN}wrk is installed${NC}"
fi

echo -e "${GREEN}Setup completed!${NC}"

