use storekit::{BillingPlanType, PurchaseOption};

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

#[test]
fn billing_plan_type_purchase_option_serializes_raw_value() {
    let option = PurchaseOption::BillingPlanType {
        billing_plan_type: BillingPlanType::UpFront,
    };
    let json = serde_json::to_value(&option).expect("billing plan type should serialize");
    assert_eq!(json["kind"], "billingPlanType");
    assert_eq!(json["billingPlanType"], "upFront");
}
