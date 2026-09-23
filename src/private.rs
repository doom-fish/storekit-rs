#![allow(clippy::missing_errors_doc)]

use core::ffi::c_char;
use std::ffi::CString;
use std::time::Duration;

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::StoreKitError;
use crate::ffi;

pub fn cstring_from_str(value: &str, context: &str) -> Result<CString, StoreKitError> {
    CString::new(value).map_err(|error| {
        StoreKitError::InvalidArgument(format!("{context} contains an embedded NUL byte: {error}"))
    })
}

pub fn json_cstring<T: Serialize + ?Sized>(
    value: &T,
    context: &str,
) -> Result<CString, StoreKitError> {
    let json = serde_json::to_string(value).map_err(|error| {
        StoreKitError::InvalidArgument(format!("failed to encode {context} as JSON: {error}"))
    })?;
    cstring_from_str(&json, context)
}

pub unsafe fn take_string(ptr: *mut c_char) -> Option<String> {
    doom_fish_utils::ffi_string::take_owned_cstring_c(ptr, |p| ffi::sk_string_free(p))
}

pub unsafe fn parse_json_ptr<T: DeserializeOwned>(
    ptr: *mut c_char,
    context: &str,
) -> Result<T, StoreKitError> {
    // SAFETY: caller guarantees ptr is either null or a valid, NUL-terminated
    // C string allocated by the Swift bridge.
    let json = take_string(ptr).ok_or_else(|| {
        StoreKitError::InvalidArgument(format!("missing JSON payload for {context}"))
    })?;
    parse_json_str(&json, context)
}

pub fn parse_json_str<T: DeserializeOwned>(json: &str, context: &str) -> Result<T, StoreKitError> {
    serde_json::from_str(json).map_err(|error| json_error(&error, context))
}

pub fn json_error(error: &serde_json::Error, context: &str) -> StoreKitError {
    let message = error.to_string();
    let detail = if message.starts_with("missing field") || message.starts_with("duplicate field") {
        message
    } else {
        format!(
            "{:?} error at line {} column {}",
            error.classify(),
            error.line(),
            error.column()
        )
    };
    StoreKitError::InvalidArgument(format!("failed to parse {context} JSON: {detail}"))
}

pub unsafe fn parse_optional_json_ptr<T: DeserializeOwned>(
    ptr: *mut c_char,
    context: &str,
) -> Result<Option<T>, StoreKitError> {
    if ptr.is_null() {
        Ok(None)
    } else {
        parse_json_ptr(ptr, context).map(Some)
    }
}

pub unsafe fn error_from_status(status: i32, err_msg: *mut c_char) -> StoreKitError {
    // SAFETY: caller guarantees err_msg is either null or a valid, NUL-terminated
    // C string allocated by the Swift bridge (ownership is transferred here).
    crate::error::from_swift(status, err_msg)
}

pub fn duration_to_timeout_ms(duration: Duration) -> u32 {
    let millis = duration.as_millis();
    u32::try_from(millis).unwrap_or(u32::MAX)
}

pub fn decode_base64(value: &str, context: &str) -> Result<Vec<u8>, StoreKitError> {
    STANDARD.decode(value).map_err(|error| {
        StoreKitError::InvalidArgument(format!("invalid base64 in {context}: {error}"))
    })
}

pub fn decode_base64_urlsafe(value: &str, context: &str) -> Result<Vec<u8>, StoreKitError> {
    URL_SAFE_NO_PAD.decode(value).map_err(|error| {
        StoreKitError::InvalidArgument(format!("invalid base64url payload in {context}: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use super::parse_json_str;

    #[derive(Debug, serde::Deserialize)]
    #[allow(dead_code)]
    struct Probe {
        flag: bool,
        token: String,
    }

    #[test]
    fn parse_errors_do_not_echo_payload_values() {
        let json = r#"{"flag":"eyJhbGciOiJFUzI1NiJ9.c2VjcmV0.c2ln","token":"8c2f59b0-5c6e-4d4f-9f55-0d3d6f1f7a11"}"#;
        let message = parse_json_str::<Probe>(json, "probe")
            .expect_err("a string is not a bool")
            .to_string();
        assert!(message.contains("failed to parse probe JSON"));
        assert!(!message.contains("eyJhbGciOiJFUzI1NiJ9"));
        assert!(!message.contains("c2VjcmV0"));
        assert!(!message.contains("8c2f59b0"));
    }

    #[test]
    fn parse_errors_keep_missing_field_names() {
        let message = parse_json_str::<Probe>(r#"{"flag":true}"#, "probe")
            .expect_err("token is required")
            .to_string();
        assert!(message.contains("missing field `token`"));
    }

    #[test]
    fn syntax_errors_report_position_only() {
        let message = parse_json_str::<Probe>(r#"{"flag":true,"token":"abc"#, "probe")
            .expect_err("unterminated JSON")
            .to_string();
        assert!(message.contains("line 1"));
        assert!(!message.contains("abc"));
    }
}
