# Changelog

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
