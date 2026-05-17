//! Example: async storefront fetch using [`AsyncStorefront`].
//!
//! Run with:
//!   `cargo run --example 22_async_storefront --features async`
//!
//! Returns `None` on headless machines that are not signed in to the
//! App Store. The example exits 0 in both cases.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        use storekit::async_api::AsyncStorefront;

        match AsyncStorefront::current().await {
            Ok(Some(sf)) => {
                println!("AsyncStorefront: country={} id={}", sf.country_code, sf.id);
                if let Some(cc) = &sf.currency_code {
                    println!("  currency: {cc}");
                }
            }
            Ok(None) => {
                println!("AsyncStorefront: no storefront available (not signed in)");
            }
            Err(e) => {
                eprintln!("AsyncStorefront error (unexpected): {e}");
                return Err(e.into());
            }
        }

        Ok(())
    })
}
