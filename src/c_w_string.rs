// A wrapper for converting Rust strings into C wide char strings
// and vice-versa. These conversions no longer rely on the process
// locale so that they behave consistently regardless of setlocale().

// This implementation is not the most efficient way of making this
// conversion (allocation-wise), but some trade-off has to be made
// between efficiency and safety. It would not have to be this way
// if C had a better string implementation.

use std::path::Path;

#[cfg(target_arch = "wasm32")]
use std::ffi::CString;
#[cfg(not(target_arch = "wasm32"))]
use std::mem;
#[cfg(not(target_arch = "wasm32"))]
use std::slice;

#[cfg(not(target_arch = "wasm32"))]
type Wchar = libc::wchar_t;
#[cfg(target_arch = "wasm32")]
type Wchar = std::ffi::c_int;

#[allow(dead_code)]
pub struct CWcharString {
    pub data: Vec<Wchar>,
    pub n_chars: usize,
}

impl CWcharString {
    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn from_str(string: &str) -> Result<CWcharString, ()> {
        if string.chars().any(|c| c == '\0') {
            return Err(());
        }

        let mut data = if mem::size_of::<Wchar>() == 2 {
            string
                .encode_utf16()
                .map(|unit| unit as Wchar)
                .collect::<Vec<_>>()
        } else if mem::size_of::<Wchar>() == 4 {
            string
                .chars()
                .map(|ch| ch as u32 as Wchar)
                .collect::<Vec<_>>()
        } else {
            return Err(());
        };

        let n_chars = data.len();
        data.push(0);

        Ok(CWcharString { data, n_chars })
    }

    #[cfg(target_arch = "wasm32")]
    pub unsafe fn from_str(string: &str) -> Result<CWcharString, ()> {
        let c_string = CString::new(string).map_err(|_| ())?;
        let utf8_str = std::str::from_utf8_unchecked(c_string.as_bytes());

        let mut data: Vec<Wchar> = utf8_str.chars().map(|c| c as u32 as Wchar).collect();
        let n_chars = data.len();
        data.push(0);

        Ok(CWcharString { data, n_chars })
    }

    pub unsafe fn from_path(in_path: &Path) -> Result<CWcharString, ()> {
        match in_path.to_str() {
            Some(p) => unsafe { CWcharString::from_str(p) },
            None => Err(()),
        }
    }

    pub unsafe fn as_raw(&self) -> *const Wchar {
        &self.data[0] as *const Wchar
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn from_raw_to_string(raw: *const Wchar) -> Result<String, ()> {
        if raw.is_null() {
            return Err(());
        }

        if mem::size_of::<Wchar>() == 2 {
            let mut len = 0;
            while unsafe { *raw.add(len) } != 0 {
                len += 1;
            }
            let slice = unsafe { slice::from_raw_parts(raw as *const u16, len) };
            String::from_utf16(slice).map_err(|_| ())
        } else if mem::size_of::<Wchar>() == 4 {
            let mut result = String::new();
            let mut idx = 0;
            loop {
                let wchar_val = unsafe { *raw.add(idx) };
                if wchar_val == 0 {
                    break;
                }

                if let Some(ch) = std::char::from_u32(wchar_val as u32) {
                    result.push(ch);
                } else {
                    return Err(());
                }
                idx += 1;
            }
            Ok(result)
        } else {
            Err(())
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub unsafe fn from_raw_to_string(raw: *const Wchar) -> Result<String, ()> {
        if raw.is_null() {
            return Err(());
        }

        let mut chars = Vec::new();
        let mut i = 0;
        loop {
            let wchar_val = unsafe { *raw.add(i) };
            if wchar_val == 0 {
                break;
            }

            if let Some(c) = std::char::from_u32(wchar_val as u32) {
                chars.push(c);
            } else {
                return Err(());
            }
            i += 1;
        }

        Ok(chars.into_iter().collect())
    }
}
