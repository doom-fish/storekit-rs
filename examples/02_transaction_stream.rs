use std::time::Duration;

use storekit::{StoreKitError, Transaction};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Transaction::current_entitlements() {
        Ok(mut stream) => match stream.next_timeout(Duration::from_millis(250)) {
            Ok(Some(result)) => println!(
                "current entitlement: {}",
                result.payload().data().product_id
            ),
            Ok(None) => println!("no current entitlements"),
            Err(StoreKitError::TimedOut(_)) => println!("no entitlement available within timeout"),
            Err(error) => return Err(error.into()),
        },
        Err(error) => println!("transaction stream unavailable: {error}"),
    }
    Ok(())
}
