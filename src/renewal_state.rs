#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenewalState {
    Subscribed,
    Expired,
    InBillingRetryPeriod,
    InGracePeriod,
    Revoked,
    Unknown(String),
}

impl RenewalState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Subscribed => "subscribed",
            Self::Expired => "expired",
            Self::InBillingRetryPeriod => "inBillingRetryPeriod",
            Self::InGracePeriod => "inGracePeriod",
            Self::Revoked => "revoked",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub fn from_raw(raw: impl Into<String>) -> Self {
        match raw.into().as_str() {
            "subscribed" => Self::Subscribed,
            "expired" => Self::Expired,
            "inBillingRetryPeriod" => Self::InBillingRetryPeriod,
            "inGracePeriod" => Self::InGracePeriod,
            "revoked" => Self::Revoked,
            other => Self::Unknown(other.to_owned()),
        }
    }
}
