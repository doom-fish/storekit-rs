mod common;

use storekit::AppTransaction;

#[test]
fn app_transaction_queries_are_safe() {
    if !common::live_tests_enabled("app_transaction_queries_are_safe") {
        return;
    }
    match AppTransaction::shared() {
        Ok(result) => assert!(!result.unverified_payload().bundle_id.is_empty()),
        Err(error) => assert!(!error.to_string().is_empty()),
    }
}
