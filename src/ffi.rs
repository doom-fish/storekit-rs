#![allow(missing_docs)]

use core::ffi::{c_char, c_void};

extern "C" {
    pub fn sk_string_free(s: *mut c_char);

    pub fn sk_products_json(
        identifiers_json: *const c_char,
        out_products_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_product_purchase(
        product_id: *const c_char,
        options_json: *const c_char,
        out_transaction: *mut *mut c_void,
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_transaction_stream_create(
        kind: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn sk_transaction_stream_release(stream: *mut c_void);
    pub fn sk_transaction_stream_next(
        stream: *mut c_void,
        timeout_ms: u32,
        out_transaction: *mut *mut c_void,
        out_transaction_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_transaction_retain(transaction: *mut c_void) -> *mut c_void;
    pub fn sk_transaction_release(transaction: *mut c_void);
    pub fn sk_transaction_verify(
        transaction: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_transaction_finish(
        transaction: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_app_store_sync(out_error_message: *mut *mut c_char) -> i32;
    pub fn sk_app_store_show_manage_subscriptions(out_error_message: *mut *mut c_char) -> i32;
}

pub mod status {
    pub const OK: i32 = 0;
    pub const END_OF_STREAM: i32 = 1;
    pub const INVALID_ARGUMENT: i32 = -1;
    pub const TIMED_OUT: i32 = -2;
    pub const NOT_SUPPORTED: i32 = -3;
    pub const FRAMEWORK_ERROR: i32 = -4;
    pub const VERIFICATION_ERROR: i32 = -5;
    pub const UNKNOWN: i32 = -99;
}

pub mod stream_kind {
    pub const ALL: i32 = 0;
    pub const CURRENT_ENTITLEMENTS: i32 = 1;
    pub const UPDATES: i32 = 2;
}
