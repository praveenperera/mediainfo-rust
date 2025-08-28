use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs;

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
    
    // Generate embedded web worker bridge
    generate_embedded_worker_bridge(&manifest_dir, &mediainfo_src);
    
    // For Emscripten builds, link against the static libraries
        let zenlib_path = mediainfo_src.join("ZenLib/Project/GNU/Library/.libs");
        let mediainfo_path = mediainfo_src.join("MediaInfoLib/Project/GNU/Library/.libs");
        
        println!("cargo:rustc-link-search=native={}", zenlib_path.display());
        println!("cargo:rustc-link-search=native={}", mediainfo_path.display());
        
        // Link the static libraries - this creates a single WASM module
        println!("cargo:rustc-link-lib=static=mediainfo");
        println!("cargo:rustc-link-lib=static=zen");
        
        println!("cargo:warning=MediaInfo built as single WASM module with static linking");
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let b1 = chunk[0] as u32;
        let b2 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b3 = chunk.get(2).copied().unwrap_or(0) as u32;
        
        let combined = (b1 << 16) | (b2 << 8) | b3;
        
        result.push(CHARS[((combined >> 18) & 63) as usize] as char);
        result.push(CHARS[((combined >> 12) & 63) as usize] as char);
        result.push(if chunk.len() > 1 { CHARS[((combined >> 6) & 63) as usize] as char } else { '=' });
        result.push(if chunk.len() > 2 { CHARS[(combined & 63) as usize] as char } else { '=' });
    }
    
    result
}

fn generate_embedded_worker_bridge(manifest_dir: &str, mediainfo_src: &PathBuf) {
    let worker_template_path = PathBuf::from(manifest_dir).join("mediainfo-worker-template.js");
    let bridge_template_path = PathBuf::from(manifest_dir).join("mediainfo-bridge-worker.js");
    
    if !worker_template_path.exists() || !bridge_template_path.exists() {
        println!("cargo:warning=Skipping worker bridge generation - missing template files");
        return;
    }
    
    // Read template files
    let worker_template = fs::read_to_string(&worker_template_path).expect("Failed to read worker template");
    let bridge_template = fs::read_to_string(&bridge_template_path).expect("Failed to read bridge template");
    
    // Check for MediaInfo JS and WASM files
    let mediainfo_lib_dir = mediainfo_src.join("MediaInfoLib/Project/GNU/Library");
    let wasm_js_path = mediainfo_lib_dir.join("MediaInfoWasm.js");
    let wasm_file_path = mediainfo_lib_dir.join("MediaInfoWasm.wasm");
    
    if !wasm_js_path.exists() || !wasm_file_path.exists() {
        println!("cargo:warning=Skipping worker bridge generation - missing MediaInfo WASM files");
        return;
    }
    
    // Read the MediaInfo JS and WASM files
    let mediainfo_js = fs::read_to_string(&wasm_js_path).expect("Failed to read MediaInfoWasm.js");
    let mediainfo_wasm = fs::read(&wasm_file_path).expect("Failed to read MediaInfoWasm.wasm");
    
    // Convert WASM to base64
    let wasm_base64 = base64_encode(&mediainfo_wasm);
    
    // Generate embedded assets for the worker
    let embedded_assets = generate_worker_embedded_assets(&mediainfo_js, &wasm_base64);
    
    // Replace placeholder in worker template
    let embedded_worker_code = worker_template.replace("// EMBED_MEDIAINFO_ASSETS", &embedded_assets);
    
    // Generate the main bridge with embedded worker
    let embedded_bridge = bridge_template.replace(
        "function getEmbeddedWorkerCode() {\n    // EMBEDDED_WORKER_CODE_PLACEHOLDER\n    throw new Error('Worker code not embedded. This should be replaced during build.');\n}",
        &format!("function getEmbeddedWorkerCode() {{\n    return `{}`;\n}}", embedded_worker_code.replace("\\", "\\\\").replace('`', r#"\`"#).replace("${", r#"\${"#))
    );
    
    // Write the embedded worker bridge
    let out_path = PathBuf::from(manifest_dir).join("mediainfo-bridge-worker-embedded.js");
    fs::write(&out_path, embedded_bridge).expect("Failed to write embedded worker bridge");
    
    println!("cargo:warning=Generated embedded worker bridge at {}", out_path.display());
}

fn generate_worker_embedded_assets(mediainfo_js: &str, wasm_base64: &str) -> String {
    format!(r#"
// Embedded MediaInfo WASM and JS
const MEDIAINFO_WASM_BASE64 = "{}";
const MEDIAINFO_JS_CODE = `{}`;

// Create MediaInfoLib from embedded content
let MediaInfoLib;

// Function to initialize MediaInfo from embedded assets
async function initEmbeddedMediaInfo() {{
    // Convert base64 to ArrayBuffer
    const binaryString = atob(MEDIAINFO_WASM_BASE64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {{
        bytes[i] = binaryString.charCodeAt(i);
    }}
    
    // Create a module with the embedded WASM
    const wasmModule = await WebAssembly.instantiate(bytes);
    
    // Execute MediaInfo JS code in context
    const moduleFunc = new Function('Module', MEDIAINFO_JS_CODE + '; return Module;');
    MediaInfoLib = moduleFunc(wasmModule);
    
    return MediaInfoLib;
}}
"#, wasm_base64, mediainfo_js.replace("\\", "\\\\").replace('`', r#"\`"#).replace("${", r#"\${"#))
}

fn generate_bridge_with_embedded_assets(bridge_template: &str, mediainfo_js: &str, wasm_base64: &str) -> String {
    // Replace the import with embedded content
    let embedded_content = format!(r#"
// Embedded MediaInfo WASM and JS
const MEDIAINFO_WASM_BASE64 = "{}";
const MEDIAINFO_JS_CODE = `{}`;

// Create MediaInfoLib from embedded content
let MediaInfoLib;

// Function to initialize MediaInfo from embedded assets
async function initEmbeddedMediaInfo() {{
    // Convert base64 to ArrayBuffer
    const binaryString = atob(MEDIAINFO_WASM_BASE64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {{
        bytes[i] = binaryString.charCodeAt(i);
    }}
    
    // Create a module with the embedded WASM
    const wasmModule = await WebAssembly.instantiate(bytes);
    
    // Execute MediaInfo JS code in context
    const moduleFunc = new Function('Module', MEDIAINFO_JS_CODE + '; return Module;');
    MediaInfoLib = moduleFunc(wasmModule);
    
    return MediaInfoLib;
}}
"#, wasm_base64, mediainfo_js.replace("\\", "\\\\").replace('`', r#"\`"#).replace("${", r#"\${"#));

    // Replace the import line with embedded content
    bridge_template.replace(
        r#"import MediaInfoLib from './mediainfo_src/MediaInfoLib/Project/GNU/Library/MediaInfoWasm.js';"#,
        &embedded_content
    ).replace(
        "export async function initMediaInfo() {\n    if (!mediaInfoInitialized) {\n        try {\n            mediaInfoInstance = await MediaInfoLib();\n            mediaInfoInitialized = true;\n            return true;",
        "export async function initMediaInfo() {\n    if (!mediaInfoInitialized) {\n        try {\n            if (!MediaInfoLib) {\n                await initEmbeddedMediaInfo();\n            }\n            mediaInfoInstance = await MediaInfoLib();\n            mediaInfoInitialized = true;\n            return true;"
    )
}