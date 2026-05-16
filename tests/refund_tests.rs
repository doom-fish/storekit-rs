use storekit::Refund;

#[test]
fn refund_requests_surface_a_result_or_error() {
    match Refund::begin_for_transaction_id(0) {
        Ok(status) => assert!(!status.as_str().is_empty()),
        Err(error) => assert!(!error.to_string().is_empty()),
    }
}
