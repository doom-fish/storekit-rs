use storekit::{Refund, StoreKitError};

#[test]
fn refund_request_without_a_main_run_loop_fails_before_presenting_ui() {
    assert_ne!(std::thread::current().name(), Some("main"));

    match Refund::begin_for_transaction_id(0) {
        Err(StoreKitError::TimedOut(message)) => {
            assert!(message.contains("did not start"), "{message}");
            assert!(
                message.contains("no StoreKit UI was presented"),
                "{message}"
            );
        }
        other => panic!("expected a not-started timeout, got {other:?}"),
    }
}
