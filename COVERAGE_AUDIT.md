# storekit-rs coverage audit (vs MacOSX26.5.sdk)

SDK_PUBLIC_SYMBOLS: 95
VERIFIED: 85
GAPS: 0
EXEMPT: 10
COVERAGE_PCT: 100.0%

Scope notes:
- Audited the macOS-reachable StoreKit 2 commerce surface in `StoreKit.framework/Versions/A/Modules/StoreKit.swiftmodule/arm64e-apple-macos.swiftinterface`.
- Excluded legacy `SK*` StoreKit 1 Objective-C APIs plus protocol-conformance/debug noise (`Equatable`, `Hashable`, `Sendable`, `debugDescription`, etc.).
- Rows collapse closely related overloads/properties into one audit unit. Rust-side wrappers sometimes normalize `Date`, `Decimal`, `UUID`, and `Locale.Currency` into strings/bytes; these still count as VERIFIED when the StoreKit data is publicly reachable.
- Excluded `StoreDownloaderExtension` from the counts because it is a `BackgroundAssets` extension-point protocol rather than a runtime StoreKit 2 commerce API.
- Refreshed against `MacOSX26.5.sdk`; the 38 new macOS 26.4/26.5 billing-plan, pricing-terms, commitment, and revocation symbols are now wrapped.

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| `Product` | struct | `StoreKit.swiftinterface` | `storekit::Product` |
| `Product.products(for:)` | static func | `StoreKit.swiftinterface` | `Product::products_for(...)` |
| `Product.purchase(options:)` | instance func | `StoreKit.swiftinterface` | `Product::purchase(...)` |
| `Product.latestTransaction` | async property | `StoreKit.swiftinterface` | `Product::latest_transaction()` |
| `Product.currentEntitlements` | async property | `StoreKit.swiftinterface` | `Product::current_entitlements()` via filtered `Transaction` stream |
| `Product.{id,type,displayName,description,price,displayPrice,isFamilyShareable,subscription,jsonRepresentation}` | property family | `StoreKit.swiftinterface` | `Product` fields |
| `Product.ProductType` | nested type | `StoreKit.swiftinterface` | `ProductType` |
| `Product.SubscriptionOffer` | nested struct | `StoreKit.swiftinterface` | `SubscriptionOffer` |
| `Product.SubscriptionOffer.{OfferType,PaymentMode}` | nested type family | `StoreKit.swiftinterface` | `SubscriptionOfferType`, `SubscriptionPaymentMode` |
| `Product.SubscriptionPeriod` | nested struct | `StoreKit.swiftinterface` | `SubscriptionPeriod` |
| `Product.SubscriptionPeriod.Unit` | nested type | `StoreKit.swiftinterface` | `SubscriptionPeriodUnit` |
| `Product.SubscriptionInfo` | nested struct | `StoreKit.swiftinterface` | `SubscriptionInfo` |
| `Product.SubscriptionInfo.{BillingPeriod,BillingPlanType,CommitmentInfo,PricingTerms}` | nested type family | `StoreKit.swiftinterface` | `SubscriptionPeriod`, `BillingPlanType`, `SubscriptionCommitmentInfo`, `SubscriptionPricingTerms` |
| `Product.SubscriptionInfo.{introductoryOffer,promotionalOffers,winBackOffers,subscriptionGroupID,subscriptionPeriod,pricingTerms,groupLevel,groupDisplayName}` | property family | `StoreKit.swiftinterface` | `SubscriptionInfo` fields |
| `Product.SubscriptionInfo.{isEligibleForIntroOffer,isEligibleForIntroOffer(for:)}` | property + static func | `StoreKit.swiftinterface` | `SubscriptionInfo::is_eligible_for_intro_offer()`, `SubscriptionInfo::is_eligible_for_intro_offer_for(...)` |
| `Product.SubscriptionInfo.{status,status(for:)}` | property + static func | `StoreKit.swiftinterface` | `SubscriptionInfo::status()`, `SubscriptionInfo::status_for(...)` |
| `Product.SubscriptionInfo.status(transactionID:)` | static func | `StoreKit.swiftinterface` | `SubscriptionInfo::status_for_transaction(...)` |
| `Product.SubscriptionInfo.RenewalState` | nested type | `StoreKit.swiftinterface` | `RenewalState` |
| `Product.SubscriptionInfo.RenewalInfo` | nested struct | `StoreKit.swiftinterface` | `RenewalInfo` |
| `Product.SubscriptionInfo.RenewalInfo.{ExpirationReason,PriceIncreaseStatus,CommitmentInfo}` | nested type family | `StoreKit.swiftinterface` | `ExpirationReason`, `PriceIncreaseStatus`, `RenewalCommitmentInfo` |
| `Product.SubscriptionInfo.RenewalInfo.{originalTransactionID,currentProductID,willAutoRenew,autoRenewPreference,expirationReason,priceIncreaseStatus,isInBillingRetry,gracePeriodExpirationDate,offer,environment,recentSubscriptionStartDate,renewalDate,renewalPrice,commitmentInfo,renewalBillingPlanType,currency,eligibleWinBackOfferIDs,appAccountToken,appTransactionID,jsonRepresentation}` | property family | `StoreKit.swiftinterface` | `RenewalInfo` fields (currency normalized to ISO code string) |
| `Product.SubscriptionInfo.Status` | nested struct | `StoreKit.swiftinterface` | `SubscriptionStatus` |
| `Product.SubscriptionInfo.Status.{state,transaction,renewalInfo}` | property family | `StoreKit.swiftinterface` | `SubscriptionStatus` fields |
| `Product.PurchaseOption.{appAccountToken,billingPlanType(_:),quantity,simulatesAskToBuyInSandbox,custom(key:value:)}` | static func family | `StoreKit.swiftinterface` | `PurchaseOption::{AppAccountToken,BillingPlanType,Quantity,SimulatesAskToBuyInSandbox,Custom*}` |
| `Product.PurchaseOption.{promotionalOffer(_:compactJWS),introductoryOfferEligibility(compactJWS:),winBackOffer(_)}` | static func family | `StoreKit.swiftinterface` | `PurchaseOption::{PromotionalOfferCompactJws,IntroductoryOfferEligibility,WinBackOffer}` |
| `Product.PurchaseOption.onStorefrontChange(shouldContinuePurchase:)` | static func | `StoreKit.swiftinterface` | `PurchaseOption::OnStorefrontChange` (constant continue/cancel policy) |
| `Product.PurchaseResult` | nested enum | `StoreKit.swiftinterface` | `PurchaseResult` |
| `Transaction` | struct | `StoreKit.swiftinterface` | `storekit::Transaction` |
| `Transaction.{all,currentEntitlements,updates,unfinished}` | static var family | `StoreKit.swiftinterface` | `Transaction::{all,current_entitlements,updates,unfinished}` |
| `Transaction.{latest(for:),all(for:),currentEntitlements(for:)}` | static lookup/stream family | `StoreKit.swiftinterface` | `Transaction::{latest_for,all_for,current_entitlements_for}` |
| `Transaction.finish()` | instance func | `StoreKit.swiftinterface` | `Transaction::finish()` |
| `Transaction.{beginRefundRequest(in:),beginRefundRequest(for:in:),subscriptionStatus}` | func/property family | `StoreKit.swiftinterface` | `Transaction::begin_refund_request()`, `Refund::begin_for_transaction_id(...)`, `SubscriptionInfo::status_for_transaction(...)` |
| `Transaction.{Reason,RevocationReason,RevocationType,OfferType,OwnershipType,Offer,Offer.PaymentMode,RefundRequestStatus,CommitmentInfo}` | nested type family | `StoreKit.swiftinterface` | `TransactionReason`, `RevocationReason`, `RevocationType`, `OfferType`, `OwnershipType`, `TransactionOffer`, `OfferPaymentMode`, `RefundRequestStatus`, `TransactionCommitmentInfo` |
| `Transaction.{id,originalID,webOrderLineItemID,productID,subscriptionGroupID,appBundleID,purchaseDate,originalPurchaseDate,expirationDate,purchasedQuantity,isUpgraded,offer,revocationDate,revocationReason,revocationType,productType,appAccountToken,environment,reason,storefront,price,currency,billingPlanType,commitmentInfo,appTransactionID,jsonRepresentation}` | property family | `StoreKit.swiftinterface` | `TransactionData` fields + `VerificationMetadata` |
| `VerificationResult.{verified,unverified,payloadValue,unsafePayloadValue,VerificationError}` | enum/accessor family | `StoreKit.swiftinterface` | `VerificationResult<T>`, `VerificationErrorCode`, `VerificationFailure` |
| `VerificationResult<Transaction/AppTransaction/RenewalInfo>.{jwsRepresentation,headerData,payloadData,signatureData,signedData,signedDate,deviceVerification,deviceVerificationNonce}` | extension property family | `StoreKit.swiftinterface` | `VerificationMetadata` |
| `Storefront` | struct | `StoreKit.swiftinterface` | `storekit::Storefront` |
| `Storefront.{countryCode,id,currency,current,updates}` | property + async sequence family | `StoreKit.swiftinterface` | `Storefront` fields, `Storefront::current()`, `Storefront::updates()` |
| `AppStore.{canMakePayments,deviceVerificationID,sync,requestReview(in:),presentOfferCodeRedeemSheet(from:),Environment,Platform}` | enum/func/type family | `StoreKit.swiftinterface` | `AppStore`, `AppStoreEnvironment`, `AppStorePlatform` |
| `AppTransaction` | struct | `StoreKit.swiftinterface` | `storekit::AppTransaction` |
| `AppTransaction.{shared,refresh,appID,appTransactionID,appVersion,appVersionID,bundleID,environment,originalAppVersion,originalPurchaseDate,originalPlatform,preorderDate,jsonRepresentation}` | static func + property family | `StoreKit.swiftinterface` | `AppTransaction` fields + `VerificationMetadata` |
| `Product.purchase(confirmIn:options:)` | instance func | `StoreKit.swiftinterface` | `Product::purchase_in_window(...)` via `NSWindowHandle` |
| `Product formatting/localization helpers` | property/func family | `StoreKit.swiftinterface` | `ProductFormatting`, `SubscriptionPeriod` convenience constructors, and `localized_description()` helpers |
| `Product.SubscriptionInfo.Status.{updates,all}` | async stream family | `StoreKit.swiftinterface` | `SubscriptionStatus::{updates,all}()` |
| `PurchaseIntent` family | struct + async sequence | `StoreKit.swiftinterface` | `PurchaseIntent`, `PurchaseIntent::intents()` |
| `ExternalPurchase{,Link,CustomLink}` families | enum families | `StoreKit.swiftinterface` | `ExternalPurchase`, `ExternalPurchaseLink`, `ExternalPurchaseCustomLink` |
| `AppStore merchandising / advanced-commerce family` | func/type family | `StoreKit.swiftinterface` | `AppStore::present_merchandising(...)`, `AppStore::age_rating_code()`, `AdvancedCommerceProduct`, and advanced-commerce info helpers |
| `Typed StoreKit error enums` | enum/struct family | `StoreKit.swiftinterface` | `StoreKitError::typed()` + typed `StoreKit`, purchase, refund-request, and invalid-request errors |

## 🔴 GAPS
None.

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| `Message` | struct | `StoreKit.swiftinterface` | StoreKit marks `Message` unavailable on macOS; the Rust module intentionally returns `NotSupported`. | `@available(macOS, unavailable)` |
| `Message.Reason` | nested type | `StoreKit.swiftinterface` | Same macOS-unavailable family as `Message`. | `@available(macOS, unavailable)` |
| `Message.messages` | static var / async sequence | `StoreKit.swiftinterface` | Same macOS-unavailable family as `Message`. | `@available(macOS, unavailable)` |
| `Product.currentEntitlement` | async property | `StoreKit.swiftinterface` | Deprecated by Apple in favor of `currentEntitlements`; not counted against coverage. | `@available(macOS, introduced: 12.0, deprecated: 15.4, ...)` |
| `Transaction.currentEntitlement(for:)` | static func | `StoreKit.swiftinterface` | Deprecated by Apple in favor of `currentEntitlements(for:)`; not counted against coverage. | `@available(macOS, introduced: 12.0, deprecated: 15.4, ...)` |
| `Transaction.{offerType,offerID,offerPaymentModeStringRepresentation,offerPeriodStringRepresentation,environmentStringRepresentation,reasonStringRepresentation,storefrontCountryCode,currencyCode}` | compatibility property family | `StoreKit.swiftinterface` | Deprecated compatibility accessors; the newer structured properties are already wrapped. | various `@available(... deprecated ...)` |
| `RenewalInfo.{offerID,offerType,offerPaymentModeStringRepresentation,offerPeriodStringRepresentation,environmentStringRepresentation,currencyCode}` | compatibility property family | `StoreKit.swiftinterface` | Deprecated compatibility accessors; the newer structured properties are already wrapped. | various `@available(... deprecated ...)` |
| `Product.SubscriptionOffer.Signature` | nested struct | `StoreKit.swiftinterface` | Deprecated by Apple in favor of compact-JWS signing. | `@available(macOS, introduced: 14.4, deprecated: 26.0, ...)` |
| `Product.PurchaseOption.promotionalOffer(offerID:keyID:...) / promotionalOffer(offerID:signature:)` | static func overloads | `StoreKit.swiftinterface` | Deprecated by Apple in favor of `promotionalOffer(_:compactJWS:)`. | `@available(... deprecated: 26.0, ...)` |
| `AppTransaction.originalPlatformStringRepresentation` | property | `StoreKit.swiftinterface` | Deprecated by Apple in favor of `originalPlatform`. | `@available(macOS, introduced: 13.0, deprecated: 15.4, ...)` |
