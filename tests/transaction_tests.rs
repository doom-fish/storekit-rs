use storekit::{OfferPaymentMode, OfferType, TransactionReason};

#[test]
fn transaction_enums_report_expected_raw_values() {
    assert_eq!(TransactionReason::Purchase.as_str(), "purchase");
    assert_eq!(OfferType::Code.as_str(), "code");
    assert_eq!(OfferPaymentMode::FreeTrial.as_str(), "freeTrial");
}
