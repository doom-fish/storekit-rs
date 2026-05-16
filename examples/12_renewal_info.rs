use storekit::{ExpirationReason, PriceIncreaseStatus};

fn main() {
    println!(
        "expiration reason example: {}",
        ExpirationReason::BillingError.as_str()
    );
    println!(
        "price increase status example: {}",
        PriceIncreaseStatus::Pending.as_str()
    );
}
