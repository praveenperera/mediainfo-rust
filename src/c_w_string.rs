// A wrapper for converting Rust strings into C wide char strings
// and vice-versa. The conversion uses the C API, which is the
// best way that I could think of getting the correct result
// always. Since it uses the C API, a call to 'setlocale()' must
// be made before any of theses functions are called or some strange
// behaviour may be encountered.

// This implementation is not the most efficient way of making this
// conversion (allocation-wise), but some trade-off has to be made
// between efficiency and safety. It would not have to be this way
// if C had a better string implementation.

use std::ptr;
use std::path::Path;

use std::ffi::CStr;
use std::ffi::CString;

type size_t = usize;
type wchar  = std::ffi::c_int;
type c_char = std::ffi::c_char;

pub struct CWcharString {
    pub data: Vec<wchar>,
    pub n_chars: usize,
}

impl CWcharString {
    pub unsafe fn from_c_string(c_string: &CString) -> CWcharString {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let c_string_ptr =
                (&c_string.as_bytes_with_nul()[0] as *const u8) as *const c_char;

            let size_needed = unsafe { mbstowcs(ptr::null_mut(), c_string_ptr, 0) } + 1;

            let mut data = vec![0 as wchar; size_needed as usize];
            let wchar_ptr = &mut data.as_mut_slice()[0] as *mut wchar;

            let n_chars = unsafe { mbstowcs(wchar_ptr, c_string_ptr, size_needed) };

            CWcharString {
                data,
                n_chars: n_chars as usize,
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            // For WASM, convert UTF-8 to UTF-32 (wchar_t equivalent)
            let utf8_bytes = c_string.to_bytes();
            let utf8_str = std::str::from_utf8_unchecked(utf8_bytes);
            
            let mut data: Vec<wchar> = utf8_str.chars()
                .map(|c| c as u32 as wchar)
                .collect();
            data.push(0); // null terminator
            
            let n_chars = data.len() - 1; // excluding null terminator
            
            CWcharString {
                data,
                n_chars,
            }
        }
    }

    pub unsafe fn from_str(string: &str) -> Result<CWcharString, ()> {
        let c_string = CString::new(string);
        if c_string.is_err() { return Err( () ); }

        Ok(unsafe { CWcharString::from_c_string(&c_string.unwrap()) })
    }

    pub unsafe fn from_path(in_path: &Path) -> Result<CWcharString, ()> {
        match in_path.to_str() {
            Some(p) => unsafe { CWcharString::from_str(p) },
            None => Err( () ),
        }
    }

    pub unsafe fn as_raw(&self) -> *const wchar {
        &self.data[0] as *const wchar
    }

    pub unsafe fn from_raw_to_c_string(raw: *const wchar) -> Result<CString, ()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let n_bytes = unsafe { wcstombs(ptr::null_mut(), raw, 0) };

            let mut data = vec![0 as u8; (n_bytes + 1) as usize];
            let data_ptr = (&mut data[0] as *mut u8) as *mut c_char;

            unsafe { wcstombs(data_ptr, raw, n_bytes + 1); }
            let c_str = CStr::from_bytes_with_nul(data.as_slice());
            if c_str.is_err() { return Err( () ); }

            Ok(c_str.unwrap().to_owned())
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            // For WASM, convert UTF-32 back to UTF-8
            if raw.is_null() {
                return Err(());
            }
            
            let mut chars = Vec::new();
            let mut i = 0;
            loop {
                let wchar_val = *raw.offset(i);
                if wchar_val == 0 {
                    break;
                }
                
                // Convert wchar (i32) to char, handling potential invalid values
                if let Some(c) = std::char::from_u32(wchar_val as u32) {
                    chars.push(c);
                } else {
                    return Err(()); // Invalid unicode
                }
                i += 1;
            }
            
            let utf8_string: String = chars.into_iter().collect();
            CString::new(utf8_string).map_err(|_| ())
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
unsafe extern "C" {
    fn mbstowcs(__pwcs: *mut wchar, __s: *const c_char, __n: size_t) -> size_t;
    fn wcstombs(__s: *mut c_char, __pwcs: *const wchar, __n: size_t) -> size_t;
}
