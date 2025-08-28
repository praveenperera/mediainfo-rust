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
    
    // Default to wasm-bindgen builds (wasm32-unknown-unknown)
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
    
    // For wasm-bindgen builds, we don't need Emscripten-specific flags
    // The libraries will be statically linked and compatible with wasm-bindgen
    println!("cargo:warning=MediaInfo built for wasm-bindgen compatibility");
}