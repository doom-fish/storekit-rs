use storekit::{
    AdvancedCommerceProduct, AdvancedCommercePurchaseOption, AppStore,
    AppStoreMerchandisingKind,
};

fn main() {
    let kind = AppStoreMerchandisingKind::subscription_bundle("com.example.bundle");
    println!("merchandising kind: {kind:?}");
    println!("age rating code: {:?}", AppStore::age_rating_code());
    println!(
        "advanced-commerce product lookup: {:?}",
        AdvancedCommerceProduct::new("com.example.advanced")
    );
    println!(
        "advanced-commerce purchase option: {:?}",
        AdvancedCommercePurchaseOption::OnStorefrontChange {
            should_continue_purchase: true,
        }
    );
}
