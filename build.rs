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
        
        // Emscripten-specific flags for single WASM module
        println!("cargo:rustc-link-arg=-sSTANDALONE_WASM=1");
        println!("cargo:rustc-link-arg=-sNO_FILESYSTEM=1");
        println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH=1");
        println!("cargo:rustc-link-arg=-sMALLOC=emmalloc");
        
        // This is the key: export the MediaInfo functions so they're available in the WASM module
        // instead of being imported from env
        println!("cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=_MediaInfo_New,_MediaInfo_Delete,_MediaInfo_Open_Buffer_Init,_MediaInfo_Open_Buffer_Continue,_MediaInfo_Open_Buffer_Continue_GoTo_Get,_MediaInfo_Open_Buffer_Finalize,_MediaInfo_Open,_MediaInfo_Close,_MediaInfo_Option,_MediaInfo_Inform,_MediaInfo_Count_Get,_MediaInfo_Get");
        
        println!("cargo:warning=MediaInfo built as single WASM module with static linking");
    } else {
        println!("cargo:warning=MediaInfo built for native or non-Emscripten target");
    }
}