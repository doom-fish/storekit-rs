use storekit::{
    SubscriptionOfferType, SubscriptionPaymentMode, SubscriptionPeriod, SubscriptionPeriodUnit,
};

#[test]
fn subscription_types_have_stable_names() {
    let period = SubscriptionPeriod {
        unit: SubscriptionPeriodUnit::Year,
        value: 1,
    };
    assert_eq!(period.unit.as_str(), "year");
    assert_eq!(SubscriptionOfferType::Promotional.as_str(), "promotional");
    assert_eq!(SubscriptionPaymentMode::PayUpFront.as_str(), "payUpFront");
}
