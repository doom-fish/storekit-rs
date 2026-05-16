# Changelog

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
