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
    
    if mediainfo_lib_path.exists() {
        println!("cargo:warning=Found MediaInfo library path: {}", mediainfo_lib_path.display());
        println!("cargo:rustc-link-search=native={}", mediainfo_lib_path.display());
    } else {
        println!("cargo:warning=MediaInfo library path not found: {}", mediainfo_lib_path.display());
    }
    
    if zenlib_path.exists() {
        println!("cargo:warning=Found ZenLib library path: {}", zenlib_path.display());
        println!("cargo:rustc-link-search=native={}", zenlib_path.display());
    } else {
        println!("cargo:warning=ZenLib library path not found: {}", zenlib_path.display());
    }
    
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
    
    // Export symbols so they don't get imported from env
    println!("cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=_MediaInfo_New,_MediaInfo_Delete,_MediaInfo_Open_Buffer_Init,_MediaInfo_Open_Buffer_Continue,_MediaInfo_Open_Buffer_Continue_GoTo_Get,_MediaInfo_Open_Buffer_Finalize,_MediaInfo_Open,_MediaInfo_Close,_MediaInfo_Option,_MediaInfo_Inform,_MediaInfo_Count_Get,_MediaInfo_Get,_setlocale");
    
    // Don't import these functions from env - they should be resolved from static libs
    println!("cargo:rustc-link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0");
}
