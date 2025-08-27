#!/bin/bash
set -e

# Script to build MediaInfo for WebAssembly
# Usage: ./build_mediainfo_wasm.sh /path/to/mediainfo/source

MEDIAINFO_SRC="$1"
if [ -z "$MEDIAINFO_SRC" ]; then
    echo "Usage: $0 /path/to/mediainfo/source"
    exit 1
fi

if [ ! -d "$MEDIAINFO_SRC" ]; then
    echo "Error: MediaInfo source directory not found: $MEDIAINFO_SRC"
    exit 1
fi

echo "Building MediaInfo for WebAssembly..."

# Build ZenLib dependency first
cd "$MEDIAINFO_SRC/../../ZenLib/Project/GNU/Library"
emconfigure ./configure --enable-static --disable-shared --host=wasm32
emmake make -j$(nproc)

# Build MediaInfoLib
cd "$MEDIAINFO_SRC/../MediaInfoLib/Project/GNU/Library"
emconfigure ./configure --enable-static --disable-shared --host=wasm32 --with-libzen-static
emmake make -j$(nproc)

echo "MediaInfo WebAssembly build complete!"
echo "Set MEDIAINFO_SOURCE=$MEDIAINFO_SRC when building Rust crate"