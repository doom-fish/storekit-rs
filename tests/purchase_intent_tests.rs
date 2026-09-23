use std::time::Duration;

use storekit::{PurchaseIntent, StoreKitError};

#[test]
fn purchase_intent_stream_is_callable() {
    match PurchaseIntent::intents() {
        Ok(mut stream) => {
            match stream.next_timeout(Duration::from_millis(1)) {
                Err(StoreKitError::TimedOut(_)) | Ok(Some(_)) => {}
                other => panic!("an open purchase intent stream reported {other:?}"),
            }
            assert!(!stream.is_finished());
        }
        Err(error) => assert!(matches!(error, StoreKitError::NotSupported(_))),
    }
}
