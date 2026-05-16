use core::ptr;
use std::ffi::c_void;

use serde::Deserialize;

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{
    cstring_from_str, decode_base64, error_from_status, json_cstring, parse_json_ptr,
};
pub use crate::purchase_option::{PurchaseOption, PurchaseResult};
pub use crate::subscription::{
    SubscriptionOffer, SubscriptionOfferType, SubscriptionPaymentMode, SubscriptionPeriod,
    SubscriptionPeriodUnit,
};
pub use crate::subscription_info::SubscriptionInfo;

use crate::purchase_option::PurchaseResultPayload;
use crate::subscription_info::SubscriptionInfoPayload;
use crate::transaction::{Transaction, TransactionStream};
use crate::verification_result::VerificationResult;

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

    pub(crate) fn from_raw(raw: String) -> Self {
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
pub struct Product {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub price: String,
    pub display_price: String,
    pub product_type: ProductType,
    pub is_family_shareable: bool,
    pub subscription: Option<SubscriptionInfo>,
    pub currency_code: Option<String>,
    pub price_locale_identifier: Option<String>,
    pub json_representation: Vec<u8>,
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
        payloads
            .into_iter()
            .map(ProductPayload::into_product)
            .collect::<Result<Vec<_>, _>>()
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

        let payload =
            unsafe { parse_json_ptr::<PurchaseResultPayload>(result_json, "purchase result") };
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

    pub fn latest_transaction(
        &self,
    ) -> Result<Option<VerificationResult<Transaction>>, StoreKitError> {
        Transaction::latest_for(&self.id)
    }

    pub fn current_entitlements(&self) -> Result<TransactionStream, StoreKitError> {
        Transaction::current_entitlements_for(&self.id)
    }
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
    #[serde(rename = "isFamilyShareable")]
    is_family_shareable: bool,
    subscription: Option<SubscriptionInfoPayload>,
    #[serde(rename = "currencyCode")]
    currency_code: Option<String>,
    #[serde(rename = "priceLocaleIdentifier")]
    price_locale_identifier: Option<String>,
    #[serde(rename = "jsonRepresentationBase64")]
    json_representation_base64: String,
}

impl ProductPayload {
    fn into_product(self) -> Result<Product, StoreKitError> {
        Ok(Product {
            id: self.id,
            display_name: self.display_name,
            description: self.description,
            price: self.price,
            display_price: self.display_price,
            product_type: ProductType::from_raw(self.product_type),
            is_family_shareable: self.is_family_shareable,
            subscription: self
                .subscription
                .map(SubscriptionInfoPayload::into_subscription_info),
            currency_code: self.currency_code,
            price_locale_identifier: self.price_locale_identifier,
            json_representation: decode_base64(
                &self.json_representation_base64,
                "product JSON representation",
            )?,
        })
    }
}
