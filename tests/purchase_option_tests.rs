use storekit::PurchaseOption;

#[test]
fn purchase_options_serialize_with_expected_tags() {
    let option = PurchaseOption::CustomString {
        key: "source".to_owned(),
        value: "tests".to_owned(),
    };
    let json = serde_json::to_string(&option).expect("purchase option should serialize");
    assert!(json.contains("customString"));
    assert!(json.contains("source"));
}
