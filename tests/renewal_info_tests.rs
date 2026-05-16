use storekit::{ExpirationReason, PriceIncreaseStatus};

#[test]
fn renewal_info_enums_have_expected_names() {
    assert_eq!(ExpirationReason::BillingError.as_str(), "billingError");
    assert_eq!(PriceIncreaseStatus::Pending.as_str(), "pending");
}
