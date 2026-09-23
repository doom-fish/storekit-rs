//! Async API for `StoreKit` — Tier 1 Future wrappers.
//!
//! This module requires the **`async`** Cargo feature:
//! ```toml
//! storekit-rs = { version = "0.5", features = ["async"] }
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
//! println!("bundle: {}", app_tx.payload_value()?.bundle_id);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # })?;
//! # Ok(())
//! # }
//! ```

use std::ffi::{c_char, c_void, CStr};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use doom_fish_utils::completion::{AsyncCompletion, AsyncCompletionFuture};
use doom_fish_utils::panic_safe::catch_user_panic;

use crate::advanced_commerce::{
    AppStoreMerchandisingKind, AppStoreMerchandisingPresentationResult,
    AppStoreMerchandisingPresentationResultPayload,
};
use crate::app_transaction::{AppTransaction, AppTransactionPayload};
use crate::error::{from_status_message, StoreKitError};
use crate::ffi;
use crate::private::{cstring_from_str, json_cstring, parse_json_str};
use crate::product::{Product, ProductPayload};
use crate::purchase_option::{PurchaseOption, PurchaseResult, PurchaseResultPayload};
use crate::storefront::{Storefront, StorefrontPayload};
use crate::transaction::TransactionHandle;
use crate::verification_result::{VerificationResult, VerificationResultPayload};
use crate::window::NSWindowHandle;

// ============================================================================
// Internal helpers
// ============================================================================

struct BridgeReply {
    status: i32,
    json: Option<String>,
    transaction: Option<TransactionHandle>,
    error: Option<String>,
}

unsafe fn copy_c_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

unsafe extern "C" fn bridge_callback(
    ctx: *mut c_void,
    status: i32,
    json: *const c_char,
    transaction: *mut c_void,
    error: *const c_char,
) {
    let transaction = unsafe { TransactionHandle::from_raw(transaction) };
    catch_user_panic("storekit async callback", || {
        let reply = BridgeReply {
            status,
            json: unsafe { copy_c_str(json) },
            transaction,
            error: unsafe { copy_c_str(error) },
        };
        unsafe { AsyncCompletion::complete_ok(ctx, reply) };
    });
}

type BridgeSuccess = (Option<String>, Option<TransactionHandle>);

fn reply_result(reply: Result<BridgeReply, String>) -> Result<BridgeSuccess, StoreKitError> {
    let reply = reply.map_err(StoreKitError::Unknown)?;
    if reply.status == ffi::status::OK {
        Ok((reply.json, reply.transaction))
    } else {
        Err(from_status_message(reply.status, reply.error))
    }
}

fn reply_json(json: Option<String>, context: &str) -> Result<String, StoreKitError> {
    json.ok_or_else(|| {
        StoreKitError::InvalidArgument(format!("missing JSON payload for {context}"))
    })
}

// ============================================================================
// AsyncProducts — Product.products(for:) async throws -> [Product]
// ============================================================================

/// Future for [`AsyncProducts::fetch`].
pub struct ProductsFuture {
    inner: AsyncCompletionFuture<BridgeReply>,
}

impl std::fmt::Debug for ProductsFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProductsFuture").finish_non_exhaustive()
    }
}

impl Future for ProductsFuture {
    type Output = Result<Vec<Product>, StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|reply| {
            let (json, _) = reply_result(reply)?;
            let payloads: Vec<ProductPayload> =
                parse_json_str(&reply_json(json, "products")?, "products")?;
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
        unsafe { ffi::sk_products_async(ids_json.as_ptr(), bridge_callback, ctx) }
        Ok(ProductsFuture { inner: future })
    }
}

// ============================================================================
// AsyncPurchase — Product.purchase(options:) async throws -> Product.PurchaseResult
// ============================================================================

/// Future for [`AsyncPurchase::buy`].
pub struct PurchaseFuture {
    inner: AsyncCompletionFuture<BridgeReply>,
}

impl std::fmt::Debug for PurchaseFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PurchaseFuture").finish_non_exhaustive()
    }
}

impl Future for PurchaseFuture {
    type Output = Result<PurchaseResult, StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|reply| {
            let (json, transaction) = reply_result(reply)?;
            parse_json_str::<PurchaseResultPayload>(
                &reply_json(json, "purchase result")?,
                "purchase result",
            )?
            .into_purchase_result(transaction)
        })
    }
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
    pub fn buy(
        product_id: &str,
        options: &[PurchaseOption],
    ) -> Result<PurchaseFuture, StoreKitError> {
        let id = cstring_from_str(product_id, "product id")?;
        let opts = json_cstring(options, "purchase options")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe { ffi::sk_product_purchase_async(id.as_ptr(), opts.as_ptr(), bridge_callback, ctx) }
        Ok(PurchaseFuture { inner: future })
    }

    pub fn buy_in_window(
        product_id: &str,
        window: &NSWindowHandle,
        options: &[PurchaseOption],
    ) -> Result<PurchaseFuture, StoreKitError> {
        let id = cstring_from_str(product_id, "product id")?;
        let opts = json_cstring(options, "purchase options")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::sk_product_purchase_in_window_async(
                id.as_ptr(),
                window.as_raw(),
                opts.as_ptr(),
                bridge_callback,
                ctx,
            );
        }
        Ok(PurchaseFuture { inner: future })
    }
}

// ============================================================================
// AsyncAppStore — AppStore.requestReview() / showManageSubscriptions()
// ============================================================================

/// Future for [`AsyncAppStore::request_review`].
pub struct RequestReviewFuture {
    inner: AsyncCompletionFuture<BridgeReply>,
}

impl std::fmt::Debug for RequestReviewFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestReviewFuture")
            .finish_non_exhaustive()
    }
}

impl Future for RequestReviewFuture {
    type Output = Result<(), StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|reply| reply_result(reply).map(|_| ()))
    }
}

/// Future for [`AsyncAppStore::show_manage_subscriptions`].
pub struct ShowManageSubscriptionsFuture {
    inner: AsyncCompletionFuture<BridgeReply>,
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
            .map(|reply| reply_result(reply).map(|_| ()))
    }
}

pub struct PresentMerchandisingFuture {
    inner: AsyncCompletionFuture<BridgeReply>,
}

impl std::fmt::Debug for PresentMerchandisingFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresentMerchandisingFuture")
            .finish_non_exhaustive()
    }
}

impl Future for PresentMerchandisingFuture {
    type Output = Result<AppStoreMerchandisingPresentationResult, StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|reply| {
            let (json, transaction) = reply_result(reply)?;
            parse_json_str::<AppStoreMerchandisingPresentationResultPayload>(
                &reply_json(json, "App Store merchandising presentation result")?,
                "App Store merchandising presentation result",
            )?
            .into_result(transaction)
        })
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
        unsafe { ffi::sk_app_store_request_review_async(bridge_callback, ctx) }
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
        unsafe { ffi::sk_app_store_show_manage_subscriptions_async(bridge_callback, ctx) }
        ShowManageSubscriptionsFuture { inner: future }
    }

    pub fn present_merchandising(
        kind: &AppStoreMerchandisingKind,
        window: &NSWindowHandle,
    ) -> Result<PresentMerchandisingFuture, StoreKitError> {
        let kind_json = json_cstring(kind, "App Store merchandising kind")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::sk_app_store_present_merchandising_async(
                kind_json.as_ptr(),
                window.as_raw(),
                bridge_callback,
                ctx,
            );
        }
        Ok(PresentMerchandisingFuture { inner: future })
    }
}

// ============================================================================
// AsyncAppTransaction — AppTransaction.shared async throws
// ============================================================================

/// Future for [`AsyncAppTransaction::shared`].
pub struct AppTransactionFuture {
    inner: AsyncCompletionFuture<BridgeReply>,
}

impl std::fmt::Debug for AppTransactionFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppTransactionFuture")
            .finish_non_exhaustive()
    }
}

impl Future for AppTransactionFuture {
    type Output = Result<VerificationResult<AppTransaction>, StoreKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|reply| {
            let (json, _) = reply_result(reply)?;
            parse_json_str::<VerificationResultPayload<AppTransactionPayload>>(
                &reply_json(json, "app transaction")?,
                "app transaction",
            )?
            .into_result(AppTransactionPayload::into_app_transaction)
        })
    }
}

/// Async wrapper for `AppTransaction.shared`.
///
/// Returns a `VerificationResult<AppTransaction>`; use
/// [`VerificationResult::payload_value`] to read the app transaction only
/// after `StoreKit` has verified it.
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
        unsafe { ffi::sk_app_transaction_shared_async(bridge_callback, ctx) }
        AppTransactionFuture { inner: future }
    }
}

// ============================================================================
// AsyncStorefront — Storefront.current async
// ============================================================================

/// JSON envelope emitted by `sk_storefront_current_async`.
#[derive(serde::Deserialize)]
struct StorefrontCurrentPayload {
    storefront: Option<StorefrontPayload>,
}

/// Future for [`AsyncStorefront::current`].
pub struct StorefrontCurrentFuture {
    inner: AsyncCompletionFuture<BridgeReply>,
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
        Pin::new(&mut self.inner).poll(cx).map(|reply| {
            let (json, _) = reply_result(reply)?;
            let wrapper: StorefrontCurrentPayload =
                parse_json_str(&reply_json(json, "storefront")?, "storefront")?;
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
        unsafe { ffi::sk_storefront_current_async(bridge_callback, ctx) }
        StorefrontCurrentFuture { inner: future }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::ptr;

    use super::{
        bridge_callback, AsyncCompletion, PurchaseFuture, RequestReviewFuture,
        StorefrontCurrentFuture,
    };
    use crate::error::{ProductPurchaseErrorCode, StoreKitError, VerificationErrorCode};
    use crate::ffi::status;

    fn fire(ctx: *mut std::ffi::c_void, status: i32, json: Option<&str>, error: Option<&str>) {
        let json = json.map(|value| CString::new(value).expect("json without NUL"));
        let error = error.map(|value| CString::new(value).expect("error without NUL"));
        unsafe {
            bridge_callback(
                ctx,
                status,
                json.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
                ptr::null_mut(),
                error.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
            );
        }
    }

    #[test]
    fn framework_errors_keep_their_typed_details() {
        let (inner, ctx) = AsyncCompletion::create();
        fire(
            ctx,
            status::FRAMEWORK_ERROR,
            None,
            Some(
                r#"{"kind":"purchaseError","code":"invalidQuantity","errorDescription":"bad quantity","failureReason":null,"recoverySuggestion":null}"#,
            ),
        );
        let error = pollster::block_on(PurchaseFuture { inner }).expect_err("framework error");
        assert!(matches!(error, StoreKitError::Framework(_)));
        assert_eq!(
            error.product_purchase_error().map(|typed| typed.code),
            Some(ProductPurchaseErrorCode::InvalidQuantity)
        );
    }

    #[test]
    fn bridge_statuses_map_to_their_error_variants() {
        let (inner, ctx) = AsyncCompletion::create();
        fire(ctx, status::NOT_SUPPORTED, None, Some("needs macOS 13.0+"));
        match pollster::block_on(RequestReviewFuture { inner }) {
            Err(StoreKitError::NotSupported(message)) => assert_eq!(message, "needs macOS 13.0+"),
            other => panic!("expected NotSupported, got {other:?}"),
        }

        let (inner, ctx) = AsyncCompletion::create();
        fire(
            ctx,
            status::VERIFICATION_ERROR,
            None,
            Some(
                r#"{"kind":"verification","code":"invalidSignature","localizedDescription":"bad signature"}"#,
            ),
        );
        match pollster::block_on(PurchaseFuture { inner }) {
            Err(StoreKitError::Verification(failure)) => {
                assert_eq!(failure.code, VerificationErrorCode::InvalidSignature);
            }
            other => panic!("expected a verification error, got {other:?}"),
        }

        let (inner, ctx) = AsyncCompletion::create();
        fire(ctx, status::TIMED_OUT, None, None);
        assert!(matches!(
            pollster::block_on(RequestReviewFuture { inner }),
            Err(StoreKitError::TimedOut(_))
        ));
    }

    #[test]
    fn successful_replies_decode_their_payload() {
        let (inner, ctx) = AsyncCompletion::create();
        fire(
            ctx,
            status::OK,
            Some(r#"{"storefront":{"countryCode":"USA","id":"143441","currencyCode":"USD"}}"#),
            None,
        );
        let storefront = pollster::block_on(StorefrontCurrentFuture { inner })
            .expect("storefront reply")
            .expect("storefront present");
        assert_eq!(storefront.country_code, "USA");
        assert_eq!(storefront.currency_code.as_deref(), Some("USD"));

        let (inner, ctx) = AsyncCompletion::create();
        fire(ctx, status::OK, None, None);
        assert!(matches!(
            pollster::block_on(StorefrontCurrentFuture { inner }),
            Err(StoreKitError::InvalidArgument(_))
        ));
    }
}
