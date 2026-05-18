use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Product.SubscriptionPeriod.Unit`.
pub enum SubscriptionPeriodUnit {
    /// Represents the `Day` `StoreKit` case.
    Day,
    /// Represents the `Week` `StoreKit` case.
    Week,
    /// Represents the `Month` `StoreKit` case.
    Month,
    /// Represents the `Year` `StoreKit` case.
    Year,
    /// Preserves an unrecognized `StoreKit` case.
    Unknown(String),
}

impl SubscriptionPeriodUnit {
    /// Returns the raw `StoreKit` string for this subscription period unit.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub(crate) fn from_raw(raw: String) -> Self {
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
/// Wraps `StoreKit.Product.SubscriptionPeriod`.
pub struct SubscriptionPeriod {
    /// Subscription period unit reported by `StoreKit`.
    pub unit: SubscriptionPeriodUnit,
    /// Value returned by `StoreKit`.
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Product.SubscriptionOffer.OfferType`.
pub enum SubscriptionOfferType {
    /// Represents the `Introductory` `StoreKit` case.
    Introductory,
    /// Represents the `Promotional` `StoreKit` case.
    Promotional,
    /// Represents the `WinBack` `StoreKit` case.
    WinBack,
    /// Preserves an unrecognized `StoreKit` case.
    Unknown(String),
}

impl SubscriptionOfferType {
    /// Returns the raw `StoreKit` string for this subscription offer type.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Introductory => "introductory",
            Self::Promotional => "promotional",
            Self::WinBack => "winBack",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub(crate) fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "introductory" => Self::Introductory,
            "promotional" => Self::Promotional,
            "winBack" => Self::WinBack,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Product.SubscriptionOffer.PaymentMode`.
pub enum SubscriptionPaymentMode {
    /// Represents the `PayAsYouGo` `StoreKit` case.
    PayAsYouGo,
    /// Represents the `PayUpFront` `StoreKit` case.
    PayUpFront,
    /// Represents the `FreeTrial` `StoreKit` case.
    FreeTrial,
    /// Preserves an unrecognized `StoreKit` case.
    Unknown(String),
}

impl SubscriptionPaymentMode {
    /// Returns the raw `StoreKit` string for this subscription payment mode.
    pub fn as_str(&self) -> &str {
        match self {
            Self::PayAsYouGo => "payAsYouGo",
            Self::PayUpFront => "payUpFront",
            Self::FreeTrial => "freeTrial",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub(crate) fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "payAsYouGo" => Self::PayAsYouGo,
            "payUpFront" => Self::PayUpFront,
            "freeTrial" => Self::FreeTrial,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Product.SubscriptionOffer`.
pub struct SubscriptionOffer {
    /// `StoreKit` identifier for this value.
    pub id: Option<String>,
    /// Offer type reported by `StoreKit`.
    pub offer_type: SubscriptionOfferType,
    /// Price reported by `StoreKit`.
    pub price: String,
    /// Localized display price reported by `StoreKit`.
    pub display_price: String,
    /// Subscription period reported by `StoreKit`.
    pub period: SubscriptionPeriod,
    /// Number of periods reported by `StoreKit`.
    pub period_count: i64,
    /// Payment mode reported by `StoreKit`.
    pub payment_mode: SubscriptionPaymentMode,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubscriptionPeriodPayload {
    unit: String,
    value: i64,
}

impl SubscriptionPeriodPayload {
    pub(crate) fn into_subscription_period(self) -> SubscriptionPeriod {
        SubscriptionPeriod {
            unit: SubscriptionPeriodUnit::from_raw(self.unit),
            value: self.value,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubscriptionOfferPayload {
    id: Option<String>,
    #[serde(rename = "type")]
    offer_type: String,
    price: String,
    #[serde(rename = "displayPrice")]
    display_price: String,
    period: SubscriptionPeriodPayload,
    #[serde(rename = "periodCount")]
    period_count: i64,
    #[serde(rename = "paymentMode")]
    payment_mode: String,
}

impl SubscriptionOfferPayload {
    pub(crate) fn into_subscription_offer(self) -> SubscriptionOffer {
        SubscriptionOffer {
            id: self.id,
            offer_type: SubscriptionOfferType::from_raw(self.offer_type),
            price: self.price,
            display_price: self.display_price,
            period: self.period.into_subscription_period(),
            period_count: self.period_count,
            payment_mode: SubscriptionPaymentMode::from_raw(self.payment_mode),
        }
    }
}
