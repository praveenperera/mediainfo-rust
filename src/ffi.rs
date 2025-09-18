use crate::c_w_string::CWcharString;
use std::ffi::CString;
use std::path::Path;

type Uint64 = u64;
type Uint8 = u8;
type SizeT = usize;
type Wchar = libc::wchar_t;
type CChar = std::ffi::c_char;
type CInt = std::ffi::c_int;
type Void = libc::c_void;

type CMediaInfoStream = std::ffi::c_int;
type CMediaInfoInfo = std::ffi::c_int;

const LC_CTYPE: CInt = 0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaInfoStream {
    General = 0,
    Video,
    Audio,
    Text,
    Other,
    Image,
    Menu,
    Max,
}

impl MediaInfoStream {
    fn c_compatible(self) -> CMediaInfoStream {
        self as std::ffi::c_int
    }

    pub fn variants() -> Vec<MediaInfoStream> {
        // NOTE: Excluding GeneralStream, since every MedinInfo result has a GeneralStream.
        vec![
            MediaInfoStream::Video,
            MediaInfoStream::Audio,
            MediaInfoStream::Text,
            MediaInfoStream::Other,
            MediaInfoStream::Image,
            MediaInfoStream::Menu,
            MediaInfoStream::Max,
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MediaInfoInfo {
    Name = 0,
    Text,
    Measure,
    Options,
    NameText,
    MeasureText,
    Info,
    HowTo,
    Max,
}

impl MediaInfoInfo {
    fn c_compatible(self) -> CMediaInfoInfo {
        self as std::ffi::c_int
    }
}

#[derive(Debug)]
pub struct MediaInfo {
    handle: *mut Void,
}

impl Default for MediaInfo {
    fn default() -> Self {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            {
                // NOTE(erick): Setting the locale so we can
                // work properly with c wide strings.
                let empty_c_str = CString::new("").unwrap();
                setlocale(LC_CTYPE, empty_c_str.as_ptr());
                MediaInfo {
                    handle: MediaInfo_New(),
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                ensure_worker_initialized();
                let handle_id = MediaInfo_New();
                MediaInfo {
                    handle: handle_id as *mut Void,
                }
            }
        }
    }
}

impl MediaInfo {
    pub fn new() -> MediaInfo {
        Default::default()
    }

    pub fn open(&mut self, path: &Path) -> MediaInfoResult<usize> {
        unsafe {
            let path_w_string = CWcharString::from_path(path);
            if path_w_string.is_err() {
                return Err(MediaInfoError::RustToCString);
            }

            let path_w_string = path_w_string.unwrap();
            let path_ptr = path_w_string.as_raw();

            #[cfg(not(target_arch = "wasm32"))]
            let result = MediaInfo_Open(self.handle, path_ptr);

            #[cfg(target_arch = "wasm32")]
            let result = {
                // For WASM, we need to convert path to C string since JS bridge expects char* not wchar*
                let path_str = path.to_string_lossy();
                let path_c_str = CString::new(path_str.as_ref()).unwrap();
                // Note: MediaInfo_Open is not available in the JS bridge
                // This would need to be implemented differently for WASM
                0 // Placeholder - file opening in WASM typically uses buffer-based approach
            };

            Ok(result as usize)
        }
    }

    pub fn close(&mut self) {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            MediaInfo_Close(self.handle);

            #[cfg(target_arch = "wasm32")]
            {
                // Note: MediaInfo_Close is not available in the JS bridge
                // The JS bridge handles cleanup when instances are deleted
            }
        }
    }

    pub fn option(&mut self, parameter: &str, value: &str) -> MediaInfoResult<String> {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let param_w_string =
                    CWcharString::from_str(parameter).map_err(|_| MediaInfoError::RustToCString)?;
                let value_w_string =
                    CWcharString::from_str(value).map_err(|_| MediaInfoError::RustToCString)?;

                let result_ptr = MediaInfo_Option(
                    self.handle,
                    param_w_string.as_raw(),
                    value_w_string.as_raw(),
                );

                let result = CWcharString::from_raw_to_string(result_ptr)
                    .map_err(|_| MediaInfoError::CToRust)?;

                if result.is_empty() {
                    return Err(MediaInfoError::ZeroLengthResult);
                }

                Ok(result)
            }

            #[cfg(target_arch = "wasm32")]
            {
                let result = MediaInfo_Option(self.handle as u32, parameter, value);
                return Ok(result);
            }
        }
    }

    pub fn inform(&mut self) -> MediaInfoResult<String> {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let result_ptr = MediaInfo_Inform(self.handle, 0 as SizeT);

                let result = CWcharString::from_raw_to_string(result_ptr)
                    .map_err(|_| MediaInfoError::CToRust)?;

                if result.is_empty() {
                    return Err(MediaInfoError::ZeroLengthResult);
                }

                Ok(result)
            }

            #[cfg(target_arch = "wasm32")]
            {
                let result = MediaInfo_Inform(self.handle as u32);
                return Ok(result);
            }
        }
    }

    pub fn count_get(&mut self, stream_kind: MediaInfoStream) -> usize {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            let result = MediaInfo_Count_Get(
                self.handle,
                stream_kind.c_compatible(),
                (usize::MAX) as SizeT,
            ) as usize;

            #[cfg(target_arch = "wasm32")]
            let result =
                MediaInfo_Count_Get(self.handle as u32, stream_kind.c_compatible(), usize::MAX)
                    as usize;

            result
        }
    }

    pub fn get(
        &mut self,
        info_stream: MediaInfoStream,
        stream_number: usize,
        parameter: &str,
        info_kind: MediaInfoInfo,
        search_kind: MediaInfoInfo,
    ) -> MediaInfoResult<String> {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let param_w_string =
                    CWcharString::from_str(parameter).map_err(|_| MediaInfoError::RustToCString)?;

                let result_ptr = MediaInfo_Get(
                    self.handle,
                    info_stream.c_compatible(),
                    stream_number as SizeT,
                    param_w_string.as_raw(),
                    info_kind.c_compatible(),
                    search_kind.c_compatible(),
                );

                let result = CWcharString::from_raw_to_string(result_ptr)
                    .map_err(|_| MediaInfoError::CToRust)?;

                if result.is_empty() {
                    return Err(MediaInfoError::ZeroLengthResult);
                }

                Ok(result)
            }

            #[cfg(target_arch = "wasm32")]
            {
                let result = MediaInfo_Get(
                    self.handle as u32,
                    info_stream.c_compatible(),
                    stream_number,
                    parameter,
                    info_kind.c_compatible(),
                    search_kind.c_compatible(),
                );
                return Ok(result);
            }
        }
    }

    pub fn available_parameters(&mut self) -> MediaInfoResult<String> {
        self.option("Info_Parameters", "")
    }

    pub fn open_buffer_init(&mut self, buffer_size: u64, offset: u64) -> usize {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            let result = MediaInfo_Open_Buffer_Init(self.handle, buffer_size, offset) as usize;

            #[cfg(target_arch = "wasm32")]
            let result =
                MediaInfo_Open_Buffer_Init(self.handle as u32, buffer_size, offset) as usize;

            result
        }
    }

    pub fn open_buffer_continue(&mut self, data: &[u8]) -> usize {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let bytes_ptr = &data[0] as *const Uint8;
                let result =
                    MediaInfo_Open_Buffer_Continue(self.handle, bytes_ptr, data.len() as SizeT);
                result as usize
            }

            #[cfg(target_arch = "wasm32")]
            {
                let result = MediaInfo_Open_Buffer_Continue(self.handle as u32, data);
                result as usize
            }
        }
    }

    pub fn open_buffer_continue_goto_get_lower(&mut self) -> usize {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            let result = MediaInfo_Open_Buffer_Continue_GoTo_Get(self.handle) as usize;

            #[cfg(target_arch = "wasm32")]
            let result = MediaInfo_Open_Buffer_Continue_GoTo_Get_Lower(self.handle as u32) as usize;

            result
        }
    }

    #[allow(arithmetic_overflow)]
    pub fn open_buffer_continue_goto_get_upper(&mut self) -> usize {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            let result = (MediaInfo_Open_Buffer_Continue_GoTo_Get(self.handle) >> 32) as usize;

            #[cfg(target_arch = "wasm32")]
            let result = MediaInfo_Open_Buffer_Continue_GoTo_Get_Upper(self.handle as u32) as usize;

            result
        }
    }

    pub fn open_buffer_continue_goto_get(&mut self) -> usize {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            let result = MediaInfo_Open_Buffer_Continue_GoTo_Get(self.handle) as usize;

            #[cfg(target_arch = "wasm32")]
            let result = MediaInfo_Open_Buffer_Continue_GoTo_Get(self.handle as u32) as usize;

            result
        }
    }

    pub fn open_buffer_finalize(&mut self) -> usize {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            let result = MediaInfo_Open_Buffer_Finalize(self.handle) as usize;

            #[cfg(target_arch = "wasm32")]
            let result = MediaInfo_Open_Buffer_Finalize(self.handle as u32) as usize;

            result
        }
    }
}

impl Drop for MediaInfo {
    fn drop(&mut self) {
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            MediaInfo_Delete(self.handle);

            #[cfg(target_arch = "wasm32")]
            MediaInfo_Delete(self.handle as u32);
        }
    }
}

// the underlying C library is thread-safe, so we can safely implement Send and Sync
unsafe impl Send for MediaInfo {}
unsafe impl Sync for MediaInfo {}

#[derive(Debug)]
pub enum MediaInfoError {
    RustToCString,
    CToRust,
    ZeroLengthResult,
    NonNumericResult,
    NoDataOpen,
}

pub type MediaInfoResult<T> = Result<T, MediaInfoError>;

// NOTE(erick): This was needed in rust 1.6, keeping
// here for historical purpose.
// #[link(name="mediainfo")]

#[cfg(not(target_arch = "wasm32"))]
unsafe extern "C" {
    fn MediaInfo_New() -> *mut Void;

    fn MediaInfo_Delete(handle: *mut Void);

    fn MediaInfo_Open_Buffer_Init(handle: *mut Void, buffer_size: Uint64, offset: Uint64) -> SizeT;

    fn MediaInfo_Open_Buffer_Continue(
        handle: *mut Void,
        bytes: *const Uint8,
        length: SizeT,
    ) -> SizeT;

    fn MediaInfo_Open_Buffer_Continue_GoTo_Get(handle: *mut Void) -> SizeT;

    fn MediaInfo_Open_Buffer_Finalize(handle: *mut Void) -> SizeT;

    fn MediaInfo_Open(handle: *mut Void, path: *const Wchar) -> SizeT;

    fn MediaInfo_Close(handle: *mut Void);

    fn MediaInfo_Option(
        handle: *mut Void,
        parameter: *const Wchar,
        value: *const Wchar,
    ) -> *const Wchar;

    fn MediaInfo_Inform(handle: *mut Void, reserved: SizeT) -> *const Wchar;

    fn MediaInfo_Count_Get(
        handle: *mut Void,
        stream_kind: CMediaInfoStream,
        stream_number: SizeT,
    ) -> SizeT;

    fn MediaInfo_Get(
        handle: *mut Void,
        info_stream: CMediaInfoStream,
        stream_number: SizeT,
        parameter: *const Wchar,
        info_kind: CMediaInfoInfo,
        search_kind: CMediaInfoInfo,
    ) -> *const Wchar;

    fn setlocale(category: CInt, locale: *const CChar) -> *const CChar;
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use std::ffi::CStr;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/mediainfo-bridge-worker-embedded.js")]
extern "C" {
    #[wasm_bindgen(js_name = initMediaInfo)]
    fn init_mediainfo() -> bool;

    #[wasm_bindgen(js_name = mediainfo_new)]
    fn MediaInfo_New() -> u32;

    #[wasm_bindgen(js_name = mediainfo_delete)]
    fn MediaInfo_Delete(handle: u32);

    #[wasm_bindgen(js_name = mediainfo_open_buffer_init)]
    fn MediaInfo_Open_Buffer_Init(handle: u32, buffer_size: u64, offset: u64) -> u32;

    #[wasm_bindgen(js_name = mediainfo_open_buffer_continue)]
    fn MediaInfo_Open_Buffer_Continue(handle: u32, bytes: &[u8]) -> u32;

    #[wasm_bindgen(js_name = mediainfo_open_buffer_continue_goto_get)]
    fn MediaInfo_Open_Buffer_Continue_GoTo_Get(handle: u32) -> u32;

    #[wasm_bindgen(js_name = mediainfo_open_buffer_continue_goto_get_lower)]
    fn MediaInfo_Open_Buffer_Continue_GoTo_Get_Lower(handle: u32) -> u32;

    #[wasm_bindgen(js_name = mediainfo_open_buffer_continue_goto_get_upper)]
    fn MediaInfo_Open_Buffer_Continue_GoTo_Get_Upper(handle: u32) -> u32;

    #[wasm_bindgen(js_name = mediainfo_open_buffer_finalize)]
    fn MediaInfo_Open_Buffer_Finalize(handle: u32) -> u32;

    #[wasm_bindgen(js_name = mediainfo_count_get)]
    fn MediaInfo_Count_Get(handle: u32, stream_kind: CMediaInfoStream, stream_number: usize)
    -> u32;

    #[wasm_bindgen(js_name = mediainfo_get)]
    fn MediaInfo_Get(
        handle: u32,
        info_stream: CMediaInfoStream,
        stream_number: usize,
        parameter: &str,
        info_kind: CMediaInfoInfo,
        search_kind: CMediaInfoInfo,
    ) -> String;

    #[wasm_bindgen(js_name = mediainfo_inform)]
    fn MediaInfo_Inform(handle: u32) -> String;

    #[wasm_bindgen(js_name = mediainfo_option)]
    fn MediaInfo_Option(handle: u32, parameter: &str, value: &str) -> String;

    #[wasm_bindgen(js_name = mediainfo_free_string)]
    fn MediaInfo_Free_String(ptr: u32);
}

#[cfg(target_arch = "wasm32")]
static WORKER_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(target_arch = "wasm32")]
fn ensure_worker_initialized() {
    WORKER_INIT.call_once(|| {
        init_mediainfo();
    });
}
