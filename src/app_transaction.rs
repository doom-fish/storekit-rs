use core::ptr;

use serde::Deserialize;

use crate::app_store::{AppStoreEnvironment, AppStorePlatform};
use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{decode_base64, error_from_status, parse_json_ptr};
use crate::verification_result::{VerificationResult, VerificationResultPayload};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppTransaction {
    pub app_id: Option<u64>,
    pub app_transaction_id: String,
    pub app_version: String,
    pub app_version_id: Option<u64>,
    pub bundle_id: String,
    pub environment: AppStoreEnvironment,
    pub original_app_version: String,
    pub original_purchase_date: String,
    pub original_platform: Option<AppStorePlatform>,
    pub preorder_date: Option<String>,
    pub json_representation: Vec<u8>,
}

impl AppTransaction {
    pub fn shared() -> Result<VerificationResult<Self>, StoreKitError> {
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status =
            unsafe { ffi::sk_app_transaction_shared(&mut result_json, &mut error_message) };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        let payload = unsafe {
            parse_json_ptr::<VerificationResultPayload<AppTransactionPayload>>(
                result_json,
                "app transaction",
            )
        }?;
        payload.into_result(AppTransactionPayload::into_app_transaction)
    }

    pub fn refresh() -> Result<VerificationResult<Self>, StoreKitError> {
        let mut result_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status =
            unsafe { ffi::sk_app_transaction_refresh(&mut result_json, &mut error_message) };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }
        let payload = unsafe {
            parse_json_ptr::<VerificationResultPayload<AppTransactionPayload>>(
                result_json,
                "refreshed app transaction",
            )
        }?;
        payload.into_result(AppTransactionPayload::into_app_transaction)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct AppTransactionPayload {
    #[serde(rename = "appID")]
    app_id: Option<u64>,
    #[serde(rename = "appTransactionID")]
    app_transaction_id: String,
    #[serde(rename = "appVersion")]
    app_version: String,
    #[serde(rename = "appVersionID")]
    app_version_id: Option<u64>,
    #[serde(rename = "bundleID")]
    bundle_id: String,
    environment: String,
    #[serde(rename = "originalAppVersion")]
    original_app_version: String,
    #[serde(rename = "originalPurchaseDate")]
    original_purchase_date: String,
    #[serde(rename = "originalPlatform")]
    original_platform: Option<String>,
    #[serde(rename = "preorderDate")]
    preorder_date: Option<String>,
    #[serde(rename = "jsonRepresentationBase64")]
    json_representation_base64: String,
}

impl AppTransactionPayload {
    fn into_app_transaction(self) -> Result<AppTransaction, StoreKitError> {
        Ok(AppTransaction {
            app_id: self.app_id,
            app_transaction_id: self.app_transaction_id,
            app_version: self.app_version,
            app_version_id: self.app_version_id,
            bundle_id: self.bundle_id,
            environment: AppStoreEnvironment::from_raw(self.environment),
            original_app_version: self.original_app_version,
            original_purchase_date: self.original_purchase_date,
            original_platform: self.original_platform.map(AppStorePlatform::from_raw),
            preorder_date: self.preorder_date,
            json_representation: decode_base64(
                &self.json_representation_base64,
                "app transaction JSON representation",
            )?,
        })
    }
}
