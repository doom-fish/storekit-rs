# Changelog

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
