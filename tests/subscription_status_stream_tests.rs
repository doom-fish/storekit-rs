use std::time::Duration;

use storekit::SubscriptionStatus;

#[test]
fn subscription_status_streams_are_callable() {
    match SubscriptionStatus::updates() {
        Ok(mut stream) => match stream.next_timeout(Duration::from_millis(1)) {
            Ok(_) => {}
            Err(error) => assert!(!error.to_string().is_empty()),
        },
        Err(error) => assert!(!error.to_string().is_empty()),
    }

    match SubscriptionStatus::all() {
        Ok(mut stream) => match stream.next_timeout(Duration::from_millis(1)) {
            Ok(_) => {}
            Err(error) => assert!(!error.to_string().is_empty()),
        },
        Err(error) => assert!(!error.to_string().is_empty()),
    }
}
