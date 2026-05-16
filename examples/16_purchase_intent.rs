use std::time::Duration;

use storekit::PurchaseIntent;

fn main() {
    match PurchaseIntent::intents() {
        Ok(mut stream) => println!(
            "purchase intent update: {:?}",
            stream.next_timeout(Duration::from_millis(1))
        ),
        Err(error) => eprintln!("purchase intents unavailable: {error}"),
    }
}
