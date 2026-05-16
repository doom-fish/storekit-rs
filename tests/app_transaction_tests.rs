use storekit::AppTransaction;

#[test]
fn app_transaction_queries_are_safe() {
    match AppTransaction::shared() {
        Ok(result) => assert!(!result.payload().bundle_id.is_empty()),
        Err(error) => assert!(!error.to_string().is_empty()),
    }
}
