use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionPeriodUnit {
    Day,
    Week,
    Month,
    Year,
    Unknown(String),
}

impl SubscriptionPeriodUnit {
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
pub struct SubscriptionPeriod {
    pub unit: SubscriptionPeriodUnit,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionOfferType {
    Introductory,
    Promotional,
    WinBack,
    Unknown(String),
}

impl SubscriptionOfferType {
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
pub enum SubscriptionPaymentMode {
    PayAsYouGo,
    PayUpFront,
    FreeTrial,
    Unknown(String),
}

impl SubscriptionPaymentMode {
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
pub struct SubscriptionOffer {
    pub id: Option<String>,
    pub offer_type: SubscriptionOfferType,
    pub price: String,
    pub display_price: String,
    pub period: SubscriptionPeriod,
    pub period_count: i64,
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
