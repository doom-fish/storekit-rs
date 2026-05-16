use core::ptr;
use std::ffi::c_void;

use serde::{Deserialize, Serialize};

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{cstring_from_str, error_from_status, json_cstring, parse_json_ptr};
use crate::transaction::{Transaction, TransactionPayload, VerificationResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductType {
    Consumable,
    NonConsumable,
    AutoRenewable,
    NonRenewing,
    Unknown(String),
}

impl ProductType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Consumable => "consumable",
            Self::NonConsumable => "nonConsumable",
            Self::AutoRenewable => "autoRenewable",
            Self::NonRenewing => "nonRenewing",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "consumable" => Self::Consumable,
            "nonConsumable" => Self::NonConsumable,
            "autoRenewable" => Self::AutoRenewable,
            "nonRenewing" => Self::NonRenewing,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionPeriodUnit {
    Day,
    Week,
    Month,
    Year,
    Unknown(String),
}

impl SubscriptionPeriodUnit {
    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "day" => Self::Day,
            "week" => Self::Week,
            "month" => Self::Month,
            "year" => Self::Year,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionPeriod {
    pub unit: SubscriptionPeriodUnit,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionInfo {
    pub subscription_group_id: String,
    pub subscription_period: SubscriptionPeriod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub price: String,
    pub display_price: String,
    pub product_type: ProductType,
    pub subscription: Option<SubscriptionInfo>,
}

impl Product {
    pub fn products_for<I, S>(identifiers: I) -> Result<Vec<Self>, StoreKitError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let identifiers: Vec<String> = identifiers
            .into_iter()
            .map(|identifier| identifier.as_ref().to_owned())
            .collect();
        let identifiers_json = json_cstring(&identifiers, "product identifiers")?;
        let mut products_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_products_json(
                identifiers_json.as_ptr(),
                &mut products_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        let payloads = unsafe { parse_json_ptr::<Vec<ProductPayload>>(products_json, "products") }?;
        Ok(payloads.into_iter().map(ProductPayload::into_product).collect())
    }

    pub fn purchase(&self, options: &[PurchaseOption]) -> Result<PurchaseResult, StoreKitError> {
        let product_id = cstring_from_str(&self.id, "product id")?;
        let options_json = json_cstring(options, "purchase options")?;
        let mut transaction_handle: *mut c_void = ptr::null_mut();
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_product_purchase(
                product_id.as_ptr(),
                options_json.as_ptr(),
                &mut transaction_handle,
                &mut result_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }

        let payload = unsafe { parse_json_ptr::<PurchaseResultPayload>(result_json, "purchase result") };
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum PurchaseResult {
    Success(VerificationResult<Transaction>),
    UserCancelled,
    Pending,
}

#[derive(Debug, Deserialize)]
struct ProductPayload {
    id: String,
    #[serde(rename = "displayName")]
    display_name: String,
    description: String,
    price: String,
    #[serde(rename = "displayPrice")]
    display_price: String,
    #[serde(rename = "type")]
    product_type: String,
    subscription: Option<SubscriptionInfoPayload>,
}

impl ProductPayload {
    fn into_product(self) -> Product {
        Product {
            id: self.id,
            display_name: self.display_name,
            description: self.description,
            price: self.price,
            display_price: self.display_price,
            product_type: ProductType::from_raw(self.product_type),
            subscription: self.subscription.map(SubscriptionInfoPayload::into_subscription_info),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SubscriptionInfoPayload {
    #[serde(rename = "subscriptionGroupID")]
    subscription_group_id: String,
    #[serde(rename = "subscriptionPeriod")]
    subscription_period: SubscriptionPeriodPayload,
}

impl SubscriptionInfoPayload {
    fn into_subscription_info(self) -> SubscriptionInfo {
        SubscriptionInfo {
            subscription_group_id: self.subscription_group_id,
            subscription_period: self.subscription_period.into_subscription_period(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SubscriptionPeriodPayload {
    unit: String,
    value: i64,
}

impl SubscriptionPeriodPayload {
    fn into_subscription_period(self) -> SubscriptionPeriod {
        SubscriptionPeriod {
            unit: SubscriptionPeriodUnit::from_raw(self.unit),
            value: self.value,
        }
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Deserialize)]
struct PurchaseResultPayload {
    kind: String,
    transaction: Option<TransactionPayload>,
}

impl PurchaseResultPayload {
    fn into_purchase_result(
        self,
        transaction_handle: *mut c_void,
    ) -> Result<PurchaseResult, StoreKitError> {
        match self.kind.as_str() {
            "success" => {
                let payload = self.transaction.ok_or_else(|| {
                    StoreKitError::Unknown(
                        "StoreKit reported a successful purchase without a transaction payload"
                            .to_owned(),
                    )
                })?;
                let transaction = Transaction::from_raw_parts(transaction_handle, payload)?;
                Ok(PurchaseResult::Success(transaction.into_verification_result()))
            }
            "userCancelled" => {
                if !transaction_handle.is_null() {
                    unsafe { ffi::sk_transaction_release(transaction_handle) };
                }
                Ok(PurchaseResult::UserCancelled)
            }
            "pending" => {
                if !transaction_handle.is_null() {
                    unsafe { ffi::sk_transaction_release(transaction_handle) };
                }
                Ok(PurchaseResult::Pending)
            }
            other => {
                if !transaction_handle.is_null() {
                    unsafe { ffi::sk_transaction_release(transaction_handle) };
                }
                Err(StoreKitError::Unknown(format!(
                    "StoreKit returned an unknown purchase result kind '{other}'"
                )))
            }
        }
    }
}
