use core::ptr;

use serde::Deserialize;

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
pub struct SubscriptionInfo {
    pub introductory_offer: Option<SubscriptionOffer>,
    pub promotional_offers: Vec<SubscriptionOffer>,
    pub win_back_offers: Vec<SubscriptionOffer>,
    pub subscription_group_id: String,
    pub subscription_period: SubscriptionPeriod,
    pub group_level: Option<i64>,
    pub group_display_name: Option<String>,
}

impl SubscriptionInfo {
    pub fn is_eligible_for_intro_offer(&self) -> Result<bool, StoreKitError> {
        Self::is_eligible_for_intro_offer_for(&self.subscription_group_id)
    }

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

    pub fn status(&self) -> Result<Vec<SubscriptionStatus>, StoreKitError> {
        Self::status_for(&self.subscription_group_id)
    }

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
pub struct SubscriptionStatus {
    pub state: RenewalState,
    pub transaction: VerificationResult<Transaction>,
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
    #[serde(rename = "groupLevel")]
    group_level: Option<i64>,
    #[serde(rename = "groupDisplayName")]
    group_display_name: Option<String>,
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
