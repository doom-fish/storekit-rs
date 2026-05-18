use core::ffi::c_void;
use core::ptr;
use std::ptr::NonNull;
use std::time::Duration;

use serde::Deserialize;

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{duration_to_timeout_ms, error_from_status, parse_json_ptr};
use crate::product::{Product, ProductPayload};
use crate::subscription::{SubscriptionOffer, SubscriptionOfferPayload};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.PurchaseIntent`.
pub struct PurchaseIntent {
    /// `StoreKit`-provided `product` value.
    pub product: Product,
    /// Offer metadata reported by `StoreKit`.
    pub offer: Option<SubscriptionOffer>,
}

impl PurchaseIntent {
    /// Returns the product identifier reported by `StoreKit` for this purchase intent.
    pub fn id(&self) -> &str {
        &self.product.id
    }

    /// Creates a stream backed by `StoreKit` purchase intents.
    pub fn intents() -> Result<PurchaseIntentStream, StoreKitError> {
        PurchaseIntentStream::new()
    }
}

#[derive(Debug)]
/// Wraps the `StoreKit` purchase intent stream.
pub struct PurchaseIntentStream {
    handle: NonNull<c_void>,
    finished: bool,
}

impl Drop for PurchaseIntentStream {
    fn drop(&mut self) {
        unsafe { ffi::sk_purchase_intent_stream_release(self.handle.as_ptr()) };
    }
}

impl PurchaseIntentStream {
    fn new() -> Result<Self, StoreKitError> {
        let mut error_message = ptr::null_mut();
        let handle = unsafe { ffi::sk_purchase_intent_stream_create(&mut error_message) };
        let handle = NonNull::new(handle)
            .ok_or_else(|| unsafe { error_from_status(ffi::status::UNKNOWN, error_message) })?;
        Ok(Self {
            handle,
            finished: false,
        })
    }

    /// Returns whether this `StoreKit` stream has reached the end of the sequence.
    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    #[allow(clippy::should_implement_trait)]
    /// Waits for the next value from the `StoreKit` stream using the default timeout.
    pub fn next(&mut self) -> Result<Option<PurchaseIntent>, StoreKitError> {
        self.next_timeout(Duration::from_secs(30))
    }

    /// Waits for the next value from the `StoreKit` stream up to the supplied timeout.
    pub fn next_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<PurchaseIntent>, StoreKitError> {
        let mut payload_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_purchase_intent_stream_next(
                self.handle.as_ptr(),
                duration_to_timeout_ms(timeout),
                &mut payload_json,
                &mut error_message,
            )
        };

        match status {
            ffi::status::OK => {
                let payload = unsafe {
                    parse_json_ptr::<PurchaseIntentPayload>(payload_json, "purchase intent")
                }?;
                payload.into_purchase_intent().map(Some)
            }
            ffi::status::END_OF_STREAM => {
                self.finished = true;
                Ok(None)
            }
            ffi::status::TIMED_OUT => Ok(None),
            _ => Err(unsafe { error_from_status(status, error_message) }),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PurchaseIntentPayload {
    product: ProductPayload,
    offer: Option<SubscriptionOfferPayload>,
}

impl PurchaseIntentPayload {
    fn into_purchase_intent(self) -> Result<PurchaseIntent, StoreKitError> {
        Ok(PurchaseIntent {
            product: self.product.into_product()?,
            offer: self
                .offer
                .map(SubscriptionOfferPayload::into_subscription_offer),
        })
    }
}
