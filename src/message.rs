use crate::error::StoreKitError;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Message.Reason`.
pub enum MessageReason {
    /// Represents the `Generic` `StoreKit` case.
    Generic,
    /// Represents the `PriceIncreaseConsent` `StoreKit` case.
    PriceIncreaseConsent,
    /// Represents the `BillingIssue` `StoreKit` case.
    BillingIssue,
    /// Represents the `WinBackOffer` `StoreKit` case.
    WinBackOffer,
    /// Preserves an unrecognized `StoreKit` case.
    Unknown(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Message`.
pub struct Message {
    /// Reason reported by `StoreKit`.
    pub reason: MessageReason,
    /// Localized reason reported by `StoreKit`.
    pub localized_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
/// Represents the `StoreKit` message stream.
pub struct MessageStream;

impl Message {
    /// Returns whether `StoreKit.Message` is available on this platform.
    pub const fn is_supported() -> bool {
        false
    }

    /// Returns the `StoreKit` message stream when the API is available on this platform.
    pub fn messages() -> Result<MessageStream, StoreKitError> {
        Err(StoreKitError::NotSupported(
            "StoreKit.Message is unavailable on macOS".to_owned(),
        ))
    }
}

impl MessageStream {
    #[allow(clippy::should_implement_trait)]
    /// Waits for the next value from the `StoreKit` stream using the default timeout.
    pub fn next(&mut self) -> Result<Option<Message>, StoreKitError> {
        Err(StoreKitError::NotSupported(
            "StoreKit.Message is unavailable on macOS".to_owned(),
        ))
    }
}
