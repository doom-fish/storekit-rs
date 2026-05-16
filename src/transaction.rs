use core::ffi::c_void;
use core::ptr;
use std::ptr::NonNull;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::app_store::AppStoreEnvironment;
use crate::error::{StoreKitError, VerificationFailure};
use crate::ffi;
use crate::private::{
    cstring_from_str, decode_base64, duration_to_timeout_ms, error_from_status, json_cstring,
    parse_json_ptr, parse_optional_json_ptr,
};
use crate::product::ProductType;
use crate::refund::{Refund, RefundRequestStatus};
use crate::storefront::{Storefront, StorefrontPayload};
use crate::subscription::{SubscriptionPeriod, SubscriptionPeriodPayload};
pub use crate::verification_result::VerificationResult;

use crate::verification_result::VerificationResultPayload;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionReason {
    Purchase,
    Renewal,
    Unknown(String),
}

impl TransactionReason {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Purchase => "purchase",
            Self::Renewal => "renewal",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "purchase" => Self::Purchase,
            "renewal" => Self::Renewal,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevocationReason {
    DeveloperIssue,
    Other,
    Unknown(String),
}

impl RevocationReason {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DeveloperIssue => "developerIssue",
            Self::Other => "other",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "developerIssue" => Self::DeveloperIssue,
            "other" => Self::Other,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfferType {
    Introductory,
    Promotional,
    Code,
    WinBack,
    Unknown(String),
}

impl OfferType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Introductory => "introductory",
            Self::Promotional => "promotional",
            Self::Code => "code",
            Self::WinBack => "winBack",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "introductory" => Self::Introductory,
            "promotional" => Self::Promotional,
            "code" => Self::Code,
            "winBack" => Self::WinBack,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfferPaymentMode {
    FreeTrial,
    PayAsYouGo,
    PayUpFront,
    OneTime,
    Unknown(String),
}

impl OfferPaymentMode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::FreeTrial => "freeTrial",
            Self::PayAsYouGo => "payAsYouGo",
            Self::PayUpFront => "payUpFront",
            Self::OneTime => "oneTime",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "freeTrial" => Self::FreeTrial,
            "payAsYouGo" => Self::PayAsYouGo,
            "payUpFront" => Self::PayUpFront,
            "oneTime" => Self::OneTime,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionOffer {
    pub id: Option<String>,
    pub offer_type: OfferType,
    pub payment_mode: Option<OfferPaymentMode>,
    pub period: Option<SubscriptionPeriod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipType {
    Purchased,
    FamilyShared,
    Unknown(String),
}

impl OwnershipType {
    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "purchased" => Self::Purchased,
            "familyShared" => Self::FamilyShared,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionData {
    pub id: u64,
    pub original_id: u64,
    pub web_order_line_item_id: Option<String>,
    pub product_id: String,
    pub subscription_group_id: Option<String>,
    pub app_bundle_id: String,
    pub purchase_date: String,
    pub original_purchase_date: String,
    pub expiration_date: Option<String>,
    pub purchased_quantity: u64,
    pub is_upgraded: bool,
    pub ownership_type: OwnershipType,
    pub signed_date: String,
    pub jws_representation: String,
    pub verification_failure: Option<VerificationFailure>,
    pub revocation_date: Option<String>,
    pub revocation_reason: Option<RevocationReason>,
    pub product_type: Option<ProductType>,
    pub app_account_token: Option<String>,
    pub environment: Option<AppStoreEnvironment>,
    pub reason: Option<TransactionReason>,
    pub storefront: Option<Storefront>,
    pub price: Option<String>,
    pub currency_code: Option<String>,
    pub app_transaction_id: Option<String>,
    pub offer: Option<TransactionOffer>,
    pub json_representation: Vec<u8>,
}

#[derive(Debug)]
pub struct Transaction {
    handle: Option<NonNull<c_void>>,
    data: TransactionData,
}

impl Clone for Transaction {
    fn clone(&self) -> Self {
        let handle = self.handle.map(|handle| {
            let retained = unsafe { ffi::sk_transaction_retain(handle.as_ptr()) };
            NonNull::new(retained).expect("StoreKit transaction retain returned null")
        });
        Self {
            handle,
            data: self.data.clone(),
        }
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Some(handle) = self.handle {
            unsafe { ffi::sk_transaction_release(handle.as_ptr()) };
        }
    }
}

impl Transaction {
    pub fn current_entitlements() -> Result<TransactionStream, StoreKitError> {
        TransactionStream::new(&TransactionStreamConfig::current_entitlements())
    }

    pub fn all() -> Result<TransactionStream, StoreKitError> {
        TransactionStream::new(&TransactionStreamConfig::all())
    }

    pub fn updates() -> Result<TransactionStream, StoreKitError> {
        TransactionStream::new(&TransactionStreamConfig::updates())
    }

    pub fn unfinished() -> Result<TransactionStream, StoreKitError> {
        TransactionStream::new(&TransactionStreamConfig::unfinished())
    }

    pub fn all_for(product_id: &str) -> Result<TransactionStream, StoreKitError> {
        TransactionStream::new(&TransactionStreamConfig::all_for(product_id))
    }

    pub fn current_entitlements_for(product_id: &str) -> Result<TransactionStream, StoreKitError> {
        TransactionStream::new(&TransactionStreamConfig::current_entitlements_for(
            product_id,
        ))
    }

    pub fn latest_for(product_id: &str) -> Result<Option<VerificationResult<Self>>, StoreKitError> {
        let product_id = cstring_from_str(product_id, "product id")?;
        let mut transaction_handle = ptr::null_mut();
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_transaction_latest_for(
                product_id.as_ptr(),
                &mut transaction_handle,
                &mut result_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }

        let payload = unsafe {
            parse_optional_json_ptr::<VerificationResultPayload<TransactionPayload>>(
                result_json,
                "latest transaction",
            )
        }?;
        payload
            .map(|payload| {
                payload.into_result(|payload| Self::from_raw_parts(transaction_handle, payload))
            })
            .transpose()
    }

    pub fn current_entitlement_for(
        product_id: &str,
    ) -> Result<Option<VerificationResult<Self>>, StoreKitError> {
        let product_id = cstring_from_str(product_id, "product id")?;
        let mut transaction_handle = ptr::null_mut();
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_transaction_current_entitlement_for(
                product_id.as_ptr(),
                &mut transaction_handle,
                &mut result_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }

        let payload = unsafe {
            parse_optional_json_ptr::<VerificationResultPayload<TransactionPayload>>(
                result_json,
                "current entitlement transaction",
            )
        }?;
        payload
            .map(|payload| {
                payload.into_result(|payload| Self::from_raw_parts(transaction_handle, payload))
            })
            .transpose()
    }

    pub const fn data(&self) -> &TransactionData {
        &self.data
    }

    pub const fn has_live_handle(&self) -> bool {
        self.handle.is_some()
    }

    pub fn verify(&self) -> Result<(), StoreKitError> {
        self.handle.map_or_else(
            || {
                self.data
                    .verification_failure
                    .clone()
                    .map_or(Ok(()), |failure| Err(StoreKitError::Verification(failure)))
            },
            |handle| {
                let mut error_message = ptr::null_mut();
                let status =
                    unsafe { ffi::sk_transaction_verify(handle.as_ptr(), &mut error_message) };
                if status == ffi::status::OK {
                    Ok(())
                } else {
                    Err(unsafe { error_from_status(status, error_message) })
                }
            },
        )
    }

    pub fn finish(&self) -> Result<(), StoreKitError> {
        self.handle.map_or_else(
            || {
                Err(StoreKitError::NotSupported(
                    "transaction snapshots cannot be finished because they do not carry a live StoreKit handle"
                        .to_owned(),
                ))
            },
            |handle| {
                let mut error_message = ptr::null_mut();
                let status = unsafe { ffi::sk_transaction_finish(handle.as_ptr(), &mut error_message) };
                if status == ffi::status::OK {
                    Ok(())
                } else {
                    Err(unsafe { error_from_status(status, error_message) })
                }
            },
        )
    }

    pub fn begin_refund_request(&self) -> Result<RefundRequestStatus, StoreKitError> {
        Refund::begin_for_transaction_id(self.data.id)
    }

    pub(crate) fn from_raw_parts(
        handle: *mut c_void,
        payload: TransactionPayload,
    ) -> Result<Self, StoreKitError> {
        Ok(Self {
            handle: NonNull::new(handle),
            data: payload.into_transaction_data()?,
        })
    }

    pub(crate) fn from_snapshot_payload(
        payload: TransactionPayload,
    ) -> Result<Self, StoreKitError> {
        Ok(Self {
            handle: None,
            data: payload.into_transaction_data()?,
        })
    }
}

#[derive(Debug)]
pub struct TransactionStream {
    handle: NonNull<c_void>,
    finished: bool,
}

impl Drop for TransactionStream {
    fn drop(&mut self) {
        unsafe { ffi::sk_transaction_stream_release(self.handle.as_ptr()) };
    }
}

impl TransactionStream {
    fn new(config: &TransactionStreamConfig) -> Result<Self, StoreKitError> {
        let config_json = json_cstring(config, "transaction stream config")?;
        let mut error_message = ptr::null_mut();
        let handle =
            unsafe { ffi::sk_transaction_stream_create(config_json.as_ptr(), &mut error_message) };
        let handle = NonNull::new(handle)
            .ok_or_else(|| unsafe { error_from_status(ffi::status::UNKNOWN, error_message) })?;
        Ok(Self {
            handle,
            finished: false,
        })
    }

    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<VerificationResult<Transaction>>, StoreKitError> {
        self.next_timeout(Duration::from_secs(30))
    }

    pub fn next_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<VerificationResult<Transaction>>, StoreKitError> {
        let mut transaction_handle = ptr::null_mut();
        let mut verification_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_transaction_stream_next(
                self.handle.as_ptr(),
                duration_to_timeout_ms(timeout),
                &mut transaction_handle,
                &mut verification_json,
                &mut error_message,
            )
        };

        match status {
            ffi::status::OK => {
                let payload = unsafe {
                    parse_json_ptr::<VerificationResultPayload<TransactionPayload>>(
                        verification_json,
                        "transaction verification result",
                    )
                };
                match payload {
                    Ok(payload) => payload
                        .into_result(|payload| {
                            Transaction::from_raw_parts(transaction_handle, payload)
                        })
                        .map(Some),
                    Err(error) => {
                        if !transaction_handle.is_null() {
                            unsafe { ffi::sk_transaction_release(transaction_handle) };
                        }
                        Err(error)
                    }
                }
            }
            ffi::status::END_OF_STREAM => {
                self.finished = true;
                Ok(None)
            }
            ffi::status::TIMED_OUT => Ok(None),
            _ => Err(unsafe { error_from_status(status, error_message) }),
        }
    }
}

#[derive(Debug, Serialize)]
struct TransactionStreamConfig {
    kind: &'static str,
    #[serde(rename = "productID", skip_serializing_if = "Option::is_none")]
    product_id: Option<String>,
}

impl TransactionStreamConfig {
    const fn all() -> Self {
        Self {
            kind: "all",
            product_id: None,
        }
    }

    const fn current_entitlements() -> Self {
        Self {
            kind: "currentEntitlements",
            product_id: None,
        }
    }

    const fn updates() -> Self {
        Self {
            kind: "updates",
            product_id: None,
        }
    }

    const fn unfinished() -> Self {
        Self {
            kind: "unfinished",
            product_id: None,
        }
    }

    fn all_for(product_id: &str) -> Self {
        Self {
            kind: "allFor",
            product_id: Some(product_id.to_owned()),
        }
    }

    fn current_entitlements_for(product_id: &str) -> Self {
        Self {
            kind: "currentEntitlementsFor",
            product_id: Some(product_id.to_owned()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct TransactionOfferPayload {
    id: Option<String>,
    #[serde(rename = "type")]
    offer_type: String,
    #[serde(rename = "paymentMode")]
    payment_mode: Option<String>,
    period: Option<SubscriptionPeriodPayload>,
}

impl TransactionOfferPayload {
    pub(crate) fn into_transaction_offer(self) -> TransactionOffer {
        TransactionOffer {
            id: self.id,
            offer_type: OfferType::from_raw(self.offer_type),
            payment_mode: self.payment_mode.map(OfferPaymentMode::from_raw),
            period: self
                .period
                .map(SubscriptionPeriodPayload::into_subscription_period),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct TransactionPayload {
    id: u64,
    #[serde(rename = "originalID")]
    original_id: u64,
    #[serde(rename = "webOrderLineItemID")]
    web_order_line_item_id: Option<String>,
    #[serde(rename = "productID")]
    product_id: String,
    #[serde(rename = "subscriptionGroupID")]
    subscription_group_id: Option<String>,
    #[serde(rename = "appBundleID")]
    app_bundle_id: String,
    #[serde(rename = "purchaseDate")]
    purchase_date: String,
    #[serde(rename = "originalPurchaseDate")]
    original_purchase_date: String,
    #[serde(rename = "expirationDate")]
    expiration_date: Option<String>,
    #[serde(rename = "purchasedQuantity")]
    purchased_quantity: u64,
    #[serde(rename = "isUpgraded")]
    is_upgraded: bool,
    #[serde(rename = "ownershipType")]
    ownership_type: String,
    #[serde(rename = "signedDate")]
    signed_date: String,
    #[serde(rename = "jwsRepresentation")]
    jws_representation: String,
    #[serde(rename = "verificationError")]
    verification_error: Option<crate::error::VerificationErrorPayload>,
    #[serde(rename = "revocationDate")]
    revocation_date: Option<String>,
    #[serde(rename = "revocationReason")]
    revocation_reason: Option<String>,
    #[serde(rename = "productType")]
    product_type: Option<String>,
    #[serde(rename = "appAccountToken")]
    app_account_token: Option<String>,
    environment: Option<String>,
    reason: Option<String>,
    storefront: Option<StorefrontPayload>,
    price: Option<String>,
    #[serde(rename = "currencyCode")]
    currency_code: Option<String>,
    #[serde(rename = "appTransactionID")]
    app_transaction_id: Option<String>,
    offer: Option<TransactionOfferPayload>,
    #[serde(rename = "jsonRepresentationBase64")]
    json_representation_base64: String,
}

impl TransactionPayload {
    fn into_transaction_data(self) -> Result<TransactionData, StoreKitError> {
        Ok(TransactionData {
            id: self.id,
            original_id: self.original_id,
            web_order_line_item_id: self.web_order_line_item_id,
            product_id: self.product_id,
            subscription_group_id: self.subscription_group_id,
            app_bundle_id: self.app_bundle_id,
            purchase_date: self.purchase_date,
            original_purchase_date: self.original_purchase_date,
            expiration_date: self.expiration_date,
            purchased_quantity: self.purchased_quantity,
            is_upgraded: self.is_upgraded,
            ownership_type: OwnershipType::from_raw(self.ownership_type),
            signed_date: self.signed_date,
            jws_representation: self.jws_representation,
            verification_failure: self
                .verification_error
                .map(crate::error::VerificationFailure::from_payload),
            revocation_date: self.revocation_date,
            revocation_reason: self.revocation_reason.map(RevocationReason::from_raw),
            product_type: self.product_type.map(ProductType::from_raw),
            app_account_token: self.app_account_token,
            environment: self.environment.map(AppStoreEnvironment::from_raw),
            reason: self.reason.map(TransactionReason::from_raw),
            storefront: self.storefront.map(StorefrontPayload::into_storefront),
            price: self.price,
            currency_code: self.currency_code,
            app_transaction_id: self.app_transaction_id,
            offer: self
                .offer
                .map(TransactionOfferPayload::into_transaction_offer),
            json_representation: decode_base64(
                &self.json_representation_base64,
                "transaction JSON representation",
            )?,
        })
    }
}
