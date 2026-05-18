use serde::Deserialize;

use crate::app_store::AppStore;
use crate::app_transaction::AppTransaction;
use crate::error::StoreKitError;
use crate::ffi;
use crate::product::Product;
use crate::receipt_validator::{AppReceipt, ReceiptValidator};
use crate::storefront::Storefront;
use crate::transaction::{Transaction, TransactionStream};
use crate::verification_result::VerificationResult;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Collects bundle context used alongside `StoreKit` APIs.
pub struct StoreContext {
    /// Bundle identifier reported by `StoreKit`.
    pub bundle_identifier: Option<String>,
    /// Bundle display name reported by `StoreKit`.
    pub bundle_name: Option<String>,
    /// Bundle version reported by `StoreKit`.
    pub bundle_version: Option<String>,
    /// Receipt URL reported by `StoreKit`.
    pub receipt_url: Option<String>,
    /// Whether `StoreKit` reports that payments can be made.
    pub can_make_payments: bool,
    /// Device verification identifier reported by `StoreKit`.
    pub device_verification_id: Option<String>,
    /// Whether `StoreKit` reported that the app is bundled.
    pub is_bundled: bool,
    /// Executable path reported by `StoreKit`.
    pub executable_path: Option<String>,
}

impl StoreContext {
    /// Fetches the current bundle context used alongside `StoreKit` APIs.
    pub fn current() -> Result<Self, StoreKitError> {
        let mut context_json = core::ptr::null_mut();
        let mut error_message = core::ptr::null_mut();
        let status = unsafe { ffi::sk_store_context_json(&mut context_json, &mut error_message) };
        if status != ffi::status::OK {
            return Err(unsafe { crate::private::error_from_status(status, error_message) });
        }
        let payload = unsafe {
            crate::private::parse_json_ptr::<StoreContextPayload>(context_json, "store context")
        }?;
        Ok(payload.into_store_context())
    }

    /// Returns whether `StoreKit` reports that payments can be made.
    pub fn can_make_payments() -> Result<bool, StoreKitError> {
        AppStore::can_make_payments()
    }

    /// Fetches the `StoreKit` device verification identifier.
    pub fn device_verification_id() -> Result<Option<String>, StoreKitError> {
        AppStore::device_verification_id()
    }

    /// Fetches current `StoreKit` products for the supplied identifiers.
    pub fn current_products<I, S>(identifiers: I) -> Result<Vec<Product>, StoreKitError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Product::products_for(identifiers)
    }

    /// Creates a stream of current `StoreKit` entitlements.
    pub fn current_entitlements() -> Result<TransactionStream, StoreKitError> {
        Transaction::current_entitlements()
    }

    /// Creates a stream of `StoreKit` transaction updates.
    pub fn transaction_updates() -> Result<TransactionStream, StoreKitError> {
        Transaction::updates()
    }

    /// Fetches the current `StoreKit` storefront.
    pub fn current_storefront() -> Result<Option<Storefront>, StoreKitError> {
        Storefront::current()
    }

    /// Fetches the shared `StoreKit` app transaction.
    pub fn app_transaction() -> Result<VerificationResult<AppTransaction>, StoreKitError> {
        AppTransaction::shared()
    }

    /// Fetches the current app receipt used by `StoreKit`.
    pub fn receipt() -> Result<Option<AppReceipt>, StoreKitError> {
        ReceiptValidator::current_receipt()
    }
}

#[derive(Debug, Deserialize)]
struct StoreContextPayload {
    #[serde(rename = "bundleIdentifier")]
    bundle_identifier: Option<String>,
    #[serde(rename = "bundleName")]
    bundle_name: Option<String>,
    #[serde(rename = "bundleVersion")]
    bundle_version: Option<String>,
    #[serde(rename = "receiptURL")]
    receipt_url: Option<String>,
    #[serde(rename = "canMakePayments")]
    can_make_payments: bool,
    #[serde(rename = "deviceVerificationID")]
    device_verification_id: Option<String>,
    #[serde(rename = "isBundled")]
    is_bundled: bool,
    #[serde(rename = "executablePath")]
    executable_path: Option<String>,
}

impl StoreContextPayload {
    fn into_store_context(self) -> StoreContext {
        StoreContext {
            bundle_identifier: self.bundle_identifier,
            bundle_name: self.bundle_name,
            bundle_version: self.bundle_version,
            receipt_url: self.receipt_url,
            can_make_payments: self.can_make_payments,
            device_verification_id: self.device_verification_id,
            is_bundled: self.is_bundled,
            executable_path: self.executable_path,
        }
    }
}
