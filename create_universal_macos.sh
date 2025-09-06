#!/usr/bin/env bash

##################################################################
# Optional Lipo Script for Universal macOS Libraries
#
# This script creates universal (fat) archives for macOS from the
# per-architecture static libraries built by the main build system.
# Only run this after building both arm64 and x86_64 artifacts.
##################################################################

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARTIFACTS_DIR="$SCRIPT_DIR/artifacts"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Creating universal macOS libraries...${NC}"

# Check if required directories exist
if [ ! -d "$ARTIFACTS_DIR/macos-arm64" ]; then
    echo -e "${RED}Error: macos-arm64 artifacts directory not found${NC}"
    echo "Run the build for aarch64-apple-darwin target first"
    exit 1
fi

if [ ! -d "$ARTIFACTS_DIR/macos-x86_64" ]; then
    echo -e "${RED}Error: macos-x86_64 artifacts directory not found${NC}"
    echo "Run the build for x86_64-apple-darwin target first"
    exit 1
fi

# Check if required libraries exist
ARM64_MEDIAINFO="$ARTIFACTS_DIR/macos-arm64/libmediainfo.a"
ARM64_ZEN="$ARTIFACTS_DIR/macos-arm64/libzen.a"
X86_64_MEDIAINFO="$ARTIFACTS_DIR/macos-x86_64/libmediainfo.a"
X86_64_ZEN="$ARTIFACTS_DIR/macos-x86_64/libzen.a"

for lib in "$ARM64_MEDIAINFO" "$ARM64_ZEN" "$X86_64_MEDIAINFO" "$X86_64_ZEN"; do
    if [ ! -f "$lib" ]; then
        echo -e "${RED}Error: Required library not found: $lib${NC}"
        echo "Build the missing architecture first"
        exit 1
    fi
done

# Create universal directory
UNIVERSAL_DIR="$ARTIFACTS_DIR/macos-universal"
mkdir -p "$UNIVERSAL_DIR"

echo -e "${YELLOW}Creating universal libmediainfo.a...${NC}"
lipo -create \
    "$ARM64_MEDIAINFO" \
    "$X86_64_MEDIAINFO" \
    -output "$UNIVERSAL_DIR/libmediainfo.a"

echo -e "${YELLOW}Creating universal libzen.a...${NC}"
lipo -create \
    "$ARM64_ZEN" \
    "$X86_64_ZEN" \
    -output "$UNIVERSAL_DIR/libzen.a"

# Verify the universal libraries
echo -e "${GREEN}Verifying universal libraries:${NC}"
echo -e "${YELLOW}libmediainfo.a:${NC}"
lipo -info "$UNIVERSAL_DIR/libmediainfo.a"

echo -e "${YELLOW}libzen.a:${NC}"
lipo -info "$UNIVERSAL_DIR/libzen.a"

# Create README with build information
README_FILE="$UNIVERSAL_DIR/README.txt"
cat > "$README_FILE" << EOF
Universal macOS Static Libraries
================================

This directory contains universal (fat) static libraries created by combining
the ARM64 and x86_64 architectures.

Libraries:
- libmediainfo.a: Universal MediaInfo library
- libzen.a: Universal ZenLib library

Source architectures:
- ARM64: $ARM64_MEDIAINFO
- x86_64: $X86_64_MEDIAINFO

Created: $(date)
Script: $(basename "$0")

These libraries can be used to build applications that run natively on both
Apple Silicon (M1/M2) and Intel-based Macs.

To use these libraries in your Rust build, you can set up your build.rs to
look for artifacts in the macos-universal directory when building for macOS
without a specific architecture preference.
EOF

echo -e "${GREEN}Universal macOS libraries created successfully!${NC}"
echo -e "Libraries are available in: ${YELLOW}$UNIVERSAL_DIR${NC}"
echo -e "Build information saved to: ${YELLOW}$README_FILE${NC}"
echo ""
echo -e "${GREEN}To use these universal libraries:${NC}"
echo "1. These can be used when you need a single binary that works on both architectures"
echo "2. For Cargo builds, the per-architecture libraries in macos-arm64/ and macos-x86_64/ are preferred"
echo "3. The universal libraries are larger than individual architecture libraries"