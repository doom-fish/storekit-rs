use storekit::{ProductType, PurchaseOption};

#[test]
fn product_type_and_options_are_exposed() {
    assert_eq!(ProductType::AutoRenewable.as_str(), "autoRenewable");
    let option = PurchaseOption::Quantity { quantity: 2 };
    let json = serde_json::to_string(&option).expect("purchase option should serialize");
    assert!(json.contains("quantity"));
}
