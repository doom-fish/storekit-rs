use serde::{Deserialize, Serialize};

use crate::error::StoreKitError;
use crate::transaction::{Transaction, TransactionPayload};
use crate::verification_result::{VerificationResult, VerificationResultPayload};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PurchaseOption {
    AppAccountToken {
        app_account_token: String,
    },
    Quantity {
        quantity: i64,
    },
    SimulatesAskToBuyInSandbox {
        simulate_ask_to_buy_in_sandbox: bool,
    },
    CustomString {
        key: String,
        value: String,
    },
    CustomNumber {
        key: String,
        value: f64,
    },
    CustomBool {
        key: String,
        value: bool,
    },
    CustomData {
        key: String,
        value_base64: String,
    },
    PromotionalOfferSignature {
        offer_id: String,
        key_id: String,
        nonce: String,
        signature_base64: String,
        timestamp: i64,
    },
    PromotionalOfferCompactJws {
        offer_id: String,
        compact_jws: String,
    },
    IntroductoryOfferEligibility {
        compact_jws: String,
    },
    WinBackOffer {
        offer_id: String,
    },
    OnStorefrontChange {
        should_continue_purchase: bool,
    },
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum PurchaseResult {
    Success(VerificationResult<Transaction>),
    UserCancelled,
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
