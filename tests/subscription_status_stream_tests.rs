use std::time::Duration;

use storekit::{StoreKitError, SubscriptionStatus};

#[test]
fn subscription_status_streams_are_callable() {
    match SubscriptionStatus::updates() {
        Ok(mut stream) => {
            match stream.next_timeout(Duration::from_millis(1)) {
                Err(StoreKitError::TimedOut(_)) | Ok(Some(_)) => {}
                other => panic!("an open status update stream reported {other:?}"),
            }
            assert!(!stream.is_finished());
        }
        Err(error) => assert!(!error.to_string().is_empty()),
    }

    match SubscriptionStatus::all() {
        Ok(mut stream) => match stream.next_timeout(Duration::from_millis(500)) {
            Ok(Some(_)) | Err(StoreKitError::TimedOut(_)) => assert!(!stream.is_finished()),
            Ok(None) => assert!(stream.is_finished()),
            Err(error) => panic!("unexpected subscription status error: {error}"),
        },
        Err(error) => assert!(matches!(error, StoreKitError::NotSupported(_))),
    }
}
