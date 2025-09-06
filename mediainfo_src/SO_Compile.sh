#! /bin/sh

##################################################################

Parallel_Make () {
    local numprocs=1
    case $OS in
    'linux')
        numprocs=`grep -c ^processor /proc/cpuinfo 2>/dev/null`
        ;;
    'mac')
        if type sysctl &> /dev/null; then
            numprocs=`sysctl -n hw.ncpu`
        fi
        ;;
    #"solaris')
    #    on Solaris you need to use psrinfo -p instead
    #    ;;
    #'freebsd')
    #    ;;
    *) ;;
    esac
    if [ "$numprocs" = "" ] || [ "$numprocs" = "0" ]; then
        numprocs=1
    fi
   $Make -s -j$numprocs
}

##################################################################
# Init

Home=`pwd`
Make="make"
ZenLib_Options=""

# Check if TARGET environment variable is set (e.g., from Rust build.rs)
if [ -z "$TARGET" ]; then
    TARGET=""
fi

# Function to determine deployment target for macOS
get_macos_deployment_target() {
    # Precedence: MEDIAINFO_MACOSX_DEPLOYMENT_TARGET > RUSTFLAGS > MACOSX_DEPLOYMENT_TARGET > default 11.0
    if [ ! -z "$MEDIAINFO_MACOSX_DEPLOYMENT_TARGET" ]; then
        echo "$MEDIAINFO_MACOSX_DEPLOYMENT_TARGET"
    elif [ ! -z "$RUSTFLAGS" ] && echo "$RUSTFLAGS" | grep -q "mmacosx-version-min="; then
        echo "$RUSTFLAGS" | sed -n 's/.*-mmacosx-version-min=\([0-9.]*\).*/\1/p' | head -n1
    elif [ ! -z "$CARGO_ENCODED_RUSTFLAGS" ] && echo "$CARGO_ENCODED_RUSTFLAGS" | grep -q "mmacosx-version-min="; then
        # Decode CARGO_ENCODED_RUSTFLAGS (\x1f separated)
        echo "$CARGO_ENCODED_RUSTFLAGS" | tr '\x1f' ' ' | sed -n 's/.*-mmacosx-version-min=\([0-9.]*\).*/\1/p' | head -n1
    elif [ ! -z "$MACOSX_DEPLOYMENT_TARGET" ]; then
        echo "$MACOSX_DEPLOYMENT_TARGET"
    else
        echo "11.0"
    fi
}

# Function to setup target-specific environment
setup_target_env() {
    local target="$1"
    case "$target" in
        "aarch64-apple-darwin")
            export CC="clang -arch arm64"
            export CXX="clang++ -arch arm64"
            local deployment_target=$(get_macos_deployment_target)
            export CFLAGS="-arch arm64 -mmacosx-version-min=$deployment_target $CFLAGS"
            export CXXFLAGS="-arch arm64 -mmacosx-version-min=$deployment_target $CXXFLAGS"
            export LDFLAGS="-arch arm64 -mmacosx-version-min=$deployment_target $LDFLAGS"
            export MACOSX_DEPLOYMENT_TARGET="$deployment_target"
            echo "Setup environment for macOS ARM64 (deployment target: $deployment_target)"
            ;;
        "x86_64-apple-darwin")
            export CC="clang -arch x86_64"
            export CXX="clang++ -arch x86_64"
            local deployment_target=$(get_macos_deployment_target)
            export CFLAGS="-arch x86_64 -mmacosx-version-min=$deployment_target $CFLAGS"
            export CXXFLAGS="-arch x86_64 -mmacosx-version-min=$deployment_target $CXXFLAGS"
            export LDFLAGS="-arch x86_64 -mmacosx-version-min=$deployment_target $LDFLAGS"
            export MACOSX_DEPLOYMENT_TARGET="$deployment_target"
            echo "Setup environment for macOS x86_64 (deployment target: $deployment_target)"
            ;;
        "x86_64-unknown-linux-gnu"|"aarch64-unknown-linux-gnu")
            # For Linux, use default compiler setup
            echo "Setup environment for Linux $target"
            ;;
        *)
            echo "Using default environment for target: $target"
            ;;
    esac
}

# Function to get host triplet from TARGET
get_host_triplet() {
    local target="$1"
    case "$target" in
        "aarch64-apple-darwin") echo "aarch64-apple-darwin" ;;
        "x86_64-apple-darwin") echo "x86_64-apple-darwin" ;;
        "x86_64-unknown-linux-gnu") echo "x86_64-linux-gnu" ;;
        "aarch64-unknown-linux-gnu") echo "aarch64-linux-gnu" ;;
        *) echo "" ;;
    esac
}

# Function to get artifact directory name from TARGET
get_artifact_dir() {
    local target="$1"
    case "$target" in
        "aarch64-apple-darwin") echo "macos-arm64" ;;
        "x86_64-apple-darwin") echo "macos-x86_64" ;;
        "x86_64-unknown-linux-gnu") echo "linux-x86_64" ;;
        "aarch64-unknown-linux-gnu") echo "linux-aarch64" ;;
        *) echo "unknown" ;;
    esac
}

# Setup environment for the target if specified
if [ ! -z "$TARGET" ]; then
    setup_target_env "$TARGET"
fi

OS=$(uname -s)
# expr isn’t available on mac
if [ "$OS" = "Darwin" ]; then
    OS="mac"
# if the 5 first caracters of $OS equal "Linux"
elif [ "$(expr substr $OS 1 5)" = "Linux" ]; then
    OS="linux"
#elif [ "$(expr substr $OS 1 5)" = "SunOS" ]; then
#    OS="solaris"
#elif [ "$(expr substr $OS 1 7)" = "FreeBSD" ]; then
#    OS="freebsd"
fi

# Check for WASM target architecture from environment or explicit flag
if [ "$1" = "--emscripten-lib" ] || [ "$TARGET" = "wasm32-unknown-emscripten" ] || [ "$TARGET" = "wasm32-wasi" ]; then
    if [ "$1" = "--emscripten-lib" ]; then
        shift
    fi
    
    OS="emscripten"
    Make="emmake make"
    CFLAGS="$CFLAGS -Oz -s EMBIND_STD_STRING_IS_UTF8=1"
    CXXFLAGS="$CXXFLAGS -Oz -s EMBIND_STD_STRING_IS_UTF8=1 -fno-exceptions"
    MediaInfoLib_CXXFLAGS="-I ../../../Source -I ../../../../ZenLib/Source -s USE_ZLIB=1 \
                           -DMEDIAINFO_ADVANCED_YES \
                           -DMEDIAINFO_MINIMAL_YES \
                           -DMEDIAINFO_EXPORT_YES \
                           -DMEDIAINFO_SEEK_YES \
                           -DMEDIAINFO_READER_NO \
                           -DMEDIAINFO_REFERENCES_NO \
                           -DMEDIAINFO_GRAPH_NO \
                           -DMEDIAINFO_GRAPHVIZ_NO \
                           -DMEDIAINFO_ARCHIVE_NO \
                           -DMEDIAINFO_FIXITY_NO \
                           -DMEDIAINFO_CSV_NO \
                           -DMEDIAINFO_CUSTOM_NO \
                           -DMEDIAINFO_EBUCORE_NO \
                           -DMEDIAINFO_FIMS_NO \
                           -DMEDIAINFO_MPEG7_NO \
                           -DMEDIAINFO_PBCORE_NO \
                           -DMEDIAINFO_REVTMD_NO \
                           -DMEDIAINFO_NISO_NO \
                           -DMEDIAINFO_MINIMIZESIZE \
                           -DMEDIAINFO_TRACE_NO \
                           -DMEDIAINFO_FILTER_NO \
                           -DMEDIAINFO_DUPLICATE_NO \
                           -DMEDIAINFO_MACROBLOCKS_NO \
                           -DMEDIAINFO_NEXTPACKET_NO \
                           -DMEDIAINFO_EVENTS_NO \
                           -DMEDIAINFO_DEMUX_NO \
                           -DMEDIAINFO_IBI_NO \
                           -DMEDIAINFO_CONFORMANCE_YES \
                           -DMEDIAINFO_DIRECTORY_NO \
                           -DMEDIAINFO_LIBCURL_NO \
                           -DMEDIAINFO_LIBMMS_NO \
                           -DMEDIAINFO_READTHREAD_NO \
                           -DMEDIAINFO_MD5_NO \
                           -DMEDIAINFO_SHA1_NO \
                           -DMEDIAINFO_SHA2_NO \
                           -DMEDIAINFO_AES_NO \
                           -DMEDIAINFO_JNI_NO \
                           -DMEDIAINFO_TRACE_FFV1CONTENT_NO \
                           -DMEDIAINFO_COMPRESS \
                           -DMEDIAINFO_DECODE_NO \
                           -DMEDIAINFO_IBIUSAGE_NO \
                           -DMEDIAINFO_TINYXML2_NO"
    echo "Detected WASM target: $TARGET - configuring for Emscripten build"
elif [ "$TARGET" = "wasm32-unknown-unknown" ]; then
    OS="wasm-bindgen"
    Make="make"
    
    # Use clang with WASM target for wasm-bindgen compatibility
    CC="clang --target=wasm32-unknown-unknown"
    CXX="clang++ --target=wasm32-unknown-unknown"
    
    CFLAGS="$CFLAGS -Os -fno-exceptions -fno-rtti"
    CXXFLAGS="$CXXFLAGS -Os -fno-exceptions -fno-rtti -std=c++17"
    
    # Configure flags for minimal MediaInfo build compatible with wasm-bindgen
    # Note: We enable unicode for DLL interface but disable wide character strings in ZenLib
    ZenLib_Options="--host=wasm32-unknown-unknown --enable-unicode --enable-static --disable-shared --disable-dll"
    MediaInfoLib_CXXFLAGS="-I ../../../Source -I ../../../../ZenLib/Source \
                           -DUNICODE \
                           -DMEDIAINFO_MINIMAL_YES \
                           -DMEDIAINFO_EXPORT_YES \
                           -DMEDIAINFO_SEEK_YES \
                           -DMEDIAINFO_READER_NO \
                           -DMEDIAINFO_REFERENCES_NO \
                           -DMEDIAINFO_GRAPH_NO \
                           -DMEDIAINFO_GRAPHVIZ_NO \
                           -DMEDIAINFO_ARCHIVE_NO \
                           -DMEDIAINFO_FIXITY_NO \
                           -DMEDIAINFO_CSV_NO \
                           -DMEDIAINFO_CUSTOM_NO \
                           -DMEDIAINFO_EBUCORE_NO \
                           -DMEDIAINFO_FIMS_NO \
                           -DMEDIAINFO_MPEG7_NO \
                           -DMEDIAINFO_PBCORE_NO \
                           -DMEDIAINFO_REVTMD_NO \
                           -DMEDIAINFO_NISO_NO \
                           -DMEDIAINFO_MINIMIZESIZE \
                           -DMEDIAINFO_TRACE_NO \
                           -DMEDIAINFO_FILTER_NO \
                           -DMEDIAINFO_DUPLICATE_NO \
                           -DMEDIAINFO_MACROBLOCKS_NO \
                           -DMEDIAINFO_NEXTPACKET_NO \
                           -DMEDIAINFO_EVENTS_NO \
                           -DMEDIAINFO_DEMUX_NO \
                           -DMEDIAINFO_IBI_NO \
                           -DMEDIAINFO_CONFORMANCE_YES \
                           -DMEDIAINFO_DIRECTORY_NO \
                           -DMEDIAINFO_LIBCURL_NO \
                           -DMEDIAINFO_LIBMMS_NO \
                           -DMEDIAINFO_READTHREAD_NO \
                           -DMEDIAINFO_MD5_NO \
                           -DMEDIAINFO_SHA1_NO \
                           -DMEDIAINFO_SHA2_NO \
                           -DMEDIAINFO_AES_NO \
                           -DMEDIAINFO_JNI_NO \
                           -DMEDIAINFO_TRACE_FFV1CONTENT_NO \
                           -DMEDIAINFO_COMPRESS \
                           -DMEDIAINFO_DECODE_NO \
                           -DMEDIAINFO_IBIUSAGE_NO \
                           -DMEDIAINFO_TINYXML2_NO \
                           -DMEDIAINFO_STATIC \
                           -DMEDIAINFODLL_EXPORTS \
                           -D__WASM__ \
                           -DUNIX"
    echo "Detected WASM target: $TARGET - configuring for wasm-bindgen build"
fi

##################################################################
# ZenLib

if test -e ZenLib/Project/GNU/Library/configure; then
    cd ZenLib/Project/GNU/Library/
    test -e Makefile && rm Makefile
    chmod +x configure

    # Bootstrap autotools helpers if missing (e.g. on fresh checkout)
    if [ ! -f "config.guess" ] || [ ! -f "config.sub" ] || [ ! -f "ltmain.sh" ] || [ ! -f "install-sh" ] || [ ! -f "compile" ]; then
        echo "Bootstrapping ZenLib autotools files (config.guess/config.sub/ltmain.sh)"
        if [ -x "./autogen.sh" ]; then
            sh ./autogen.sh || true
        fi
        if [ ! -f "config.guess" ] || [ ! -f "config.sub" ] || [ ! -f "ltmain.sh" ]; then
            if command -v autoreconf >/dev/null 2>&1; then
                autoreconf -fi || true
            fi
        fi
    fi

    if [ "$OS" = "emscripten" ]; then
        emconfigure ./configure --host=le32-unknown-nacl --disable-unicode --enable-static --disable-shared --disable-dll CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS"
    elif [ "$OS" = "wasm-bindgen" ]; then
        ./configure $ZenLib_Options CC="$CC" CXX="$CXX" CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS"
    else
        # Use host triplet if TARGET is specified
        host_arg=""
        if [ ! -z "$TARGET" ]; then
            host_triplet=$(get_host_triplet "$TARGET")
            if [ ! -z "$host_triplet" ]; then
                host_arg="--host=$host_triplet"
            fi
        fi
        ./configure --enable-static --disable-shared $host_arg $ZenLib_Options $*
    fi
    if test -e Makefile; then
        make clean
        Parallel_Make
        if test -e libzen.la; then
            echo ZenLib compiled
        else
            echo Problem while compiling ZenLib
            exit
        fi
    else
        echo Problem while configuring ZenLib
        exit
    fi
else
    echo ZenLib directory is not found
    exit
fi
cd $Home

##################################################################
# MediaInfoLib

if test -e MediaInfoLib/Project/GNU/Library/configure; then
    cd MediaInfoLib/Project/GNU/Library/
    test -e Makefile && rm Makefile
    chmod +x configure
    if [ "$OS" = "emscripten" ]; then
        emconfigure ./configure --host=le32-unknown-nacl --enable-static --disable-shared --disable-dll $* CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS $MediaInfoLib_CXXFLAGS"
    elif [ "$OS" = "wasm-bindgen" ]; then
        ./configure --host=wasm32-unknown-unknown --enable-static --disable-shared --disable-dll CC="$CC" CXX="$CXX" CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS $MediaInfoLib_CXXFLAGS" $*
    else
        # Use host triplet if TARGET is specified
        host_arg=""
        if [ ! -z "$TARGET" ]; then
            host_triplet=$(get_host_triplet "$TARGET")
            if [ ! -z "$host_triplet" ]; then
                host_arg="--host=$host_triplet"
            fi
        fi
        ./configure --enable-static --disable-shared --with-libcurl=runtime --with-graphviz=runtime $host_arg $MediaInfoLib_Options $*
    fi
    if test -e Makefile; then
        make clean
        Parallel_Make
        if test -e libmediainfo.la; then
            echo MediaInfoLib compiled
        else
            echo Problem while compiling MediaInfoLib
            exit
        fi
    else
        echo Problem while configuring MediaInfoLib
        exit
    fi
else
    echo MediaInfoLib directory is not found
    exit
fi
cd $Home

##################################################################
# MediaInfoDLL Interface (for wasm-bindgen builds)
if [ "$OS" = "wasm-bindgen" ]; then
    echo "Building MediaInfoDLL interface for wasm-bindgen"
    cd MediaInfoLib/Project/GNU/Library/
    
    # Compile MediaInfoDLL.cpp specifically for WASM
    $CXX $CXXFLAGS $MediaInfoLib_CXXFLAGS \
        -c ../../../Source/MediaInfoDLL/MediaInfoDLL.cpp \
        -o MediaInfoDLL.o
    
    # Create a static library that includes the DLL interface
    ar rcs libmediainfodll.a MediaInfoDLL.o
    
    if test -e libmediainfodll.a; then
        echo "MediaInfoDLL interface compiled for wasm-bindgen"
        # Copy to .libs for consistency
        mkdir -p .libs
        cp libmediainfodll.a .libs/
    else
        echo "Problem compiling MediaInfoDLL interface"
        exit 1
    fi
    
    cd $Home
fi

##################################################################
# JavaScript Interface (only for Emscripten builds)
if [ "$OS" = "emscripten" ]; then
    cd MediaInfoLib/Project/GNU/Library/
    em++ $CXXFLAGS $MediaInfoLib_CXXFLAGS --bind -c ../../../Source/MediaInfoDLL/MediaInfoJS.cpp -o MediaInfoJS.o

    em++ -s WASM=0 $CXXFLAGS $MediaInfoLib_CXXFLAGS -s TOTAL_MEMORY=134217728 -s NO_FILESYSTEM=1 -s MODULARIZE=1 --closure 0 \
         --bind MediaInfoJS.o .libs/libmediainfo.a ../../../../ZenLib/Project/GNU/Library/.libs/libzen.a \
         --post-js ../../../Source/Resource/JavaScript/Post.js \
         -s EXPORT_NAME="'MediaInfoLib'" \
         -o MediaInfo.js

    em++ -s WASM=1 $CXXFLAGS $MediaInfoLib_CXXFLAGS -s TOTAL_MEMORY=33554432 -s ALLOW_MEMORY_GROWTH=1 -s NO_FILESYSTEM=1 -s MODULARIZE=1 --closure 0 \
         --bind MediaInfoJS.o .libs/libmediainfo.a ../../../../ZenLib/Project/GNU/Library/.libs/libzen.a \
         --post-js ../../../Source/Resource/JavaScript/Post.js \
         -s EXPORT_NAME="'MediaInfoLib'" \
         -o MediaInfoWasm.js

    if test -e MediaInfo.js; then
        echo MediaInfoLib JavaScript interface compiled
    else
        echo Problem while compiling MediaInfoLib JavaScript interface
        exit
    fi

    echo "MediaInfoLib JavaScript interface is in MediaInfoLib/Project/GNU/Library"
    echo "Static libraries are also available in .libs directories for Rust linking"
    cd $Home
fi

##################################################################
# Copy artifacts to target-specific directory

if [ ! -z "$TARGET" ] && [ "$OS" != "emscripten" ] && [ "$OS" != "wasm-bindgen" ]; then
    artifact_dir=$(get_artifact_dir "$TARGET")
    if [ "$artifact_dir" != "unknown" ]; then
        echo "Copying artifacts to $artifact_dir directory"
        mkdir -p "$Home/../artifacts/$artifact_dir"
        
        # Copy ZenLib
        if [ -f "ZenLib/Project/GNU/Library/.libs/libzen.a" ]; then
            cp "ZenLib/Project/GNU/Library/.libs/libzen.a" "$Home/../artifacts/$artifact_dir/"
            echo "Copied libzen.a to artifacts/$artifact_dir/"
        else
            echo "Warning: libzen.a not found"
        fi
        
        # Copy MediaInfoLib
        if [ -f "MediaInfoLib/Project/GNU/Library/.libs/libmediainfo.a" ]; then
            cp "MediaInfoLib/Project/GNU/Library/.libs/libmediainfo.a" "$Home/../artifacts/$artifact_dir/"
            echo "Copied libmediainfo.a to artifacts/$artifact_dir/"
        else
            echo "Warning: libmediainfo.a not found"
        fi
        
        # Create a build info file
        echo "Target: $TARGET" > "$Home/../artifacts/$artifact_dir/README.txt"
        echo "Host: $(get_host_triplet "$TARGET")" >> "$Home/../artifacts/$artifact_dir/README.txt"
        if [ "$TARGET" = "aarch64-apple-darwin" ] || [ "$TARGET" = "x86_64-apple-darwin" ]; then
            echo "Deployment Target: $(get_macos_deployment_target)" >> "$Home/../artifacts/$artifact_dir/README.txt"
        fi
        echo "Build Date: $(date)" >> "$Home/../artifacts/$artifact_dir/README.txt"
        echo "Created build info: artifacts/$artifact_dir/README.txt"
    fi
fi

##################################################################

echo "MediaInfo shared object is in MediaInfoLib/Project/GNU/Library/.libs"
if [ ! -z "$TARGET" ] && [ "$artifact_dir" != "unknown" ]; then
    echo "Static libraries also copied to artifacts/$artifact_dir/"
fi
echo "For installing ZenLib, cd ZenLib/Project/GNU/Library && make install"
echo "For installing MediaInfoLib, cd MediaInfoLib/Project/GNU/Library && make install"

unset -v Home ZenLib_Options JsOptions OS
