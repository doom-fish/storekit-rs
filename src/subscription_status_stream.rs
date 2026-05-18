use core::ffi::c_void;
use core::ptr;
use std::ptr::NonNull;
use std::time::Duration;

use serde::Deserialize;

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{duration_to_timeout_ms, error_from_status, parse_json_ptr};
use crate::subscription_info::{SubscriptionStatus, SubscriptionStatusPayload};

#[derive(Debug, Clone)]
/// Carries the `StoreKit` statuses for a subscription group.
pub struct SubscriptionGroupStatuses {
    /// Subscription group identifier reported by `StoreKit`.
    pub group_id: String,
    /// `StoreKit`-provided `statuses` value.
    pub statuses: Vec<SubscriptionStatus>,
}

impl SubscriptionStatus {
    /// Creates a stream backed by `StoreKit` subscription status updates.
    pub fn updates() -> Result<SubscriptionStatusStream, StoreKitError> {
        SubscriptionStatusStream::new()
    }

    /// Creates a stream backed by `StoreKit` subscription-group statuses.
    pub fn all() -> Result<SubscriptionGroupStatusStream, StoreKitError> {
        SubscriptionGroupStatusStream::new()
    }
}

#[derive(Debug)]
/// Wraps the `StoreKit` subscription status update stream.
pub struct SubscriptionStatusStream {
    handle: NonNull<c_void>,
    finished: bool,
}

impl Drop for SubscriptionStatusStream {
    fn drop(&mut self) {
        unsafe { ffi::sk_subscription_status_stream_release(self.handle.as_ptr()) };
    }
}

impl SubscriptionStatusStream {
    fn new() -> Result<Self, StoreKitError> {
        let mut error_message = ptr::null_mut();
        let handle = unsafe { ffi::sk_subscription_status_stream_create(&mut error_message) };
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
    pub fn next(&mut self) -> Result<Option<SubscriptionStatus>, StoreKitError> {
        self.next_timeout(Duration::from_secs(30))
    }

    /// Waits for the next value from the `StoreKit` stream up to the supplied timeout.
    pub fn next_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<SubscriptionStatus>, StoreKitError> {
        let mut status_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_subscription_status_stream_next(
                self.handle.as_ptr(),
                duration_to_timeout_ms(timeout),
                &mut status_json,
                &mut error_message,
            )
        };

        match status {
            ffi::status::OK => {
                let payload = unsafe {
                    parse_json_ptr::<SubscriptionStatusPayload>(
                        status_json,
                        "subscription status update",
                    )
                }?;
                payload.into_subscription_status().map(Some)
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

#[derive(Debug)]
/// Wraps the `StoreKit` subscription-group status stream.
pub struct SubscriptionGroupStatusStream {
    handle: NonNull<c_void>,
    finished: bool,
}

impl Drop for SubscriptionGroupStatusStream {
    fn drop(&mut self) {
        unsafe { ffi::sk_subscription_group_status_stream_release(self.handle.as_ptr()) };
    }
}

impl SubscriptionGroupStatusStream {
    fn new() -> Result<Self, StoreKitError> {
        let mut error_message = ptr::null_mut();
        let handle = unsafe { ffi::sk_subscription_group_status_stream_create(&mut error_message) };
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
    pub fn next(&mut self) -> Result<Option<SubscriptionGroupStatuses>, StoreKitError> {
        self.next_timeout(Duration::from_secs(30))
    }

    /// Waits for the next value from the `StoreKit` stream up to the supplied timeout.
    pub fn next_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<SubscriptionGroupStatuses>, StoreKitError> {
        let mut payload_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_subscription_group_status_stream_next(
                self.handle.as_ptr(),
                duration_to_timeout_ms(timeout),
                &mut payload_json,
                &mut error_message,
            )
        };

        match status {
            ffi::status::OK => {
                let payload = unsafe {
                    parse_json_ptr::<SubscriptionGroupStatusesPayload>(
                        payload_json,
                        "subscription group statuses",
                    )
                }?;
                payload.into_group_statuses().map(Some)
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
struct SubscriptionGroupStatusesPayload {
    #[serde(rename = "groupID")]
    group_id: String,
    statuses: Vec<SubscriptionStatusPayload>,
}

impl SubscriptionGroupStatusesPayload {
    fn into_group_statuses(self) -> Result<SubscriptionGroupStatuses, StoreKitError> {
        Ok(SubscriptionGroupStatuses {
            group_id: self.group_id,
            statuses: self
                .statuses
                .into_iter()
                .map(SubscriptionStatusPayload::into_subscription_status)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
