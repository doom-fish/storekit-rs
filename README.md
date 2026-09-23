# storekit-rs

Safe Rust bindings for Apple's [StoreKit](https://developer.apple.com/documentation/storekit) framework on macOS.

> **Status:** v0.5.0. The `async_api` module, gated on `--features async`, wraps `StoreKit` 2's Swift async APIs as Rust `Future`s via `doom_fish_utils::completion`.

## Installation

```toml
[dependencies]
storekit-rs = "0.5"
```

The package is `storekit-rs`; the library is imported as `storekit`. It runs on macOS 12 or later, and APIs that need a newer macOS return `StoreKitError::NotSupported` on older systems. Building compiles a Swift bridge, which needs Xcode with the macOS 26.4 SDK or newer.

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

## Verifying transactions

`StoreKit` returns every transaction, subscription status and app transaction together with the result of its JWS signature check. Read payloads through the checked accessors before granting access:

```rust,no_run
use storekit::Transaction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut entitlements = Transaction::current_entitlements()?;
    while let Some(result) = entitlements.next()? {
        match result.payload_value() {
            Ok(transaction) => println!("entitled to {}", transaction.data().product_id),
            Err(error) => eprintln!("ignoring an unverified transaction: {error}"),
        }
    }
    Ok(())
}
```

- `payload_value()` and `into_payload_value()` return the payload only when `StoreKit` verified it, and `StoreKitError::Verification` otherwise, like Swift's throwing `payloadValue`.
- `unverified_payload()` and `into_unverified_payload()` skip the check, like `unsafePayloadValue`. Don't grant entitlements from them.
- `Transaction::finish()` refuses to finish a transaction that failed verification.

## Threading and timeouts

- Synchronous calls block the calling thread until `StoreKit` answers. Calls that don't present UI give up after 30 seconds: the `StoreKit` task is cancelled and `StoreKitError::TimedOut` is returned.
- Calls that present `StoreKit` UI run on the main actor: `Product::purchase`, `Product::purchase_in_window`, `AppStore::present_merchandising`, `AdvancedCommerceProduct::purchase_in_window`, `AppStore::present_offer_code_redeem_sheet`, `Refund::begin_for_transaction_id` and `Transaction::begin_refund_request`.
  - On the main thread they return `StoreKitError::NotSupported` right away, because the main actor could never run while they waited.
  - On any other thread they need the main thread to be running its run loop (for example `NSApplication.run`). If the main actor doesn't start the work within 10 seconds, they return `TimedOut` and nothing is presented. Once the main actor has started it they wait up to 10 minutes, then cancel it and return `TimedOut`; a purchase that still completes afterwards is delivered through `Transaction::updates()`.
- `AppStore::request_review()` is synchronous in `StoreKit` and runs inline when called on the main thread.
- `ExternalPurchase::present_notice_sheet`, `ExternalPurchaseLink::open` and `open_url`, and `ExternalPurchaseCustomLink::show_notice` let `StoreKit` present its own sheet and use the 30-second limit.
- To start a purchase or merchandising flow from the main thread without blocking it, enable the `async` feature and use `AsyncPurchase::buy`, `AsyncPurchase::buy_in_window`, `AsyncAppStore::present_merchandising` or `AsyncAppStore::request_review`. The futures are `Send`, so they can be spawned onto any executor; don't block the main thread on them.
- Stream `next()` waits until the next value arrives or the sequence ends. `next_timeout()` returns `StoreKitError::TimedOut` when nothing arrives in time, and the stream stays open. Dropping a stream cancels its `StoreKit` listener. Keep a `Transaction::updates()` stream open for the life of the app, typically on its own thread, so renewals, refunds and Ask to Buy approvals aren't missed.
- `Transaction`, `VerificationResult<Transaction>` and `PurchaseResult` are `Send + Sync`, so a purchase made on a worker thread can be handed back to the UI thread.

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
storekit-rs = { version = "0.5", features = ["async"] }
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

The following async types are available: `AsyncProducts`, `AsyncPurchase`, `AsyncAppStore`, `AsyncAppTransaction` and `AsyncStorefront`.

## Notes

- `StoreKit.Message` is unavailable on macOS, so the message module reports `NotSupported` there.
- `AppStore::show_manage_subscriptions()` and `AsyncAppStore::show_manage_subscriptions()` always return `NotSupported`: `AppStore.showManageSubscriptions(in:)` is scene-based and isn't part of the macOS `StoreKit` SDK.
- Refund, review, and offer-code presentation helpers use the key window's `NSViewController` and return `NotSupported` when there is none. Without a running main run loop they return `TimedOut` after 10 seconds (see [Threading and timeouts](#threading-and-timeouts)).
- Window-based purchase and merchandising APIs accept caller-owned `NSWindowHandle` values when the host app has an `AppKit` window to lend to `StoreKit`. The window is retained for the duration of the flow, so it only has to be alive when the call is made.
- The crate does **not** publish or call `cargo publish`; release tagging is separate from drip-publisher rollout.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
