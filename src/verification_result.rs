use serde::Deserialize;

use crate::error::{StoreKitError, VerificationErrorPayload, VerificationFailure};
use crate::private::decode_base64;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Carries metadata returned alongside `StoreKit.VerificationResult`.
pub struct VerificationMetadata {
    /// `StoreKit`-provided `jws_representation` value.
    pub jws_representation: String,
    /// Decoded JWS header bytes returned by `StoreKit`.
    pub header_data: Vec<u8>,
    /// Decoded JWS payload bytes returned by `StoreKit`.
    pub payload_data: Vec<u8>,
    /// Decoded JWS signature bytes returned by `StoreKit`.
    pub signature_data: Vec<u8>,
    /// Decoded signed-data bytes returned by `StoreKit`.
    pub signed_data: Vec<u8>,
    /// Signature timestamp reported by `StoreKit`.
    pub signed_date: String,
    /// Decoded device-verification bytes returned by `StoreKit`.
    pub device_verification: Vec<u8>,
    /// Device verification nonce reported by `StoreKit`.
    pub device_verification_nonce: String,
}

impl VerificationMetadata {
    /// Returns the compact JWS representation returned by `StoreKit`.
    pub fn jws_representation(&self) -> &str {
        &self.jws_representation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Wraps `StoreKit.VerificationResult`.
pub enum VerificationResult<T> {
    /// `StoreKit` verified the payload signature.
    Verified {
        /// Decoded payload returned by `StoreKit`.
        payload: T,
        /// Verification metadata returned by `StoreKit`.
        metadata: VerificationMetadata,
    },
    /// `StoreKit` returned the payload but verification failed.
    Unverified {
        /// Decoded payload returned by `StoreKit`.
        payload: T,
        /// Verification metadata returned by `StoreKit`.
        metadata: VerificationMetadata,
        /// Verification failure returned by `StoreKit`.
        failure: VerificationFailure,
    },
}

impl<T> VerificationResult<T> {
    /// Returns `true` when `StoreKit` verified the payload.
    pub const fn is_verified(&self) -> bool {
        matches!(self, Self::Verified { .. })
    }

    /// Returns the decoded payload returned by `StoreKit`.
    pub const fn payload(&self) -> &T {
        match self {
            Self::Verified { payload, .. } | Self::Unverified { payload, .. } => payload,
        }
    }

    /// Consumes the wrapper and returns the decoded `StoreKit` payload.
    pub fn into_payload(self) -> T {
        match self {
            Self::Verified { payload, .. } | Self::Unverified { payload, .. } => payload,
        }
    }

    /// Returns the verification metadata returned by `StoreKit`.
    pub const fn metadata(&self) -> &VerificationMetadata {
        match self {
            Self::Verified { metadata, .. } | Self::Unverified { metadata, .. } => metadata,
        }
    }

    /// Returns the verification failure reported by `StoreKit`, if any.
    pub const fn verification_failure(&self) -> Option<&VerificationFailure> {
        match self {
            Self::Verified { .. } => None,
            Self::Unverified { failure, .. } => Some(failure),
        }
    }

    /// Consumes the wrapper and returns the payload, metadata, and optional verification failure.
    pub fn into_parts(self) -> (T, VerificationMetadata, Option<VerificationFailure>) {
        match self {
            Self::Verified { payload, metadata } => (payload, metadata, None),
            Self::Unverified {
                payload,
                metadata,
                failure,
            } => (payload, metadata, Some(failure)),
        }
    }

    /// Returns the compact JWS representation returned by `StoreKit`.
    pub fn jws_representation(&self) -> &str {
        self.metadata().jws_representation()
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct VerificationMetadataPayload {
    #[serde(rename = "jwsRepresentation")]
    jws_representation: String,
    #[serde(rename = "headerDataBase64")]
    header_data_base64: String,
    #[serde(rename = "payloadDataBase64")]
    payload_data_base64: String,
    #[serde(rename = "signatureDataBase64")]
    signature_data_base64: String,
    #[serde(rename = "signedDataBase64")]
    signed_data_base64: String,
    #[serde(rename = "signedDate")]
    signed_date: String,
    #[serde(rename = "deviceVerificationBase64")]
    device_verification_base64: String,
    #[serde(rename = "deviceVerificationNonce")]
    device_verification_nonce: String,
}

impl VerificationMetadataPayload {
    fn into_metadata(self) -> Result<VerificationMetadata, StoreKitError> {
        Ok(VerificationMetadata {
            jws_representation: self.jws_representation,
            header_data: decode_base64(&self.header_data_base64, "verification header data")?,
            payload_data: decode_base64(&self.payload_data_base64, "verification payload data")?,
            signature_data: decode_base64(
                &self.signature_data_base64,
                "verification signature data",
            )?,
            signed_data: decode_base64(&self.signed_data_base64, "verification signed data")?,
            signed_date: self.signed_date,
            device_verification: decode_base64(
                &self.device_verification_base64,
                "verification device data",
            )?,
            device_verification_nonce: self.device_verification_nonce,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct VerificationResultPayload<T> {
    kind: String,
    payload: T,
    metadata: VerificationMetadataPayload,
    #[serde(rename = "verificationError")]
    verification_error: Option<VerificationErrorPayload>,
}

impl<T> VerificationResultPayload<T> {
    pub(crate) fn into_result<U, F>(
        self,
        map_payload: F,
    ) -> Result<VerificationResult<U>, StoreKitError>
    where
        F: FnOnce(T) -> Result<U, StoreKitError>,
    {
        let payload = map_payload(self.payload)?;
        let metadata = self.metadata.into_metadata()?;
        match self.kind.as_str() {
            "verified" => Ok(VerificationResult::Verified { payload, metadata }),
            "unverified" => {
                let failure = self
                    .verification_error
                    .map(VerificationFailure::from_payload)
                    .ok_or_else(|| {
                        StoreKitError::Unknown(
                            "StoreKit returned an unverified result without a verification error"
                                .to_owned(),
                        )
                    })?;
                Ok(VerificationResult::Unverified {
                    payload,
                    metadata,
                    failure,
                })
            }
            other => Err(StoreKitError::Unknown(format!(
                "StoreKit returned an unknown verification result kind '{other}'"
            ))),
        }
    }
}
