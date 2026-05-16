use storekit::{
    ExpirationReason, OfferType, OwnershipType, PriceIncreaseStatus, ProductType, RenewalState,
    RevocationReason, SubscriptionOfferType, SubscriptionPaymentMode, SubscriptionPeriod,
    SubscriptionPeriodUnit,
};

#[test]
fn formatting_and_localization_helpers_are_exposed() {
    assert_eq!(SubscriptionPeriod::weekly().value, 1);
    assert_eq!(SubscriptionPeriod::every_six_months().value, 6);
    assert_eq!(SubscriptionPeriod::every_six_months().unit.as_str(), "month");

    for result in [
        ProductType::AutoRenewable.localized_description(),
        RenewalState::Expired.localized_description(),
        ExpirationReason::BillingError.localized_description(),
        PriceIncreaseStatus::Pending.localized_description(),
        SubscriptionOfferType::Promotional.localized_description(),
        OfferType::Code.localized_description(),
        SubscriptionPaymentMode::PayUpFront.localized_description(),
        SubscriptionPeriodUnit::Month.localized_description(),
        RevocationReason::Other.localized_description(),
        OwnershipType::Purchased.localized_description(),
    ] {
        match result {
            Ok(_) => {}
            Err(error) => assert!(!error.to_string().is_empty()),
        }
    }
}
