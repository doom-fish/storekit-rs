use core::ffi::c_char;
use std::fmt;

use serde::Deserialize;

use crate::ffi;
use crate::private::take_string;

#[derive(Debug, Clone)]
pub enum StoreKitError {
    InvalidArgument(String),
    TimedOut(String),
    NotSupported(String),
    Framework(StoreKitFrameworkError),
    Verification(VerificationFailure),
    Unknown(String),
}

impl fmt::Display for StoreKitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message)
            | Self::TimedOut(message)
            | Self::NotSupported(message)
            | Self::Unknown(message) => formatter.write_str(message),
            Self::Framework(error) => write!(
                formatter,
                "{} (domain={}, code={})",
                error.localized_description, error.domain, error.code
            ),
            Self::Verification(error) => write!(
                formatter,
                "{} ({})",
                error.localized_description,
                error.code.as_str()
            ),
        }
    }
}

impl std::error::Error for StoreKitError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreKitFrameworkError {
    pub domain: String,
    pub code: i64,
    pub localized_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationFailure {
    pub code: VerificationErrorCode,
    pub localized_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationErrorCode {
    RevokedCertificate,
    InvalidCertificateChain,
    InvalidDeviceVerification,
    InvalidEncoding,
    InvalidSignature,
    MissingRequiredProperties,
    Unknown(String),
}

impl VerificationErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::RevokedCertificate => "revokedCertificate",
            Self::InvalidCertificateChain => "invalidCertificateChain",
            Self::InvalidDeviceVerification => "invalidDeviceVerification",
            Self::InvalidEncoding => "invalidEncoding",
            Self::InvalidSignature => "invalidSignature",
            Self::MissingRequiredProperties => "missingRequiredProperties",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub(crate) fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "revokedCertificate" => Self::RevokedCertificate,
            "invalidCertificateChain" => Self::InvalidCertificateChain,
            "invalidDeviceVerification" => Self::InvalidDeviceVerification,
            "invalidEncoding" => Self::InvalidEncoding,
            "invalidSignature" => Self::InvalidSignature,
            "missingRequiredProperties" => Self::MissingRequiredProperties,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct VerificationErrorPayload {
    #[allow(dead_code)]
    kind: String,
    code: String,
    #[serde(rename = "localizedDescription")]
    localized_description: String,
}

impl VerificationFailure {
    pub(crate) fn from_payload(payload: VerificationErrorPayload) -> Self {
        Self {
            code: VerificationErrorCode::from_raw(payload.code),
            localized_description: payload.localized_description,
        }
    }
}

#[derive(Debug, Deserialize)]
struct FrameworkErrorPayload {
    #[allow(dead_code)]
    kind: String,
    domain: String,
    code: i64,
    #[serde(rename = "localizedDescription")]
    localized_description: String,
}

pub(crate) unsafe fn from_swift(status: i32, err_msg: *mut c_char) -> StoreKitError {
    let message = take_string(err_msg);
    match status {
        ffi::status::INVALID_ARGUMENT => StoreKitError::InvalidArgument(
            message.unwrap_or_else(|| "StoreKit reported an invalid argument".to_owned()),
        ),
        ffi::status::TIMED_OUT => StoreKitError::TimedOut(
            message.unwrap_or_else(|| "StoreKit operation timed out".to_owned()),
        ),
        ffi::status::NOT_SUPPORTED => StoreKitError::NotSupported(
            message.unwrap_or_else(|| "StoreKit operation is not supported".to_owned()),
        ),
        ffi::status::FRAMEWORK_ERROR => parse_framework_error(message),
        ffi::status::VERIFICATION_ERROR => parse_verification_error(message),
        _ => StoreKitError::Unknown(
            message.unwrap_or_else(|| format!("StoreKit bridge returned unexpected status {status}")),
        ),
    }
}

fn parse_framework_error(message: Option<String>) -> StoreKitError {
    message.map_or_else(
        || {
            StoreKitError::Framework(StoreKitFrameworkError {
                domain: "StoreKit".to_owned(),
                code: i64::from(ffi::status::FRAMEWORK_ERROR),
                localized_description: "StoreKit framework error".to_owned(),
            })
        },
        |json| {
            serde_json::from_str::<FrameworkErrorPayload>(&json).map_or_else(
                |_| {
                    StoreKitError::Framework(StoreKitFrameworkError {
                        domain: "StoreKit".to_owned(),
                        code: i64::from(ffi::status::FRAMEWORK_ERROR),
                        localized_description: json,
                    })
                },
                |payload| {
                    StoreKitError::Framework(StoreKitFrameworkError {
                        domain: payload.domain,
                        code: payload.code,
                        localized_description: payload.localized_description,
                    })
                },
            )
        },
    )
}

fn parse_verification_error(message: Option<String>) -> StoreKitError {
    message.map_or_else(
        || {
            StoreKitError::Verification(VerificationFailure {
                code: VerificationErrorCode::Unknown("unknown".to_owned()),
                localized_description: "StoreKit verification failed".to_owned(),
            })
        },
        |json| {
            serde_json::from_str::<VerificationErrorPayload>(&json).map_or_else(
                |_| {
                    StoreKitError::Verification(VerificationFailure {
                        code: VerificationErrorCode::Unknown("unknown".to_owned()),
                        localized_description: json,
                    })
                },
                |payload| {
                    StoreKitError::Verification(VerificationFailure::from_payload(payload))
                },
            )
        },
    )
}
