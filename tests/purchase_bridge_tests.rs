use std::time::{Duration, Instant};

use storekit::{Product, ProductType, StoreKitError};

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

#[test]
fn purchase_without_a_main_run_loop_fails_before_presenting_ui() {
    assert_ne!(std::thread::current().name(), Some("main"));

    let started = Instant::now();
    let result = unknown_product().purchase(&[]);
    let elapsed = started.elapsed();

    match result {
        Err(StoreKitError::TimedOut(message)) => {
            assert!(message.contains("did not start"), "{message}");
            assert!(
                message.contains("no StoreKit UI was presented"),
                "{message}"
            );
        }
        other => panic!("expected a not-started timeout, got {other:?}"),
    }
    assert!(
        elapsed < Duration::from_secs(30),
        "purchase waited {elapsed:?}"
    );
}
