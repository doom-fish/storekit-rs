use storekit::{
    SubscriptionOfferType, SubscriptionPaymentMode, SubscriptionPeriod, SubscriptionPeriodUnit,
};

fn main() {
    let period = SubscriptionPeriod {
        unit: SubscriptionPeriodUnit::Month,
        value: 1,
    };
    println!(
        "subscription period: {} {}",
        period.value,
        period.unit.as_str()
    );
    println!(
        "offer type: {}",
        SubscriptionOfferType::Promotional.as_str()
    );
    println!(
        "payment mode: {}",
        SubscriptionPaymentMode::PayAsYouGo.as_str()
    );
}
