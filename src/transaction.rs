use core::ffi::c_void;
use core::ptr;
use std::ptr::NonNull;
use std::time::Duration;

use serde::Deserialize;

use crate::error::{StoreKitError, VerificationFailure};
use crate::ffi;
use crate::private::{duration_to_timeout_ms, error_from_status, parse_json_ptr};

#[derive(Debug, Clone)]
pub enum VerificationResult<T> {
    Verified(T),
    Unverified(T, VerificationFailure),
}

impl<T> VerificationResult<T> {
    pub const fn is_verified(&self) -> bool {
        matches!(self, Self::Verified(_))
    }

    pub const fn payload(&self) -> &T {
        match self {
            Self::Verified(payload) | Self::Unverified(payload, _) => payload,
        }
    }

    pub fn into_payload(self) -> T {
        match self {
            Self::Verified(payload) | Self::Unverified(payload, _) => payload,
        }
    }

    pub const fn verification_failure(&self) -> Option<&VerificationFailure> {
        match self {
            Self::Verified(_) => None,
            Self::Unverified(_, failure) => Some(failure),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipType {
    Purchased,
    FamilyShared,
    Unknown(String),
}

impl OwnershipType {
    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "purchased" => Self::Purchased,
            "familyShared" => Self::FamilyShared,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionData {
    pub id: u64,
    pub original_id: u64,
    pub web_order_line_item_id: Option<String>,
    pub product_id: String,
    pub subscription_group_id: Option<String>,
    pub app_bundle_id: String,
    pub purchase_date: String,
    pub original_purchase_date: String,
    pub expiration_date: Option<String>,
    pub purchased_quantity: u64,
    pub is_upgraded: bool,
    pub ownership_type: OwnershipType,
    pub signed_date: String,
    pub jws_representation: String,
    pub verification_failure: Option<VerificationFailure>,
}

#[derive(Debug)]
pub struct Transaction {
    handle: NonNull<c_void>,
    data: TransactionData,
}

impl Clone for Transaction {
    fn clone(&self) -> Self {
        let retained = unsafe { ffi::sk_transaction_retain(self.handle.as_ptr()) };
        let handle = NonNull::new(retained).expect("StoreKit transaction retain returned null");
        Self {
            handle,
            data: self.data.clone(),
        }
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        unsafe { ffi::sk_transaction_release(self.handle.as_ptr()) };
    }
}

impl Transaction {
    pub fn current_entitlements() -> Result<TransactionStream, StoreKitError> {
        TransactionStream::new(ffi::stream_kind::CURRENT_ENTITLEMENTS)
    }

    pub fn all() -> Result<TransactionStream, StoreKitError> {
        TransactionStream::new(ffi::stream_kind::ALL)
    }

    pub fn updates() -> Result<TransactionStream, StoreKitError> {
        TransactionStream::new(ffi::stream_kind::UPDATES)
    }

    pub const fn data(&self) -> &TransactionData {
        &self.data
    }

    pub fn verify(&self) -> Result<(), StoreKitError> {
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::sk_transaction_verify(self.handle.as_ptr(), &mut error_message) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub fn finish(&self) -> Result<(), StoreKitError> {
        let mut error_message = ptr::null_mut();
        let status = unsafe { ffi::sk_transaction_finish(self.handle.as_ptr(), &mut error_message) };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { error_from_status(status, error_message) })
        }
    }

    pub(crate) fn from_raw_parts(
        handle: *mut c_void,
        payload: TransactionPayload,
    ) -> Result<Self, StoreKitError> {
        let handle = NonNull::new(handle).ok_or_else(|| {
            StoreKitError::Unknown("StoreKit returned a null transaction handle".to_owned())
        })?;
        Ok(Self {
            handle,
            data: payload.into_transaction_data(),
        })
    }

    pub(crate) fn into_verification_result(self) -> VerificationResult<Self> {
        if let Some(failure) = self.data.verification_failure.clone() {
            VerificationResult::Unverified(self, failure)
        } else {
            VerificationResult::Verified(self)
        }
    }
}

#[derive(Debug)]
pub struct TransactionStream {
    handle: NonNull<c_void>,
    finished: bool,
}

impl Drop for TransactionStream {
    fn drop(&mut self) {
        unsafe { ffi::sk_transaction_stream_release(self.handle.as_ptr()) };
    }
}

impl TransactionStream {
    fn new(kind: i32) -> Result<Self, StoreKitError> {
        let mut error_message = ptr::null_mut();
        let handle = unsafe { ffi::sk_transaction_stream_create(kind, &mut error_message) };
        let handle = NonNull::new(handle)
            .ok_or_else(|| unsafe { error_from_status(ffi::status::UNKNOWN, error_message) })?;
        Ok(Self {
            handle,
            finished: false,
        })
    }

    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<VerificationResult<Transaction>>, StoreKitError> {
        self.next_timeout(Duration::from_secs(30))
    }

    pub fn next_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<VerificationResult<Transaction>>, StoreKitError> {
        let mut transaction_handle = ptr::null_mut();
        let mut transaction_json = ptr::null_mut();
        let mut error_message = ptr::null_mut();
        let status = unsafe {
            ffi::sk_transaction_stream_next(
                self.handle.as_ptr(),
                duration_to_timeout_ms(timeout),
                &mut transaction_handle,
                &mut transaction_json,
                &mut error_message,
            )
        };

        match status {
            ffi::status::OK => {
                let payload = unsafe { parse_json_ptr::<TransactionPayload>(transaction_json, "transaction") };
                match payload {
                    Ok(payload) => {
                        let transaction = Transaction::from_raw_parts(transaction_handle, payload)?;
                        Ok(Some(transaction.into_verification_result()))
                    }
                    Err(error) => {
                        if !transaction_handle.is_null() {
                            unsafe { ffi::sk_transaction_release(transaction_handle) };
                        }
                        Err(error)
                    }
                }
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
pub(crate) struct TransactionPayload {
    id: u64,
    #[serde(rename = "originalID")]
    original_id: u64,
    #[serde(rename = "webOrderLineItemID")]
    web_order_line_item_id: Option<String>,
    #[serde(rename = "productID")]
    product_id: String,
    #[serde(rename = "subscriptionGroupID")]
    subscription_group_id: Option<String>,
    #[serde(rename = "appBundleID")]
    app_bundle_id: String,
    #[serde(rename = "purchaseDate")]
    purchase_date: String,
    #[serde(rename = "originalPurchaseDate")]
    original_purchase_date: String,
    #[serde(rename = "expirationDate")]
    expiration_date: Option<String>,
    #[serde(rename = "purchasedQuantity")]
    purchased_quantity: u64,
    #[serde(rename = "isUpgraded")]
    is_upgraded: bool,
    #[serde(rename = "ownershipType")]
    ownership_type: String,
    #[serde(rename = "signedDate")]
    signed_date: String,
    #[serde(rename = "jwsRepresentation")]
    jws_representation: String,
    #[serde(rename = "verificationError")]
    verification_error: Option<crate::error::VerificationErrorPayload>,
}

impl TransactionPayload {
    fn into_transaction_data(self) -> TransactionData {
        TransactionData {
            id: self.id,
            original_id: self.original_id,
            web_order_line_item_id: self.web_order_line_item_id,
            product_id: self.product_id,
            subscription_group_id: self.subscription_group_id,
            app_bundle_id: self.app_bundle_id,
            purchase_date: self.purchase_date,
            original_purchase_date: self.original_purchase_date,
            expiration_date: self.expiration_date,
            purchased_quantity: self.purchased_quantity,
            is_upgraded: self.is_upgraded,
            ownership_type: OwnershipType::from_raw(self.ownership_type),
            signed_date: self.signed_date,
            jws_representation: self.jws_representation,
            verification_failure: self
                .verification_error
                .map(crate::error::VerificationFailure::from_payload),
        }
    }
}
