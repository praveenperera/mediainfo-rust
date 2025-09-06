use std::env;
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    build_from_source(&target);
}

fn build_from_source(target: &str) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set by cargo"));
    let mediainfo_src = PathBuf::from(&manifest_dir).join("mediainfo_src");
    let build_rs = PathBuf::from(&manifest_dir).join("build.rs");
    let so_compile = mediainfo_src.join("SO_Compile.sh");

    println!("cargo:rerun-if-changed={}", mediainfo_src.display());
    println!("cargo:rerun-if-changed={}", build_rs.display());
    println!("cargo:rerun-if-changed={}", so_compile.display());

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
            build_single_target(&mediainfo_src, target, &out_dir);
            return;
        }
    };

    // We'll build in a writable temp dir under OUT_DIR and place artifacts there
    let work_dir = out_dir.join("mediainfo_build");
    let artifact_root = out_dir.join("artifacts");
    let artifact_path = artifact_root.join(artifact_dir);

    println!("cargo:rerun-if-changed={}", artifact_path.display());

    if !artifact_path.exists() {
        fs::create_dir_all(&artifact_path).expect("Failed to create artifact directory");
    }

    // Check if pre-built artifacts exist
    let zenlib_artifact = artifact_path.join("libzen.a");
    let mediainfo_artifact = artifact_path.join("libmediainfo.a");

    if zenlib_artifact.exists() && mediainfo_artifact.exists() {
        println!("cargo:info=Using pre-built static libraries for {target}");
    } else {
        println!("cargo:info=Building MediaInfo static libraries for {target}");
        // Prepare an isolated, writable copy of mediainfo_src under OUT_DIR
        let copied_src = work_dir.join("mediainfo_src");
        if copied_src.exists() {
            fs::remove_dir_all(&copied_src).expect("Failed to clean previous build dir");
        }

        copy_dir_all(&mediainfo_src, &copied_src).expect("Failed to copy mediainfo_src to OUT_DIR");

        // Build using the copied source so we don't write into the crate source (read-only in registry)
        build_single_target(&copied_src, target, &artifact_root);
    }

    // Set up linking for the specific target
    setup_linking(&artifact_path, target);
}

fn build_single_target(mediainfo_src: &PathBuf, target: &str, artifact_parent: &Path) {
    println!("cargo:warning=Building MediaInfo for single {target}");
    let compile_script_path = mediainfo_src.join("SO_Compile.sh");

    // Execute the compilation script for the specific target
    let compile_script_full_path = compile_script_path
        .canonicalize()
        .expect("Failed to canonicalize compile script path");

    println!(
        "cargo:warning=Compiling MediaInfo for single {}, target={target}",
        compile_script_full_path.display()
    );
    // Invoke via sh for portability across filesystems preserving exec bits
    let mut compile_script = Command::new("sh");
    compile_script
        .arg(compile_script_full_path)
        .current_dir(mediainfo_src)
        .env("TARGET", target)
        .env("ARTIFACT_PARENT_DIR", artifact_parent);

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

    println!("cargo:warning=MediaInfo configured for static linking with target={target}",);
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
        } else {
            // Skip symlinks and others for simplicity
        }
    }
    Ok(())
}
