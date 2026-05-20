#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Product.SubscriptionInfo.RenewalState`.
pub enum RenewalState {
    /// Represents the `Subscribed` `StoreKit` case.
    Subscribed,
    /// Represents the `Expired` `StoreKit` case.
    Expired,
    /// Represents the `InBillingRetryPeriod` `StoreKit` case.
    InBillingRetryPeriod,
    /// Represents the `InGracePeriod` `StoreKit` case.
    InGracePeriod,
    /// Represents the `Revoked` `StoreKit` case.
    Revoked,
    /// Preserves an unrecognized `StoreKit` case.
    Unknown(String),
}

impl RenewalState {
    /// Returns the raw `StoreKit` string for this renewal state.
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

    /// Builds a renewal state from the raw `StoreKit` string.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_renewal_states_round_trip() {
        let cases = [
            ("subscribed", RenewalState::Subscribed),
            ("expired", RenewalState::Expired),
            ("inBillingRetryPeriod", RenewalState::InBillingRetryPeriod),
            ("inGracePeriod", RenewalState::InGracePeriod),
            ("revoked", RenewalState::Revoked),
        ];

        for (raw, state) in cases {
            assert_eq!(RenewalState::from_raw(raw), state);
            assert_eq!(state.as_str(), raw);
        }
    }

    #[test]
    fn unknown_renewal_state_is_preserved() {
        let state = RenewalState::from_raw("paused");

        assert_eq!(state, RenewalState::Unknown("paused".into()));
        assert_eq!(state.as_str(), "paused");
    }
}
