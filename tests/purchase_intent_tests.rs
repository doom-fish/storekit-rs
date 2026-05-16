use std::time::Duration;

use storekit::PurchaseIntent;

#[test]
fn purchase_intent_stream_is_callable() {
    match PurchaseIntent::intents() {
        Ok(mut stream) => match stream.next_timeout(Duration::from_millis(1)) {
            Ok(_) => {}
            Err(error) => assert!(!error.to_string().is_empty()),
        },
        Err(error) => assert!(!error.to_string().is_empty()),
    }
}
