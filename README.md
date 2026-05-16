# storekit-rs

Safe Rust bindings for Apple's [StoreKit](https://developer.apple.com/documentation/storekit) framework on macOS.

> **Status:** v0.1.0 focuses on the StoreKit 2 surface: product lookup, purchase initiation, transaction verification state, transaction streams, and restore/sync helpers.

## Quick start

```rust,no_run
use storekit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let products = Product::products_for(["com.example.pro.monthly"])?;
    for product in products {
        println!("{} — {}", product.display_name, product.display_price);
    }
    Ok(())
}
```

## Highlights

- `Product::products_for(...)` wraps `Product.products(for:)`
- `Product` exposes `id`, `display_name`, `description`, `price`, `display_price`, `product_type`, and `subscription`
- `Product::purchase(...)` returns `PurchaseResult`
- `Transaction::current_entitlements()`, `Transaction::all()`, and `Transaction::updates()` expose `StoreKit` transaction streams
- `Transaction::verify()` replays `StoreKit`'s verification status for the originating JWS
- `Transaction::finish()` marks verified transactions as complete
- `AppStore::sync()` wraps restore/sync
- `AppStore::show_manage_subscriptions()` currently returns `NotSupported` on macOS because the scene-based API is unavailable there

## Smoke example

Run the non-destructive product-fetch smoke test with:

```bash
cargo run --example 01_storekit_smoke
```

It relaunches itself from a minimal `.app` bundle (`StoreKit` 2 expects a bundle context), requests `Product::products_for(["nonexistent.product.id"])`, expects an empty result, and prints:

```text
✅ storekit product fetch (empty) OK
```

No purchase flow is triggered.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
