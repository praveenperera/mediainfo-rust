# WASM Integration Guide

## Problem: `env` Import Error

When using this MediaInfo crate in a WASM project, you may encounter this error:
```
Failed to resolve import "env" from "../pkg/index.js"
```

## Solution

The issue occurs because `wasm-bindgen` generates imports from the `env` module by default. To fix this, the **consuming crate** (not this MediaInfo crate) needs specific configuration.

### For the consuming crate's `Cargo.toml`:

Add these flags to your `wasm32-unknown-emscripten` target configuration:

```toml
[target.wasm32-unknown-emscripten]
rustflags = [
    "-C", "link-arg=-sSTANDALONE_WASM=1",
    "-C", "link-arg=-sNO_FILESYSTEM=1", 
    "-C", "link-arg=-sMALLOC=emmalloc",
    "-C", "link-arg=-sALLOW_MEMORY_GROWTH=1",
    "-C", "link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0",
    "-C", "link-arg=-sIMPORT_MEMORY=0",
    "-C", "link-arg=-sEXPORT_ALL=0",
    "-C", "link-arg=-sENVIRONMENT=web,worker"
]
```

### For `wasm-pack` builds:

If using `wasm-pack`, add these flags to your build command or `.cargo/config.toml`:

```bash
wasm-pack build --target web -- --features wasm
```

With `.cargo/config.toml`:
```toml
[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "link-arg=--export-dynamic",
    "-C", "link-arg=-sSTANDALONE_WASM=1",
]
```

### Environment Variables

Set these environment variables before building:
```bash
export EMCC_CFLAGS="-sSTANDALONE_WASM=1 -sNO_FILESYSTEM=1"
export EMMAKEN_CFLAGS="-sSTANDALONE_WASM=1"
```

## Explanation

The MediaInfo crate compiles C++ libraries to WASM using Emscripten. The `env` import issue occurs when:

1. The final WASM module tries to import functions from JavaScript environment
2. `wasm-bindgen` generates bindings that expect these imports
3. Vite/bundlers can't resolve the `env` module

The solution ensures:
- All dependencies are statically linked (`STANDALONE_WASM=1`)
- No dynamic imports from JavaScript environment
- Self-contained WASM module that doesn't need `env` imports