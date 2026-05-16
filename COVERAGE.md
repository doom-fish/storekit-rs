# StoreKit 2 coverage audit (v0.2.0)

Scope: `StoreKit.framework` on macOS, focused on the StoreKit 2 Swift API surface used by this crate.

Legend:

- ✅ implemented
- 🟡 partial
- ⏭️ skipped (unavailable on macOS, requires an injected AppKit scene/controller, or intentionally deferred new macOS 15.4+/26.x advanced-commerce APIs)

## Product

| API | Status | Notes |
| --- | --- | --- |
| `Product.products(for:)` | ✅ | `Product::products_for(...)` |
| `Product.purchase(options:)` | ✅ | `Product::purchase(...)` |
| `Product.purchase(confirmIn:options:)` | ⏭️ | Requires caller-owned `NSWindow`; not exposed in headless-safe API |
| `Product.latestTransaction` | ✅ | `Product::latest_transaction()` |
| `Product.currentEntitlements` / filtered entitlements | ✅ | `Product::current_entitlements()` via filtered transaction stream |
| `Product.id`, `type`, `displayName`, `description`, `price`, `displayPrice` | ✅ | Exposed on `Product` |
| `Product.isFamilyShareable` | ✅ | Exposed on `Product` |
| `Product.subscription` | ✅ | Exposed on `Product` |
| `Product.jsonRepresentation` | ✅ | Exposed as raw bytes |
| `Product.priceFormatStyle` / locale formatting helpers | 🟡 | Raw display price preserved; locale-specific format style is not wrapped |

## Transaction

| API | Status | Notes |
| --- | --- | --- |
| `Transaction.all` | ✅ | `Transaction::all()` |
| `Transaction.currentEntitlements` | ✅ | `Transaction::current_entitlements()` |
| `Transaction.updates` | ✅ | `Transaction::updates()` |
| `Transaction.unfinished` | ✅ | `Transaction::unfinished()` |
| `Transaction.latest(for:)` | ✅ | `Transaction::latest_for(...)` |
| `Transaction.currentEntitlement(for:)` | ✅ | `Transaction::current_entitlement_for(...)` |
| `Transaction.all(for:)` | ✅ | `Transaction::all_for(...)` via filtered stream |
| `Transaction.currentEntitlements(for:)` | ✅ | `Transaction::current_entitlements_for(...)` via filtered stream |
| `Transaction.finish()` | ✅ | `Transaction::finish()` |
| `VerificationResult<Transaction>.payloadValue` equivalent | ✅ | `VerificationResult<Transaction>` + `Transaction::verify()` |
| Core transaction fields (`id`, `originalID`, dates, quantity, ownership, bundle, JWS, signed date`) | ✅ | Exposed on `TransactionData` |
| `environment`, `reason`, `storefront`, `offer`, `currencyCode`, `appTransactionID` | ✅ | Exposed when available; absent on older runtimes remain `None` |
| `beginRefundRequest(for:in:)` | 🟡 | Headless-safe wrapper auto-discovers the first `NSViewController`; returns `NotSupported` without one |
| `advancedCommerceInfo` | ⏭️ | Deferred advanced-commerce payloads from newer macOS 15.4+/26.x APIs |

## AppStore

| API | Status | Notes |
| --- | --- | --- |
| `AppStore.canMakePayments` | ✅ | `AppStore::can_make_payments()` |
| `AppStore.deviceVerificationID` | ✅ | `AppStore::device_verification_id()` |
| `AppStore.sync()` | ✅ | `AppStore::sync()` |
| `AppStore.requestReview(in:)` | ✅ | `AppStore::request_review()`; requires discovered `NSViewController` |
| `AppStore.presentOfferCodeRedeemSheet(from:)` | ✅ | `AppStore::present_offer_code_redeem_sheet()`; requires discovered `NSViewController` |
| `AppStore.showManageSubscriptions(...)` | ⏭️ | Scene-based API remains unavailable on macOS StoreKit |
| `AppStore.presentMerchandising(...)` | ⏭️ | macOS 26.2 advanced merchandising deferred |

## Storefront

| API | Status | Notes |
| --- | --- | --- |
| `Storefront.current` | ✅ | `Storefront::current()` |
| `Storefront.updates` | ✅ | `Storefront::updates()` |
| `Storefront.countryCode`, `id`, `currency` | ✅ | Exposed on `Storefront` |

## Subscription

| API | Status | Notes |
| --- | --- | --- |
| `Product.SubscriptionPeriod` | ✅ | `SubscriptionPeriod` + `SubscriptionPeriodUnit` |
| `Product.SubscriptionOffer` | ✅ | `SubscriptionOffer` |
| `Product.SubscriptionOffer.OfferType` | ✅ | `SubscriptionOfferType` |
| `Product.SubscriptionOffer.PaymentMode` | ✅ | `SubscriptionPaymentMode` |
| `Product.SubscriptionOffer.Signature` | ✅ | Covered through purchase-option signature fields |

## SubscriptionInfo

| API | Status | Notes |
| --- | --- | --- |
| `introductoryOffer`, `promotionalOffers`, `winBackOffers` | ✅ | Exposed on `SubscriptionInfo` |
| `subscriptionGroupID`, `subscriptionPeriod` | ✅ | Exposed on `SubscriptionInfo` |
| `groupLevel`, `groupDisplayName` | ✅ | Exposed when available |
| `isEligibleForIntroOffer(for:)` | ✅ | `SubscriptionInfo::is_eligible_for_intro_offer_for(...)` |
| `status(for:)` | ✅ | `SubscriptionInfo::status_for(...)` |
| `status(transactionID:)` | ✅ | `SubscriptionInfo::status_for_transaction(...)` |
| `Status.updates` / `Status.all` streams | 🟡 | Synchronous point queries implemented; stream surface intentionally deferred |

## RenewalInfo

| API | Status | Notes |
| --- | --- | --- |
| Core renewal info fields (`originalTransactionID`, `currentProductID`, `willAutoRenew`, retry/grace dates`) | ✅ | Exposed on `RenewalInfo` |
| `expirationReason`, `priceIncreaseStatus` | ✅ | Exposed as Rust enums |
| `offer`, `environment`, `recentSubscriptionStartDate`, `renewalDate`, `renewalPrice`, `currency` | ✅ | Exposed when available |
| `eligibleWinBackOfferIDs`, `appAccountToken`, `appTransactionID` | ✅ | Exposed when available |
| `advancedCommerceInfo` | ⏭️ | Deferred advanced-commerce payloads |

## RenewalState

| API | Status | Notes |
| --- | --- | --- |
| `subscribed`, `expired`, `inBillingRetryPeriod`, `inGracePeriod`, `revoked` | ✅ | `RenewalState` enum |

## PurchaseOption

| API | Status | Notes |
| --- | --- | --- |
| `appAccountToken(_:)` | ✅ | `PurchaseOption::AppAccountToken` |
| `quantity(_:)` | ✅ | `PurchaseOption::Quantity` |
| `simulatesAskToBuyInSandbox(_:)` | ✅ | `PurchaseOption::SimulatesAskToBuyInSandbox` |
| `custom(key:value:)` string/double/bool/data | ✅ | `CustomString`, `CustomNumber`, `CustomBool`, `CustomData` |
| `promotionalOffer(...)` signature form | ✅ | `PromotionalOfferSignature` |
| `promotionalOffer(_:compactJWS:)` | ✅ | `PromotionalOfferCompactJws` |
| `introductoryOfferEligibility(compactJWS:)` | ✅ | `IntroductoryOfferEligibility` |
| `winBackOffer(_:)` | ✅ | `WinBackOffer` via offer-id lookup on the fetched product |
| `onStorefrontChange(...)` | 🟡 | Exposed as a constant continue/cancel policy; arbitrary Rust closures are not bridged |

## VerificationResult

| API | Status | Notes |
| --- | --- | --- |
| `verified` / `unverified` cases | ✅ | `VerificationResult<T>` |
| `payloadValue` / `unsafePayloadValue` equivalents | ✅ | `payload()`, `into_payload()`, `verification_failure()` |
| JWS metadata (`jwsRepresentation`, header/payload/signature/signed data, signed date, device verification`) | ✅ | `VerificationMetadata` |
| `VerificationResult<Transaction>` | ✅ | Purchase, transaction streams, and lookups |
| `VerificationResult<RenewalInfo>` | ✅ | Subscription status queries |
| `VerificationResult<AppTransaction>` | ✅ | App transaction queries |

## AppTransaction

| API | Status | Notes |
| --- | --- | --- |
| `AppTransaction.shared` | ✅ | `AppTransaction::shared()` |
| `AppTransaction.refresh()` | ✅ | `AppTransaction::refresh()` |
| Core app transaction fields (`bundleID`, versions, purchase date, environment`) | ✅ | Exposed on `AppTransaction` |
| `originalPlatform` | ✅ | Exposed when available |
| `jsonRepresentation` | ✅ | Exposed as raw bytes |

## Refund

| API | Status | Notes |
| --- | --- | --- |
| `Transaction.beginRefundRequest(for:in:)` | 🟡 | Headless-safe wrapper uses the first discovered `NSViewController` and otherwise returns `NotSupported` |
| `RefundRequestStatus.success` / `userCancelled` | ✅ | `RefundRequestStatus` |

## Message

| API | Status | Notes |
| --- | --- | --- |
| `Message` / `Message.messages` | ⏭️ | Apple marks `StoreKit.Message` unavailable on macOS; Rust module returns `NotSupported` |

## ReceiptValidator (crate utility, not Apple StoreKit API)

| API | Status | Notes |
| --- | --- | --- |
| Read current app receipt bytes | ✅ | `ReceiptValidator::current_receipt()` |
| Decode JWS payload without verification | ✅ | `ReceiptValidator::extract_unverified_payload(...)` |
| Validate via `AppTransaction` bridge | ✅ | Delegates to `AppTransaction::shared()` / `refresh()` |

## StoreContext (crate utility, not Apple StoreKit API)

| API | Status | Notes |
| --- | --- | --- |
| Bundle / executable / receipt summary | ✅ | `StoreContext::current()` |
| Payment and device-verification convenience helpers | ✅ | Delegates to `AppStore` |
| Product, entitlement, storefront, receipt convenience wrappers | ✅ | Delegates to the corresponding modules |
