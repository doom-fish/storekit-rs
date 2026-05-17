#![allow(clippy::missing_errors_doc)]

use core::ffi::c_char;
use std::ffi::{CStr, CString};
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
    if ptr.is_null() {
        return None;
    }
    // SAFETY: caller guarantees ptr is a valid, NUL-terminated C string.
    let string = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    // SAFETY: ptr was allocated by the Swift bridge (sk_string_free is the matching free).
    ffi::sk_string_free(ptr);
    Some(string)
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
    serde_json::from_str(&json).map_err(|error| {
        StoreKitError::InvalidArgument(format!(
            "failed to parse {context} JSON: {error}; payload={json}"
        ))
    })
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
