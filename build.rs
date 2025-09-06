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
    println!(
        "cargo:rerun-if-changed={}",
        mediainfo_src.join("SO_Compile.sh").display()
    );

    if target.contains("windows") {
        panic!("Windows builds not yet supported - please implement Windows-specific build logic");
    }

    // Map target to artifact directory
    let artifact_dir = match target {
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x86_64",
        "x86_64-unknown-linux-gnu" => "linux-x86_64",
        "aarch64-unknown-linux-gnu" => "linux-aarch64",
        _ => {
            println!("cargo:warning=Unknown target {target}, falling back to source build",);
            build_single_target(&mediainfo_src, target);
            return;
        }
    };

    let artifact_path = PathBuf::from(&manifest_dir)
        .join("artifacts")
        .join(artifact_dir);

    println!("cargo:rerun-if-changed={}", artifact_path.display());
    if !artifact_path.exists() {
        std::fs::create_dir_all(&artifact_path).expect("Failed to create artifact directory");
    }

    // Check if pre-built artifacts exist
    let zenlib_artifact = artifact_path.join("libzen.a");
    let mediainfo_artifact = artifact_path.join("libmediainfo.a");

    if zenlib_artifact.exists() && mediainfo_artifact.exists() {
        println!("cargo:info=Using pre-built static libraries for {target}");
    } else {
        println!("cargo:info=Building MediaInfo static libraries for {target}",);
        build_single_target(&mediainfo_src, target);
    }

    // Set up linking for the specific target
    setup_linking(&artifact_path, target);
}

fn build_single_target(mediainfo_src: &PathBuf, target: &str) {
    println!("cargo:info=Building MediaInfo for single {target}");
    let compile_script_path = mediainfo_src.join("SO_Compile.sh");

    // Execute the compilation script for the specific target
    let mut compile_script = Command::new("bash");
    compile_script
        .arg(compile_script_path.file_name().unwrap())
        .current_dir(mediainfo_src)
        .env("TARGET", target);

    let output = compile_script
        .output()
        .expect("Failed to execute compilation script");

    if !output.status.success() {
        panic!(
            "MediaInfo compilation failed for target {}:\nstdout: {}\nstderr: {}",
            target,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn setup_linking(artifact_path: &PathBuf, target: &str) {
    // Add search path for the target-specific artifacts
    println!("cargo:rustc-link-search=native={}", artifact_path.display());

    // Link the static libraries
    println!("cargo:rustc-link-lib=static=mediainfo");
    println!("cargo:rustc-link-lib=static=zen");

    // Add OS-specific system libraries
    if target.contains("darwin") {
        // macOS system libraries that MediaInfo depends on
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=z");
    } else if target.contains("linux") {
        // Linux system libraries
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=dl");
        println!("cargo:rustc-link-lib=z");
    }

    println!(
        "cargo:warning=MediaInfo configured for static linking with target: {}",
        target
    );
}
