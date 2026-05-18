use serde::Deserialize;

use crate::app_store::AppStoreEnvironment;
use crate::transaction::{TransactionOffer, TransactionOfferPayload};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.RenewalInfo.ExpirationReason`.
pub enum ExpirationReason {
    /// Represents the `AutoRenewDisabled` `StoreKit` case.
    AutoRenewDisabled,
    /// Represents the `BillingError` `StoreKit` case.
    BillingError,
    /// Represents the `DidNotConsentToPriceIncrease` `StoreKit` case.
    DidNotConsentToPriceIncrease,
    /// Represents the `ProductUnavailable` `StoreKit` case.
    ProductUnavailable,
    /// Preserves an unrecognized `StoreKit` case.
    Unknown(String),
}

impl ExpirationReason {
    /// Returns the raw `StoreKit` string for this expiration reason.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AutoRenewDisabled => "autoRenewDisabled",
            Self::BillingError => "billingError",
            Self::DidNotConsentToPriceIncrease => "didNotConsentToPriceIncrease",
            Self::ProductUnavailable => "productUnavailable",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub(crate) fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "autoRenewDisabled" => Self::AutoRenewDisabled,
            "billingError" => Self::BillingError,
            "didNotConsentToPriceIncrease" => Self::DidNotConsentToPriceIncrease,
            "productUnavailable" => Self::ProductUnavailable,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.RenewalInfo.PriceIncreaseStatus`.
pub enum PriceIncreaseStatus {
    /// Represents the `NoIncreasePending` `StoreKit` case.
    NoIncreasePending,
    /// The `StoreKit` flow is pending further action.
    Pending,
    /// Represents the `Agreed` `StoreKit` case.
    Agreed,
    /// Preserves an unrecognized `StoreKit` case.
    Unknown(String),
}

impl PriceIncreaseStatus {
    /// Returns the raw `StoreKit` string for this price increase status.
    pub fn as_str(&self) -> &str {
        match self {
            Self::NoIncreasePending => "noIncreasePending",
            Self::Pending => "pending",
            Self::Agreed => "agreed",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub(crate) fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "noIncreasePending" => Self::NoIncreasePending,
            "pending" => Self::Pending,
            "agreed" => Self::Agreed,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.RenewalInfo`.
pub struct RenewalInfo {
    /// Original `StoreKit` transaction identifier.
    pub original_transaction_id: u64,
    /// Current product identifier reported by `StoreKit`.
    pub current_product_id: String,
    /// Whether `StoreKit` reports that the subscription will auto-renew.
    pub will_auto_renew: bool,
    /// Preferred renewal product identifier reported by `StoreKit`.
    pub auto_renew_preference: Option<String>,
    /// Expiration reason reported by `StoreKit`.
    pub expiration_reason: Option<ExpirationReason>,
    /// Price increase status reported by `StoreKit`.
    pub price_increase_status: PriceIncreaseStatus,
    /// Whether `StoreKit` reports that the subscription is in billing retry.
    pub is_in_billing_retry: bool,
    /// Grace-period expiration date reported by `StoreKit`.
    pub grace_period_expiration_date: Option<String>,
    /// Offer metadata reported by `StoreKit`.
    pub offer: Option<TransactionOffer>,
    /// Environment reported by `StoreKit`.
    pub environment: Option<AppStoreEnvironment>,
    /// Recent subscription start date reported by `StoreKit`.
    pub recent_subscription_start_date: String,
    /// Renewal date reported by `StoreKit`.
    pub renewal_date: Option<String>,
    /// Renewal price reported by `StoreKit`.
    pub renewal_price: Option<String>,
    /// Currency code reported by `StoreKit`.
    pub currency_code: Option<String>,
    /// Eligible win-back offer identifiers reported by `StoreKit`.
    pub eligible_win_back_offer_ids: Vec<String>,
    /// App account token reported by `StoreKit`.
    pub app_account_token: Option<String>,
    /// App transaction identifier reported by `StoreKit`.
    pub app_transaction_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RenewalInfoPayload {
    #[serde(rename = "originalTransactionID")]
    original_transaction_id: u64,
    #[serde(rename = "currentProductID")]
    current_product_id: String,
    #[serde(rename = "willAutoRenew")]
    will_auto_renew: bool,
    #[serde(rename = "autoRenewPreference")]
    auto_renew_preference: Option<String>,
    #[serde(rename = "expirationReason")]
    expiration_reason: Option<String>,
    #[serde(rename = "priceIncreaseStatus")]
    price_increase_status: String,
    #[serde(rename = "isInBillingRetry")]
    is_in_billing_retry: bool,
    #[serde(rename = "gracePeriodExpirationDate")]
    grace_period_expiration_date: Option<String>,
    offer: Option<TransactionOfferPayload>,
    environment: Option<String>,
    #[serde(rename = "recentSubscriptionStartDate")]
    recent_subscription_start_date: String,
    #[serde(rename = "renewalDate")]
    renewal_date: Option<String>,
    #[serde(rename = "renewalPrice")]
    renewal_price: Option<String>,
    #[serde(rename = "currencyCode")]
    currency_code: Option<String>,
    #[serde(rename = "eligibleWinBackOfferIDs")]
    eligible_win_back_offer_ids: Vec<String>,
    #[serde(rename = "appAccountToken")]
    app_account_token: Option<String>,
    #[serde(rename = "appTransactionID")]
    app_transaction_id: Option<String>,
}

impl RenewalInfoPayload {
    pub(crate) fn into_renewal_info(self) -> RenewalInfo {
        RenewalInfo {
            original_transaction_id: self.original_transaction_id,
            current_product_id: self.current_product_id,
            will_auto_renew: self.will_auto_renew,
            auto_renew_preference: self.auto_renew_preference,
            expiration_reason: self.expiration_reason.map(ExpirationReason::from_raw),
            price_increase_status: PriceIncreaseStatus::from_raw(self.price_increase_status),
            is_in_billing_retry: self.is_in_billing_retry,
            grace_period_expiration_date: self.grace_period_expiration_date,
            offer: self
                .offer
                .map(TransactionOfferPayload::into_transaction_offer),
            environment: self.environment.map(AppStoreEnvironment::from_raw),
            recent_subscription_start_date: self.recent_subscription_start_date,
            renewal_date: self.renewal_date,
            renewal_price: self.renewal_price,
            currency_code: self.currency_code,
            eligible_win_back_offer_ids: self.eligible_win_back_offer_ids,
            app_account_token: self.app_account_token,
            app_transaction_id: self.app_transaction_id,
        }
    }
}
