use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
struct BuildConfig {
    target: String,
    manifest_dir: PathBuf,
    out_dir: PathBuf,
    mediainfo_src: PathBuf,
    artifact_dir: String,
    known_target: bool,
}

#[derive(Debug, Clone)]
struct BuildPaths {
    work_dir: PathBuf,
    artifact_root: PathBuf,
    artifact_path: PathBuf,
    zenlib_artifact: PathBuf,
    mediainfo_artifact: PathBuf,
}

fn main() {
    let config = BuildConfig::new();

    if config.target.contains("windows") {
        panic!("Windows builds not yet supported - please implement Windows-specific build logic");
    }

    setup_rerun_triggers(&config);

    // Ensure required tools are present for this target before attempting build
    preflight_check_tools(&config.target);

    if config.known_target {
        let paths = BuildPaths::new(&config);

        if artifacts_exist(&paths) {
            println!(
                "cargo:warning=Using pre-built static libraries for {}",
                config.target
            );
        } else {
            println!(
                "cargo:warning=Building MediaInfo static libraries for {}",
                config.target
            );
            build_libraries(&config, &paths);
        }

        setup_linking(&paths, &config.target);
    } else {
        // Fallback: build inside OUT_DIR copy and link directly from .libs
        let paths = BuildPaths::new(&config);
        println!(
            "cargo:warning=Unknown target {}; building from source in OUT_DIR",
            config.target
        );
        prepare_build_directory(&config, &paths);
        run_build_script(&config, &paths);
        setup_linking_from_local_libs(&paths, &config.target);
    }
}

impl BuildConfig {
    fn new() -> Self {
        let target = env::var("TARGET").unwrap();
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set by cargo"));
        let mediainfo_src = manifest_dir.join("mediainfo_src");

        let (artifact_dir, known_target) = match target.as_str() {
            "aarch64-apple-darwin" => ("macos-arm64", true),
            "x86_64-apple-darwin" => ("macos-x86_64", true),
            "x86_64-unknown-linux-gnu" => ("linux-x86_64", true),
            "aarch64-unknown-linux-gnu" => ("linux-aarch64", true),
            _ => ("unknown", false),
        };

        Self {
            target,
            manifest_dir,
            out_dir,
            mediainfo_src,
            artifact_dir: artifact_dir.to_string(),
            known_target,
        }
    }
}

impl BuildPaths {
    fn new(config: &BuildConfig) -> Self {
        let work_dir = config.out_dir.join("mediainfo_build");
        let artifact_root = config.out_dir.join("artifacts");
        let artifact_path = artifact_root.join(&config.artifact_dir);

        if !artifact_path.exists() {
            fs::create_dir_all(&artifact_path).expect("Failed to create artifact directory");
        }

        let zenlib_artifact = artifact_path.join("libzen.a");
        let mediainfo_artifact = artifact_path.join("libmediainfo.a");

        Self {
            work_dir,
            artifact_root,
            artifact_path,
            zenlib_artifact,
            mediainfo_artifact,
        }
    }
}

fn setup_rerun_triggers(config: &BuildConfig) {
    let build_rs = config.manifest_dir.join("build.rs");
    let so_compile = config.mediainfo_src.join("SO_Compile.sh");

    println!("cargo:rerun-if-changed={}", config.mediainfo_src.display());
    println!("cargo:rerun-if-changed={}", build_rs.display());
    println!("cargo:rerun-if-changed={}", so_compile.display());
    // Rebuild if relevant environment variables affecting the build change
    println!("cargo:rerun-if-env-changed=MEDIAINFO_MACOSX_DEPLOYMENT_TARGET");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=TARGET");
}

fn artifacts_exist(paths: &BuildPaths) -> bool {
    paths.zenlib_artifact.exists() && paths.mediainfo_artifact.exists()
}

fn build_libraries(config: &BuildConfig, paths: &BuildPaths) {
    prepare_build_directory(config, paths);
    run_build_script(config, paths);
}

fn prepare_build_directory(config: &BuildConfig, paths: &BuildPaths) {
    let copied_src = paths.work_dir.join("mediainfo_src");

    if copied_src.exists() {
        fs::remove_dir_all(&copied_src).expect("Failed to clean previous build dir");
    }

    copy_dir_all(&config.mediainfo_src, &copied_src)
        .expect("Failed to copy mediainfo_src to OUT_DIR");
}

fn run_build_script(config: &BuildConfig, paths: &BuildPaths) {
    let copied_src = paths.work_dir.join("mediainfo_src");
    let compile_script = copied_src.join("SO_Compile.sh");
    let compile_script_full_path = compile_script
        .canonicalize()
        .expect("Failed to canonicalize compile script path");

    println!(
        "cargo:warning=Compiling MediaInfo for {}, script={}",
        config.target,
        compile_script_full_path.display()
    );

    let output = Command::new("bash")
        .arg(compile_script_full_path)
        .current_dir(&copied_src)
        .env("TARGET", &config.target)
        .env("ARTIFACT_PARENT_DIR", &paths.artifact_root)
        .output()
        .expect("Failed to execute compilation script");

    if !output.status.success() {
        panic!(
            "MediaInfo compilation failed for target {}:\nstdout: {}\nstderr: {}",
            config.target,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn setup_linking(paths: &BuildPaths, target: &str) {
    println!(
        "cargo:rustc-link-search=native={}",
        paths.artifact_path.display()
    );
    println!("cargo:rustc-link-lib=static=mediainfo");
    println!("cargo:rustc-link-lib=static=zen");

    link_system_libraries(target);

    println!("cargo:warning=MediaInfo configured for static linking with target={target}");
}

fn setup_linking_from_local_libs(paths: &BuildPaths, target: &str) {
    let mediainfo_lib_dir = paths
        .work_dir
        .join("mediainfo_src/MediaInfoLib/Project/GNU/Library/.libs");
    let zenlib_lib_dir = paths
        .work_dir
        .join("mediainfo_src/ZenLib/Project/GNU/Library/.libs");

    println!(
        "cargo:rustc-link-search=native={}",
        mediainfo_lib_dir.display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        zenlib_lib_dir.display()
    );
    println!("cargo:rustc-link-lib=static=mediainfo");
    println!("cargo:rustc-link-lib=static=zen");

    link_system_libraries(target);

    println!(
        "cargo:warning=MediaInfo configured for static linking with local .libs for target={target}"
    );
}

fn link_system_libraries(target: &str) {
    if target.contains("darwin") {
        // macOS system libraries
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
}

fn build_from_source_fallback(mediainfo_src: &PathBuf, target: &str, out_dir: &PathBuf) {
    println!("cargo:warning=Building MediaInfo for fallback target {target}");
    let compile_script_path = mediainfo_src.join("SO_Compile.sh");
    let compile_script_full_path = compile_script_path
        .canonicalize()
        .expect("Failed to canonicalize compile script path");

    let output = Command::new("bash")
        .arg(compile_script_full_path)
        .current_dir(mediainfo_src)
        .env("TARGET", target)
        .env("ARTIFACT_PARENT_DIR", out_dir)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildType {
    Native,
    Emscripten,
    WasmBindgen,
}

fn detect_build_type(target: &str) -> BuildType {
    if target == "wasm32-unknown-emscripten" || target == "wasm32-wasi" {
        BuildType::Emscripten
    } else if target == "wasm32-unknown-unknown" {
        BuildType::WasmBindgen
    } else {
        BuildType::Native
    }
}

fn preflight_check_tools(target: &str) {
    let build_type = detect_build_type(target);
    let mut required: Vec<&str> = vec!["bash", "make"];

    match build_type {
        BuildType::Emscripten => {
            required.extend(["emconfigure", "emmake", "em++"].iter().copied());
        }
        BuildType::WasmBindgen => {
            required.extend(["clang", "clang++", "ar"].iter().copied());
        }
        BuildType::Native => {
            if target.contains("apple-darwin") {
                required.extend(["clang", "clang++", "ar"].iter().copied());
            } else if target.contains("linux") {
                // Prefer generic drivers so either gcc/clang works
                required.extend(["cc", "c++", "ar"].iter().copied());
            }
        }
    }

    let mut missing: Vec<&str> = Vec::new();
    for tool in required {
        if !command_exists(tool) {
            missing.push(tool);
        }
    }

    if !missing.is_empty() {
        let hint = match build_type {
            BuildType::Emscripten => "Install Emscripten and activate env (source emsdk_env.sh).",
            BuildType::WasmBindgen => {
                if target.contains("apple-darwin") {
                    "Install Xcode Command Line Tools (xcode-select --install)."
                } else {
                    "Install clang/LLVM and binutils (ar)."
                }
            }
            BuildType::Native => {
                if target.contains("apple-darwin") {
                    "Install Xcode Command Line Tools (xcode-select --install)."
                } else if target.contains("linux") {
                    "Install build tools (make, binutils) and a C/C++ compiler (gcc/g++ or clang/clang++)."
                } else {
                    "Install a C/C++ toolchain and make."
                }
            }
        };

        panic!(
            "Required tools missing for target {}: {}\n{}",
            target,
            missing.join(", "),
            hint
        );
    }
}

fn command_exists(tool: &str) -> bool {
    // Use shell builtin `command -v` to detect presence
    match Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {} >/dev/null 2>&1", tool))
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
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
        }
        // Skip symlinks and others for simplicity
    }
    Ok(())
}
