use std::fmt::Debug;
use std::time::{Duration, Instant};

use storekit::{AppStore, Product, ProductType, Refund, StoreKitError};

fn unknown_product() -> Product {
    Product {
        id: "com.example.storekit-rs.missing".to_owned(),
        display_name: String::new(),
        description: String::new(),
        price: "0".to_owned(),
        display_price: String::new(),
        product_type: ProductType::NonConsumable,
        is_family_shareable: false,
        subscription: None,
        currency_code: None,
        price_locale_identifier: None,
        json_representation: Vec::new(),
    }
}

fn assert_returns_without_blocking<T: Debug>(
    label: &str,
    call: impl FnOnce() -> Result<T, StoreKitError>,
) {
    let started = Instant::now();
    let result = call();
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(5),
        "{label} blocked the main thread for {elapsed:?}"
    );
    match result {
        Err(StoreKitError::NotSupported(message)) => assert!(
            message.contains("main thread") || message.contains("requires macOS"),
            "{label}: unexpected message {message}"
        ),
        other => panic!("{label}: expected NotSupported, got {other:?}"),
    }
}

fn main() {
    assert_eq!(std::thread::current().name(), Some("main"));

    assert_returns_without_blocking("Product::purchase", || unknown_product().purchase(&[]));
    assert_returns_without_blocking(
        "AppStore::present_offer_code_redeem_sheet",
        AppStore::present_offer_code_redeem_sheet,
    );
    assert_returns_without_blocking("Refund::begin_for_transaction_id", || {
        Refund::begin_for_transaction_id(1)
    });

    println!("main-thread UI calls returned without blocking");
}
