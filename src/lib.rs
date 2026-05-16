#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

pub mod app_store;
pub mod error;
mod ffi;
mod private;
pub mod product;
pub mod transaction;

pub use app_store::AppStore;
pub use error::{
    StoreKitError, StoreKitFrameworkError, VerificationErrorCode, VerificationFailure,
};
pub use product::{
    Product, ProductType, PurchaseOption, PurchaseResult, SubscriptionInfo, SubscriptionPeriod,
    SubscriptionPeriodUnit,
};
pub use transaction::{OwnershipType, Transaction, TransactionData, TransactionStream, VerificationResult};

/// Common imports.
pub mod prelude {
    pub use crate::app_store::AppStore;
    pub use crate::error::{
        StoreKitError, StoreKitFrameworkError, VerificationErrorCode, VerificationFailure,
    };
    pub use crate::product::{
        Product, ProductType, PurchaseOption, PurchaseResult, SubscriptionInfo,
        SubscriptionPeriod, SubscriptionPeriodUnit,
    };
    pub use crate::transaction::{
        OwnershipType, Transaction, TransactionData, TransactionStream, VerificationResult,
    };
}
