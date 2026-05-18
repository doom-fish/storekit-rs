use core::ffi::c_void;
use core::ptr;
use std::ptr::NonNull;
use std::time::Duration;

use serde::Deserialize;

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{
    duration_to_timeout_ms, error_from_status, parse_json_ptr, parse_optional_json_ptr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.Storefront`.
pub struct Storefront {
    /// Country code reported by `StoreKit`.
    pub country_code: String,
    /// `StoreKit` identifier for this value.
    pub id: String,
    /// Currency code reported by `StoreKit`.
    pub currency_code: Option<String>,
}

impl Storefront {
    /// Fetches the current `StoreKit.Storefront`.
    pub fn current() -> Result<Option<Self>, StoreKitError> {
        let mut storefront_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status =
            unsafe { ffi::sk_storefront_current_json(&mut storefront_json, &mut error_message) };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        unsafe { parse_optional_json_ptr::<StorefrontPayload>(storefront_json, "storefront") }
            .map(|payload| payload.map(StorefrontPayload::into_storefront))
    }

    /// Creates a stream backed by `StoreKit.Storefront` updates.
    pub fn updates() -> Result<StorefrontStream, StoreKitError> {
        StorefrontStream::new()
    }
}

#[derive(Debug)]
/// Wraps the `StoreKit` storefront update stream.
pub struct StorefrontStream {
    handle: NonNull<c_void>,
    finished: bool,
}

impl Drop for StorefrontStream {
    fn drop(&mut self) {
        unsafe { ffi::sk_storefront_stream_release(self.handle.as_ptr()) };
    }
}

impl StorefrontStream {
    fn new() -> Result<Self, StoreKitError> {
        let mut error_message = ptr::null_mut();
        let handle = unsafe { ffi::sk_storefront_stream_create(&mut error_message) };
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
    pub fn next(&mut self) -> Result<Option<Storefront>, StoreKitError> {
        self.next_timeout(Duration::from_secs(30))
    }

    /// Waits for the next value from the `StoreKit` stream up to the supplied timeout.
    pub fn next_timeout(&mut self, timeout: Duration) -> Result<Option<Storefront>, StoreKitError> {
        let mut storefront_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_storefront_stream_next(
                self.handle.as_ptr(),
                duration_to_timeout_ms(timeout),
                &mut storefront_json,
                &mut error_message,
            )
        };

        match status {
            ffi::status::OK => {
                let payload =
                    unsafe { parse_json_ptr::<StorefrontPayload>(storefront_json, "storefront") }?;
                Ok(Some(payload.into_storefront()))
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
pub(crate) struct StorefrontPayload {
    #[serde(rename = "countryCode")]
    country_code: String,
    id: String,
    #[serde(rename = "currencyCode")]
    currency_code: Option<String>,
}

impl StorefrontPayload {
    pub(crate) fn into_storefront(self) -> Storefront {
        Storefront {
            country_code: self.country_code,
            id: self.id,
            currency_code: self.currency_code,
        }
    }
}
