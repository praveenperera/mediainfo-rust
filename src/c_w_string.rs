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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_string_conversion() {
        unsafe {
            let test_str = "Hello World";
            let cw_string =
                CWcharString::from_str(test_str).expect("Failed to create CWcharString");

            let result = CWcharString::from_raw_to_string(cw_string.as_raw())
                .expect("Failed to convert back to string");

            assert_eq!(result, test_str);
        }
    }

    #[test]
    fn test_utf8_basic_unicode() {
        unsafe {
            let test_str = "Hello 世界";
            let cw_string =
                CWcharString::from_str(test_str).expect("Failed to create CWcharString");

            let result = CWcharString::from_raw_to_string(cw_string.as_raw())
                .expect("Failed to convert back to string");

            assert_eq!(result, test_str);
        }
    }

    #[test]
    fn test_utf8_emoji() {
        unsafe {
            let test_str = "Hello 👋 World 🌍";
            let cw_string =
                CWcharString::from_str(test_str).expect("Failed to create CWcharString");

            let result = CWcharString::from_raw_to_string(cw_string.as_raw())
                .expect("Failed to convert back to string");

            assert_eq!(result, test_str);
        }
    }

    #[test]
    fn test_utf8_mixed_scripts() {
        unsafe {
            let test_str = "English français 中文 العربية русский язык 日本語";
            let cw_string =
                CWcharString::from_str(test_str).expect("Failed to create CWcharString");

            let result = CWcharString::from_raw_to_string(cw_string.as_raw())
                .expect("Failed to convert back to string");

            assert_eq!(result, test_str);
        }
    }

    #[test]
    fn test_utf8_mathematical_symbols() {
        unsafe {
            let test_str = "π ≈ 3.14159 ∑∞ ∫∂x ℝ²";
            let cw_string =
                CWcharString::from_str(test_str).expect("Failed to create CWcharString");

            let result = CWcharString::from_raw_to_string(cw_string.as_raw())
                .expect("Failed to convert back to string");

            assert_eq!(result, test_str);
        }
    }

    #[test]
    fn test_utf8_special_whitespace() {
        unsafe {
            let test_str = "Normal\tTab\nNewline\r\nCRLF　Full-width space";
            let cw_string =
                CWcharString::from_str(test_str).expect("Failed to create CWcharString");

            let result = CWcharString::from_raw_to_string(cw_string.as_raw())
                .expect("Failed to convert back to string");

            assert_eq!(result, test_str);
        }
    }

    #[test]
    fn test_empty_string() {
        unsafe {
            let test_str = "";
            let cw_string =
                CWcharString::from_str(test_str).expect("Failed to create CWcharString");

            let result = CWcharString::from_raw_to_string(cw_string.as_raw())
                .expect("Failed to convert back to string");

            assert_eq!(result, test_str);
        }
    }

    #[test]
    fn test_null_byte_rejection() {
        unsafe {
            let test_str = "Hello\0World";
            let result = CWcharString::from_str(test_str);

            assert!(
                result.is_err(),
                "Should reject strings containing null bytes"
            );
        }
    }

    #[test]
    fn test_path_conversion() {
        unsafe {
            let path_str = "/path/to/file with spaces and unicode 文件.txt";
            let path = Path::new(path_str);
            let cw_string =
                CWcharString::from_path(path).expect("Failed to create CWcharString from path");

            let result = CWcharString::from_raw_to_string(cw_string.as_raw())
                .expect("Failed to convert back to string");

            assert_eq!(result, path_str);
        }
    }

    #[test]
    fn test_wchar_size_consistency() {
        // This test ensures our implementation correctly handles different wchar_t sizes
        unsafe {
            let test_strings = vec![
                "ASCII only",
                "Mixed ASCII and Unicode: café",
                "Chinese: 你好世界",
                "Japanese: こんにちは",
                "Arabic: مرحبا",
                "Emoji: 🚀🌟✨",
                "Mathematical: ∀x∈ℝ: x²≥0",
            ];

            for test_str in test_strings {
                let cw_string = CWcharString::from_str(test_str)
                    .unwrap_or_else(|_| panic!("Failed to create CWcharString for: {}", test_str));

                let result =
                    CWcharString::from_raw_to_string(cw_string.as_raw()).unwrap_or_else(|_| {
                        panic!("Failed to convert back to string for: {}", test_str)
                    });

                assert_eq!(result, test_str, "Mismatch for string: {}", test_str);
            }
        }
    }

    #[test]
    fn test_surrogate_pairs() {
        // Test characters that require surrogate pairs in UTF-16
        unsafe {
            let test_str = "𝕎𝕠𝕣𝕝𝕕 𝔬𝔣 𝔪𝔞𝔱𝔥: 𝒻(𝓍) = 𝓍²"; // Mathematical script characters
            let cw_string =
                CWcharString::from_str(test_str).expect("Failed to create CWcharString");

            let result = CWcharString::from_raw_to_string(cw_string.as_raw())
                .expect("Failed to convert back to string");

            assert_eq!(result, test_str);
        }
    }

    #[test]
    fn test_mediainfo_parameter_strings() {
        // Test strings commonly used as MediaInfo parameters
        unsafe {
            let parameters = vec![
                "Complete",
                "Inform",
                "Output",
                "Language",
                "Details",
                "Internet",
                "File_Duplicate",
                "ParseSpeed",
                "ReadByHuman",
                "Legacy",
                "EncodeTime",
                "File_CheckSideCarFiles",
                "File_KeepInfo",
                "File_StopAfterFilled",
                "File_StopSubStreamAfterFilled",
            ];

            for param in parameters {
                let cw_string = CWcharString::from_str(param).unwrap_or_else(|_| {
                    panic!("Failed to create CWcharString for parameter: {}", param)
                });

                let result =
                    CWcharString::from_raw_to_string(cw_string.as_raw()).unwrap_or_else(|_| {
                        panic!("Failed to convert back to string for parameter: {}", param)
                    });

                assert_eq!(result, param, "Mismatch for parameter: {}", param);
            }
        }
    }

    #[test]
    fn test_roundtrip_consistency() {
        // Helper function to test roundtrip conversion
        fn assert_roundtrip(input: &str) {
            unsafe {
                let wide = CWcharString::from_str(input).expect("from_str should succeed");
                assert_eq!(wide.n_chars, input.chars().count());
                assert_eq!(wide.data.last().copied(), Some(0));

                let back = CWcharString::from_raw_to_string(wide.as_raw()).expect("roundtrip");
                assert_eq!(back, input);
            }
        }

        assert_roundtrip("MediaInfo");
        assert_roundtrip("Grüße 🌍");
        assert_roundtrip("Test with emoji: 👋🌍🚀");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_invalid_wide_data_rejection() {
        use std::mem;

        // Test rejection of invalid Unicode code points
        unsafe {
            if mem::size_of::<Wchar>() == 4 {
                // Invalid code point above Unicode range
                let data = [0x110000 as Wchar, 0];
                assert!(CWcharString::from_raw_to_string(data.as_ptr()).is_err());
            } else if mem::size_of::<Wchar>() == 2 {
                // Lone surrogate in UTF-16
                let data = [0xD800 as Wchar, 0];
                assert!(CWcharString::from_raw_to_string(data.as_ptr()).is_err());
            }
        }
    }
}
