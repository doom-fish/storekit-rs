use storekit::{
    AdvancedCommerceProduct, AdvancedCommercePurchaseOption, AppStore,
    AppStoreMerchandisingKind, AppStoreMerchandisingPresentationResult, NSWindowHandle,
    Product, PurchaseOption, PurchaseResult, StoreKitError,
};

#[test]
fn advanced_commerce_helpers_are_exposed() {
    let kind = AppStoreMerchandisingKind::subscription_bundle("com.example.bundle");
    let kind_json = serde_json::to_string(&kind).expect("merchandising kind should serialize");
    assert!(kind_json.contains("subscriptionBundle"));

    let option = AdvancedCommercePurchaseOption::OnStorefrontChange {
        should_continue_purchase: true,
    };
    let option_json = serde_json::to_string(&option).expect("advanced-commerce option should serialize");
    assert!(option_json.contains("onStorefrontChange"));

    match AppStore::age_rating_code() {
        Ok(_) => {}
        Err(error) => assert!(!error.to_string().is_empty()),
    }

    match AdvancedCommerceProduct::new("com.example.advanced") {
        Ok(product) => {
            let _ = product.latest_transaction();
        }
        Err(error) => assert!(!error.to_string().is_empty()),
    }

    let _: fn(
        &AppStoreMerchandisingKind,
        &NSWindowHandle,
    ) -> Result<AppStoreMerchandisingPresentationResult, StoreKitError> =
        AppStore::present_merchandising;
    let _: fn(
        &Product,
        &NSWindowHandle,
        &[PurchaseOption],
    ) -> Result<PurchaseResult, StoreKitError> = Product::purchase_in_window;
    let _: fn(
        &AdvancedCommerceProduct,
        &str,
        &NSWindowHandle,
        &[AdvancedCommercePurchaseOption],
    ) -> Result<PurchaseResult, StoreKitError> = AdvancedCommerceProduct::purchase_in_window;
}
