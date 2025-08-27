use std::env;
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
    let mediainfo_src = env::var("MEDIAINFO_SOURCE")
        .unwrap_or_else(|_| panic!("MEDIAINFO_SOURCE environment variable must be set for wasm builds"));
    
    // Build libmediainfo with emscripten
    let status = Command::new("emconfigure")
        .args(&["./configure", "--enable-static", "--disable-shared"])
        .current_dir(&mediainfo_src)
        .status()
        .expect("Failed to configure MediaInfo");
    
    if !status.success() {
        panic!("Failed to configure MediaInfo for wasm");
    }
    
    let status = Command::new("emmake")
        .args(&["make", "-j4"])
        .current_dir(&mediainfo_src)
        .status()
        .expect("Failed to build MediaInfo");
    
    if !status.success() {
        panic!("Failed to build MediaInfo for wasm");
    }
    
    // Link the built library
    println!("cargo:rustc-link-search=native={}/Source/MediaInfo/.libs", mediainfo_src);
    println!("cargo:rustc-link-lib=static=mediainfo");
    
    // Also need to link MediaInfoLib dependencies
    println!("cargo:rustc-link-search=native={}/Source/MediaInfoLib/.libs", mediainfo_src);
    println!("cargo:rustc-link-lib=static=mediainfolib");
}
