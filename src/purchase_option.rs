use serde::{Deserialize, Serialize};

use crate::error::StoreKitError;
use crate::transaction::{Transaction, TransactionPayload};
use crate::verification_result::{VerificationResult, VerificationResultPayload};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
/// Represents options passed to `StoreKit.Product.purchase(options:)`.
pub enum PurchaseOption {
    /// Represents the `AppAccountToken` `StoreKit` case.
    AppAccountToken {
        /// App account token reported by `StoreKit`.
        app_account_token: String,
    },
    /// Represents the `Quantity` `StoreKit` case.
    Quantity {
        /// Value forwarded to `StoreKit` for `quantity`.
        quantity: i64,
    },
    /// Represents the `SimulatesAskToBuyInSandbox` `StoreKit` case.
    SimulatesAskToBuyInSandbox {
        /// Value forwarded to `StoreKit` for `simulate_ask_to_buy_in_sandbox`.
        simulate_ask_to_buy_in_sandbox: bool,
    },
    /// Represents the `CustomString` `StoreKit` case.
    CustomString {
        /// Value forwarded to `StoreKit` for `key`.
        key: String,
        /// Value returned by `StoreKit`.
        value: String,
    },
    /// Represents the `CustomNumber` `StoreKit` case.
    CustomNumber {
        /// Value forwarded to `StoreKit` for `key`.
        key: String,
        /// Value returned by `StoreKit`.
        value: f64,
    },
    /// Represents the `CustomBool` `StoreKit` case.
    CustomBool {
        /// Value forwarded to `StoreKit` for `key`.
        key: String,
        /// Value returned by `StoreKit`.
        value: bool,
    },
    /// Represents the `CustomData` `StoreKit` case.
    CustomData {
        /// Value forwarded to `StoreKit` for `key`.
        key: String,
        /// Value forwarded to `StoreKit` for `value_base64`.
        value_base64: String,
    },
    /// Represents the `PromotionalOfferSignature` `StoreKit` case.
    PromotionalOfferSignature {
        /// Value forwarded to `StoreKit` for `offer_id`.
        offer_id: String,
        /// Value forwarded to `StoreKit` for `key_id`.
        key_id: String,
        /// Value forwarded to `StoreKit` for `nonce`.
        nonce: String,
        /// Value forwarded to `StoreKit` for `signature_base64`.
        signature_base64: String,
        /// Value forwarded to `StoreKit` for `timestamp`.
        timestamp: i64,
    },
    /// Represents the `PromotionalOfferCompactJws` `StoreKit` case.
    PromotionalOfferCompactJws {
        /// Value forwarded to `StoreKit` for `offer_id`.
        offer_id: String,
        /// Value forwarded to `StoreKit` for `compact_jws`.
        compact_jws: String,
    },
    /// Represents the `IntroductoryOfferEligibility` `StoreKit` case.
    IntroductoryOfferEligibility {
        /// Value forwarded to `StoreKit` for `compact_jws`.
        compact_jws: String,
    },
    /// Represents the `WinBackOffer` `StoreKit` case.
    WinBackOffer {
        /// Value forwarded to `StoreKit` for `offer_id`.
        offer_id: String,
    },
    /// Represents the `OnStorefrontChange` `StoreKit` case.
    OnStorefrontChange {
        /// Value forwarded to `StoreKit` for `should_continue_purchase`.
        should_continue_purchase: bool,
    },
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
/// Represents the result returned by `StoreKit.Product.purchase(options:)`.
pub enum PurchaseResult {
    /// The `StoreKit` operation succeeded.
    Success(VerificationResult<Transaction>),
    /// The person cancelled the `StoreKit` flow.
    UserCancelled,
    /// The `StoreKit` flow is pending further action.
    Pending,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Deserialize)]
pub(crate) struct PurchaseResultPayload {
    kind: String,
    #[serde(rename = "verificationResult")]
    verification_result: Option<VerificationResultPayload<TransactionPayload>>,
}

impl PurchaseResultPayload {
    pub(crate) fn into_purchase_result(
        self,
        transaction_handle: *mut core::ffi::c_void,
    ) -> Result<PurchaseResult, StoreKitError> {
        match self.kind.as_str() {
            "success" => {
                let verification_result = self.verification_result.ok_or_else(|| {
                    StoreKitError::Unknown(
                        "StoreKit reported a successful purchase without a verification result"
                            .to_owned(),
                    )
                })?;
                let transaction = verification_result.into_result(|payload| {
                    Transaction::from_raw_parts(transaction_handle, payload)
                })?;
                Ok(PurchaseResult::Success(transaction))
            }
            "userCancelled" => {
                if !transaction_handle.is_null() {
                    unsafe { crate::ffi::sk_transaction_release(transaction_handle) };
                }
                Ok(PurchaseResult::UserCancelled)
            }
            "pending" => {
                if !transaction_handle.is_null() {
                    unsafe { crate::ffi::sk_transaction_release(transaction_handle) };
                }
                Ok(PurchaseResult::Pending)
            }
            other => {
                if !transaction_handle.is_null() {
                    unsafe { crate::ffi::sk_transaction_release(transaction_handle) };
                }
                Err(StoreKitError::Unknown(format!(
                    "StoreKit returned an unknown purchase result kind '{other}'"
                )))
            }
        }
    }
}
