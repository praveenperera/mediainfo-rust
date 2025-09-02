use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    build_from_source(&target);
}

fn build_from_source(target: &str) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mediainfo_src = PathBuf::from(&manifest_dir).join("mediainfo_src");
    
    println!("cargo:rerun-if-changed={}", mediainfo_src.display());
    println!("cargo:rerun-if-changed={}", mediainfo_src.join("SO_Compile.sh").display());
    
    let compile_script_path = if target.contains("windows") {
        panic!("Windows builds not yet supported - please implement Windows-specific build logic");
    } else {
        mediainfo_src.join("SO_Compile.sh")
    };
    
    // Execute the compilation script
    let mut compile_script = Command::new("bash");
    compile_script
        .arg(compile_script_path.file_name().unwrap())
        .current_dir(&mediainfo_src)
        .env("TARGET", target);
    
    let output = compile_script.output().expect("Failed to execute compilation script");
    
    if !output.status.success() {
        panic!(
            "MediaInfo compilation failed for target {}:\nstdout: {}\nstderr: {}",
            target,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    println!("cargo:warning=MediaInfo compilation completed successfully for target: {}", target);
    
    // Set up linking paths for the static libraries
    let zenlib_path = mediainfo_src.join("ZenLib/Project/GNU/Library/.libs");
    let mediainfo_path = mediainfo_src.join("MediaInfoLib/Project/GNU/Library/.libs");
    
    println!("cargo:rustc-link-search=native={}", zenlib_path.display());
    println!("cargo:rustc-link-search=native={}", mediainfo_path.display());
    
    // Link the static libraries
    println!("cargo:rustc-link-lib=static=mediainfo");
    println!("cargo:rustc-link-lib=static=zen");
    
    if target.contains("darwin") {
        // macOS system libraries that MediaInfo depends on
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=IOKit");
    } else if target.contains("linux") {
        // Linux system libraries
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=dl");
    }
    
    println!("cargo:warning=MediaInfo built with static linking for target: {}", target);
}