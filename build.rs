use std::env;
use std::path::PathBuf;

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
    
    // Link against the compiled MediaInfo libraries
    let mediainfo_lib_path = mediainfo_src.join("MediaInfoLib/Project/GNU/Library/.libs");
    let zenlib_path = mediainfo_src.join("ZenLib/Project/GNU/Library/.libs");
    
    println!("cargo:rustc-link-search=native={}", mediainfo_lib_path.display());
    println!("cargo:rustc-link-search=native={}", zenlib_path.display());
    
    // Link to the actual static library files (without lib prefix for -l flag)
    // Note: mediainfo depends on zen, so zen must come after mediainfo
    println!("cargo:rustc-link-lib=static=mediainfo");
    println!("cargo:rustc-link-lib=static=zen");
    
    // Add WASM-specific linker flags
    println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH=1");
    println!("cargo:rustc-link-arg=-sMALLOC=emmalloc");
    println!("cargo:rustc-link-arg=-sASSERTIONS=0");
    println!("cargo:rustc-link-arg=-sNO_FILESYSTEM=1");
    println!("cargo:rustc-link-arg=-sINITIAL_MEMORY=33554432");
}
