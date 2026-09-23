# Changelog

All notable changes to `storekit-rs` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - Unreleased

### Security

- The synchronous purchase, window purchase, merchandising, advanced-commerce
  purchase, `Transaction::latest_for` and `Transaction::current_entitlement_for`
  bridges no longer use caller memory after they return. After a timeout, the
  Swift task used to read the freed product-ID string and window pointer, could
  still show a real payment sheet, and wrote a retained transaction into a dead
  stack slot. Inputs are now copied or retained first, results go through
  lock-protected heap state that only the calling thread reads, and a
  timed-out task is cancelled.
- A purchase called on the main thread no longer deadlocks for 60 seconds. It
  returns `StoreKitError::NotSupported` right away.
- Added checked verification accessors so that entitlements can be granted
  only on verified transactions (see Added and Changed), and fixed the
  transaction-stream example, which read an entitlement without checking it.
- JSON parse errors no longer include the payload, which could contain the
  transaction JWS or the `appAccountToken`.

### Fixed

- `PurchaseOption` fields are now sent under the camelCase names the bridge
  decodes. `AppAccountToken`, `CustomNumber`, `CustomBool`, `CustomData`, both
  promotional offers, `IntroductoryOfferEligibility` and `WinBackOffer` always
  failed. `SimulatesAskToBuyInSandbox(true)` was applied as false and
  `OnStorefrontChange(false)` as true. The bridge now rejects a missing
  boolean instead of substituting a default.
- Dropping `Transaction::updates()` or any other `StoreKit` stream now cancels
  its listener. Before, the listener task held the stream box strongly, so the
  box leaked and its queue kept growing.
- Transaction handles are released on every error path.
- Async API errors keep their type (`NotSupported`, `TimedOut`,
  `Verification` and the typed framework errors) instead of becoming
  `StoreKitError::Unknown`.
- Filtered transaction streams reject a missing product ID instead of
  returning every transaction.
- Off the main thread, a UI call made while the main run loop isn't running
  fails after 10 seconds with nothing presented, instead of blocking for up
  to 60 seconds.
- `next_timeout(Duration::MAX)` waits instead of passing an out-of-range
  deadline to the bridge.

### Changed

- **Breaking:** `VerificationResult::payload()` and `into_payload()` are
  renamed to `unverified_payload()` and `into_unverified_payload()`. Grant
  access through the new `payload_value()` or `into_payload_value()`.
- **Breaking:** `VerificationResult::advanced_commerce_info()` returns
  `StoreKitError::Verification` for unverified results.
- **Breaking:** stream `next()` waits until the next value arrives or the
  sequence ends, instead of returning `Ok(None)` after 30 seconds.
  `next_timeout()` returns `StoreKitError::TimedOut` instead of `Ok(None)`
  when nothing arrives in time, so `Ok(None)` always means the end of the
  sequence. This applies to the transaction, storefront, purchase intent and
  subscription status streams.
- **Breaking:** the synchronous UI calls (`Product::purchase`,
  `Product::purchase_in_window`, `AppStore::present_merchandising`,
  `AdvancedCommerceProduct::purchase_in_window`,
  `AppStore::present_offer_code_redeem_sheet`,
  `Refund::begin_for_transaction_id` and `Transaction::begin_refund_request`)
  return `NotSupported` on the main thread. Off the main thread they wait up
  to 10 seconds for the main actor to start the work, then up to 10 minutes
  for it to finish. They used to wait 30 or 60 seconds in total.
  `AppStore::request_review()` runs inline on the main thread.
- **Breaking:** `PurchaseOption` serializes its struct-variant fields in
  camelCase.
- Timed-out `StoreKit` tasks are cancelled, and timeout errors name the
  operation.
- Requires `doom-fish-utils` `>=0.4.1, <0.5` and serde 1.0.183 or later.
  `rust-version` is now 1.82 (was 1.76).
- README: the install snippets named the unrelated `storekit` crate. They now
  use `storekit-rs`, and the README documents verification, threading and
  timeouts, the macOS 12 minimum and the macOS 26.4 SDK needed to build. Its
  broken rustdoc links are fixed.
- COVERAGE: corrected wrong method and variant names, recorded that the
  `PurchaseOption` rows were marked covered while broken, and listed the
  uncovered macOS 26.5 purchase error case and StoreKit 27.0 additions.

### Added

- `VerificationResult::payload_value()` and `into_payload_value()`, the
  equivalents of Swift's throwing `payloadValue`.
- `AsyncPurchase::buy_in_window` and `AsyncAppStore::present_merchandising`
  (with `PresentMerchandisingFuture`), which start window-based flows from the
  main thread without blocking it.
- `Transaction`, `VerificationResult<Transaction>` and `PurchaseResult` are
  `Send + Sync`.
- `PurchaseOption` and `BillingPlanType` implement `Deserialize`.

### Removed

- The unused Swift bridge C header and the `publicHeadersPath` setting.

## [0.4.6] - 2026-05-20

- Migrated local `take_string` body to call `doom_fish_utils::ffi_string::take_owned_cstring_c`. Centralises the duplicated FFI take-string pattern fleet-wide. No public API change.

## [0.4.5] - 2026-05-20

- Added in-`src/` unit tests across src/product.rs, src/renewal_info.rs, src/renewal_state.rs, src/subscription.rs, and src/subscription_info.rs (Tier 2 quality polish), providing fast `cargo test --lib` fail-fast signal alongside the existing integration tests under `tests/`.

## [0.4.4] - 2026-05-20

- Clippy hygiene sweep: cleared all `-D warnings` lints across the crate. No public API change.

## [0.4.3] - 2026-05-20

- Widen `doom-fish-utils` dependency bound to `<0.4` so the 0.3.x SPSC-ring release resolves cleanly. No source changes.

## [0.4.2] - 2026-05-19

- Bump MSRV from 1.70 to 1.76 to match fleet baseline.

## [0.4.1] - 2026-05-19

- Document 23 legacy StoreKit 1 Obj-C classes as EXEMPT (superseded by StoreKit 2 Swift API already wrapped).

## [0.4.0] - 2026-05-19

### Added

- Added `BillingPlanType` plus `PurchaseOption::BillingPlanType` for StoreKit's subscription billing-plan purchase option.
- Added `SubscriptionCommitmentInfo`, `SubscriptionPricingTerms`, and `SubscriptionInfo::pricing_terms` for subscription pricing-term and commitment metadata.
- Added `RenewalCommitmentInfo` plus `RenewalInfo::{commitment_info, renewal_billing_plan_type}` for renewal commitment snapshots.
- Added `TransactionCommitmentInfo`, `RevocationType`, and `TransactionData::{revocation_type, billing_plan_type, commitment_info}` for transaction commitment and revocation metadata.
- Added integration tests covering billing-plan, pricing-terms, renewal-commitment, transaction-commitment, and revocation wrappers.

### Changed

- Refreshed `COVERAGE_AUDIT.md` against `MacOSX26.5.sdk` and moved the 38 newly audited StoreKit symbols to VERIFIED.
- Bumped crate version from `0.3.2` to `0.4.0`.

## [0.3.2] - 2026-05-18

### Changed

- Added `///` docs across the public `src/` API surface (excluding `src/ffi.rs`), with StoreKit counterpart references for the wrapped modules, types, fields, variants, and methods; nightly rustdoc coverage now reports 100.0% documented items in `src/`.

## [0.3.1] - 2026-05-17

### Fixed

- **Async callbacks are now panic-safe**: wrapped all five `extern "C" fn` callback bodies
  (`products_cb`, `purchase_cb`, `void_cb`, `app_transaction_cb`, `storefront_cb`) in
  `doom_fish_utils::panic_safe::catch_user_panic` — a Rust panic escaping across the FFI
  boundary into Swift is undefined behaviour.
- **`RawPurchaseBox` now has a `Drop` impl** that calls `sk_purchase_async_result_release`;
  previously, if `extract_purchase_result` returned early (e.g. JSON parse error), the
  retained `SKPurchaseAsyncResult` pointer was leaked and never released.
- **`// SAFETY:` comments** added to all `unsafe {}` blocks in `async_api.rs` and the
  core helpers in `private.rs` and `transaction.rs`.
- **`doom-fish-utils` version range** widened from `"0.1"` to `">=0.1, <0.3"` to allow
  compatible patch/minor upgrades without a forced lockfile bump.

## [0.3.0] - 2026-05-17

### Added

- `async_api` module (enabled by `--features async`) providing Rust `Future`-based wrappers over StoreKit 2's async Swift APIs using `doom_fish_utils::completion`:
  - `products_async(ids)` — fetch products by identifier
  - `purchase_async(product, options)` — purchase a product, returning a `VerificationResult<Transaction>`
  - `request_review_async()` — request an App Store review
  - `show_manage_subscriptions_async()` — open the subscription management sheet (always `NotSupported` on macOS)
  - `app_transaction_shared_async()` — fetch the verified `AppTransaction`
  - `storefront_current_async()` — fetch the current `Storefront` (if set)
- Three new numbered examples: `20_async_products`, `21_async_app_transaction`, `22_async_storefront`.
- New `doom-fish-utils` dependency (path = `"../doom-fish-utils"`) for `AsyncCompletion` / `AsyncCompletionFuture`.

## [0.2.1] - 2026-05-16

### Added

- Closed the remaining audited StoreKit 2 gaps with `Product::purchase_in_window(...)`, product formatting/localization helpers, `SubscriptionStatus::{updates,all}`, `PurchaseIntent::intents()`, `ExternalPurchase{,Link,CustomLink}`, App Store merchandising/advanced-commerce APIs, and typed framework-error decoding.
- Added numbered examples `16_purchase_intent.rs` through `19_typed_errors.rs` plus new integration tests covering the newly wrapped surfaces.

### Changed

- `Transaction` now exposes advanced-commerce info when StoreKit includes it in transaction payloads.
- `VerificationResult<Transaction>` and `VerificationResult<RenewalInfo>` now expose advanced-commerce info extracted from signed payload metadata.
- Coverage documentation now reflects full audited StoreKit 2 coverage on macOS.

## [0.2.0] - 2026-05-16

### Added

- Expanded StoreKit 2 coverage across Product, Transaction, AppStore, Storefront, Subscription, SubscriptionInfo, Refund, Message, AppTransaction, RenewalInfo, RenewalState, PurchaseOption, VerificationResult, ReceiptValidator, and StoreContext.
- Product metadata now includes family-sharing and JSON representation bytes, plus helpers for latest transactions and filtered entitlement streams.
- Transaction streams now cover `unfinished`, filtered product streams, `latest_for`, `current_entitlement_for`, refund entry points, and richer transaction metadata.
- App-level APIs now expose App Store environment/platform helpers, device verification IDs, storefront queries, and app transactions.
- Subscription wrappers now include introductory/promotional/win-back offers, renewal state, renewal info, and subscription status queries.
- New receipt and store-context utilities support app-receipt reads, JWS payload decoding, and bundle-context inspection.
- Added numbered examples `01_product_lookup.rs` through `15_verification_result.rs`.
- Added per-area integration tests under `tests/`.

## [0.1.0] - 2026-05-16

### Added

- Initial `storekit-rs` release.
- StoreKit 2 product lookup via `Product::products_for(...)`.
- `Product` metadata for identifiers, localized names/descriptions, prices, product types, and subscription details.
- Purchase initiation via `Product::purchase(...)` with support for quantity, app-account-token, and Ask to Buy sandbox options.
- `PurchaseResult` + `VerificationResult<Transaction>` wrappers.
- Transaction streams for `current_entitlements`, `all`, and `updates`.
- Transaction verification + finishing helpers.
- `AppStore::sync()` and a macOS `show_manage_subscriptions()` stub that reports the API as unavailable.
- Smoke example `examples/01_storekit_smoke.rs`.
