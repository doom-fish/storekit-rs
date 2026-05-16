use core::ptr;

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{error_from_status, take_string};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppStoreEnvironment {
    Production,
    Sandbox,
    Xcode,
    Unknown(String),
}

impl AppStoreEnvironment {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Production => "production",
            Self::Sandbox => "sandbox",
            Self::Xcode => "xcode",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub(crate) fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "production" => Self::Production,
            "sandbox" => Self::Sandbox,
            "xcode" => Self::Xcode,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppStorePlatform {
    IOS,
    MacOS,
    TvOS,
    VisionOS,
    Unknown(String),
}

impl AppStorePlatform {
    pub fn as_str(&self) -> &str {
        match self {
            Self::IOS => "iOS",
            Self::MacOS => "macOS",
            Self::TvOS => "tvOS",
            Self::VisionOS => "visionOS",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub(crate) fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "iOS" => Self::IOS,
            "macOS" => Self::MacOS,
            "tvOS" => Self::TvOS,
            "visionOS" => Self::VisionOS,
            _ => Self::Unknown(raw),
        }
    }
}

/// Helpers backed by `StoreKit.AppStore`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AppStore;

impl AppStore {
    pub fn can_make_payments() -> Result<bool, StoreKitError> {
        let mut raw_value = 0;
        let mut error_message = ptr::null_mut();
        let status =
            unsafe { ffi::sk_app_store_can_make_payments(&mut raw_value, &mut error_message) };
        if status == ffi::status::OK {
            Ok(raw_value != 0)
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn device_verification_id() -> Result<Option<String>, StoreKitError> {
        let mut uuid_ptr = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status =
            unsafe { ffi::sk_app_store_device_verification_id(&mut uuid_ptr, &mut error_message) };
        if status == ffi::status::OK {
            Ok(unsafe { take_string(uuid_ptr) })
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn sync() -> Result<(), StoreKitError> {
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::sk_app_store_sync(&mut error_message) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn show_manage_subscriptions() -> Result<(), StoreKitError> {
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::sk_app_store_show_manage_subscriptions(&mut error_message) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn request_review() -> Result<(), StoreKitError> {
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::sk_app_store_request_review(&mut error_message) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn present_offer_code_redeem_sheet() -> Result<(), StoreKitError> {
        let mut error_message = ptr::null_mut();
        let status =
            unsafe { ffi::sk_app_store_present_offer_code_redeem_sheet(&mut error_message) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }
}
