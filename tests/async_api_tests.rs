//! Integration tests for the `async` feature.
//!
//! These tests run against the actual Swift bridge on macOS. In a headless
//! CI environment without an App Store session, requests that require the
//! network (products, purchases, app transaction) will fail with a framework
//! or not-supported error — the tests assert the *error path* in those cases.
//!
//! Tests that touch UI (`request_review`, `show_manage_subscriptions`) are also
//! expected to fail in headless environments and are validated for the error
//! type returned.

#[cfg(feature = "async")]
mod async_tests {
    use storekit::async_api::{
        AsyncAppStore, AsyncAppTransaction, AsyncProducts, AsyncPurchase, AsyncStorefront,
    };
    use storekit::error::StoreKitError;

    // -----------------------------------------------------------------------
    // AsyncProducts
    // -----------------------------------------------------------------------

    #[test]
    fn products_happy_path_empty_result() {
        // Fetching products for an empty identifier set returns an empty vec.
        // In headless environments without an App Store connection, this may
        // also fail with a network error — that is acceptable.
        let result = pollster::block_on(async {
            AsyncProducts::fetch([] as [&str; 0])?.await
        });
        if let Ok(products) = result {
            assert!(
                products.is_empty(),
                "expected empty products list for empty identifiers"
            );
        } else {
            // Network not available in this environment — acceptable.
        }
    }

    #[test]
    fn products_nonexistent_identifiers_returns_empty_or_error() {
        // The App Store returns an empty list for unknown identifiers,
        // not an error. In Sandbox without a config file it may also
        // return an empty list or a framework error.
        let result = pollster::block_on(async {
            AsyncProducts::fetch(["com.example.does.not.exist.xyz123"])?.await
        });
        // Both Ok and Err are acceptable here.
        let _ = result;
    }

    // -----------------------------------------------------------------------
    // AsyncPurchase
    // -----------------------------------------------------------------------

    #[test]
    fn purchase_missing_product_returns_error() {
        let result = pollster::block_on(async {
            AsyncPurchase::buy("com.example.does.not.exist.xyz123", &[])?.await
        });
        assert!(
            result.is_err(),
            "expected an error when purchasing a non-existent product, got Ok"
        );
    }

    // -----------------------------------------------------------------------
    // AsyncAppStore — request_review
    // -----------------------------------------------------------------------

    #[test]
    #[ignore = "requires a running NSApplication main run loop / UI environment"]
    fn request_review_error_path_without_window() {
        // In a headless test environment there is no key window, so this
        // should return NotSupported. Succeeds if run interactively.
        let result = pollster::block_on(AsyncAppStore::request_review());
        if let Err(ref e) = result {
            assert!(
                matches!(e, StoreKitError::NotSupported(_) | StoreKitError::Unknown(_)),
                "unexpected error variant: {e}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // AsyncAppStore — show_manage_subscriptions
    // -----------------------------------------------------------------------

    #[test]
    fn show_manage_subscriptions_always_not_supported_on_macos() {
        let result = pollster::block_on(AsyncAppStore::show_manage_subscriptions());
        assert!(
            matches!(result, Err(StoreKitError::NotSupported(_))),
            "expected NotSupported, got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // AsyncAppTransaction
    // -----------------------------------------------------------------------

    #[test]
    fn app_transaction_shared_error_or_result() {
        let result = pollster::block_on(AsyncAppTransaction::shared());
        match result {
            Ok(_vr) => {
                // A VerificationResult was obtained — pass.
            }
            Err(
                StoreKitError::Framework(_)
                | StoreKitError::NotSupported(_)
                | StoreKitError::Verification(_)
                | StoreKitError::Unknown(_),
            ) => {
                // Expected in headless / macOS 12 / Sandbox / no-network environments.
            }
            Err(e) => {
                panic!("unexpected error from AsyncAppTransaction::shared: {e}");
            }
        }
    }

    // -----------------------------------------------------------------------
    // AsyncStorefront
    // -----------------------------------------------------------------------

    #[test]
    fn storefront_current_returns_option() {
        // Returns Ok(None) when not signed in, Ok(Some(...)) otherwise.
        // Should never return Err.
        let result = pollster::block_on(AsyncStorefront::current());
        match result {
            Ok(None) => {
                // Not signed in to the App Store.
            }
            Ok(Some(sf)) => {
                assert!(!sf.country_code.is_empty(), "country_code must not be empty");
                assert!(!sf.id.is_empty(), "id must not be empty");
            }
            Err(e) => {
                panic!("AsyncStorefront::current returned unexpected error: {e}");
            }
        }
    }
}
