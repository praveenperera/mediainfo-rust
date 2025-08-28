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
    ZenLib_Options="--host=wasm32-unknown-unknown --disable-unicode --enable-static --disable-shared --disable-dll"
    MediaInfoLib_CXXFLAGS="-I ../../../Source -I ../../../../ZenLib/Source \
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

    if [ "$OS" = "emscripten" ]; then
        emconfigure ./configure --host=le32-unknown-nacl --disable-unicode --enable-static --disable-shared --disable-dll CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS"
    elif [ "$OS" = "wasm-bindgen" ]; then
        ./configure $ZenLib_Options CC="$CC" CXX="$CXX" CFLAGS="$CFLAGS" CXXFLAGS="$CXXFLAGS"
    else
        ./configure --enable-static --disable-shared $ZenLib_Options $*
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
        ./configure --enable-staticlibs --enable-shared --disable-static --with-libcurl=runtime --with-graphviz=runtime $MediaInfoLib_Options $*
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

echo "MediaInfo shared object is in MediaInfoLib/Project/GNU/Library/.libs"
echo "For installing ZenLib, cd ZenLib/Project/GNU/Library && make install"
echo "For installing MediaInfoLib, cd MediaInfoLib/Project/GNU/Library && make install"

unset -v Home ZenLib_Options JsOptions OS
