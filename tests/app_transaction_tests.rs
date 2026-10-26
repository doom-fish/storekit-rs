mod common;

use storekit::AppTransaction;

#[test]
fn app_transaction_queries_are_safe() {
    if !common::live_tests_enabled("app_transaction_queries_are_safe") {
        return;
    }
    match AppTransaction::shared() {
        Ok(result) => assert_ne!(result.unverified_payload().bundle_id, ""),
        Err(error) => assert_ne!(error.to_string(), ""),
    }
}
