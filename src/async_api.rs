//! Async API for `StoreKit` — Tier 1 Future wrappers.
//!
//! This module requires the **`async`** Cargo feature:
//! ```toml
//! storekit-rs = { version = "0.3", features = ["async"] }
//! ```
//!
//! Every public type is an executor-agnostic [`Future`] backed by a
//! `doom_fish_utils` completion handler.  The futures work with any async
//! runtime (`tokio`, `async-std`, `smol`, `pollster`, …).
//!
//! ## Available types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`AsyncProducts`] | Fetch products by identifier |
//! | [`AsyncPurchase`] | Purchase a product |
//! | [`AsyncAppStore`] | Request review / manage subscriptions |
//! | [`AsyncAppTransaction`] | Fetch the app transaction |
//! | [`AsyncStorefront`] | Fetch the current storefront |
//!
//! ## `AsyncSequence` APIs — deferred to Tier 2
//!
//! The following `StoreKit` APIs expose `AsyncSequence` (multi-fire streams).
//! They are intentionally **not** included here; they will be wrapped as
//! `Stream` types in the Tier 2 rollout:
//!
//! - `Transaction.updates` — use [`crate::transaction::TransactionStream`] in the meantime
//! - `Transaction.currentEntitlements` — use [`crate::transaction::TransactionStream`]
//! - `Transaction.unfinished` — use [`crate::transaction::TransactionStream`]
//!
//! ## Examples
//!
//! ```rust,no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # pollster::block_on(async {
//! use storekit::async_api::{AsyncProducts, AsyncAppTransaction};
//!
//! let products = AsyncProducts::fetch(["com.example.pro"])?.await?;
//! println!("{} product(s) found", products.len());
//!
//! let app_tx = AsyncAppTransaction::shared().await?;
//! println!("bundle: {}", app_tx.payload().bundle_id);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # })?;
//! # Ok(())
//! # }
//! ```

use std::ffi::{c_void, CStr};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};

use crate::app_transaction::{AppTransaction, AppTransactionPayload};
use crate::error::StoreKitError;
use crate::private::{cstring_from_str, json_cstring, take_string};
use crate::product::{Product, ProductPayload};
use crate::purchase_option::{PurchaseOption, PurchaseResult, PurchaseResultPayload};
use crate::storefront::{Storefront, StorefrontPayload};
use crate::verification_result::{VerificationResult, VerificationResultPayload};

// ============================================================================
// Internal helpers
// ============================================================================

/// Read a transient JSON C-string from a `*const c_void` result pointer.
///
/// The caller's Swift thunk passes a `&str`-borrowed `CStr` as `UnsafeRawPointer`.
/// We copy it to an owned `String` immediately so the borrow lifetime in Swift
/// is satisfied before the callback returns.
unsafe fn json_from_result_ptr(result: *const c_void) -> String {
    CStr::from_ptr(result.cast::<i8>())
        .to_string_lossy()
        .into_owned()
}

// ============================================================================
// AsyncProducts — Product.products(for:) async throws -> [Product]
// ============================================================================

extern "C" fn products_cb(result: *const c_void, error: *const i8, ctx: *mut c_void) {
    if !error.is_null() {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<String>::complete_err(ctx, msg) };
    } else if !result.is_null() {
        let json = unsafe { json_from_result_ptr(result) };
        unsafe { AsyncCompletion::complete_ok(ctx, json) };
    } else {
        unsafe { AsyncCompletion::<String>::complete_err(ctx, "no result from sk_products_async".into()) };
    }
}

/// Future for [`AsyncProducts::fetch`].
pub struct ProductsFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for ProductsFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProductsFuture").finish_non_exhaustive()
    }
}

impl Future for ProductsFuture {
    type Output = Result<Vec<Product>, StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            let json = r.map_err(StoreKitError::Unknown)?;
            let payloads: Vec<ProductPayload> = serde_json::from_str(&json).map_err(|e| {
                StoreKitError::InvalidArgument(format!("failed to parse products JSON: {e}"))
            })?;
            payloads
                .into_iter()
                .map(ProductPayload::into_product)
                .collect()
        })
    }
}

/// Async wrapper for `Product.products(for:)`.
///
/// Fetches products from the App Store for a set of product identifiers.
///
/// # Notes
///
/// On macOS Sandbox / Xcode previews you must configure a `StoreKit
/// Configuration File` in your scheme for products to be returned.
#[derive(Debug, Clone, Copy)]
pub struct AsyncProducts;

impl AsyncProducts {
    /// Asynchronously fetch products for the given identifiers.
    ///
    /// Equivalent to `Product.products(for: identifiers)` in Swift.
    ///
    /// # Errors
    ///
    /// Returns an error if the App Store request fails or the product
    /// identifiers cannot be encoded.
    pub fn fetch<I, S>(identifiers: I) -> Result<ProductsFuture, StoreKitError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let ids: Vec<String> = identifiers
            .into_iter()
            .map(|s| s.as_ref().to_owned())
            .collect();
        let ids_json = json_cstring(&ids, "product identifiers")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe { crate::ffi::sk_products_async(ids_json.as_ptr(), products_cb, ctx) }
        Ok(ProductsFuture { inner: future })
    }
}

// ============================================================================
// AsyncPurchase — Product.purchase(options:) async throws -> Product.PurchaseResult
// ============================================================================

/// Wrapper that carries the opaque Swift `SKPurchaseAsyncResult` pointer.
/// `Send` is safe because the pointer is a retained Swift object with no
/// thread-affinity restrictions after it has been constructed.
struct RawPurchaseBox(*mut c_void);
unsafe impl Send for RawPurchaseBox {}

extern "C" fn purchase_cb(result: *const c_void, error: *const i8, ctx: *mut c_void) {
    if !error.is_null() {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<RawPurchaseBox>::complete_err(ctx, msg) };
    } else if !result.is_null() {
        // result_ptr is a *retained* SKPurchaseAsyncResult — take ownership.
        let boxed = RawPurchaseBox(result.cast_mut());
        unsafe { AsyncCompletion::complete_ok(ctx, boxed) };
    } else {
        unsafe {
            AsyncCompletion::<RawPurchaseBox>::complete_err(
                ctx,
                "no result from sk_product_purchase_async".into(),
            );
        };
    }
}

/// Future for [`AsyncPurchase::buy`].
pub struct PurchaseFuture {
    inner: AsyncCompletionFuture<RawPurchaseBox>,
}

impl std::fmt::Debug for PurchaseFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PurchaseFuture").finish_non_exhaustive()
    }
}

impl Future for PurchaseFuture {
    type Output = Result<PurchaseResult, StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            let raw_box = r.map_err(StoreKitError::Unknown)?;
            let ptr = raw_box.0;
            // SAFETY: ptr is a retained Swift SKPurchaseAsyncResult; we own it
            // and must release it before returning.
            let result = unsafe { extract_purchase_result(ptr) };
            // Release the box regardless of parse outcome.
            unsafe { crate::ffi::sk_purchase_async_result_release(ptr) };
            result
        })
    }
}

/// Extract `PurchaseResult` from a retained `SKPurchaseAsyncResult` pointer.
///
/// # Safety
///
/// `ptr` must be a valid, retained `SKPurchaseAsyncResult` pointer.
unsafe fn extract_purchase_result(ptr: *mut c_void) -> Result<PurchaseResult, StoreKitError> {
    let json_ptr = crate::ffi::sk_purchase_async_result_json(ptr);
    let json = take_string(json_ptr).ok_or_else(|| {
        StoreKitError::InvalidArgument("missing JSON from purchase async result".into())
    })?;
    let transaction_handle = crate::ffi::sk_purchase_async_result_take_handle(ptr);

    let payload: PurchaseResultPayload = serde_json::from_str(&json).map_err(|e| {
        if !transaction_handle.is_null() {
            crate::ffi::sk_transaction_release(transaction_handle);
        }
        StoreKitError::InvalidArgument(format!(
            "failed to parse purchase result JSON: {e}; payload={json}"
        ))
    })?;
    payload.into_purchase_result(transaction_handle)
}

/// Async wrapper for `Product.purchase(options:)`.
///
/// Initiates an in-app purchase and resolves to the `PurchaseResult`.
/// The purchase sheet is shown on the **main actor** (equivalent to Swift's
/// `@MainActor`).
///
/// # Notes
///
/// - The future must be awaited until completion; cancellation is not
///   supported by the `StoreKit` 2 purchase API.
/// - `Transaction.currentEntitlements` and `Transaction.updates`
///   (multi-fire streams) are deferred to Tier 2.
#[derive(Debug, Clone, Copy)]
pub struct AsyncPurchase;

impl AsyncPurchase {
    /// Asynchronously purchase a product.
    ///
    /// Equivalent to `product.purchase(options:)` in Swift.
    ///
    /// # Errors
    ///
    /// Returns an error if the product cannot be found, the purchase fails,
    /// or the options cannot be encoded.
    pub fn buy(product_id: &str, options: &[PurchaseOption]) -> Result<PurchaseFuture, StoreKitError> {
        let id = cstring_from_str(product_id, "product id")?;
        let opts = json_cstring(options, "purchase options")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe { crate::ffi::sk_product_purchase_async(id.as_ptr(), opts.as_ptr(), purchase_cb, ctx) }
        Ok(PurchaseFuture { inner: future })
    }
}

// ============================================================================
// AsyncAppStore — AppStore.requestReview() / showManageSubscriptions()
// ============================================================================

extern "C" fn void_cb(_result: *const c_void, error: *const i8, ctx: *mut c_void) {
    if error.is_null() {
        // Ignore result_ptr; void APIs use a sentinel 0x1 which we don't need.
        unsafe { AsyncCompletion::complete_ok(ctx, ()) };
    } else {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<()>::complete_err(ctx, msg) };
    }
}

/// Future for [`AsyncAppStore::request_review`].
pub struct RequestReviewFuture {
    inner: AsyncCompletionFuture<()>,
}

impl std::fmt::Debug for RequestReviewFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestReviewFuture").finish_non_exhaustive()
    }
}

impl Future for RequestReviewFuture {
    type Output = Result<(), StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(StoreKitError::Unknown))
    }
}

/// Future for [`AsyncAppStore::show_manage_subscriptions`].
pub struct ShowManageSubscriptionsFuture {
    inner: AsyncCompletionFuture<()>,
}

impl std::fmt::Debug for ShowManageSubscriptionsFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShowManageSubscriptionsFuture")
            .finish_non_exhaustive()
    }
}

impl Future for ShowManageSubscriptionsFuture {
    type Output = Result<(), StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(StoreKitError::NotSupported))
    }
}

/// Async wrapper for `AppStore` UI APIs.
///
/// # Notes
///
/// - [`AsyncAppStore::request_review`] requires macOS 13.0+ and an
///   `NSViewController`-backed window.
/// - [`AsyncAppStore::show_manage_subscriptions`] is scene-based (`SwiftUI`)
///   and always returns a `NotSupported` error on macOS.  Consider opening
///   the `itms-apps://` URL directly for `AppKit` apps.
#[derive(Debug, Clone, Copy, Default)]
pub struct AsyncAppStore;

impl AsyncAppStore {
    /// Asynchronously prompt the user for an App Store review.
    ///
    /// Equivalent to `AppStore.requestReview(in:)` in Swift.
    ///
    /// # Errors
    ///
    /// Returns a `NotSupported` error when:
    /// - Running on macOS < 13.0.
    /// - No `NSViewController`-backed key window is available.
    #[must_use = "futures do nothing unless polled"]
    pub fn request_review() -> RequestReviewFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe { crate::ffi::sk_app_store_request_review_async(void_cb, ctx) }
        RequestReviewFuture { inner: future }
    }

    /// Asynchronously show the "Manage Subscriptions" sheet.
    ///
    /// Equivalent to `AppStore.showManageSubscriptions(in:)` in Swift.
    ///
    /// # Errors
    ///
    /// Always returns a `NotSupported` error on macOS because
    /// `showManageSubscriptions(in:)` is scene-based and unavailable in
    /// the macOS `StoreKit` SDK.
    #[must_use = "futures do nothing unless polled"]
    pub fn show_manage_subscriptions() -> ShowManageSubscriptionsFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe { crate::ffi::sk_app_store_show_manage_subscriptions_async(void_cb, ctx) }
        ShowManageSubscriptionsFuture { inner: future }
    }
}

// ============================================================================
// AsyncAppTransaction — AppTransaction.shared async throws
// ============================================================================

extern "C" fn app_transaction_cb(result: *const c_void, error: *const i8, ctx: *mut c_void) {
    if !error.is_null() {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<String>::complete_err(ctx, msg) };
    } else if !result.is_null() {
        let json = unsafe { json_from_result_ptr(result) };
        unsafe { AsyncCompletion::complete_ok(ctx, json) };
    } else {
        unsafe {
            AsyncCompletion::<String>::complete_err(
                ctx,
                "no result from sk_app_transaction_shared_async".into(),
            );
        };
    }
}

/// Future for [`AsyncAppTransaction::shared`].
pub struct AppTransactionFuture {
    inner: AsyncCompletionFuture<String>,
}

impl std::fmt::Debug for AppTransactionFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppTransactionFuture").finish_non_exhaustive()
    }
}

impl Future for AppTransactionFuture {
    type Output = Result<VerificationResult<AppTransaction>, StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            let json = r.map_err(StoreKitError::Unknown)?;
            let payload: VerificationResultPayload<AppTransactionPayload> =
                serde_json::from_str(&json).map_err(|e| {
                    StoreKitError::InvalidArgument(format!(
                        "failed to parse app transaction JSON: {e}"
                    ))
                })?;
            payload.into_result(AppTransactionPayload::into_app_transaction)
        })
    }
}

/// Async wrapper for `AppTransaction.shared`.
///
/// Returns a `VerificationResult<AppTransaction>` that can be verified
/// with `.verified()` to confirm the transaction's authenticity.
///
/// Requires macOS 13.0+.
///
/// # Notes
///
/// `AppTransaction.shared` may prompt the user to authenticate with the
/// App Store, so it should be awaited before presenting any gated content.
#[derive(Debug, Clone, Copy)]
pub struct AsyncAppTransaction;

impl AsyncAppTransaction {
    /// Asynchronously fetch `AppTransaction.shared`.
    ///
    /// Equivalent to `AppTransaction.shared` in Swift.
    ///
    /// # Errors
    ///
    /// Returns a `NotSupported` error on macOS < 13.0.
    #[must_use = "futures do nothing unless polled"]
    pub fn shared() -> AppTransactionFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe { crate::ffi::sk_app_transaction_shared_async(app_transaction_cb, ctx) }
        AppTransactionFuture { inner: future }
    }
}

// ============================================================================
// AsyncStorefront — Storefront.current async
// ============================================================================

/// Wrapper that carries the optional storefront JSON.
struct StorefrontResult(Option<String>);

extern "C" fn storefront_cb(result: *const c_void, error: *const i8, ctx: *mut c_void) {
    if !error.is_null() {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<StorefrontResult>::complete_err(ctx, msg) };
    } else if !result.is_null() {
        let json = unsafe { json_from_result_ptr(result) };
        unsafe { AsyncCompletion::complete_ok(ctx, StorefrontResult(Some(json))) };
    } else {
        // nil result_ptr means success but nil storefront
        unsafe { AsyncCompletion::complete_ok(ctx, StorefrontResult(None)) };
    }
}

/// JSON envelope emitted by `sk_storefront_current_async`.
#[derive(serde::Deserialize)]
struct StorefrontCurrentPayload {
    storefront: Option<StorefrontPayload>,
}

/// Future for [`AsyncStorefront::current`].
pub struct StorefrontCurrentFuture {
    inner: AsyncCompletionFuture<StorefrontResult>,
}

impl std::fmt::Debug for StorefrontCurrentFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorefrontCurrentFuture")
            .finish_non_exhaustive()
    }
}

impl Future for StorefrontCurrentFuture {
    type Output = Result<Option<Storefront>, StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|r| {
            let StorefrontResult(maybe_json) = r.map_err(StoreKitError::Unknown)?;
            let Some(json) = maybe_json else { return Ok(None) };
            let wrapper: StorefrontCurrentPayload =
                serde_json::from_str(&json).map_err(|e| {
                    StoreKitError::InvalidArgument(format!(
                        "failed to parse storefront JSON: {e}"
                    ))
                })?;
            Ok(wrapper.storefront.map(StorefrontPayload::into_storefront))
        })
    }
}

/// Async wrapper for `Storefront.current`.
///
/// Returns the current App Store storefront, or `None` if no storefront is
/// available (e.g. the device is not connected to the App Store).
#[derive(Debug, Clone, Copy)]
pub struct AsyncStorefront;

impl AsyncStorefront {
    /// Asynchronously fetch `Storefront.current`.
    ///
    /// Equivalent to `Storefront.current` in Swift.
    #[must_use = "futures do nothing unless polled"]
    pub fn current() -> StorefrontCurrentFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe { crate::ffi::sk_storefront_current_async(storefront_cb, ctx) }
        StorefrontCurrentFuture { inner: future }
    }
}
