#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod async_api;

/// `StoreKit` advanced-commerce wrappers and related value types.
pub mod advanced_commerce;
/// Wrappers for `StoreKit.AppStore` APIs.
pub mod app_store;
/// Wrappers for `StoreKit.AppTransaction`.
pub mod app_transaction;
/// Typed errors surfaced by the `StoreKit` framework.
pub mod error;
/// Wrappers for `StoreKit` external purchase APIs.
pub mod external_purchase;
mod ffi;
/// Wrappers for `StoreKit.Message` and related APIs.
pub mod message;
mod private;
/// Wrappers for `StoreKit.Product` and related value types.
pub mod product;
mod product_helpers;
/// Wrappers for `StoreKit.PurchaseIntent` and its stream APIs.
pub mod purchase_intent;
/// Purchase options and results used with `StoreKit.Product.purchase`.
pub mod purchase_option;
/// Helpers for `StoreKit` receipts, app transactions, and JWS payloads.
pub mod receipt_validator;
/// Helpers for `StoreKit` refund request APIs.
pub mod refund;
/// Wrappers for `StoreKit.RenewalInfo` and related enums.
pub mod renewal_info;
/// Wrappers for `StoreKit.Product.SubscriptionInfo.RenewalState`.
pub mod renewal_state;
/// Helpers that collect bundle context used alongside `StoreKit` APIs.
pub mod store_context;
/// Wrappers for `StoreKit.Storefront` and storefront streams.
pub mod storefront;
/// Wrappers for `StoreKit.Product.SubscriptionInfo` value types.
pub mod subscription;
/// Wrappers for `StoreKit.Product.SubscriptionInfo.Status` and related APIs.
pub mod subscription_info;
mod subscription_status_stream;
/// Wrappers for `StoreKit.Transaction` and transaction streams.
pub mod transaction;
/// Wrappers for `StoreKit.VerificationResult` and verification metadata.
pub mod verification_result;
/// Opaque `NSWindow` handles used with `StoreKit` APIs that present UI.
pub mod window;

pub use advanced_commerce::{
    AdvancedCommerceProduct, AdvancedCommercePurchaseOption, AppStoreMerchandisingKind,
    AppStoreMerchandisingPresentationResult, RenewalInfoAdvancedCommerceInfo,
    RenewalInfoAdvancedCommerceItem, RenewalInfoAdvancedCommercePriceIncreaseInfo,
    RenewalInfoAdvancedCommercePriceIncreaseStatus, TransactionAdvancedCommerceInfo,
    TransactionAdvancedCommerceItem, TransactionAdvancedCommerceItemDetails,
    TransactionAdvancedCommerceOffer, TransactionAdvancedCommerceOfferReason,
    TransactionAdvancedCommerceRefund, TransactionAdvancedCommerceRefundReason,
    TransactionAdvancedCommerceRefundType,
};
pub use app_store::{AppStore, AppStoreEnvironment, AppStorePlatform};
pub use app_transaction::AppTransaction;
pub use error::{
    InvalidRequestError, ProductPurchaseError, ProductPurchaseErrorCode, RefundRequestError,
    RefundRequestErrorCode, StoreKitApiError, StoreKitApiErrorCode, StoreKitError,
    StoreKitFrameworkError, TypedStoreKitError, VerificationErrorCode, VerificationFailure,
};
pub use external_purchase::{
    ExternalPurchase, ExternalPurchaseCustomLink, ExternalPurchaseCustomLinkNoticeResult,
    ExternalPurchaseCustomLinkNoticeType, ExternalPurchaseCustomLinkToken, ExternalPurchaseLink,
    ExternalPurchaseNoticeResult,
};
pub use message::{Message, MessageReason, MessageStream};
pub use product::{Product, ProductType};
pub use product_helpers::ProductFormatting;
pub use purchase_intent::{PurchaseIntent, PurchaseIntentStream};
pub use purchase_option::{PurchaseOption, PurchaseResult};
pub use receipt_validator::{AppReceipt, ReceiptValidator};
pub use refund::{Refund, RefundRequestStatus};
pub use renewal_info::{ExpirationReason, PriceIncreaseStatus, RenewalCommitmentInfo, RenewalInfo};
pub use renewal_state::RenewalState;
pub use store_context::StoreContext;
pub use storefront::{Storefront, StorefrontStream};
pub use subscription::{
    SubscriptionOffer, SubscriptionOfferType, SubscriptionPaymentMode, SubscriptionPeriod,
    SubscriptionPeriodUnit,
};
pub use subscription_info::{
    BillingPlanType, SubscriptionCommitmentInfo, SubscriptionInfo, SubscriptionPricingTerms,
    SubscriptionStatus,
};
pub use subscription_status_stream::{
    SubscriptionGroupStatusStream, SubscriptionGroupStatuses, SubscriptionStatusStream,
};
pub use transaction::{
    OfferPaymentMode, OfferType, OwnershipType, RevocationReason, RevocationType, Transaction,
    TransactionCommitmentInfo, TransactionData, TransactionOffer, TransactionReason,
    TransactionStream,
};
pub use verification_result::{VerificationMetadata, VerificationResult};
pub use window::NSWindowHandle;

/// Common imports.
pub mod prelude {
    pub use crate::advanced_commerce::{
        AdvancedCommerceProduct, AdvancedCommercePurchaseOption, AppStoreMerchandisingKind,
        AppStoreMerchandisingPresentationResult, RenewalInfoAdvancedCommerceInfo,
        RenewalInfoAdvancedCommerceItem, RenewalInfoAdvancedCommercePriceIncreaseInfo,
        RenewalInfoAdvancedCommercePriceIncreaseStatus, TransactionAdvancedCommerceInfo,
        TransactionAdvancedCommerceItem, TransactionAdvancedCommerceItemDetails,
        TransactionAdvancedCommerceOffer, TransactionAdvancedCommerceOfferReason,
        TransactionAdvancedCommerceRefund, TransactionAdvancedCommerceRefundReason,
        TransactionAdvancedCommerceRefundType,
    };
    pub use crate::app_store::{AppStore, AppStoreEnvironment, AppStorePlatform};
    pub use crate::app_transaction::AppTransaction;
    pub use crate::error::{
        InvalidRequestError, ProductPurchaseError, ProductPurchaseErrorCode, RefundRequestError,
        RefundRequestErrorCode, StoreKitApiError, StoreKitApiErrorCode, StoreKitError,
        StoreKitFrameworkError, TypedStoreKitError, VerificationErrorCode, VerificationFailure,
    };
    pub use crate::external_purchase::{
        ExternalPurchase, ExternalPurchaseCustomLink, ExternalPurchaseCustomLinkNoticeResult,
        ExternalPurchaseCustomLinkNoticeType, ExternalPurchaseCustomLinkToken,
        ExternalPurchaseLink, ExternalPurchaseNoticeResult,
    };
    pub use crate::message::{Message, MessageReason, MessageStream};
    pub use crate::product::{Product, ProductType};
    pub use crate::product_helpers::ProductFormatting;
    pub use crate::purchase_intent::{PurchaseIntent, PurchaseIntentStream};
    pub use crate::purchase_option::{PurchaseOption, PurchaseResult};
    pub use crate::receipt_validator::{AppReceipt, ReceiptValidator};
    pub use crate::refund::{Refund, RefundRequestStatus};
    pub use crate::renewal_info::{
        ExpirationReason, PriceIncreaseStatus, RenewalCommitmentInfo, RenewalInfo,
    };
    pub use crate::renewal_state::RenewalState;
    pub use crate::store_context::StoreContext;
    pub use crate::storefront::{Storefront, StorefrontStream};
    pub use crate::subscription::{
        SubscriptionOffer, SubscriptionOfferType, SubscriptionPaymentMode, SubscriptionPeriod,
        SubscriptionPeriodUnit,
    };
    pub use crate::subscription_info::{
        BillingPlanType, SubscriptionCommitmentInfo, SubscriptionInfo, SubscriptionPricingTerms,
        SubscriptionStatus,
    };
    pub use crate::subscription_status_stream::{
        SubscriptionGroupStatusStream, SubscriptionGroupStatuses, SubscriptionStatusStream,
    };
    pub use crate::transaction::{
        OfferPaymentMode, OfferType, OwnershipType, RevocationReason, RevocationType, Transaction,
        TransactionCommitmentInfo, TransactionData, TransactionOffer, TransactionReason,
        TransactionStream,
    };
    pub use crate::verification_result::{VerificationMetadata, VerificationResult};
    pub use crate::window::NSWindowHandle;
}
