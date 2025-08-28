// MediaInfo Bridge Module
// This module loads the MediaInfo WASM and provides a bridge to Rust WASM

import MediaInfoLib from './mediainfo_src/MediaInfoLib/Project/GNU/Library/MediaInfoWasm.js';

let mediaInfoInstance = null;
let mediaInfoInitialized = false;

// Initialize MediaInfo module
export async function initMediaInfo() {
    if (!mediaInfoInitialized) {
        try {
            mediaInfoInstance = await MediaInfoLib();
            mediaInfoInitialized = true;
            return true;
        } catch (error) {
            console.error('Failed to initialize MediaInfo:', error);
            return false;
        }
    }
    return true;
}

// Bridge functions that will be called from Rust via wasm-bindgen
// These need to be exported as ES module functions for wasm-bindgen

// MediaInfo instance management
export function mediainfo_new() {
    if (!mediaInfoInitialized) return 0;
    try {
        const instance = new mediaInfoInstance.MediaInfo();
        // Store instance reference and return a handle
        if (!globalThis.mediaInfoInstances) {
            globalThis.mediaInfoInstances = new Map();
            globalThis.mediaInfoNextHandle = 1;
        }
        const handle = globalThis.mediaInfoNextHandle++;
        globalThis.mediaInfoInstances.set(handle, instance);
        return handle;
    } catch (error) {
        console.error('Failed to create MediaInfo instance:', error);
        return 0;
    }
}

export function mediainfo_delete(handle) {
    if (!globalThis.mediaInfoInstances) return;
    const instance = globalThis.mediaInfoInstances.get(handle);
    if (instance) {
        try {
            instance.delete();
            globalThis.mediaInfoInstances.delete(handle);
        } catch (error) {
            console.error('Failed to delete MediaInfo instance:', error);
        }
    }
}

// Buffer-based file opening for WASM
export function mediainfo_open_buffer_init(handle, fileSize, fileOffset) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Init(fileSize, fileOffset) ? 1 : 0;
    } catch (error) {
        console.error('Failed to init buffer:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_continue(handle, buffer) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        // wasm-bindgen passes the buffer directly as Uint8Array
        return instance.Open_Buffer_Continue(buffer);
    } catch (error) {
        console.error('Failed to continue buffer:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_finalize(handle) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Finalize() ? 1 : 0;
    } catch (error) {
        console.error('Failed to finalize buffer:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_continue_goto_get(handle) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Continue_GoTo_Get();
    } catch (error) {
        console.error('Failed to get goto position:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_continue_goto_get_lower(handle) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Continue_GoTo_Get_Lower();
    } catch (error) {
        console.error('Failed to get goto lower position:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_continue_goto_get_upper(handle) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Continue_GoTo_Get_Upper();
    } catch (error) {
        console.error('Failed to get goto upper position:', error);
        return 0;
    }
}

// Information retrieval
export function mediainfo_get(handle, streamKind, streamNumber, parameter, infoKind, searchKind) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return "";
    try {
        // wasm-bindgen passes strings directly
        const result = instance.Get(streamKind, streamNumber, parameter || "", infoKind || "", searchKind || "");
        return result || "";
    } catch (error) {
        console.error('Failed to get info:', error);
        return "";
    }
}

export function mediainfo_count_get(handle, streamKind) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Count_Get(streamKind);
    } catch (error) {
        console.error('Failed to get count:', error);
        return 0;
    }
}

export function mediainfo_inform(handle) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return "";
    try {
        const result = instance.Inform();
        return result || "";
    } catch (error) {
        console.error('Failed to get inform:', error);
        return "";
    }
}

export function mediainfo_option(handle, option, value) {
    const instance = globalThis.mediaInfoInstances?.get(handle);
    if (!instance) return "";
    try {
        // wasm-bindgen passes strings directly
        const result = instance.Option(option || "", value || "");
        return result || "";
    } catch (error) {
        console.error('Failed to set option:', error);
        return "";
    }
}

// Free allocated strings - for wasm-bindgen, this is typically handled automatically
export function mediainfo_free_string(_ptr) {
    // With wasm-bindgen string marshalling, manual freeing is usually not needed
    // This function is kept for API compatibility
}

// Export initialization for ES modules
export { initMediaInfo as default };