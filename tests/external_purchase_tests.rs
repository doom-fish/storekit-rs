use storekit::{
    ExternalPurchase, ExternalPurchaseCustomLink, ExternalPurchaseCustomLinkNoticeResult,
    ExternalPurchaseCustomLinkNoticeType, ExternalPurchaseLink, ExternalPurchaseNoticeResult,
    StoreKitError,
};

#[test]
fn external_purchase_helpers_are_exposed() {
    assert_eq!(ExternalPurchaseCustomLinkNoticeType::Browser.as_raw(), 1);

    for result in [
        ExternalPurchase::can_present().map(|_| ()),
        ExternalPurchaseLink::can_open().map(|_| ()),
        ExternalPurchaseCustomLink::is_eligible().map(|_| ()),
    ] {
        match result {
            Ok(()) => {}
            Err(error) => assert!(!error.to_string().is_empty()),
        }
    }

    let _: fn() -> Result<ExternalPurchaseNoticeResult, StoreKitError> =
        ExternalPurchase::present_notice_sheet;
    let _: fn(&str) -> Result<(), StoreKitError> = ExternalPurchaseLink::open_url;
    let _: fn(
        ExternalPurchaseCustomLinkNoticeType,
    ) -> Result<ExternalPurchaseCustomLinkNoticeResult, StoreKitError> =
        ExternalPurchaseCustomLink::show_notice;
}
