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
    let target = "wasm32-unknown-emscripten";
    
    println!("cargo:rerun-if-changed={}", mediainfo_src.display());
    println!("cargo:rerun-if-changed={}", mediainfo_src.join("SO_Compile.sh").display());
    
    // Use the actual target for building - this allows both emscripten and wasm32-unknown-unknown
    unsafe {
        env::set_var("TARGET", &target);
    }
    
    // Use SO_Compile.sh which properly handles Emscripten builds
    let mut compile_script = Command::new("bash");
    compile_script
        .arg("SO_Compile.sh")
        .current_dir(&mediainfo_src)
        .env("TARGET", &target);
    
    let output = compile_script.output().expect("Failed to execute SO_Compile.sh");
    
    if !output.status.success() {
        panic!(
            "MediaInfo compilation failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    println!("cargo:warning=MediaInfo compilation completed successfully");
    
    // For Emscripten builds, link against the static libraries
    if target == "wasm32-unknown-emscripten" {
        let zenlib_path = mediainfo_src.join("ZenLib/Project/GNU/Library/.libs");
        let mediainfo_path = mediainfo_src.join("MediaInfoLib/Project/GNU/Library/.libs");
        
        println!("cargo:rustc-link-search=native={}", zenlib_path.display());
        println!("cargo:rustc-link-search=native={}", mediainfo_path.display());
        
        // Link the static libraries - this creates a single WASM module
        println!("cargo:rustc-link-lib=static=mediainfo");
        println!("cargo:rustc-link-lib=static=zen");
        
        println!("cargo:warning=MediaInfo built as single WASM module with static linking");
    } else {
        println!("cargo:warning=MediaInfo built for native or non-Emscripten target");
    }
}