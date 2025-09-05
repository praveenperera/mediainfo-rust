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

    let compile_script_path = if target.contains("windows") {
        panic!("Windows builds not yet supported - please implement Windows-specific build logic");
    } else {
        mediainfo_src.join("SO_Compile.sh")
    };

    if target.contains("darwin") {
        build_universal_macos(&mediainfo_src, &compile_script_path, target);
    } else {
        // Execute the compilation script for non-macOS targets
        let mut compile_script = Command::new("bash");
        compile_script
            .arg(compile_script_path.file_name().unwrap())
            .current_dir(&mediainfo_src)
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
}

fn build_universal_macos(mediainfo_src: &PathBuf, compile_script_path: &PathBuf, target: &str) {
    let architectures = ["x86_64", "arm64"];
    let mut lib_paths = Vec::new();

    for arch in &architectures {
        println!(
            "cargo:warning=Building MediaInfo static libraries for {}",
            arch
        );

        // Clean previous builds
        let _ = Command::new("make")
            .arg("clean")
            .current_dir(mediainfo_src.join("ZenLib/Project/GNU/Library"))
            .output();
        let _ = Command::new("make")
            .arg("clean")
            .current_dir(mediainfo_src.join("MediaInfoLib/Project/GNU/Library"))
            .output();

        // Set architecture-specific environment variables
        let mut compile_script = Command::new("bash");
        compile_script
            .arg(compile_script_path.file_name().unwrap())
            .current_dir(&mediainfo_src)
            .env("TARGET", format!("{}-apple-darwin", arch))
            .env("ARCHFLAGS", format!("-arch {}", arch));

        if env::var("CFLAGS").is_err() {
            compile_script.env(
                "CFLAGS",
                format!("-arch {} -mmacosx-version-min=10.9", arch),
            );
        }

        if env::var("CXXFLAGS").is_err() {
            compile_script.env(
                "CXXFLAGS",
                format!("-arch {} -mmacosx-version-min=10.9", arch),
            );
        }

        compile_script.env("LDFLAGS", format!("-arch {}", arch));

        let output = compile_script
            .output()
            .expect("Failed to execute compilation script");

        if !output.status.success() {
            panic!(
                "MediaInfo compilation failed for {}:\nstdout: {}\nstderr: {}",
                arch,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Create architecture-specific directories and copy libraries
        let arch_dir = mediainfo_src.join(format!("libs_{}", arch));
        std::fs::create_dir_all(&arch_dir).expect("Failed to create architecture directory");

        let zenlib_src = mediainfo_src.join("ZenLib/Project/GNU/Library/.libs/libzen.a");
        let mediainfo_src_lib =
            mediainfo_src.join("MediaInfoLib/Project/GNU/Library/.libs/libmediainfo.a");

        let zenlib_dest = arch_dir.join("libzen.a");
        let mediainfo_dest = arch_dir.join("libmediainfo.a");

        std::fs::copy(&zenlib_src, &zenlib_dest).expect("Failed to copy libzen.a");
        std::fs::copy(&mediainfo_src_lib, &mediainfo_dest).expect("Failed to copy libmediainfo.a");

        lib_paths.push((arch.to_string(), zenlib_dest, mediainfo_dest));
    }

    // Create universal libraries using lipo
    let universal_dir = mediainfo_src.join("libs_universal");
    std::fs::create_dir_all(&universal_dir).expect("Failed to create universal directory");

    let universal_zenlib = universal_dir.join("libzen.a");
    let universal_mediainfo = universal_dir.join("libmediainfo.a");

    // Create universal libzen.a
    let mut lipo_zen = Command::new("lipo");
    lipo_zen
        .arg("-create")
        .arg("-output")
        .arg(&universal_zenlib);
    for (_, zenlib_path, _) in &lib_paths {
        lipo_zen.arg(zenlib_path);
    }

    let output = lipo_zen
        .output()
        .expect("Failed to execute lipo for libzen.a");
    if !output.status.success() {
        panic!(
            "Failed to create universal libzen.a:\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Create universal libmediainfo.a
    let mut lipo_mediainfo = Command::new("lipo");
    lipo_mediainfo
        .arg("-create")
        .arg("-output")
        .arg(&universal_mediainfo);
    for (_, _, mediainfo_path) in &lib_paths {
        lipo_mediainfo.arg(mediainfo_path);
    }

    let output = lipo_mediainfo
        .output()
        .expect("Failed to execute lipo for libmediainfo.a");
    if !output.status.success() {
        panic!(
            "Failed to create universal libmediainfo.a:\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    println!("cargo:warning=Successfully created universal static libraries for macOS");

    // Update the library search path to use universal libraries
    println!("cargo:rustc-link-search=native={}", universal_dir.display());

    // Set up linking paths for the static libraries
    if target.contains("darwin") {
        // Universal libraries are already set up in build_universal_macos
    } else {
        let zenlib_path = mediainfo_src.join("ZenLib/Project/GNU/Library/.libs");
        let mediainfo_path = mediainfo_src.join("MediaInfoLib/Project/GNU/Library/.libs");

        println!("cargo:rustc-link-search=native={}", zenlib_path.display());
        println!(
            "cargo:rustc-link-search=native={}",
            mediainfo_path.display()
        );
    }

    // Link the static libraries
    println!("cargo:rustc-link-lib=static=mediainfo");
    println!("cargo:rustc-link-lib=static=zen");

    // Link zlib (required by MediaInfo)
    println!("cargo:rustc-link-lib=z");

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

    println!(
        "cargo:warning=MediaInfo built with static linking for target: {}",
        target
    );
}
