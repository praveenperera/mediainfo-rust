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

// Bridge functions that will be called from Rust via extern "C"
// These need to be exposed to the global scope for Rust to access

// MediaInfo instance management
window.mediainfo_new = function() {
    if (!mediaInfoInitialized) return 0;
    try {
        const instance = new mediaInfoInstance.MediaInfo();
        // Store instance reference and return a handle
        if (!window.mediaInfoInstances) {
            window.mediaInfoInstances = new Map();
            window.mediaInfoNextHandle = 1;
        }
        const handle = window.mediaInfoNextHandle++;
        window.mediaInfoInstances.set(handle, instance);
        return handle;
    } catch (error) {
        console.error('Failed to create MediaInfo instance:', error);
        return 0;
    }
};

window.mediainfo_delete = function(handle) {
    if (!window.mediaInfoInstances) return;
    const instance = window.mediaInfoInstances.get(handle);
    if (instance) {
        try {
            instance.delete();
            window.mediaInfoInstances.delete(handle);
        } catch (error) {
            console.error('Failed to delete MediaInfo instance:', error);
        }
    }
};

// Buffer-based file opening for WASM
window.mediainfo_open_buffer_init = function(handle, fileSize, fileOffset) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Init(fileSize, fileOffset) ? 1 : 0;
    } catch (error) {
        console.error('Failed to init buffer:', error);
        return 0;
    }
};

window.mediainfo_open_buffer_continue = function(handle, bufferPtr, bufferSize) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        // Get buffer from WASM memory
        const buffer = new Uint8Array(Module.HEAPU8.buffer, bufferPtr, bufferSize);
        return instance.Open_Buffer_Continue(buffer);
    } catch (error) {
        console.error('Failed to continue buffer:', error);
        return 0;
    }
};

window.mediainfo_open_buffer_finalize = function(handle) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Finalize() ? 1 : 0;
    } catch (error) {
        console.error('Failed to finalize buffer:', error);
        return 0;
    }
};

window.mediainfo_open_buffer_continue_goto_get = function(handle) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Continue_GoTo_Get();
    } catch (error) {
        console.error('Failed to get goto position:', error);
        return 0;
    }
};

window.mediainfo_open_buffer_continue_goto_get_lower = function(handle) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Continue_GoTo_Get_Lower();
    } catch (error) {
        console.error('Failed to get goto lower position:', error);
        return 0;
    }
};

window.mediainfo_open_buffer_continue_goto_get_upper = function(handle) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Open_Buffer_Continue_GoTo_Get_Upper();
    } catch (error) {
        console.error('Failed to get goto upper position:', error);
        return 0;
    }
};

// Information retrieval
window.mediainfo_get = function(handle, streamKind, streamNumber, parameterPtr, infoKindPtr, searchKindPtr) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        // Convert C strings from WASM memory
        const parameter = parameterPtr ? UTF8ToString(parameterPtr) : "";
        const infoKind = infoKindPtr ? UTF8ToString(infoKindPtr) : "";
        const searchKind = searchKindPtr ? UTF8ToString(searchKindPtr) : "";
        
        const result = instance.Get(streamKind, streamNumber, parameter, infoKind, searchKind);
        
        // Allocate WASM memory for result string
        const resultPtr = Module._malloc(lengthBytesUTF8(result) + 1);
        stringToUTF8(result, resultPtr, lengthBytesUTF8(result) + 1);
        return resultPtr;
    } catch (error) {
        console.error('Failed to get info:', error);
        return 0;
    }
};

window.mediainfo_count_get = function(handle, streamKind) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        return instance.Count_Get(streamKind);
    } catch (error) {
        console.error('Failed to get count:', error);
        return 0;
    }
};

window.mediainfo_inform = function(handle) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        const result = instance.Inform();
        // Allocate WASM memory for result string
        const resultPtr = Module._malloc(lengthBytesUTF8(result) + 1);
        stringToUTF8(result, resultPtr, lengthBytesUTF8(result) + 1);
        return resultPtr;
    } catch (error) {
        console.error('Failed to get inform:', error);
        return 0;
    }
};

window.mediainfo_option = function(handle, optionPtr, valuePtr) {
    const instance = window.mediaInfoInstances?.get(handle);
    if (!instance) return 0;
    try {
        const option = optionPtr ? UTF8ToString(optionPtr) : "";
        const value = valuePtr ? UTF8ToString(valuePtr) : "";
        
        const result = instance.Option(option, value);
        
        // Allocate WASM memory for result string
        const resultPtr = Module._malloc(lengthBytesUTF8(result) + 1);
        stringToUTF8(result, resultPtr, lengthBytesUTF8(result) + 1);
        return resultPtr;
    } catch (error) {
        console.error('Failed to set option:', error);
        return 0;
    }
};

// Free allocated strings
window.mediainfo_free_string = function(ptr) {
    if (ptr) {
        Module._free(ptr);
    }
};

// Export initialization for ES modules
export { initMediaInfo as default };