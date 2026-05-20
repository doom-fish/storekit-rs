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
    pub fn sk_product_purchase_in_window(
        product_id: *const c_char,
        window: *mut c_void,
        options_json: *const c_char,
        out_transaction: *mut *mut c_void,
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_product_formatting_json(
        product_id: *const c_char,
        out_formatting_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_localized_description(
        kind: *const c_char,
        raw_value: *const c_char,
        out_localized_description: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_transaction_stream_create(
        config_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn sk_transaction_stream_release(stream: *mut c_void);
    pub fn sk_transaction_stream_next(
        stream: *mut c_void,
        timeout_ms: u32,
        out_transaction: *mut *mut c_void,
        out_verification_json: *mut *mut c_char,
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
    pub fn sk_transaction_latest_for(
        product_id: *const c_char,
        out_transaction: *mut *mut c_void,
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_transaction_current_entitlement_for(
        product_id: *const c_char,
        out_transaction: *mut *mut c_void,
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_app_store_can_make_payments(
        out_value: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_app_store_device_verification_id(
        out_uuid: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_app_store_sync(out_error_message: *mut *mut c_char) -> i32;
    pub fn sk_app_store_show_manage_subscriptions(out_error_message: *mut *mut c_char) -> i32;
    pub fn sk_app_store_request_review(out_error_message: *mut *mut c_char) -> i32;
    pub fn sk_app_store_present_offer_code_redeem_sheet(out_error_message: *mut *mut c_char)
        -> i32;
    pub fn sk_app_store_age_rating_code(
        out_value: *mut i64,
        out_has_value: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_app_store_present_merchandising(
        kind_json: *const c_char,
        window: *mut c_void,
        out_transaction: *mut *mut c_void,
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_storefront_current_json(
        out_storefront_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_storefront_stream_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn sk_storefront_stream_release(stream: *mut c_void);
    pub fn sk_storefront_stream_next(
        stream: *mut c_void,
        timeout_ms: u32,
        out_storefront_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_subscription_info_is_eligible_for_intro_offer(
        group_id: *const c_char,
        out_value: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_subscription_info_statuses_json(
        group_id: *const c_char,
        out_statuses_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_subscription_info_status_for_transaction(
        transaction_id: *const c_char,
        out_status_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_subscription_status_stream_create(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn sk_subscription_status_stream_release(stream: *mut c_void);
    pub fn sk_subscription_status_stream_next(
        stream: *mut c_void,
        timeout_ms: u32,
        out_status_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_subscription_group_status_stream_create(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn sk_subscription_group_status_stream_release(stream: *mut c_void);
    pub fn sk_subscription_group_status_stream_next(
        stream: *mut c_void,
        timeout_ms: u32,
        out_payload_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_app_transaction_shared(
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_app_transaction_refresh(
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_purchase_intent_stream_create(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn sk_purchase_intent_stream_release(stream: *mut c_void);
    pub fn sk_purchase_intent_stream_next(
        stream: *mut c_void,
        timeout_ms: u32,
        out_payload_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_external_purchase_can_present(
        out_value: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_external_purchase_present_notice_result_json(
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_external_purchase_link_can_open(
        out_value: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_external_purchase_link_eligible_urls_json(
        out_urls_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_external_purchase_link_open(
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_external_purchase_link_open_url(
        url: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_external_purchase_custom_link_is_eligible(
        out_value: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_external_purchase_custom_link_show_notice_result_json(
        notice_type: i32,
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_external_purchase_custom_link_token_json(
        token_type: *const c_char,
        out_token_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_advanced_commerce_product_json(
        product_id: *const c_char,
        out_product_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn sk_advanced_commerce_product_purchase(
        product_id: *const c_char,
        compact_jws: *const c_char,
        window: *mut c_void,
        options_json: *const c_char,
        out_transaction: *mut *mut c_void,
        out_result_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_refund_begin_request_for_transaction_id(
        transaction_id: *const c_char,
        out_status: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_receipt_json(
        out_receipt_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn sk_store_context_json(
        out_context_json: *mut *mut c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    // -------------------------------------------------------------------------
    // Async callback-based FFI (used by the `async` feature)
    // -------------------------------------------------------------------------

    #[cfg(feature = "async")]
    pub fn sk_products_async(
        identifiers_json: *const c_char,
        cb: extern "C" fn(*const c_void, *const c_char, *mut c_void),
        ctx: *mut c_void,
    );

    #[cfg(feature = "async")]
    pub fn sk_product_purchase_async(
        product_id: *const c_char,
        options_json: *const c_char,
        cb: extern "C" fn(*const c_void, *const c_char, *mut c_void),
        ctx: *mut c_void,
    );

    /// Read the JSON string out of a retained `SKPurchaseAsyncResult`.
    /// Returns a `strdup`'d C string — caller must free with `sk_string_free`.
    #[cfg(feature = "async")]
    pub fn sk_purchase_async_result_json(ptr: *mut c_void) -> *mut c_char;

    /// Steal the live transaction handle from a retained `SKPurchaseAsyncResult`.
    /// Returns null when the result is not a `.success`.
    /// Transfers handle ownership to the caller.
    #[cfg(feature = "async")]
    pub fn sk_purchase_async_result_take_handle(ptr: *mut c_void) -> *mut c_void;

    /// Release a retained `SKPurchaseAsyncResult`.
    #[cfg(feature = "async")]
    pub fn sk_purchase_async_result_release(ptr: *mut c_void);

    #[cfg(feature = "async")]
    pub fn sk_app_store_request_review_async(
        cb: extern "C" fn(*const c_void, *const c_char, *mut c_void),
        ctx: *mut c_void,
    );

    #[cfg(feature = "async")]
    pub fn sk_app_store_show_manage_subscriptions_async(
        cb: extern "C" fn(*const c_void, *const c_char, *mut c_void),
        ctx: *mut c_void,
    );

    #[cfg(feature = "async")]
    pub fn sk_app_transaction_shared_async(
        cb: extern "C" fn(*const c_void, *const c_char, *mut c_void),
        ctx: *mut c_void,
    );

    #[cfg(feature = "async")]
    pub fn sk_storefront_current_async(
        cb: extern "C" fn(*const c_void, *const c_char, *mut c_void),
        ctx: *mut c_void,
    );
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
