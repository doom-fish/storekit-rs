//! Example: async product fetch using [`AsyncProducts`].
//!
//! Run with:
//!   `cargo run --example 20_async_products --features async`
//!
//! In a Sandbox / headless environment this will typically return an empty
//! product list (no `StoreKit` configuration file present). The example exits
//! 0 in that case.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        use storekit::async_api::AsyncProducts;

        let identifiers = ["com.example.pro", "com.example.monthly"];
        let products = match AsyncProducts::fetch(identifiers)?.await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("products fetch returned an error (expected in Sandbox): {e}");
                Vec::new()
            }
        };

        println!("AsyncProducts: {} product(s) returned", products.len());
        for product in &products {
            println!("  {} — {}", product.id, product.display_price);
        }

        Ok(())
    })
}
