use std::time::Duration;

use storekit::{StoreKitError, Storefront};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("current storefront: {:?}", Storefront::current());
    match Storefront::updates() {
        Ok(mut updates) => match updates.next_timeout(Duration::from_millis(100)) {
            Ok(update) => println!("storefront update within timeout: {update:?}"),
            Err(StoreKitError::TimedOut(_)) => println!("no storefront update within timeout"),
            Err(error) => return Err(error.into()),
        },
        Err(error) => println!("storefront updates unavailable: {error}"),
    }
    Ok(())
}
