use core::ptr;

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{cstring_from_str, error_from_status, take_string};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefundRequestStatus {
    Success,
    UserCancelled,
    Unknown(String),
}

impl RefundRequestStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Success => "success",
            Self::UserCancelled => "userCancelled",
            Self::Unknown(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "success" => Self::Success,
            "userCancelled" => Self::UserCancelled,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Refund;

impl Refund {
    pub fn begin_for_transaction_id(
        transaction_id: u64,
    ) -> Result<RefundRequestStatus, StoreKitError> {
        let transaction_id = cstring_from_str(&transaction_id.to_string(), "transaction id")?;
        let mut status_ptr = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_refund_begin_request_for_transaction_id(
                transaction_id.as_ptr(),
                &mut status_ptr,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }

        let raw_status = unsafe { take_string(status_ptr) }
            .ok_or_else(|| StoreKitError::Unknown("missing refund status payload".to_owned()))?;
        Ok(RefundRequestStatus::from_raw(raw_status))
    }
}
