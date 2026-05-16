use storekit::AppStore;

#[test]
fn app_store_queries_do_not_crash() {
    assert!(AppStore::can_make_payments().is_ok());
    let _ = AppStore::device_verification_id();
}
