use serde::Deserialize;
use serde_json::Value;

use crate::app_transaction::AppTransaction;
use crate::error::StoreKitError;
use crate::ffi;
use crate::private::{
    decode_base64, decode_base64_urlsafe, error_from_status, parse_optional_json_ptr,
};
use crate::verification_result::VerificationResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppReceipt {
    pub path: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReceiptValidator;

impl ReceiptValidator {
    pub fn current_receipt() -> Result<Option<AppReceipt>, StoreKitError> {
        let mut receipt_json = core::ptr::null_mut();
        let mut error_message = core::ptr::null_mut();
        let status = unsafe { ffi::sk_receipt_json(&mut receipt_json, &mut error_message) };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, error_message) });
        }

        unsafe { parse_optional_json_ptr::<AppReceiptPayload>(receipt_json, "receipt") }
            .and_then(|payload| payload.map(AppReceiptPayload::into_receipt).transpose())
    }

    pub fn shared_app_transaction() -> Result<VerificationResult<AppTransaction>, StoreKitError> {
        AppTransaction::shared()
    }

    pub fn refresh_app_transaction() -> Result<VerificationResult<AppTransaction>, StoreKitError> {
        AppTransaction::refresh()
    }

    pub fn extract_unverified_payload(jws: &str) -> Result<Value, StoreKitError> {
        let mut segments = jws.split('.');
        let _header = segments.next().ok_or_else(|| {
            StoreKitError::InvalidArgument("JWS is missing a header segment".to_owned())
        })?;
        let payload = segments.next().ok_or_else(|| {
            StoreKitError::InvalidArgument("JWS is missing a payload segment".to_owned())
        })?;
        let _signature = segments.next().ok_or_else(|| {
            StoreKitError::InvalidArgument("JWS is missing a signature segment".to_owned())
        })?;
        if segments.next().is_some() {
            return Err(StoreKitError::InvalidArgument(
                "JWS contains too many segments".to_owned(),
            ));
        }
        let payload_bytes = decode_base64_urlsafe(payload, "JWS payload")?;
        serde_json::from_slice::<Value>(&payload_bytes).map_err(|error| {
            StoreKitError::InvalidArgument(format!("failed to decode JWS payload JSON: {error}"))
        })
    }
}

#[derive(Debug, Deserialize)]
struct AppReceiptPayload {
    path: String,
    #[serde(rename = "dataBase64")]
    data_base64: String,
}

impl AppReceiptPayload {
    fn into_receipt(self) -> Result<AppReceipt, StoreKitError> {
        Ok(AppReceipt {
            path: self.path,
            data: decode_base64(&self.data_base64, "receipt data")?,
        })
    }
}
