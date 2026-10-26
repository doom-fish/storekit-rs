mod common;

use std::time::Duration;

use storekit::{StoreKitError, Storefront};

#[test]
fn storefront_query_is_safe() {
    if !common::live_tests_enabled("storefront_query_is_safe") {
        return;
    }
    match Storefront::current() {
        Ok(Some(storefront)) => assert_ne!(storefront.id, ""),
        Ok(None) => {}
        Err(error) => assert_ne!(error.to_string(), ""),
    }
}

#[test]
fn storefront_update_stream_times_out_instead_of_ending() {
    if !common::live_tests_enabled("storefront_update_stream_times_out_instead_of_ending") {
        return;
    }
    let mut updates = Storefront::updates().expect("Storefront.updates stream");
    match updates.next_timeout(Duration::from_millis(20)) {
        Err(StoreKitError::TimedOut(_)) | Ok(Some(_)) => {}
        other => panic!("an open storefront stream reported {other:?}"),
    }
    assert!(!updates.is_finished());
}
