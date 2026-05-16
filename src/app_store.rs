use core::ptr;

use crate::error::StoreKitError;
use crate::ffi;
use crate::private::error_from_status;

/// Helpers backed by `StoreKit.AppStore`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AppStore;

impl AppStore {
    /// Triggers a restore/sync with the App Store.
    pub fn sync() -> Result<(), StoreKitError> {
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::sk_app_store_sync(&mut error_message) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    /// Opens manage-subscriptions UI when supported by the platform.
    pub fn show_manage_subscriptions() -> Result<(), StoreKitError> {
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::sk_app_store_show_manage_subscriptions(&mut error_message) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }
}
