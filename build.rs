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
    use std::process::Command;
    
    let target = env::var("TARGET").unwrap();
    println!("cargo:rerun-if-changed=mediainfo_src/SO_Compile.sh");
    println!("cargo:rerun-if-changed=mediainfo_src/");
    
    // Set TARGET environment variable for the compile script
    env::set_var("TARGET", &target);
    
    // Run the SO_Compile.sh script to build MediaInfo for WASM
    let output = Command::new("sh")
        .arg("mediainfo_src/SO_Compile.sh")
        .env("TARGET", &target)
        .output()
        .expect("Failed to execute SO_Compile.sh");
    
    if !output.status.success() {
        panic!(
            "SO_Compile.sh failed with status: {}\nstdout: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    println!("SO_Compile.sh output: {}", String::from_utf8_lossy(&output.stdout));
    
    // Link against the compiled MediaInfo libraries
    // The script builds libraries in MediaInfoLib/Project/GNU/Library/.libs/
    let mediainfo_lib_path = "mediainfo_src/MediaInfoLib/Project/GNU/Library/.libs";
    let zenlib_path = "mediainfo_src/ZenLib/Project/GNU/Library/.libs";
    
    println!("cargo:rustc-link-search=native={}", mediainfo_lib_path);
    println!("cargo:rustc-link-search=native={}", zenlib_path);
    println!("cargo:rustc-link-lib=static=mediainfo");
    println!("cargo:rustc-link-lib=static=zen");
    
    // Add WASM-specific linker flags
    println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH=1");
    println!("cargo:rustc-link-arg=-sMALLOC=emmalloc");
    println!("cargo:rustc-link-arg=-sASSERTIONS=0");
    println!("cargo:rustc-link-arg=-sNO_FILESYSTEM=1");
    println!("cargo:rustc-link-arg=-sINITIAL_MEMORY=33554432");
}
