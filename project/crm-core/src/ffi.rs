// Proteus Core — Cross-Platform C-FFI Export Layer (Windows .dll, macOS .dylib, Android NDK, iOS)
// Designed from first principles. Zero copied third-party boilerplate.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub const CORE_VERSION: &str = "0.1.0-native";

/// Returns the static semantic version string of Proteus Core.
#[no_mangle]
pub extern "C" fn proteus_core_version() -> *const c_char {
    concat!("0.1.0-native", "\0").as_ptr() as *const c_char
}

/// Parses a database connection URL and returns JSON representation of the ConnectionConfig.
/// Caller must free the returned string with `proteus_free_string`.
#[no_mangle]
pub unsafe extern "C" fn proteus_parse_connection_url(url_ptr: *const c_char) -> *mut c_char {
    if url_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = match CStr::from_ptr(url_ptr).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match crate::drivers::ConnectionConfig::parse_url(c_str) {
        Ok(cfg) => match serde_json::to_string(&cfg) {
            Ok(json) => match CString::new(json) {
                Ok(c) => c.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        },
        Err(e) => {
            let err_obj = serde_json::json!({ "error": e }).to_string();
            CString::new(err_obj).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut())
        }
    }
}

/// Evaluates a business rule against a JSON context. Writes result JSON into out_buf.
/// Returns 0 on success, -1 on invalid pointers, -2 on parse error.
#[no_mangle]
pub unsafe extern "C" fn proteus_eval_rule(
    rule_json_ptr: *const c_char,
    ctx_json_ptr: *const c_char,
    out_buf: *mut c_char,
    max_len: usize,
) -> i32 {
    if rule_json_ptr.is_null() || ctx_json_ptr.is_null() || out_buf.is_null() || max_len == 0 {
        return -1;
    }

    let rule_str = match CStr::from_ptr(rule_json_ptr).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let ctx_str = match CStr::from_ptr(ctx_json_ptr).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let rule: crate::rules::BusinessRule = match serde_json::from_str(rule_str) {
        Ok(r) => r,
        Err(_) => return -2,
    };
    let mut data: serde_json::Value = match serde_json::from_str(ctx_str) {
        Ok(c) => c,
        Err(_) => return -2,
    };

    let entity = rule.entity_type.clone();
    let engine = crate::rules::BusinessRulesEngine::new(vec![rule]);
    let outcome = engine.execute(&entity, &mut data);
    let out_obj = serde_json::json!({
        "outcome": outcome,
        "data": data,
    });
    let out_json = match serde_json::to_string(&out_obj) {
        Ok(s) => s,
        Err(_) => return -2,
    };

    write_to_c_buffer(&out_json, out_buf, max_len)
}

/// Decodes GS1-128 barcode Application Identifiers. Writes decoded JSON into out_buf.
#[no_mangle]
pub unsafe extern "C" fn proteus_parse_gs1(
    barcode_ptr: *const c_char,
    out_buf: *mut c_char,
    max_len: usize,
) -> i32 {
    if barcode_ptr.is_null() || out_buf.is_null() || max_len == 0 {
        return -1;
    }
    let barcode = match CStr::from_ptr(barcode_ptr).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let elements = crate::gs1::Gs1Parser::parse(barcode);
    let out_json = match serde_json::to_string(&elements) {
        Ok(s) => s,
        Err(_) => return -2,
    };

    write_to_c_buffer(&out_json, out_buf, max_len)
}

/// Frees a C-allocated string returned by Proteus Core.
#[no_mangle]
pub unsafe extern "C" fn proteus_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

unsafe fn write_to_c_buffer(src: &str, out_buf: *mut c_char, max_len: usize) -> i32 {
    let bytes = src.as_bytes();
    if bytes.len() + 1 > max_len {
        return -3; // Buffer too small
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf as *mut u8, bytes.len());
    *out_buf.add(bytes.len()) = 0; // Null terminator
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_version_ffi() {
        let v = proteus_core_version();
        let s = unsafe { CStr::from_ptr(v).to_str().unwrap() };
        assert_eq!(s, "0.1.0-native");
    }

    #[test]
    fn test_parse_connection_url_ffi() {
        let url = CString::new("sqlite:///my/data.db").unwrap();
        let ptr = unsafe { proteus_parse_connection_url(url.as_ptr()) };
        assert!(!ptr.is_null());
        let json_str = unsafe { CStr::from_ptr(ptr).to_str().unwrap() };
        assert!(json_str.contains("Sqlite"));
        unsafe { proteus_free_string(ptr) };
    }

    #[test]
    fn test_parse_gs1_ffi() {
        let barcode = CString::new("(01)08412345678905(10)BATCH99").unwrap();
        let mut buf = [0i8; 512];
        let res = unsafe { proteus_parse_gs1(barcode.as_ptr(), buf.as_mut_ptr(), buf.len()) };
        assert_eq!(res, 0);
        let out_str = unsafe { CStr::from_ptr(buf.as_ptr()).to_str().unwrap() };
        assert!(out_str.contains("08412345678905"));
        assert!(out_str.contains("BATCH99"));
    }
}
