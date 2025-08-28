// MediaInfo Worker Bridge (Synchronous API)
// This module creates a web worker for MediaInfo WASM and provides synchronous bridge functions

let worker = null;
let workerInitialized = false;
let requestId = 1;
let responseCache = new Map();

// Initialize the MediaInfo worker
export function initMediaInfo() {
    if (workerInitialized) return true;
    
    try {
        // Create worker from embedded code
        const workerCode = getEmbeddedWorkerCode();
        const blob = new Blob([workerCode], { type: 'application/javascript' });
        const workerUrl = URL.createObjectURL(blob);
        
        worker = new Worker(workerUrl);
        
        // Handle worker messages
        worker.onmessage = (event) => {
            const { id, success, result, error } = event.data;
            responseCache.set(id, { success, result, error });
        };
        
        // Handle worker errors
        worker.onerror = (error) => {
            console.error('MediaInfo worker error:', error);
        };
        
        // Initialize MediaInfo in the worker (synchronously)
        const initResult = sendWorkerMessageSync('init', {});
        if (!initResult.success) {
            throw new Error('Failed to initialize worker');
        }
        
        workerInitialized = true;
        
        // Clean up the blob URL
        URL.revokeObjectURL(workerUrl);
        
        return true;
    } catch (error) {
        console.error('Failed to initialize MediaInfo worker:', error);
        return false;
    }
}

// Send a message to the worker and wait synchronously for response
function sendWorkerMessageSync(method, params) {
    if (!worker) {
        return { success: false, error: 'Worker not initialized' };
    }
    
    const id = requestId++;
    worker.postMessage({ id, method, params });
    
    // Busy wait for response (this works in WASM single-threaded environment)
    const maxWait = 10000; // 10 seconds
    const startTime = Date.now();
    
    while (!responseCache.has(id)) {
        if (Date.now() - startTime > maxWait) {
            return { success: false, error: 'Request timeout' };
        }
        
        // Allow other tasks to run
        // In a real environment, this would be replaced with proper sync/await handling
        continue;
    }
    
    const response = responseCache.get(id);
    responseCache.delete(id);
    return response;
}

// Bridge functions - these maintain the synchronous API
export function mediainfo_new() {
    try {
        const response = sendWorkerMessageSync('new', {});
        return response.success ? response.result : 0;
    } catch (error) {
        console.error('Failed to create MediaInfo instance:', error);
        return 0;
    }
}

export function mediainfo_delete(handle) {
    try {
        const response = sendWorkerMessageSync('delete', { handle });
        return response.success ? response.result : false;
    } catch (error) {
        console.error('Failed to delete MediaInfo instance:', error);
    }
}

export function mediainfo_open_buffer_init(handle, fileSize, fileOffset) {
    try {
        const response = sendWorkerMessageSync('openBufferInit', { handle, fileSize, fileOffset });
        return response.success ? response.result : 0;
    } catch (error) {
        console.error('Failed to init buffer:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_continue(handle, buffer) {
    try {
        // Convert buffer to transferable array
        const bufferArray = Array.from(buffer);
        const response = sendWorkerMessageSync('openBufferContinue', { handle, buffer: bufferArray });
        return response.success ? response.result : 0;
    } catch (error) {
        console.error('Failed to continue buffer:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_finalize(handle) {
    try {
        const response = sendWorkerMessageSync('openBufferFinalize', { handle });
        return response.success ? response.result : 0;
    } catch (error) {
        console.error('Failed to finalize buffer:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_continue_goto_get(handle) {
    try {
        const response = sendWorkerMessageSync('openBufferContinueGoToGet', { handle });
        return response.success ? response.result : 0;
    } catch (error) {
        console.error('Failed to get goto position:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_continue_goto_get_lower(handle) {
    try {
        const response = sendWorkerMessageSync('openBufferContinueGoToGetLower', { handle });
        return response.success ? response.result : 0;
    } catch (error) {
        console.error('Failed to get goto lower position:', error);
        return 0;
    }
}

export function mediainfo_open_buffer_continue_goto_get_upper(handle) {
    try {
        const response = sendWorkerMessageSync('openBufferContinueGoToGetUpper', { handle });
        return response.success ? response.result : 0;
    } catch (error) {
        console.error('Failed to get goto upper position:', error);
        return 0;
    }
}

export function mediainfo_get(handle, streamKind, streamNumber, parameter, infoKind, searchKind) {
    try {
        const response = sendWorkerMessageSync('get', { 
            handle, streamKind, streamNumber, parameter, infoKind, searchKind 
        });
        return response.success ? response.result : "";
    } catch (error) {
        console.error('Failed to get info:', error);
        return "";
    }
}

export function mediainfo_count_get(handle, streamKind, streamNumber) {
    try {
        const response = sendWorkerMessageSync('countGet', { handle, streamKind, streamNumber });
        return response.success ? response.result : 0;
    } catch (error) {
        console.error('Failed to get count:', error);
        return 0;
    }
}

export function mediainfo_inform(handle) {
    try {
        const response = sendWorkerMessageSync('inform', { handle });
        return response.success ? response.result : "";
    } catch (error) {
        console.error('Failed to get inform:', error);
        return "";
    }
}

export function mediainfo_option(handle, parameter, value) {
    try {
        const response = sendWorkerMessageSync('option', { handle, parameter, value });
        return response.success ? response.result : "";
    } catch (error) {
        console.error('Failed to set option:', error);
        return "";
    }
}

export function mediainfo_free_string(_ptr) {
    // No-op for worker implementation
}

// This function will be replaced by build.rs with the actual embedded worker code
function getEmbeddedWorkerCode() {
    // EMBEDDED_WORKER_CODE_PLACEHOLDER
    throw new Error('Worker code not embedded. This should be replaced during build.');
}

// Export initialization for ES modules
export { initMediaInfo as default };