# storekit-rs

Safe Rust bindings for Apple's [StoreKit](https://developer.apple.com/documentation/storekit) framework on macOS.

> **Status:** v0.2.0 expands the StoreKit 2 bridge across product lookup, purchases, transaction streams, storefronts, app transactions, subscription status, renewal info, refund entry points, receipt helpers, and verification metadata.

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

- `Product::products_for(...)`, `Product::purchase(...)`, `Product::latest_transaction()`, and `Product::current_entitlements()`
- `PurchaseOption` covers quantity, app-account tokens, custom payloads, promotional offers, win-back offers, and storefront-change policies
- `Transaction` exposes `all`, `current_entitlements`, `updates`, `unfinished`, filtered product streams, `latest_for`, `current_entitlement_for`, verification, finishing, and refund helpers
- `SubscriptionInfo::status_for(...)` and `status_for_transaction(...)` surface renewal state, verified transactions, and verified renewal info
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

Run them all with:

```bash
for ex in examples/*.rs; do cargo run --example "$(basename "$ex" .rs)"; done
```

## Notes

- `StoreKit.Message` is unavailable on macOS, so the message module reports `NotSupported` there.
- Refund, review, and offer-code presentation helpers require an `NSViewController`-backed window; headless callers get `NotSupported` instead of hanging.
- The crate does **not** publish or call `cargo publish`; release tagging is separate from drip-publisher rollout.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
