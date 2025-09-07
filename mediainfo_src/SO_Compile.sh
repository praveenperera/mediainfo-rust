#!/bin/bash

set -e  # Exit on any error

# =================================================================
# MediaInfo Build Script
# Builds ZenLib and MediaInfoLib static libraries for various targets
# =================================================================

# Global configuration
readonly SCRIPT_DIR
SCRIPT_DIR="$(pwd)"
readonly DEFAULT_MAKE="make"

# =================================================================
# Utility Functions
# =================================================================

log() {
    echo "[$0] $*"
}

error() {
    echo "ERROR: $*" >&2
    exit 1
}

get_cpu_count() {
    local cpu_count=1
    case "$(uname -s)" in
        Darwin)
            command -v sysctl >/dev/null && cpu_count=$(sysctl -n hw.ncpu)
            ;;
        Linux)
            cpu_count=$(grep -c ^processor /proc/cpuinfo 2>/dev/null || echo 1)
            ;;
    esac
    echo "${cpu_count:-1}"
}

parallel_make() {
    local jobs=$(get_cpu_count)
    log "Building with $jobs parallel jobs"
    ${MAKE:-$DEFAULT_MAKE} -s -j"$jobs"
}

# =================================================================
# Target Configuration
# =================================================================

get_artifact_dir() {
    case "$1" in
        aarch64-apple-darwin) echo "macos-arm64" ;;
        x86_64-apple-darwin) echo "macos-x86_64" ;;
        x86_64-unknown-linux-gnu) echo "linux-x86_64" ;;
        aarch64-unknown-linux-gnu) echo "linux-aarch64" ;;
        *) echo "unknown" ;;
    esac
}

get_host_triplet() {
    case "$1" in
        aarch64-apple-darwin) echo "aarch64-apple-darwin" ;;
        x86_64-apple-darwin) echo "x86_64-apple-darwin" ;;
        x86_64-unknown-linux-gnu) echo "x86_64-linux-gnu" ;;
        aarch64-unknown-linux-gnu) echo "aarch64-linux-gnu" ;;
        *) echo "" ;;
    esac
}

get_macos_deployment_target() {
    # Priority: MEDIAINFO_MACOSX_DEPLOYMENT_TARGET > RUSTFLAGS > MACOSX_DEPLOYMENT_TARGET > default 11.0
    if [ -n "$MEDIAINFO_MACOSX_DEPLOYMENT_TARGET" ]; then
        echo "$MEDIAINFO_MACOSX_DEPLOYMENT_TARGET"
    elif [ -n "$RUSTFLAGS" ] && echo "$RUSTFLAGS" | grep -q "mmacosx-version-min="; then
        echo "$RUSTFLAGS" | sed -n 's/.*-mmacosx-version-min=\([0-9.]*\).*/\1/p' | head -n1
    elif [ -n "$CARGO_ENCODED_RUSTFLAGS" ] && echo "$CARGO_ENCODED_RUSTFLAGS" | grep -q "mmacosx-version-min="; then
        echo "$CARGO_ENCODED_RUSTFLAGS" | tr '\x1f' ' ' | sed -n 's/.*-mmacosx-version-min=\([0-9.]*\).*/\1/p' | head -n1
    elif [ -n "$MACOSX_DEPLOYMENT_TARGET" ]; then
        echo "$MACOSX_DEPLOYMENT_TARGET"
    else
        echo "11.0"
    fi
}

# =================================================================
# Environment Setup
# =================================================================

setup_macos_env() {
    local arch="$1"
    local deployment_target
    
    deployment_target=$(get_macos_deployment_target)
    [ -z "$deployment_target" ] && deployment_target="11.0"
    
    export CC="clang"
    export CXX="clang++"
    export MACOSX_DEPLOYMENT_TARGET="$deployment_target"
    
    local sdk_root=""
    if command -v xcrun >/dev/null 2>&1; then
        sdk_root=$(xcrun --show-sdk-path 2>/dev/null || true)
    fi
    
    local arch_flags="-arch $arch -mmacosx-version-min=$deployment_target"
    if [ -n "$sdk_root" ]; then
        arch_flags="$arch_flags -isysroot $sdk_root"
    fi
    
    export CFLAGS="$arch_flags $CFLAGS"
    export CXXFLAGS="$arch_flags $CXXFLAGS" 
    export LDFLAGS="$arch_flags $LDFLAGS"
    
    log "macOS environment: arch=$arch, deployment_target=$deployment_target"
}

setup_linux_env() {
    local target="$1"
    log "Linux environment: target=$target"
    # Use default compiler setup for Linux
}

setup_wasm_emscripten_env() {
    export MAKE="emmake make"
    export CFLAGS="$CFLAGS -Oz -s EMBIND_STD_STRING_IS_UTF8=1"
    export CXXFLAGS="$CXXFLAGS -Oz -s EMBIND_STD_STRING_IS_UTF8=1 -fno-exceptions"
    
    # Minimal MediaInfo build for Emscripten
    export MEDIAINFO_CXXFLAGS="-I ../../../Source -I ../../../../ZenLib/Source -s USE_ZLIB=1 \
        -DMEDIAINFO_MINIMAL_YES -DMEDIAINFO_EXPORT_YES -DMEDIAINFO_SEEK_YES \
        -DMEDIAINFO_READER_NO -DMEDIAINFO_REFERENCES_NO -DMEDIAINFO_GRAPH_NO \
        -DMEDIAINFO_GRAPHVIZ_NO -DMEDIAINFO_ARCHIVE_NO -DMEDIAINFO_FIXITY_NO \
        -DMEDIAINFO_CSV_NO -DMEDIAINFO_CUSTOM_NO -DMEDIAINFO_EBUCORE_NO \
        -DMEDIAINFO_FIMS_NO -DMEDIAINFO_MPEG7_NO -DMEDIAINFO_PBCORE_NO \
        -DMEDIAINFO_REVTMD_NO -DMEDIAINFO_NISO_NO -DMEDIAINFO_MINIMIZESIZE \
        -DMEDIAINFO_TRACE_NO -DMEDIAINFO_FILTER_NO -DMEDIAINFO_DUPLICATE_NO \
        -DMEDIAINFO_MACROBLOCKS_NO -DMEDIAINFO_NEXTPACKET_NO -DMEDIAINFO_EVENTS_NO \
        -DMEDIAINFO_DEMUX_NO -DMEDIAINFO_IBI_NO -DMEDIAINFO_CONFORMANCE_YES \
        -DMEDIAINFO_DIRECTORY_NO -DMEDIAINFO_LIBCURL_NO -DMEDIAINFO_LIBMMS_NO \
        -DMEDIAINFO_READTHREAD_NO -DMEDIAINFO_MD5_NO -DMEDIAINFO_SHA1_NO \
        -DMEDIAINFO_SHA2_NO -DMEDIAINFO_AES_NO -DMEDIAINFO_JNI_NO \
        -DMEDIAINFO_TRACE_FFV1CONTENT_NO -DMEDIAINFO_COMPRESS -DMEDIAINFO_DECODE_NO \
        -DMEDIAINFO_IBIUSAGE_NO -DMEDIAINFO_TINYXML2_NO"
    
    log "Emscripten WASM environment configured"
}

setup_wasm_bindgen_env() {
    export CC="clang --target=wasm32-unknown-unknown"
    export CXX="clang++ --target=wasm32-unknown-unknown"
    export CFLAGS="$CFLAGS -Os -fno-exceptions -fno-rtti"
    export CXXFLAGS="$CXXFLAGS -Os -fno-exceptions -fno-rtti -std=c++17"
    
    export ZENLIB_OPTIONS="--host=wasm32-unknown-unknown --enable-unicode --enable-static --disable-shared --disable-dll"
    export MEDIAINFO_CXXFLAGS="-I ../../../Source -I ../../../../ZenLib/Source \
        -DUNICODE -DMEDIAINFO_MINIMAL_YES -DMEDIAINFO_EXPORT_YES -DMEDIAINFO_SEEK_YES \
        -DMEDIAINFO_STATIC -DMEDIAINFODLL_EXPORTS -D__WASM__ -DUNIX"
    
    log "wasm-bindgen environment configured"
}

setup_target_environment() {
    local target="$1"
    
    case "$target" in
        aarch64-apple-darwin)
            setup_macos_env "arm64"
            ;;
        x86_64-apple-darwin)
            setup_macos_env "x86_64"
            ;;
        x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu)
            setup_linux_env "$target"
            ;;
        wasm32-unknown-emscripten|wasm32-wasi)
            setup_wasm_emscripten_env
            ;;
        wasm32-unknown-unknown)
            setup_wasm_bindgen_env
            ;;
        *)
            log "Using default environment for target: $target"
            ;;
    esac
}

# =================================================================
# Build Functions
# =================================================================

ensure_autotools() {
    # Ensure libtoolize is available (macOS uses glibtoolize)
    if ! command -v libtoolize >/dev/null 2>&1; then
        if command -v glibtoolize >/dev/null 2>&1; then
            export LIBTOOLIZE=glibtoolize
            log "Using glibtoolize as libtoolize"
        fi
    fi
}

build_zenlib() {
    log "Building ZenLib..."
    cd ZenLib/Project/GNU/Library/
    
    # Generate build files
    sh ./autogen.sh || true
    autoreconf -fi || true
    
    [ ! -f configure ] && error "ZenLib configure script not found"
    
    # Clean previous build
    [ -f Makefile ] && rm -f Makefile
    chmod +x configure
    
    # Configure based on target type
    if [ "$1" = "emscripten" ]; then
        emconfigure ./configure --host=le32-unknown-nacl --disable-unicode --enable-static --disable-shared --disable-dll \
            CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS"
    elif [ "$1" = "wasm-bindgen" ]; then
        ./configure $ZENLIB_OPTIONS CC="$CC" CXX="$CXX" CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS"
    else
        local host_arg=""
        if [ -n "$TARGET" ]; then
            local host_triplet=$(get_host_triplet "$TARGET")
            [ -n "$host_triplet" ] && host_arg="--host=$host_triplet"
        fi
        ./configure --enable-static --disable-shared $host_arg $ZENLIB_OPTIONS
    fi
    
    [ ! -f Makefile ] && error "ZenLib configuration failed"
    
    # Build
    make clean
    parallel_make
    
    [ ! -f libzen.la ] && error "ZenLib compilation failed"
    log "ZenLib compiled successfully"
    
    cd "$SCRIPT_DIR"
}

build_mediainfo() {
    log "Building MediaInfoLib..."
    cd MediaInfoLib/Project/GNU/Library/
    
    # Generate build files  
    sh ./autogen.sh || true
    autoreconf -fi || true
    
    [ ! -f configure ] && error "MediaInfoLib configure script not found"
    
    # Clean previous build
    [ -f Makefile ] && rm -f Makefile
    chmod +x configure
    
    # Configure based on target type
    if [ "$1" = "emscripten" ]; then
        emconfigure ./configure --host=le32-unknown-nacl --enable-static --disable-shared --disable-dll \
            CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS $MEDIAINFO_CXXFLAGS"
    elif [ "$1" = "wasm-bindgen" ]; then
        ./configure --host=wasm32-unknown-unknown --enable-static --disable-shared --disable-dll \
            CC="$CC" CXX="$CXX" CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS $MEDIAINFO_CXXFLAGS"
    else
        local host_arg=""
        if [ -n "$TARGET" ]; then
            local host_triplet=$(get_host_triplet "$TARGET")
            [ -n "$host_triplet" ] && host_arg="--host=$host_triplet"
        fi
        ./configure --enable-static --disable-shared --with-libcurl=runtime --with-graphviz=runtime \
            $host_arg $MEDIAINFO_OPTIONS
    fi
    
    [ ! -f Makefile ] && error "MediaInfoLib configuration failed"
    
    # Build
    make clean
    parallel_make
    
    [ ! -f libmediainfo.la ] && error "MediaInfoLib compilation failed"
    log "MediaInfoLib compiled successfully"
    
    cd "$SCRIPT_DIR"
}

build_wasm_dll_interface() {
    [ "$1" != "wasm-bindgen" ] && return
    
    log "Building MediaInfoDLL interface for wasm-bindgen"
    cd MediaInfoLib/Project/GNU/Library/
    
    # Compile MediaInfoDLL.cpp for WASM
    $CXX $CXXFLAGS $MEDIAINFO_CXXFLAGS \
        -c ../../../Source/MediaInfoDLL/MediaInfoDLL.cpp \
        -o MediaInfoDLL.o
    
    # Create static library with DLL interface
    ar rcs libmediainfodll.a MediaInfoDLL.o
    
    [ ! -f libmediainfodll.a ] && error "MediaInfoDLL interface compilation failed"
    
    # Copy to .libs for consistency
    mkdir -p .libs
    cp libmediainfodll.a .libs/
    
    log "MediaInfoDLL interface compiled successfully"
    cd "$SCRIPT_DIR"
}

build_javascript_interface() {
    [ "$1" != "emscripten" ] && return
    
    log "Building JavaScript interface for Emscripten"
    cd MediaInfoLib/Project/GNU/Library/
    
    # Compile JavaScript binding
    em++ $CXXFLAGS $MEDIAINFO_CXXFLAGS --bind \
        -c ../../../Source/MediaInfoDLL/MediaInfoJS.cpp \
        -o MediaInfoJS.o
    
    # Create asm.js version
    em++ -s WASM=0 $CXXFLAGS $MEDIAINFO_CXXFLAGS -s TOTAL_MEMORY=134217728 \
        -s NO_FILESYSTEM=1 -s MODULARIZE=1 --closure 0 --bind \
        MediaInfoJS.o .libs/libmediainfo.a ../../../../ZenLib/Project/GNU/Library/.libs/libzen.a \
        --post-js ../../../Source/Resource/JavaScript/Post.js \
        -s EXPORT_NAME="'MediaInfoLib'" \
        -o MediaInfo.js
    
    # Create WASM version
    em++ -s WASM=1 $CXXFLAGS $MEDIAINFO_CXXFLAGS -s TOTAL_MEMORY=33554432 \
        -s ALLOW_MEMORY_GROWTH=1 -s NO_FILESYSTEM=1 -s MODULARIZE=1 --closure 0 --bind \
        MediaInfoJS.o .libs/libmediainfo.a ../../../../ZenLib/Project/GNU/Library/.libs/libzen.a \
        --post-js ../../../Source/Resource/JavaScript/Post.js \
        -s EXPORT_NAME="'MediaInfoLib'" \
        -o MediaInfoWasm.js
    
    [ ! -f MediaInfo.js ] && error "JavaScript interface compilation failed"
    
    log "JavaScript interface compiled successfully"
    cd "$SCRIPT_DIR"
}

copy_artifacts() {
    [ -z "$TARGET" ] && return
    
    local artifact_dir=$(get_artifact_dir "$TARGET")
    [ "$artifact_dir" = "unknown" ] && return
    
    local parent_dir="${ARTIFACT_PARENT_DIR:-$SCRIPT_DIR/../artifacts}"
    local dest_dir="$parent_dir/$artifact_dir"
    
    log "Copying artifacts to $dest_dir"
    mkdir -p "$dest_dir"
    
    # Copy libraries
    if [ -f "ZenLib/Project/GNU/Library/.libs/libzen.a" ]; then
        cp "ZenLib/Project/GNU/Library/.libs/libzen.a" "$dest_dir/"
        log "Copied libzen.a"
    else
        log "Warning: libzen.a not found"
    fi
    
    if [ -f "MediaInfoLib/Project/GNU/Library/.libs/libmediainfo.a" ]; then
        cp "MediaInfoLib/Project/GNU/Library/.libs/libmediainfo.a" "$dest_dir/"
        log "Copied libmediainfo.a"
    else
        log "Warning: libmediainfo.a not found"
    fi
    
    # Create build info
    {
        echo "Target: $TARGET"
        echo "Host: $(get_host_triplet "$TARGET")"
        case "$TARGET" in
            *apple-darwin*)
                echo "Deployment Target: $(get_macos_deployment_target)"
                ;;
        esac
        echo "Build Date: $(date)"
    } > "$dest_dir/README.txt"
    
    log "Build info created: $dest_dir/README.txt"
}

# =================================================================
# Main Execution
# =================================================================

main() {
    log "Starting MediaInfo build process"
    
    # Determine build type from target or arguments
    local build_type="native"
    if [ "$1" = "--emscripten-lib" ] || [ "$TARGET" = "wasm32-unknown-emscripten" ] || [ "$TARGET" = "wasm32-wasi" ]; then
        build_type="emscripten"
        [ "$1" = "--emscripten-lib" ] && shift
    elif [ "$TARGET" = "wasm32-unknown-unknown" ]; then
        build_type="wasm-bindgen"
    fi
    
    log "Build type: $build_type, Target: ${TARGET:-auto-detect}"
    
    # Setup environment
    ensure_autotools
    [ -n "$TARGET" ] && setup_target_environment "$TARGET"
    
    # Build libraries
    build_zenlib "$build_type"
    build_mediainfo "$build_type"
    
    # Build additional interfaces for WASM
    build_wasm_dll_interface "$build_type"
    build_javascript_interface "$build_type"
    
    # Copy artifacts for Rust builds
    copy_artifacts
    
    log "Build completed successfully"
    log "Static libraries location: MediaInfoLib/Project/GNU/Library/.libs"
    
    if [ -n "$TARGET" ] && [ "$(get_artifact_dir "$TARGET")" != "unknown" ]; then
        local parent_dir="${ARTIFACT_PARENT_DIR:-$SCRIPT_DIR/../artifacts}"
        log "Artifacts also copied to: $parent_dir/$(get_artifact_dir "$TARGET")/"
    fi
    
    log "For installation: cd <library>/Project/GNU/Library && make install"
}

# Run main function with all arguments
main "$@"