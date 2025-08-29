// MediaInfo Web Worker Template
// This worker runs MediaInfo WASM in isolation to avoid main thread conflicts

// Embedded MediaInfo assets will be injected here during build
// EMBED_MEDIAINFO_ASSETS

let mediaInfoInstance = null;
let mediaInfoInstances = new Map();
let nextHandle = 1;

// SharedArrayBuffer for synchronization
let syncBuffer = null;
let syncView = null;
const RESPONSE_READY_INDEX = 0; // Index for response ready flag
const REQUEST_ID_INDEX = 1; // Index for current request ID

// Initialize MediaInfo in the worker context
async function initializeMediaInfo() {
    try {
        if (!MediaInfoLib) {
            await initEmbeddedMediaInfo();
        }
        mediaInfoInstance = await MediaInfoLib();
        return true;
    } catch (error) {
        console.error('Worker: Failed to initialize MediaInfo:', error);
        return false;
    }
}

self.onerror = function(error) {
    console.error('Worker: Error:', error);
};

self.onabort = function(error) {
    console.error('Worker: Abort:', error);
};

self.onmessageerror = function(error) {
    console.error('Worker: Message Error:', error);
};

// Message handler for communication with main thread
self.onmessage = async function(event) {
    if (event.data.type === 'init_sync') {
        syncBuffer = event.data.newSync;
        syncView = new Int32Array(syncBuffer);
        return;
    }

    const { type, id, method, params } = event.data;
    
    try {
        let result;
        
        switch (method) {
            case 'init':
                result = await initializeMediaInfo();
                break;
                
            case 'new':
                result = await handleNew();
                break;
                
            case 'delete':
                result = await handleDelete(params.handle);
                break;
                
            case 'openBufferInit':
                result = await handleOpenBufferInit(params.handle, params.fileSize, params.fileOffset);
                break;
                
            case 'openBufferContinue':
                result = await handleOpenBufferContinue(params.handle, params.buffer);
                break;
                
            case 'openBufferFinalize':
                result = await handleOpenBufferFinalize(params.handle);
                break;
                
            case 'openBufferContinueGoToGet':
                result = await handleOpenBufferContinueGoToGet(params.handle);
                break;
                
            case 'openBufferContinueGoToGetLower':
                result = await handleOpenBufferContinueGoToGetLower(params.handle);
                break;
                
            case 'openBufferContinueGoToGetUpper':
                result = await handleOpenBufferContinueGoToGetUpper(params.handle);
                break;
                
            case 'get':
                result = await handleGet(params.handle, params.streamKind, params.streamNumber, params.parameter, params.infoKind, params.searchKind);
                break;
                
            case 'countGet':
                result = await handleCountGet(params.handle, params.streamKind, params.streamNumber);
                break;
                
            case 'inform':
                result = await handleInform(params.handle);
                break;
                
            case 'option':
                result = await handleOption(params.handle, params.parameter, params.value);
                break;
                
            default:
                throw new Error(`Unknown method: ${method}`);
        }
        
        // Send success response
        self.postMessage({
            id,
            success: true,
            result
        });
        
        // Signal that response is ready using atomics (moved from bridge)
        if (syncView && id) {
            Atomics.store(syncView, REQUEST_ID_INDEX, id);
            Atomics.store(syncView, RESPONSE_READY_INDEX, 1);
            Atomics.notify(syncView, RESPONSE_READY_INDEX, 1);
        }
        
    } catch (error) {
        // Send error response
        self.postMessage({
            id,
            success: false,
            error: error.message
        });
        
        // Signal that response is ready using atomics (even for errors)
        if (syncView && id) {
            Atomics.store(syncView, REQUEST_ID_INDEX, id);
            Atomics.store(syncView, RESPONSE_READY_INDEX, 1);
            Atomics.notify(syncView, RESPONSE_READY_INDEX, 1);
        }
    }
};

// Handler functions
async function handleNew() {
    if (!mediaInfoInstance) {
        throw new Error('MediaInfo not initialized');
    }
    
    const instance = new mediaInfoInstance.MediaInfo();
    const handle = nextHandle++;
    mediaInfoInstances.set(handle, instance);
    return handle;
}

async function handleDelete(handle) {
    const instance = mediaInfoInstances.get(handle);
    if (instance) {
        instance.delete();
        mediaInfoInstances.delete(handle);
        return true;
    }
    return false;
}

async function handleOpenBufferInit(handle, fileSize, fileOffset) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Open_Buffer_Init(fileSize, fileOffset) ? 1 : 0;
}

async function handleOpenBufferContinue(handle, buffer) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Open_Buffer_Continue(new Uint8Array(buffer));
}

async function handleOpenBufferFinalize(handle) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Open_Buffer_Finalize() ? 1 : 0;
}

async function handleOpenBufferContinueGoToGet(handle) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Open_Buffer_Continue_GoTo_Get();
}

async function handleOpenBufferContinueGoToGetLower(handle) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Open_Buffer_Continue_GoTo_Get_Lower();
}

async function handleOpenBufferContinueGoToGetUpper(handle) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Open_Buffer_Continue_GoTo_Get_Upper();
}

async function handleGet(handle, streamKind, streamNumber, parameter, infoKind, searchKind) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Get(streamKind, streamNumber, parameter || "", infoKind || "", searchKind || "") || "";
}

async function handleCountGet(handle, streamKind, streamNumber) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Count_Get(streamKind, streamNumber || 0);
}

async function handleInform(handle) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Inform() || "";
}

async function handleOption(handle, parameter, value) {
    const instance = mediaInfoInstances.get(handle);
    if (!instance) throw new Error(`No instance found for handle ${handle}`);
    return instance.Option(parameter || "", value || "") || "";
}