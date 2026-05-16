use core::ptr;

use serde::Deserialize;

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{
    cstring_from_str, error_from_status, parse_json_ptr, parse_optional_json_ptr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalPurchaseNoticeResult {
    Continued,
    Cancelled,
    ContinuedWithExternalPurchaseToken { token: String },
    Unknown(String),
}

impl ExternalPurchaseNoticeResult {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Continued => "continued",
            Self::Cancelled => "cancelled",
            Self::ContinuedWithExternalPurchaseToken { .. } => "continuedWithExternalPurchaseToken",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalPurchase;

impl ExternalPurchase {
    pub fn can_present() -> Result<bool, StoreKitError> {
        let mut raw_value = 0;
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_external_purchase_can_present(&mut raw_value, &mut error_message)
        };
        if status == ffi::status::OK {
            Ok(raw_value != 0)
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn present_notice_sheet() -> Result<ExternalPurchaseNoticeResult, StoreKitError> {
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_external_purchase_present_notice_result_json(&mut result_json, &mut error_message)
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        let payload = unsafe {
            parse_json_ptr::<ExternalPurchaseNoticeResultPayload>(
                result_json,
                "external purchase notice result",
            )
        }?;
        Ok(payload.into_notice_result())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalPurchaseLink;

impl ExternalPurchaseLink {
    pub fn can_open() -> Result<bool, StoreKitError> {
        let mut raw_value = 0;
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::sk_external_purchase_link_can_open(&mut raw_value, &mut error_message) };
        if status == ffi::status::OK {
            Ok(raw_value != 0)
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn eligible_urls() -> Result<Option<Vec<String>>, StoreKitError> {
        let mut urls_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_external_purchase_link_eligible_urls_json(&mut urls_json, &mut error_message)
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        unsafe { parse_optional_json_ptr::<Vec<String>>(urls_json, "eligible external purchase URLs") }
    }

    pub fn open() -> Result<(), StoreKitError> {
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::sk_external_purchase_link_open(&mut error_message) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn open_url(url: &str) -> Result<(), StoreKitError> {
        let url = cstring_from_str(url, "external purchase URL")?;
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_external_purchase_link_open_url(url.as_ptr(), &mut error_message)
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalPurchaseCustomLinkNoticeType {
    WithinApp,
    Browser,
    Unknown(i64),
}

impl ExternalPurchaseCustomLinkNoticeType {
    pub const fn as_raw(&self) -> i64 {
        match self {
            Self::WithinApp => 0,
            Self::Browser => 1,
            Self::Unknown(value) => *value,
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalPurchaseCustomLinkNoticeResult {
    Cancelled,
    Continued,
    Unknown(String),
}

impl ExternalPurchaseCustomLinkNoticeResult {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Cancelled => "cancelled",
            Self::Continued => "continued",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalPurchaseCustomLinkToken {
    pub value: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalPurchaseCustomLink;

impl ExternalPurchaseCustomLink {
    pub fn is_eligible() -> Result<bool, StoreKitError> {
        let mut raw_value = 0;
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_external_purchase_custom_link_is_eligible(&mut raw_value, &mut error_message)
        };
        if status == ffi::status::OK {
            Ok(raw_value != 0)
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn show_notice(
        notice_type: ExternalPurchaseCustomLinkNoticeType,
    ) -> Result<ExternalPurchaseCustomLinkNoticeResult, StoreKitError> {
        let raw_notice_type = i32::try_from(notice_type.as_raw()).map_err(|_| {
            StoreKitError::InvalidArgument(format!(
                "external purchase custom link notice type '{}' does not fit in i32",
                notice_type.as_raw()
            ))
        })?;
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_external_purchase_custom_link_show_notice_result_json(
                raw_notice_type,
                &mut result_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        let payload = unsafe {
            parse_json_ptr::<ExternalPurchaseCustomLinkNoticeResultPayload>(
                result_json,
                "external purchase custom link notice result",
            )
        }?;
        Ok(payload.into_notice_result())
    }

    pub fn token(token_type: &str) -> Result<Option<ExternalPurchaseCustomLinkToken>, StoreKitError> {
        let token_type = cstring_from_str(token_type, "external purchase custom link token type")?;
        let mut token_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_external_purchase_custom_link_token_json(
                token_type.as_ptr(),
                &mut token_json,
                &mut error_message,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        unsafe {
            parse_optional_json_ptr::<ExternalPurchaseCustomLinkTokenPayload>(
                token_json,
                "external purchase custom link token",
            )
        }
        .map(|payload| {
            payload.map(|payload| ExternalPurchaseCustomLinkToken { value: payload.value })
        })
    }
}

#[derive(Debug, Deserialize)]
struct ExternalPurchaseNoticeResultPayload {
    kind: String,
    token: Option<String>,
}

impl ExternalPurchaseNoticeResultPayload {
    fn into_notice_result(self) -> ExternalPurchaseNoticeResult {
        match self.kind.as_str() {
            "continued" => ExternalPurchaseNoticeResult::Continued,
            "cancelled" => ExternalPurchaseNoticeResult::Cancelled,
            "continuedWithExternalPurchaseToken" => {
                ExternalPurchaseNoticeResult::ContinuedWithExternalPurchaseToken {
                    token: self.token.unwrap_or_default(),
                }
            }
            other => ExternalPurchaseNoticeResult::Unknown(other.to_owned()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExternalPurchaseCustomLinkNoticeResultPayload {
    kind: String,
}

impl ExternalPurchaseCustomLinkNoticeResultPayload {
    fn into_notice_result(self) -> ExternalPurchaseCustomLinkNoticeResult {
        match self.kind.as_str() {
            "cancelled" => ExternalPurchaseCustomLinkNoticeResult::Cancelled,
            "continued" => ExternalPurchaseCustomLinkNoticeResult::Continued,
            other => ExternalPurchaseCustomLinkNoticeResult::Unknown(other.to_owned()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExternalPurchaseCustomLinkTokenPayload {
    value: String,
}
