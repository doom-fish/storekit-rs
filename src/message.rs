use crate::error::StoreKitError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageReason {
    Generic,
    PriceIncreaseConsent,
    BillingIssue,
    WinBackOffer,
    Unknown(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub reason: MessageReason,
    pub localized_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MessageStream;

impl Message {
    pub const fn is_supported() -> bool {
        false
    }

    pub fn messages() -> Result<MessageStream, StoreKitError> {
        Err(StoreKitError::NotSupported(
            "StoreKit.Message is unavailable on macOS".to_owned(),
        ))
    }
}

impl MessageStream {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<Message>, StoreKitError> {
        Err(StoreKitError::NotSupported(
            "StoreKit.Message is unavailable on macOS".to_owned(),
        ))
    }
}
