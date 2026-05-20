use core::ptr;

use serde::{Deserialize, Serialize, Serializer};

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{
    cstring_from_str, error_from_status, parse_json_ptr, parse_optional_json_ptr,
};
use crate::renewal_info::{RenewalInfo, RenewalInfoPayload};
use crate::renewal_state::RenewalState;
use crate::subscription::{
    SubscriptionOffer, SubscriptionOfferPayload, SubscriptionPeriod, SubscriptionPeriodPayload,
};
use crate::transaction::{Transaction, TransactionPayload};
use crate::verification_result::{VerificationResult, VerificationResultPayload};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Product.SubscriptionInfo.BillingPlanType`.
pub enum BillingPlanType {
    /// Represents the `Monthly` `StoreKit` case.
    Monthly,
    /// Represents the `UpFront` `StoreKit` case.
    UpFront,
    /// Preserves an unrecognized `StoreKit` case.
    Unknown(String),
}

impl BillingPlanType {
    /// Returns the raw `StoreKit` string for this billing plan type.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Monthly => "monthly",
            Self::UpFront => "upFront",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub(crate) fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "monthly" => Self::Monthly,
            "upFront" => Self::UpFront,
            _ => Self::Unknown(raw),
        }
    }
}

impl Serialize for BillingPlanType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Product.SubscriptionInfo.CommitmentInfo`.
pub struct SubscriptionCommitmentInfo {
    /// Price reported by `StoreKit`.
    pub price: String,
    /// Localized display price reported by `StoreKit`.
    pub display_price: String,
    /// Billing period reported by `StoreKit`.
    pub period: SubscriptionPeriod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Product.SubscriptionInfo.PricingTerms`.
pub struct SubscriptionPricingTerms {
    /// Billing price reported by `StoreKit`.
    pub billing_price: String,
    /// Localized billing display price reported by `StoreKit`.
    pub billing_display_price: String,
    /// Billing period reported by `StoreKit`.
    pub billing_period: SubscriptionPeriod,
    /// Billing plan type reported by `StoreKit`.
    pub billing_plan_type: BillingPlanType,
    /// Commitment info reported by `StoreKit`.
    pub commitment_info: SubscriptionCommitmentInfo,
    /// Subscription offers reported by `StoreKit`.
    pub subscription_offers: Vec<SubscriptionOffer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Product.SubscriptionInfo`.
pub struct SubscriptionInfo {
    /// Introductory offer reported by `StoreKit`.
    pub introductory_offer: Option<SubscriptionOffer>,
    /// Promotional offers reported by `StoreKit`.
    pub promotional_offers: Vec<SubscriptionOffer>,
    /// Win-back offers reported by `StoreKit`.
    pub win_back_offers: Vec<SubscriptionOffer>,
    /// Subscription group identifier reported by `StoreKit`.
    pub subscription_group_id: String,
    /// Subscription period reported by `StoreKit`.
    pub subscription_period: SubscriptionPeriod,
    /// Pricing terms reported by `StoreKit`.
    pub pricing_terms: Vec<SubscriptionPricingTerms>,
    /// Subscription group level reported by `StoreKit`.
    pub group_level: Option<i64>,
    /// Subscription group display name reported by `StoreKit`.
    pub group_display_name: Option<String>,
}

impl SubscriptionInfo {
    /// Returns whether `StoreKit` reports that this subscription group is eligible for an introductory offer.
    pub fn is_eligible_for_intro_offer(&self) -> Result<bool, StoreKitError> {
        Self::is_eligible_for_intro_offer_for(&self.subscription_group_id)
    }

    /// Returns whether `StoreKit` reports that the supplied subscription group is eligible for an introductory offer.
    pub fn is_eligible_for_intro_offer_for(group_id: &str) -> Result<bool, StoreKitError> {
        let group_id = cstring_from_str(group_id, "subscription group id")?;
        let mut raw_value = 0;
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_subscription_info_is_eligible_for_intro_offer(
                group_id.as_ptr(),
                &mut raw_value,
                &mut error_message,
            )
        };
        if status == ffi::status::OK {
            Ok(raw_value != 0)
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    /// Fetches the `StoreKit` subscription statuses for this subscription group.
    pub fn status(&self) -> Result<Vec<SubscriptionStatus>, StoreKitError> {
        Self::status_for(&self.subscription_group_id)
    }

    /// Fetches the `StoreKit` subscription statuses for the supplied subscription group identifier.
    pub fn status_for(group_id: &str) -> Result<Vec<SubscriptionStatus>, StoreKitError> {
        let group_id = cstring_from_str(group_id, "subscription group id")?;
        let mut statuses_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_subscription_info_statuses_json(
                group_id.as_ptr(),
                &mut statuses_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        let payloads = unsafe {
            parse_json_ptr::<Vec<SubscriptionStatusPayload>>(statuses_json, "subscription statuses")
        }?;
        payloads
            .into_iter()
            .map(SubscriptionStatusPayload::into_subscription_status)
            .collect::<Result<Vec<_>, _>>()
    }

    /// Fetches the `StoreKit` subscription status for the supplied transaction identifier.
    pub fn status_for_transaction(
        transaction_id: u64,
    ) -> Result<Option<SubscriptionStatus>, StoreKitError> {
        let transaction_id = cstring_from_str(&transaction_id.to_string(), "transaction id")?;
        let mut status_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_subscription_info_status_for_transaction(
                transaction_id.as_ptr(),
                &mut status_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        unsafe {
            parse_optional_json_ptr::<SubscriptionStatusPayload>(
                status_json,
                "subscription status for transaction",
            )
        }
        .and_then(|payload| {
            payload
                .map(SubscriptionStatusPayload::into_subscription_status)
                .transpose()
        })
    }
}

#[derive(Debug, Clone)]
/// Wraps `StoreKit.Product.SubscriptionInfo.Status`.
pub struct SubscriptionStatus {
    /// State reported by `StoreKit`.
    pub state: RenewalState,
    /// Transaction payload returned by `StoreKit`.
    pub transaction: VerificationResult<Transaction>,
    /// Renewal info payload returned by `StoreKit`.
    pub renewal_info: VerificationResult<RenewalInfo>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubscriptionInfoPayload {
    #[serde(rename = "introductoryOffer")]
    introductory_offer: Option<SubscriptionOfferPayload>,
    #[serde(rename = "promotionalOffers")]
    promotional_offers: Vec<SubscriptionOfferPayload>,
    #[serde(rename = "winBackOffers")]
    win_back_offers: Vec<SubscriptionOfferPayload>,
    #[serde(rename = "subscriptionGroupID")]
    subscription_group_id: String,
    #[serde(rename = "subscriptionPeriod")]
    subscription_period: SubscriptionPeriodPayload,
    #[serde(default, rename = "pricingTerms")]
    pricing_terms: Vec<SubscriptionPricingTermsPayload>,
    #[serde(rename = "groupLevel")]
    group_level: Option<i64>,
    #[serde(rename = "groupDisplayName")]
    group_display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubscriptionCommitmentInfoPayload {
    price: String,
    #[serde(rename = "displayPrice")]
    display_price: String,
    period: SubscriptionPeriodPayload,
}

impl SubscriptionCommitmentInfoPayload {
    pub(crate) fn into_subscription_commitment_info(self) -> SubscriptionCommitmentInfo {
        SubscriptionCommitmentInfo {
            price: self.price,
            display_price: self.display_price,
            period: self.period.into_subscription_period(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubscriptionPricingTermsPayload {
    #[serde(rename = "billingPrice")]
    billing_price: String,
    #[serde(rename = "billingDisplayPrice")]
    billing_display_price: String,
    #[serde(rename = "billingPeriod")]
    billing_period: SubscriptionPeriodPayload,
    #[serde(rename = "billingPlanType")]
    billing_plan_type: String,
    #[serde(rename = "commitmentInfo")]
    commitment_info: SubscriptionCommitmentInfoPayload,
    #[serde(default, rename = "subscriptionOffers")]
    subscription_offers: Vec<SubscriptionOfferPayload>,
}

impl SubscriptionPricingTermsPayload {
    pub(crate) fn into_subscription_pricing_terms(self) -> SubscriptionPricingTerms {
        SubscriptionPricingTerms {
            billing_price: self.billing_price,
            billing_display_price: self.billing_display_price,
            billing_period: self.billing_period.into_subscription_period(),
            billing_plan_type: BillingPlanType::from_raw(self.billing_plan_type),
            commitment_info: self.commitment_info.into_subscription_commitment_info(),
            subscription_offers: self
                .subscription_offers
                .into_iter()
                .map(SubscriptionOfferPayload::into_subscription_offer)
                .collect(),
        }
    }
}

impl SubscriptionInfoPayload {
    pub(crate) fn into_subscription_info(self) -> SubscriptionInfo {
        SubscriptionInfo {
            introductory_offer: self
                .introductory_offer
                .map(SubscriptionOfferPayload::into_subscription_offer),
            promotional_offers: self
                .promotional_offers
                .into_iter()
                .map(SubscriptionOfferPayload::into_subscription_offer)
                .collect(),
            win_back_offers: self
                .win_back_offers
                .into_iter()
                .map(SubscriptionOfferPayload::into_subscription_offer)
                .collect(),
            subscription_group_id: self.subscription_group_id,
            subscription_period: self.subscription_period.into_subscription_period(),
            pricing_terms: self
                .pricing_terms
                .into_iter()
                .map(SubscriptionPricingTermsPayload::into_subscription_pricing_terms)
                .collect(),
            group_level: self.group_level,
            group_display_name: self.group_display_name,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubscriptionStatusPayload {
    state: String,
    transaction: VerificationResultPayload<TransactionPayload>,
    #[serde(rename = "renewalInfo")]
    renewal_info: VerificationResultPayload<RenewalInfoPayload>,
}

impl SubscriptionStatusPayload {
    pub(crate) fn into_subscription_status(self) -> Result<SubscriptionStatus, StoreKitError> {
        Ok(SubscriptionStatus {
            state: RenewalState::from_raw(self.state),
            transaction: self
                .transaction
                .into_result(Transaction::from_snapshot_payload)?,
            renewal_info: self
                .renewal_info
                .into_result(|payload| Ok(payload.into_renewal_info()))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::subscription::{
        SubscriptionOfferType, SubscriptionPaymentMode, SubscriptionPeriodUnit,
    };

    #[test]
    fn billing_plan_types_round_trip_known_values() {
        let cases = [
            ("monthly", BillingPlanType::Monthly),
            ("upFront", BillingPlanType::UpFront),
        ];

        for (raw, billing_plan_type) in cases {
            assert_eq!(BillingPlanType::from_raw(raw.to_owned()), billing_plan_type);
            assert_eq!(billing_plan_type.as_str(), raw);
        }
    }

    #[test]
    fn unknown_billing_plan_type_is_preserved() {
        let billing_plan_type = BillingPlanType::from_raw("seasonal".to_owned());

        assert_eq!(billing_plan_type, BillingPlanType::Unknown("seasonal".into()));
        assert_eq!(billing_plan_type.as_str(), "seasonal");
    }

    #[test]
    fn billing_plan_type_serializes_as_raw_string() {
        assert_eq!(
            serde_json::to_value(BillingPlanType::Monthly).expect("serialize monthly plan"),
            json!("monthly")
        );
        assert_eq!(
            serde_json::to_value(BillingPlanType::Unknown("custom".into()))
                .expect("serialize custom plan"),
            json!("custom")
        );
    }

    #[test]
    fn subscription_pricing_terms_payload_converts_nested_fields() {
        let payload: SubscriptionPricingTermsPayload = serde_json::from_value(json!({
            "billingPrice": "59.99",
            "billingDisplayPrice": "$59.99",
            "billingPeriod": {
                "unit": "year",
                "value": 1
            },
            "billingPlanType": "upFront",
            "commitmentInfo": {
                "price": "59.99",
                "displayPrice": "$59.99",
                "period": {
                    "unit": "year",
                    "value": 1
                }
            },
            "subscriptionOffers": [{
                "id": "winback",
                "type": "winBack",
                "price": "29.99",
                "displayPrice": "$29.99",
                "period": {
                    "unit": "month",
                    "value": 3
                },
                "periodCount": 1,
                "paymentMode": "payUpFront"
            }]
        }))
        .expect("deserialize pricing terms payload");
        let pricing_terms = payload.into_subscription_pricing_terms();

        assert_eq!(pricing_terms.billing_plan_type, BillingPlanType::UpFront);
        assert_eq!(
            pricing_terms.billing_period,
            SubscriptionPeriod {
                unit: SubscriptionPeriodUnit::Year,
                value: 1,
            }
        );
        assert_eq!(pricing_terms.commitment_info.price, "59.99");
        let offer = pricing_terms
            .subscription_offers
            .first()
            .expect("one subscription offer");
        assert_eq!(offer.offer_type, SubscriptionOfferType::WinBack);
        assert_eq!(offer.payment_mode, SubscriptionPaymentMode::PayUpFront);
    }

    #[test]
    fn subscription_info_payload_converts_optional_fields() {
        let payload: SubscriptionInfoPayload = serde_json::from_value(json!({
            "promotionalOffers": [],
            "winBackOffers": [],
            "subscriptionGroupID": "group-1",
            "subscriptionPeriod": {
                "unit": "month",
                "value": 1
            },
            "pricingTerms": [],
            "groupLevel": 2,
            "groupDisplayName": "Pro"
        }))
        .expect("deserialize subscription info payload");
        let info = payload.into_subscription_info();

        assert_eq!(info.subscription_group_id, "group-1");
        assert_eq!(
            info.subscription_period,
            SubscriptionPeriod {
                unit: SubscriptionPeriodUnit::Month,
                value: 1,
            }
        );
        assert_eq!(info.group_level, Some(2));
        assert_eq!(info.group_display_name.as_deref(), Some("Pro"));
        assert!(info.pricing_terms.is_empty());
    }
}
