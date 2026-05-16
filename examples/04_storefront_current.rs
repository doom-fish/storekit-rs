use std::time::Duration;

use storekit::Storefront;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("current storefront: {:?}", Storefront::current());
    match Storefront::updates() {
        Ok(mut updates) => println!(
            "storefront update within timeout: {:?}",
            updates.next_timeout(Duration::from_millis(100))?
        ),
        Err(error) => println!("storefront updates unavailable: {error}"),
    }
    Ok(())
}
