use std::time::Duration;

use storekit::Transaction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Transaction::current_entitlements() {
        Ok(mut stream) => match stream.next_timeout(Duration::from_millis(250))? {
            Some(result) => println!(
                "current entitlement: {}",
                result.payload().data().product_id
            ),
            None => println!("no entitlement available within timeout"),
        },
        Err(error) => println!("transaction stream unavailable: {error}"),
    }
    Ok(())
}
