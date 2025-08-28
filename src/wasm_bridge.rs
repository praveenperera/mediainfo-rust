// WASM Bridge for calling MediaInfo JS functions from Rust
// This module provides extern "C" declarations for the JS bridge functions

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

// External JS functions exposed via the bridge
unsafe extern "C" {
    fn mediainfo_new() -> c_int;
    fn mediainfo_delete(handle: c_int);
    fn mediainfo_open_buffer_init(handle: c_int, file_size: c_int, file_offset: c_int) -> c_int;
    fn mediainfo_open_buffer_continue(handle: c_int, buffer: *const u8, buffer_size: c_int) -> c_int;
    fn mediainfo_open_buffer_finalize(handle: c_int) -> c_int;
    fn mediainfo_open_buffer_continue_goto_get(handle: c_int) -> c_int;
    fn mediainfo_open_buffer_continue_goto_get_lower(handle: c_int) -> c_int;
    fn mediainfo_open_buffer_continue_goto_get_upper(handle: c_int) -> c_int;
    fn mediainfo_get(
        handle: c_int,
        stream_kind: c_int,
        stream_number: c_int,
        parameter: *const c_char,
        info_kind: *const c_char,
        search_kind: *const c_char,
    ) -> *mut c_char;
    fn mediainfo_count_get(handle: c_int, stream_kind: c_int) -> c_int;
    fn mediainfo_inform(handle: c_int) -> *mut c_char;
    fn mediainfo_option(handle: c_int, option: *const c_char, value: *const c_char) -> *mut c_char;
    fn mediainfo_free_string(ptr: *mut c_char);
}

/// Stream types for MediaInfo
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamKind {
    General = 0,
    Video = 1,
    Audio = 2,
    Text = 3,
    Other = 4,
    Image = 5,
    Menu = 6,
}

/// MediaInfo handle for WASM bridge
pub struct MediaInfoHandle {
    handle: c_int,
}

impl MediaInfoHandle {
    /// Create a new MediaInfo instance
    pub fn new() -> Option<Self> {
        let handle = unsafe { mediainfo_new() };
        if handle > 0 {
            Some(MediaInfoHandle { handle })
        } else {
            None
        }
    }

    /// Initialize buffer-based file opening
    pub fn open_buffer_init(&self, file_size: usize, file_offset: usize) -> bool {
        let result = unsafe { 
            mediainfo_open_buffer_init(self.handle, file_size as c_int, file_offset as c_int) 
        };
        result != 0
    }

    /// Continue buffer processing
    pub fn open_buffer_continue(&self, buffer: &[u8]) -> i32 {
        unsafe {
            mediainfo_open_buffer_continue(
                self.handle,
                buffer.as_ptr(),
                buffer.len() as c_int,
            )
        }
    }

    /// Finalize buffer processing
    pub fn open_buffer_finalize(&self) -> bool {
        let result = unsafe { mediainfo_open_buffer_finalize(self.handle) };
        result != 0
    }

    /// Get buffer position for seeking
    pub fn open_buffer_continue_goto_get(&self) -> usize {
        unsafe { mediainfo_open_buffer_continue_goto_get(self.handle) as usize }
    }

    /// Get lower bound of buffer position for seeking
    pub fn open_buffer_continue_goto_get_lower(&self) -> usize {
        unsafe { mediainfo_open_buffer_continue_goto_get_lower(self.handle) as usize }
    }

    /// Get upper bound of buffer position for seeking
    pub fn open_buffer_continue_goto_get_upper(&self) -> usize {
        unsafe { mediainfo_open_buffer_continue_goto_get_upper(self.handle) as usize }
    }

    /// Get information about a stream
    pub fn get(
        &self,
        stream_kind: StreamKind,
        stream_number: usize,
        parameter: &str,
        info_kind: Option<&str>,
        search_kind: Option<&str>,
    ) -> Option<String> {
        let parameter_cstr = CString::new(parameter).ok()?;
        let info_kind_cstr = info_kind.and_then(|s| CString::new(s).ok());
        let search_kind_cstr = search_kind.and_then(|s| CString::new(s).ok());

        let info_kind_ptr = info_kind_cstr
            .as_ref()
            .map_or(std::ptr::null(), |s| s.as_ptr());
        let search_kind_ptr = search_kind_cstr
            .as_ref()
            .map_or(std::ptr::null(), |s| s.as_ptr());

        unsafe {
            let result_ptr = mediainfo_get(
                self.handle,
                stream_kind as c_int,
                stream_number as c_int,
                parameter_cstr.as_ptr(),
                info_kind_ptr,
                search_kind_ptr,
            );

            if result_ptr.is_null() {
                return None;
            }

            let result_cstr = CStr::from_ptr(result_ptr);
            let result = result_cstr.to_string_lossy().into_owned();
            mediainfo_free_string(result_ptr);
            Some(result)
        }
    }

    /// Get count of streams of a specific type
    pub fn count_get(&self, stream_kind: StreamKind) -> usize {
        unsafe { mediainfo_count_get(self.handle, stream_kind as c_int) as usize }
    }

    /// Get complete information summary
    pub fn inform(&self) -> Option<String> {
        unsafe {
            let result_ptr = mediainfo_inform(self.handle);
            if result_ptr.is_null() {
                return None;
            }

            let result_cstr = CStr::from_ptr(result_ptr);
            let result = result_cstr.to_string_lossy().into_owned();
            mediainfo_free_string(result_ptr);
            Some(result)
        }
    }

    /// Set MediaInfo options
    pub fn option(&self, option: &str, value: &str) -> Option<String> {
        let option_cstr = CString::new(option).ok()?;
        let value_cstr = CString::new(value).ok()?;

        unsafe {
            let result_ptr = mediainfo_option(
                self.handle,
                option_cstr.as_ptr(),
                value_cstr.as_ptr(),
            );

            if result_ptr.is_null() {
                return None;
            }

            let result_cstr = CStr::from_ptr(result_ptr);
            let result = result_cstr.to_string_lossy().into_owned();
            mediainfo_free_string(result_ptr);
            Some(result)
        }
    }
}

impl Drop for MediaInfoHandle {
    fn drop(&mut self) {
        unsafe {
            mediainfo_delete(self.handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mediainfo_handle_creation() {
        // This test will only work when the JS bridge is properly loaded
        // In a real WASM environment with the bridge module
        if let Some(handle) = MediaInfoHandle::new() {
            // Test basic functionality
            assert!(handle.handle > 0);
        }
    }
}