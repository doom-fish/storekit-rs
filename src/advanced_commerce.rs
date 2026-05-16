use core::ffi::c_void;
use core::ptr;

use serde::{Deserialize, Serialize};

use crate::app_store::AppStore;
use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{cstring_from_str, error_from_status, json_cstring, parse_json_ptr};
use crate::product::ProductType;
use crate::purchase_option::{PurchaseResult, PurchaseResultPayload};
use crate::renewal_info::RenewalInfo;
use crate::subscription::{SubscriptionPeriod, SubscriptionPeriodPayload};
use crate::transaction::{Transaction, TransactionStream};
use crate::verification_result::VerificationResult;
use crate::window::NSWindowHandle;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AppStoreMerchandisingKind {
    SubscriptionBundle {
        #[serde(rename = "groupID")]
        group_id: String,
    },
}

impl AppStoreMerchandisingKind {
    pub fn subscription_bundle(group_id: impl Into<String>) -> Self {
        Self::SubscriptionBundle {
            group_id: group_id.into(),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum AppStoreMerchandisingPresentationResult {
    Dismissed,
    PurchaseCompleted(PurchaseResult),
}

impl AppStore {
    pub fn age_rating_code() -> Result<Option<i64>, StoreKitError> {
        let mut raw_value = 0_i64;
        let mut has_value = 0;
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_app_store_age_rating_code(
                &mut raw_value,
                &mut has_value,
                &mut error_message,
            )
        };
        if status == ffi::status::OK {
            Ok((has_value != 0).then_some(raw_value))
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn present_merchandising(
        kind: &AppStoreMerchandisingKind,
        window: &NSWindowHandle,
    ) -> Result<AppStoreMerchandisingPresentationResult, StoreKitError> {
        let kind_json = json_cstring(kind, "App Store merchandising kind")?;
        let mut transaction_handle: *mut c_void = ptr::null_mut();
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_app_store_present_merchandising(
                kind_json.as_ptr(),
                window.as_raw(),
                &mut transaction_handle,
                &mut result_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        let payload = unsafe {
            parse_json_ptr::<AppStoreMerchandisingPresentationResultPayload>(
                result_json,
                "App Store merchandising presentation result",
            )
        }?;
        payload.into_result(transaction_handle)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AdvancedCommercePurchaseOption {
    OnStorefrontChange {
        #[serde(rename = "shouldContinuePurchase")]
        should_continue_purchase: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancedCommerceProduct {
    pub id: String,
    pub product_type: ProductType,
}

impl AdvancedCommerceProduct {
    pub fn new(id: &str) -> Result<Self, StoreKitError> {
        let product_id = cstring_from_str(id, "advanced commerce product id")?;
        let mut product_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_advanced_commerce_product_json(
                product_id.as_ptr(),
                &mut product_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        let payload = unsafe {
            parse_json_ptr::<AdvancedCommerceProductPayload>(
                product_json,
                "advanced commerce product",
            )
        }?;
        Ok(payload.into_product())
    }

    pub fn purchase_in_window(
        &self,
        compact_jws: &str,
        window: &NSWindowHandle,
        options: &[AdvancedCommercePurchaseOption],
    ) -> Result<PurchaseResult, StoreKitError> {
        let product_id = cstring_from_str(&self.id, "advanced commerce product id")?;
        let compact_jws = cstring_from_str(compact_jws, "advanced commerce compact JWS")?;
        let options_json = json_cstring(options, "advanced commerce purchase options")?;
        let mut transaction_handle: *mut c_void = ptr::null_mut();
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_advanced_commerce_product_purchase(
                product_id.as_ptr(),
                compact_jws.as_ptr(),
                window.as_raw(),
                options_json.as_ptr(),
                &mut transaction_handle,
                &mut result_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }

        let payload = unsafe {
            parse_json_ptr::<PurchaseResultPayload>(result_json, "advanced commerce purchase result")
        };
        match payload {
            Ok(payload) => payload.into_purchase_result(transaction_handle),
            Err(error) => {
                if !transaction_handle.is_null() {
                    unsafe { ffi::sk_transaction_release(transaction_handle) };
                }
                Err(error)
            }
        }
    }

    pub fn latest_transaction(&self) -> Result<Option<VerificationResult<Transaction>>, StoreKitError> {
        Transaction::latest_for(&self.id)
    }

    pub fn all_transactions(&self) -> Result<TransactionStream, StoreKitError> {
        Transaction::all_for(&self.id)
    }

    pub fn current_entitlements(&self) -> Result<TransactionStream, StoreKitError> {
        Transaction::current_entitlements_for(&self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAdvancedCommerceInfo {
    pub request_reference_id: String,
    pub estimated_tax: String,
    pub tax_rate: String,
    pub tax_code: String,
    pub tax_exclusive_price: String,
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub period: Option<SubscriptionPeriod>,
    pub items: Vec<TransactionAdvancedCommerceItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAdvancedCommerceItem {
    pub details: TransactionAdvancedCommerceItemDetails,
    pub refunds: Option<Vec<TransactionAdvancedCommerceRefund>>,
    pub revocation_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAdvancedCommerceItemDetails {
    pub sku: String,
    pub display_name: String,
    pub description: String,
    pub offer: Option<TransactionAdvancedCommerceOffer>,
    pub price: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAdvancedCommerceOffer {
    pub price: String,
    pub period: SubscriptionPeriod,
    pub period_count: i64,
    pub reason: TransactionAdvancedCommerceOfferReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionAdvancedCommerceOfferReason {
    Acquisition,
    Retention,
    WinBack,
    Unknown(String),
}

impl TransactionAdvancedCommerceOfferReason {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Acquisition => "acquisition",
            Self::Retention => "retention",
            Self::WinBack => "winBack",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "acquisition" => Self::Acquisition,
            "retention" => Self::Retention,
            "winBack" => Self::WinBack,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAdvancedCommerceRefund {
    pub reason: TransactionAdvancedCommerceRefundReason,
    pub refund_type: TransactionAdvancedCommerceRefundType,
    pub date: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionAdvancedCommerceRefundReason {
    Legal,
    ModifyItems,
    Unintended,
    Unfulfilled,
    Unsatisfied,
    Other,
    Unknown(String),
}

impl TransactionAdvancedCommerceRefundReason {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Legal => "legal",
            Self::ModifyItems => "modifyItems",
            Self::Unintended => "unintended",
            Self::Unfulfilled => "unfulfilled",
            Self::Unsatisfied => "unsatisfied",
            Self::Other => "other",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "legal" => Self::Legal,
            "modifyItems" => Self::ModifyItems,
            "unintended" => Self::Unintended,
            "unfulfilled" => Self::Unfulfilled,
            "unsatisfied" => Self::Unsatisfied,
            "other" => Self::Other,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionAdvancedCommerceRefundType {
    Custom,
    ProRated,
    Full,
    Unknown(String),
}

impl TransactionAdvancedCommerceRefundType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Custom => "custom",
            Self::ProRated => "proRated",
            Self::Full => "full",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "custom" => Self::Custom,
            "proRated" => Self::ProRated,
            "full" => Self::Full,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewalInfoAdvancedCommerceInfo {
    pub consistency_token: String,
    pub request_reference_id: String,
    pub tax_code: String,
    pub description: String,
    pub display_name: String,
    pub period: SubscriptionPeriod,
    pub items: Vec<RenewalInfoAdvancedCommerceItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewalInfoAdvancedCommerceItem {
    pub details: TransactionAdvancedCommerceItemDetails,
    pub price_increase_info: Option<RenewalInfoAdvancedCommercePriceIncreaseInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewalInfoAdvancedCommercePriceIncreaseInfo {
    pub status: RenewalInfoAdvancedCommercePriceIncreaseStatus,
    pub price: String,
    pub dependent_skus: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenewalInfoAdvancedCommercePriceIncreaseStatus {
    Pending,
    Accepted,
    Scheduled,
    Unknown(String),
}

impl RenewalInfoAdvancedCommercePriceIncreaseStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Scheduled => "scheduled",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "pending" => Self::Pending,
            "accepted" => Self::Accepted,
            "scheduled" => Self::Scheduled,
            _ => Self::Unknown(raw),
        }
    }
}

impl VerificationResult<Transaction> {
    pub fn advanced_commerce_info(&self) -> Result<Option<TransactionAdvancedCommerceInfo>, StoreKitError> {
        parse_transaction_advanced_commerce_info_payload(&self.metadata().payload_data)
    }
}

impl VerificationResult<RenewalInfo> {
    pub fn advanced_commerce_info(&self) -> Result<Option<RenewalInfoAdvancedCommerceInfo>, StoreKitError> {
        parse_renewal_advanced_commerce_info_payload(&self.metadata().payload_data)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct TransactionAdvancedCommerceInfoPayload {
    #[serde(rename = "requestReferenceID")]
    request_reference_id: String,
    #[serde(rename = "estimatedTax")]
    estimated_tax: String,
    #[serde(rename = "taxRate")]
    tax_rate: String,
    #[serde(rename = "taxCode")]
    tax_code: String,
    #[serde(rename = "taxExclusivePrice")]
    tax_exclusive_price: String,
    description: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    period: Option<SubscriptionPeriodPayload>,
    items: Vec<TransactionAdvancedCommerceItemPayload>,
}

impl TransactionAdvancedCommerceInfoPayload {
    pub(crate) fn into_transaction_advanced_commerce_info(self) -> TransactionAdvancedCommerceInfo {
        TransactionAdvancedCommerceInfo {
            request_reference_id: self.request_reference_id,
            estimated_tax: self.estimated_tax,
            tax_rate: self.tax_rate,
            tax_code: self.tax_code,
            tax_exclusive_price: self.tax_exclusive_price,
            description: self.description,
            display_name: self.display_name,
            period: self.period.map(SubscriptionPeriodPayload::into_subscription_period),
            items: self
                .items
                .into_iter()
                .map(TransactionAdvancedCommerceItemPayload::into_transaction_advanced_commerce_item)
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TransactionAdvancedCommerceItemPayload {
    details: TransactionAdvancedCommerceItemDetailsPayload,
    refunds: Option<Vec<TransactionAdvancedCommerceRefundPayload>>,
    #[serde(rename = "revocationDate")]
    revocation_date: Option<String>,
}

impl TransactionAdvancedCommerceItemPayload {
    fn into_transaction_advanced_commerce_item(self) -> TransactionAdvancedCommerceItem {
        TransactionAdvancedCommerceItem {
            details: self.details.into_transaction_advanced_commerce_item_details(),
            refunds: self.refunds.map(|refunds| {
                refunds
                    .into_iter()
                    .map(TransactionAdvancedCommerceRefundPayload::into_transaction_advanced_commerce_refund)
                    .collect()
            }),
            revocation_date: self.revocation_date,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TransactionAdvancedCommerceItemDetailsPayload {
    sku: String,
    #[serde(rename = "displayName")]
    display_name: String,
    description: String,
    offer: Option<TransactionAdvancedCommerceOfferPayload>,
    price: String,
}

impl TransactionAdvancedCommerceItemDetailsPayload {
    fn into_transaction_advanced_commerce_item_details(self) -> TransactionAdvancedCommerceItemDetails {
        TransactionAdvancedCommerceItemDetails {
            sku: self.sku,
            display_name: self.display_name,
            description: self.description,
            offer: self.offer.map(TransactionAdvancedCommerceOfferPayload::into_transaction_advanced_commerce_offer),
            price: self.price,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TransactionAdvancedCommerceOfferPayload {
    price: String,
    period: SubscriptionPeriodPayload,
    #[serde(rename = "periodCount")]
    period_count: i64,
    reason: String,
}

impl TransactionAdvancedCommerceOfferPayload {
    fn into_transaction_advanced_commerce_offer(self) -> TransactionAdvancedCommerceOffer {
        TransactionAdvancedCommerceOffer {
            price: self.price,
            period: self.period.into_subscription_period(),
            period_count: self.period_count,
            reason: TransactionAdvancedCommerceOfferReason::from_raw(self.reason),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TransactionAdvancedCommerceRefundPayload {
    reason: String,
    #[serde(rename = "type")]
    refund_type: String,
    date: String,
    amount: String,
}

impl TransactionAdvancedCommerceRefundPayload {
    fn into_transaction_advanced_commerce_refund(self) -> TransactionAdvancedCommerceRefund {
        TransactionAdvancedCommerceRefund {
            reason: TransactionAdvancedCommerceRefundReason::from_raw(self.reason),
            refund_type: TransactionAdvancedCommerceRefundType::from_raw(self.refund_type),
            date: self.date,
            amount: self.amount,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RenewalSignedPayload {
    #[serde(rename = "advancedCommerceInfo")]
    advanced_commerce_info: Option<RenewalInfoAdvancedCommerceInfoPayload>,
}

#[derive(Debug, Deserialize)]
struct RenewalInfoAdvancedCommerceInfoPayload {
    #[serde(rename = "consistencyToken")]
    consistency_token: String,
    #[serde(rename = "requestReferenceID")]
    request_reference_id: String,
    #[serde(rename = "taxCode")]
    tax_code: String,
    description: String,
    #[serde(rename = "displayName")]
    display_name: String,
    period: SubscriptionPeriodPayload,
    items: Vec<RenewalInfoAdvancedCommerceItemPayload>,
}

impl RenewalInfoAdvancedCommerceInfoPayload {
    fn into_renewal_info_advanced_commerce_info(self) -> RenewalInfoAdvancedCommerceInfo {
        RenewalInfoAdvancedCommerceInfo {
            consistency_token: self.consistency_token,
            request_reference_id: self.request_reference_id,
            tax_code: self.tax_code,
            description: self.description,
            display_name: self.display_name,
            period: self.period.into_subscription_period(),
            items: self
                .items
                .into_iter()
                .map(RenewalInfoAdvancedCommerceItemPayload::into_renewal_info_advanced_commerce_item)
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RenewalInfoAdvancedCommerceItemPayload {
    details: TransactionAdvancedCommerceItemDetailsPayload,
    #[serde(rename = "priceIncreaseInfo")]
    price_increase_info: Option<RenewalInfoAdvancedCommercePriceIncreaseInfoPayload>,
}

impl RenewalInfoAdvancedCommerceItemPayload {
    fn into_renewal_info_advanced_commerce_item(self) -> RenewalInfoAdvancedCommerceItem {
        RenewalInfoAdvancedCommerceItem {
            details: self.details.into_transaction_advanced_commerce_item_details(),
            price_increase_info: self.price_increase_info.map(
                RenewalInfoAdvancedCommercePriceIncreaseInfoPayload::into_renewal_info_advanced_commerce_price_increase_info,
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RenewalInfoAdvancedCommercePriceIncreaseInfoPayload {
    status: String,
    price: String,
    #[serde(rename = "dependentSKUs")]
    dependent_skus: Vec<String>,
}

impl RenewalInfoAdvancedCommercePriceIncreaseInfoPayload {
    fn into_renewal_info_advanced_commerce_price_increase_info(self) -> RenewalInfoAdvancedCommercePriceIncreaseInfo {
        RenewalInfoAdvancedCommercePriceIncreaseInfo {
            status: RenewalInfoAdvancedCommercePriceIncreaseStatus::from_raw(self.status),
            price: self.price,
            dependent_skus: self.dependent_skus,
        }
    }
}

#[derive(Debug, Deserialize)]
struct AdvancedCommerceProductPayload {
    id: String,
    #[serde(rename = "type")]
    product_type: String,
}

impl AdvancedCommerceProductPayload {
    fn into_product(self) -> AdvancedCommerceProduct {
        AdvancedCommerceProduct {
            id: self.id,
            product_type: ProductType::from_raw(self.product_type),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(clippy::unsafe_derive_deserialize)]
pub(crate) struct AppStoreMerchandisingPresentationResultPayload {
    kind: String,
    #[serde(rename = "purchaseResult")]
    purchase_result: Option<PurchaseResultPayload>,
}

impl AppStoreMerchandisingPresentationResultPayload {
    pub(crate) fn into_result(
        self,
        transaction_handle: *mut c_void,
    ) -> Result<AppStoreMerchandisingPresentationResult, StoreKitError> {
        match self.kind.as_str() {
            "dismissed" => {
                if !transaction_handle.is_null() {
                    unsafe { ffi::sk_transaction_release(transaction_handle) };
                }
                Ok(AppStoreMerchandisingPresentationResult::Dismissed)
            }
            "purchaseCompleted" => {
                let purchase_result = self.purchase_result.ok_or_else(|| {
                    StoreKitError::Unknown(
                        "App Store merchandising reported a purchase completion without a purchase result"
                            .to_owned(),
                    )
                })?;
                Ok(AppStoreMerchandisingPresentationResult::PurchaseCompleted(
                    purchase_result.into_purchase_result(transaction_handle)?,
                ))
            }
            other => {
                if !transaction_handle.is_null() {
                    unsafe { ffi::sk_transaction_release(transaction_handle) };
                }
                Err(StoreKitError::Unknown(format!(
                    "unknown App Store merchandising presentation result kind '{other}'"
                )))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct TransactionSignedPayload {
    #[serde(rename = "advancedCommerceInfo")]
    advanced_commerce_info: Option<TransactionAdvancedCommerceInfoPayload>,
}

fn parse_transaction_advanced_commerce_info_payload(
    payload_data: &[u8],
) -> Result<Option<TransactionAdvancedCommerceInfo>, StoreKitError> {
    let payload = serde_json::from_slice::<TransactionSignedPayload>(payload_data).map_err(|error| {
        StoreKitError::InvalidArgument(format!(
            "failed to parse signed transaction payload JSON: {error}"
        ))
    })?;
    Ok(payload
        .advanced_commerce_info
        .map(TransactionAdvancedCommerceInfoPayload::into_transaction_advanced_commerce_info))
}

fn parse_renewal_advanced_commerce_info_payload(
    payload_data: &[u8],
) -> Result<Option<RenewalInfoAdvancedCommerceInfo>, StoreKitError> {
    let payload = serde_json::from_slice::<RenewalSignedPayload>(payload_data).map_err(|error| {
        StoreKitError::InvalidArgument(format!(
            "failed to parse signed renewal payload JSON: {error}"
        ))
    })?;
    Ok(payload
        .advanced_commerce_info
        .map(RenewalInfoAdvancedCommerceInfoPayload::into_renewal_info_advanced_commerce_info))
}
