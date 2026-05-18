use core::ffi::c_void;
use core::ptr;

use serde::Deserialize;

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{cstring_from_str, error_from_status, parse_json_ptr, take_string};
use crate::product::{Product, ProductType, PurchaseOption, PurchaseResult};
use crate::purchase_option::PurchaseResultPayload;
use crate::renewal_info::{ExpirationReason, PriceIncreaseStatus};
use crate::renewal_state::RenewalState;
use crate::subscription::{
    SubscriptionOfferType, SubscriptionPaymentMode, SubscriptionPeriod, SubscriptionPeriodUnit,
};
use crate::transaction::{OfferType, OwnershipType, RevocationReason};
use crate::window::NSWindowHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Carries localized formatting derived from `StoreKit.Product`.
pub struct ProductFormatting {
    /// Formatted price string returned by `StoreKit`.
    pub formatted_price: String,
    /// Formatted subscription period string returned by `StoreKit`.
    pub formatted_subscription_period: Option<String>,
    /// Formatted subscription period unit string returned by `StoreKit`.
    pub formatted_subscription_period_unit: Option<String>,
}

impl Product {
    /// Calls the `StoreKit` purchase API while presenting UI in the supplied `NSWindow`.
    pub fn purchase_in_window(
        &self,
        window: &NSWindowHandle,
        options: &[PurchaseOption],
    ) -> Result<PurchaseResult, StoreKitError> {
        let product_id = cstring_from_str(&self.id, "product id")?;
        let options_json = crate::private::json_cstring(options, "purchase options")?;
        let mut transaction_handle: *mut c_void = ptr::null_mut();
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_product_purchase_in_window(
                product_id.as_ptr(),
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

    /// Fetches `StoreKit`-provided formatting for this product.
    pub fn formatting(&self) -> Result<ProductFormatting, StoreKitError> {
        let product_id = cstring_from_str(&self.id, "product id")?;
        let mut formatting_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_product_formatting_json(
                product_id.as_ptr(),
                &mut formatting_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        let payload = unsafe {
            parse_json_ptr::<ProductFormattingPayload>(formatting_json, "product formatting")
        }?;
        Ok(payload.into_product_formatting())
    }

    /// Returns the `StoreKit`-formatted price for this product.
    pub fn formatted_price(&self) -> Result<String, StoreKitError> {
        self.formatting()
            .map(|formatting| formatting.formatted_price)
    }

    /// Returns the `StoreKit`-formatted subscription period for this product, if available.
    pub fn formatted_subscription_period(&self) -> Result<Option<String>, StoreKitError> {
        self.formatting()
            .map(|formatting| formatting.formatted_subscription_period)
    }

    /// Returns the `StoreKit`-formatted subscription period unit for this product, if available.
    pub fn formatted_subscription_period_unit(&self) -> Result<Option<String>, StoreKitError> {
        self.formatting()
            .map(|formatting| formatting.formatted_subscription_period_unit)
    }
}

impl SubscriptionPeriod {
    /// Builds a `SubscriptionPeriod` matching a weekly `StoreKit` period.
    pub const fn weekly() -> Self {
        Self {
            unit: SubscriptionPeriodUnit::Week,
            value: 1,
        }
    }

    /// Builds a `SubscriptionPeriod` matching a monthly `StoreKit` period.
    pub const fn monthly() -> Self {
        Self {
            unit: SubscriptionPeriodUnit::Month,
            value: 1,
        }
    }

    /// Builds a `SubscriptionPeriod` matching a yearly `StoreKit` period.
    pub const fn yearly() -> Self {
        Self {
            unit: SubscriptionPeriodUnit::Year,
            value: 1,
        }
    }

    /// Builds a `SubscriptionPeriod` matching a three-day `StoreKit` period.
    pub const fn every_three_days() -> Self {
        Self {
            unit: SubscriptionPeriodUnit::Day,
            value: 3,
        }
    }

    /// Builds a `SubscriptionPeriod` matching a two-week `StoreKit` period.
    pub const fn every_two_weeks() -> Self {
        Self {
            unit: SubscriptionPeriodUnit::Week,
            value: 2,
        }
    }

    /// Builds a `SubscriptionPeriod` matching a two-month `StoreKit` period.
    pub const fn every_two_months() -> Self {
        Self {
            unit: SubscriptionPeriodUnit::Month,
            value: 2,
        }
    }

    /// Builds a `SubscriptionPeriod` matching a three-month `StoreKit` period.
    pub const fn every_three_months() -> Self {
        Self {
            unit: SubscriptionPeriodUnit::Month,
            value: 3,
        }
    }

    /// Builds a `SubscriptionPeriod` matching a six-month `StoreKit` period.
    pub const fn every_six_months() -> Self {
        Self {
            unit: SubscriptionPeriodUnit::Month,
            value: 6,
        }
    }
}

impl ProductType {
    /// Returns the localized `StoreKit` description for this product type.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        localized_description("productType", self.as_str())
    }
}

impl RenewalState {
    /// Returns the localized `StoreKit` description for this renewal state.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        localized_description("renewalState", self.as_str())
    }
}

impl ExpirationReason {
    /// Returns the localized `StoreKit` description for this expiration reason.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        localized_description("expirationReason", self.as_str())
    }
}

impl PriceIncreaseStatus {
    /// Returns the localized `StoreKit` description for this price increase status.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        localized_description("priceIncreaseStatus", self.as_str())
    }
}

impl SubscriptionOfferType {
    /// Returns the localized `StoreKit` description for this subscription offer type.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        localized_description("subscriptionOfferType", self.as_str())
    }
}

impl OfferType {
    /// Returns the localized `StoreKit` description for this transaction offer type.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        localized_description("transactionOfferType", self.as_str())
    }
}

impl SubscriptionPaymentMode {
    /// Returns the localized `StoreKit` description for this subscription payment mode.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        localized_description("subscriptionPaymentMode", self.as_str())
    }
}

impl SubscriptionPeriodUnit {
    /// Returns the localized `StoreKit` description for this subscription period unit.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        localized_description("subscriptionPeriodUnit", self.as_str())
    }
}

impl RevocationReason {
    /// Returns the localized `StoreKit` description for this revocation reason.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        localized_description("revocationReason", self.as_str())
    }
}

impl OwnershipType {
    /// Returns the localized `StoreKit` description for this ownership type.
    pub fn localized_description(&self) -> Result<String, StoreKitError> {
        let raw = match self {
            Self::Purchased => "purchased",
            Self::FamilyShared => "familyShared",
            Self::Unknown(value) => value.as_str(),
        };
        localized_description("ownershipType", raw)
    }
}

fn localized_description(kind: &str, raw_value: &str) -> Result<String, StoreKitError> {
    let kind = cstring_from_str(kind, "localized description kind")?;
    let raw_value = cstring_from_str(raw_value, "localized description raw value")?;
    let mut localized = ptr::null_mut();
    let mut error_message = ptr::null_mut();
    let status = unsafe {
        ffi::sk_localized_description(
            kind.as_ptr(),
            raw_value.as_ptr(),
            &mut localized,
            &mut error_message,
        )
    };
    if status != ffi::status::OK {
        return Err(unsafe { error_from_status(status, error_message) });
    }
    unsafe { take_string(localized) }
        .ok_or_else(|| StoreKitError::Unknown("missing localized description payload".to_owned()))
}

#[derive(Debug, Deserialize)]
struct ProductFormattingPayload {
    #[serde(rename = "formattedPrice")]
    price: String,
    #[serde(rename = "formattedSubscriptionPeriod")]
    subscription_period: Option<String>,
    #[serde(rename = "formattedSubscriptionPeriodUnit")]
    subscription_period_unit: Option<String>,
}

impl ProductFormattingPayload {
    fn into_product_formatting(self) -> ProductFormatting {
        ProductFormatting {
            formatted_price: self.price,
            formatted_subscription_period: self.subscription_period,
            formatted_subscription_period_unit: self.subscription_period_unit,
        }
    }
}
