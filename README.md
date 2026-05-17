# storekit-rs

Safe Rust bindings for Apple's [StoreKit](https://developer.apple.com/documentation/storekit) framework on macOS.

> **Status:** v0.3.0 adds an `async_api` module gated on `--features async`, wrapping StoreKit 2's Swift async APIs as Rust `Future`s via `doom_fish_utils::completion`.

## Quick start

```rust,no_run
use storekit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let products = Product::products_for(["com.example.pro.monthly"])?;
    for product in products {
        println!("{} — {}", product.display_name, product.display_price);
    }

    println!("payments enabled: {}", AppStore::can_make_payments()?);
    Ok(())
}
```

## Highlights

- `Product::products_for(...)`, `Product::purchase(...)`, `Product::purchase_in_window(...)`, `Product::latest_transaction()`, and `Product::current_entitlements()`
- Product formatting/localization helpers expose `StoreKit` format styles, `SubscriptionPeriod` convenience constructors, and enum/unit `localized_description()` helpers
- `PurchaseOption` covers quantity, app-account tokens, custom payloads, promotional offers, win-back offers, and storefront-change policies
- `Transaction` exposes `all`, `current_entitlements`, `updates`, `unfinished`, filtered product streams, `latest_for`, `current_entitlement_for`, verification, finishing, refund helpers, and advanced-commerce info
- `SubscriptionInfo::status_for(...)`, `status_for_transaction(...)`, `SubscriptionStatus::updates()`, and `SubscriptionStatus::all()` surface renewal state plus status streams
- `PurchaseIntent::intents()` and `ExternalPurchase{,Link,CustomLink}` cover the newer `StoreKit` purchase-intent and regulatory-link families
- `AppStore` now includes merchandising presentation, age-rating lookups, and advanced-commerce product purchase support
- `StoreKitError::typed()` recovers typed `StoreKit`, purchase, refund-request, and invalid-request framework errors
- `Storefront::current()` and `Storefront::updates()` wrap the `StoreKit` storefront APIs
- `AppTransaction::shared()` and `AppTransaction::refresh()` expose app-level verification results
- `ReceiptValidator` reads the local app receipt and decodes JWS payloads without publishing or mutating store state
- `StoreContext::current()` summarizes bundle, receipt, payment, and device-verification context for headless tooling

## Examples

The crate ships with numbered examples for each logical area:

- `01_product_lookup`
- `02_transaction_stream`
- `03_app_store_context`
- `04_storefront_current`
- `05_subscription_types`
- `06_subscription_info_status`
- `07_refund_request`
- `08_receipt_validator`
- `09_message_support`
- `10_app_transaction`
- `11_store_context`
- `12_renewal_info`
- `13_renewal_state`
- `14_purchase_option`
- `15_verification_result`
- `16_purchase_intent`
- `17_external_purchase`
- `18_advanced_commerce`
- `19_typed_errors`
- `20_async_products` *(requires `--features async`)*
- `21_async_app_transaction` *(requires `--features async`)*
- `22_async_storefront` *(requires `--features async`)*

Run them all with:

```bash
for ex in examples/*.rs; do cargo run --example "$(basename "$ex" .rs)"; done
```

## Async API

Enable the `async` feature to access `StoreKit` 2's async Swift APIs as standard Rust `Future`s:

```toml
[dependencies]
storekit = { version = "0.3", features = ["async"] }
pollster = "0.3"  # or any async runtime
```

```rust,no_run
use storekit::async_api::AsyncProducts;

fn main() {
    let products = pollster::block_on(async {
        AsyncProducts::fetch(["com.example.pro"])
            .expect("invalid identifier")
            .await
    });
    println!("{:?}", products);
}
```

The following async types are available: [`AsyncProducts`], [`AsyncPurchase`], [`AsyncAppStore`], [`AsyncAppTransaction`], [`AsyncStorefront`].

## Notes

- `StoreKit.Message` is unavailable on macOS, so the message module reports `NotSupported` there.
- Refund, review, and offer-code presentation helpers still require an `NSViewController`-backed window; headless callers get `NotSupported` instead of hanging.
- Window-based purchase and merchandising APIs accept caller-owned `NSWindowHandle` values when the host app has an `AppKit` window to lend to `StoreKit`.
- The crate does **not** publish or call `cargo publish`; release tagging is separate from drip-publisher rollout.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
