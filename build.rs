use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    if target.contains("wasm32") {
        build_for_wasm();
    } else {
        build_for_native();
    }
}

fn build_for_native() {
    let lib_mediainfo = pkg_config::probe_library("libmediainfo");
    if lib_mediainfo.is_err() {
        panic!("Could not find MediaInfo via pkgconfig");
    }
}

fn build_for_wasm() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mediainfo_src = PathBuf::from(&manifest_dir).join("mediainfo_src");
    
    println!("cargo:rerun-if-changed={}", mediainfo_src.display());
    println!("cargo:rerun-if-changed={}", mediainfo_src.join("SO_Compile.sh").display());
    
    // Set TARGET environment variable for the SO_Compile.sh script
    let target_value = "wasm32-unknown-unknown";
    unsafe {
        env::set_var("TARGET", target_value);
    }
    
    // Use the existing SO_Compile.sh script which already handles WASM compilation properly
    // This will build the libraries needed for WASM
    let mut compile_script = Command::new("bash");
    compile_script
        .arg("SO_Compile.sh")
        .current_dir(&mediainfo_src)
        .env("TARGET", target_value);
    
    let output = compile_script.output().expect("Failed to execute SO_Compile.sh");
    
    if !output.status.success() {
        panic!(
            "MediaInfo compilation failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    println!("cargo:warning=MediaInfo compilation completed successfully");
    
    // For WASM builds, the SO_Compile.sh script creates libraries in .libs directories
    // We need to add these to the link path for Rust to find them
    let zenlib_path = mediainfo_src.join("ZenLib/Project/GNU/Library/.libs");
    let mediainfo_path = mediainfo_src.join("MediaInfoLib/Project/GNU/Library/.libs");
    
    // Add search paths for the static libraries
    println!("cargo:rustc-link-search=native={}", zenlib_path.display());
    println!("cargo:rustc-link-search=native={}", mediainfo_path.display());
    
    // Link against the static libraries that were built
    println!("cargo:rustc-link-lib=static=mediainfo");
    println!("cargo:rustc-link-lib=static=zen");
    
    // The key insight: the "env" import issue comes from wasm-bindgen/wasm-pack
    // These flags ensure static linking and prevent env imports
    
    // Tell the consuming crate's build to use these Emscripten flags
    // This is critical for preventing the env import issue
    println!("cargo:rustc-link-arg=-sSTANDALONE_WASM=1");
    println!("cargo:rustc-link-arg=-sNO_FILESYSTEM=1"); 
    println!("cargo:rustc-link-arg=-sMALLOC=emmalloc");
    println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH=1");
    println!("cargo:rustc-link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0");
    
    // Most importantly: prevent automatic env imports by disabling them
    println!("cargo:rustc-link-arg=-sIMPORT_MEMORY=0");
    println!("cargo:rustc-link-arg=-sEXPORT_ALL=0");
    
    // Ensure we don't depend on JavaScript environment
    println!("cargo:rustc-link-arg=-sENVIRONMENT=web,worker");
    
    // Link C++ standard library statically to avoid runtime dependencies
    println!("cargo:rustc-link-arg=-lc++");
    
    // Set a flag that consuming crates can check to configure themselves properly
    println!("cargo:metadata=requires_standalone_wasm=1");
}