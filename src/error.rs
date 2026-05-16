use core::ffi::c_char;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

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

impl StoreKitError {
    pub fn typed(&self) -> Option<TypedStoreKitError> {
        match self {
            Self::Framework(error) => lookup_typed_framework_error(error),
            _ => None,
        }
    }

    pub fn storekit_api_error(&self) -> Option<StoreKitApiError> {
        match self.typed() {
            Some(TypedStoreKitError::StoreKit(error)) => Some(error),
            _ => None,
        }
    }

    pub fn product_purchase_error(&self) -> Option<ProductPurchaseError> {
        match self.typed() {
            Some(TypedStoreKitError::Purchase(error)) => Some(error),
            _ => None,
        }
    }

    pub fn refund_request_error(&self) -> Option<RefundRequestError> {
        match self.typed() {
            Some(TypedStoreKitError::RefundRequest(error)) => Some(error),
            _ => None,
        }
    }

    pub fn invalid_request_error(&self) -> Option<InvalidRequestError> {
        match self.typed() {
            Some(TypedStoreKitError::InvalidRequest(error)) => Some(error),
            _ => None,
        }
    }
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
pub enum TypedStoreKitError {
    StoreKit(StoreKitApiError),
    Purchase(ProductPurchaseError),
    RefundRequest(RefundRequestError),
    InvalidRequest(InvalidRequestError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreKitApiError {
    pub code: StoreKitApiErrorCode,
    pub error_description: Option<String>,
    pub failure_reason: Option<String>,
    pub recovery_suggestion: Option<String>,
    pub underlying_domain: Option<String>,
    pub underlying_code: Option<i64>,
    pub underlying_description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreKitApiErrorCode {
    Unknown,
    UserCancelled,
    NetworkError,
    SystemError,
    NotAvailableInStorefront,
    NotEntitled,
    Unsupported,
    Other(String),
}

impl StoreKitApiErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Unknown => "unknown",
            Self::UserCancelled => "userCancelled",
            Self::NetworkError => "networkError",
            Self::SystemError => "systemError",
            Self::NotAvailableInStorefront => "notAvailableInStorefront",
            Self::NotEntitled => "notEntitled",
            Self::Unsupported => "unsupported",
            Self::Other(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "unknown" => Self::Unknown,
            "userCancelled" => Self::UserCancelled,
            "networkError" => Self::NetworkError,
            "systemError" => Self::SystemError,
            "notAvailableInStorefront" => Self::NotAvailableInStorefront,
            "notEntitled" => Self::NotEntitled,
            "unsupported" => Self::Unsupported,
            _ => Self::Other(raw),
        }
    }

    const fn numeric_code(&self) -> i64 {
        match self {
            Self::Unknown => 0,
            Self::UserCancelled => 1,
            Self::NetworkError => 2,
            Self::SystemError => 3,
            Self::NotAvailableInStorefront => 4,
            Self::NotEntitled => 5,
            Self::Unsupported => 6,
            Self::Other(_) => -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductPurchaseError {
    pub code: ProductPurchaseErrorCode,
    pub error_description: Option<String>,
    pub failure_reason: Option<String>,
    pub recovery_suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductPurchaseErrorCode {
    InvalidQuantity,
    ProductUnavailable,
    PurchaseNotAllowed,
    IneligibleForOffer,
    InvalidOfferIdentifier,
    InvalidOfferPrice,
    InvalidOfferSignature,
    MissingOfferParameters,
    Other(String),
}

impl ProductPurchaseErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvalidQuantity => "invalidQuantity",
            Self::ProductUnavailable => "productUnavailable",
            Self::PurchaseNotAllowed => "purchaseNotAllowed",
            Self::IneligibleForOffer => "ineligibleForOffer",
            Self::InvalidOfferIdentifier => "invalidOfferIdentifier",
            Self::InvalidOfferPrice => "invalidOfferPrice",
            Self::InvalidOfferSignature => "invalidOfferSignature",
            Self::MissingOfferParameters => "missingOfferParameters",
            Self::Other(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "invalidQuantity" => Self::InvalidQuantity,
            "productUnavailable" => Self::ProductUnavailable,
            "purchaseNotAllowed" => Self::PurchaseNotAllowed,
            "ineligibleForOffer" => Self::IneligibleForOffer,
            "invalidOfferIdentifier" => Self::InvalidOfferIdentifier,
            "invalidOfferPrice" => Self::InvalidOfferPrice,
            "invalidOfferSignature" => Self::InvalidOfferSignature,
            "missingOfferParameters" => Self::MissingOfferParameters,
            _ => Self::Other(raw),
        }
    }

    const fn numeric_code(&self) -> i64 {
        match self {
            Self::InvalidQuantity => 0,
            Self::ProductUnavailable => 1,
            Self::PurchaseNotAllowed => 2,
            Self::IneligibleForOffer => 3,
            Self::InvalidOfferIdentifier => 4,
            Self::InvalidOfferPrice => 5,
            Self::InvalidOfferSignature => 6,
            Self::MissingOfferParameters => 7,
            Self::Other(_) => -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundRequestError {
    pub code: RefundRequestErrorCode,
    pub error_description: Option<String>,
    pub failure_reason: Option<String>,
    pub recovery_suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefundRequestErrorCode {
    DuplicateRequest,
    Failed,
    Other(String),
}

impl RefundRequestErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DuplicateRequest => "duplicateRequest",
            Self::Failed => "failed",
            Self::Other(value) => value.as_str(),
        }
    }

    fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "duplicateRequest" => Self::DuplicateRequest,
            "failed" => Self::Failed,
            _ => Self::Other(raw),
        }
    }

    const fn numeric_code(&self) -> i64 {
        match self {
            Self::DuplicateRequest => 0,
            Self::Failed => 1,
            Self::Other(_) => -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidRequestError {
    pub code: i64,
    pub message: String,
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
#[serde(tag = "kind")]
enum FrameworkErrorPayload {
    #[serde(rename = "framework")]
    Framework {
        domain: String,
        code: i64,
        #[serde(rename = "localizedDescription")]
        localized_description: String,
    },
    #[serde(rename = "storekitError")]
    StoreKit {
        code: String,
        #[serde(rename = "errorDescription")]
        error_description: Option<String>,
        #[serde(rename = "failureReason")]
        failure_reason: Option<String>,
        #[serde(rename = "recoverySuggestion")]
        recovery_suggestion: Option<String>,
        #[serde(rename = "underlyingDomain")]
        underlying_domain: Option<String>,
        #[serde(rename = "underlyingCode")]
        underlying_code: Option<i64>,
        #[serde(rename = "underlyingDescription")]
        underlying_description: Option<String>,
    },
    #[serde(rename = "purchaseError")]
    Purchase {
        code: String,
        #[serde(rename = "errorDescription")]
        error_description: Option<String>,
        #[serde(rename = "failureReason")]
        failure_reason: Option<String>,
        #[serde(rename = "recoverySuggestion")]
        recovery_suggestion: Option<String>,
    },
    #[serde(rename = "refundRequestError")]
    RefundRequest {
        code: String,
        #[serde(rename = "errorDescription")]
        error_description: Option<String>,
        #[serde(rename = "failureReason")]
        failure_reason: Option<String>,
        #[serde(rename = "recoverySuggestion")]
        recovery_suggestion: Option<String>,
    },
    #[serde(rename = "invalidRequestError")]
    InvalidRequest {
        code: i64,
        message: String,
    },
}

impl FrameworkErrorPayload {
    fn into_storekit_error(self) -> StoreKitError {
        match self {
            Self::Framework {
                domain,
                code,
                localized_description,
            } => StoreKitError::Framework(StoreKitFrameworkError {
                domain,
                code,
                localized_description,
            }),
            Self::StoreKit {
                code,
                error_description,
                failure_reason,
                recovery_suggestion,
                underlying_domain,
                underlying_code,
                underlying_description,
            } => typed_framework_error(
                "StoreKit.StoreKitError",
                StoreKitApiErrorCode::from_raw(code.clone()).numeric_code(),
                error_description
                    .clone()
                    .unwrap_or_else(|| StoreKitApiErrorCode::from_raw(code.clone()).as_str().to_owned()),
                TypedStoreKitError::StoreKit(StoreKitApiError {
                    code: StoreKitApiErrorCode::from_raw(code),
                    error_description,
                    failure_reason,
                    recovery_suggestion,
                    underlying_domain,
                    underlying_code,
                    underlying_description,
                }),
            ),
            Self::Purchase {
                code,
                error_description,
                failure_reason,
                recovery_suggestion,
            } => typed_framework_error(
                "StoreKit.Product.PurchaseError",
                ProductPurchaseErrorCode::from_raw(code.clone()).numeric_code(),
                error_description.clone().unwrap_or_else(|| {
                    ProductPurchaseErrorCode::from_raw(code.clone())
                        .as_str()
                        .to_owned()
                }),
                TypedStoreKitError::Purchase(ProductPurchaseError {
                    code: ProductPurchaseErrorCode::from_raw(code),
                    error_description,
                    failure_reason,
                    recovery_suggestion,
                }),
            ),
            Self::RefundRequest {
                code,
                error_description,
                failure_reason,
                recovery_suggestion,
            } => typed_framework_error(
                "StoreKit.Transaction.RefundRequestError",
                RefundRequestErrorCode::from_raw(code.clone()).numeric_code(),
                error_description.clone().unwrap_or_else(|| {
                    RefundRequestErrorCode::from_raw(code.clone())
                        .as_str()
                        .to_owned()
                }),
                TypedStoreKitError::RefundRequest(RefundRequestError {
                    code: RefundRequestErrorCode::from_raw(code),
                    error_description,
                    failure_reason,
                    recovery_suggestion,
                }),
            ),
            Self::InvalidRequest { code, message } => typed_framework_error(
                "StoreKit.InvalidRequestError",
                code,
                message.clone(),
                TypedStoreKitError::InvalidRequest(InvalidRequestError { code, message }),
            ),
        }
    }
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
            message
                .unwrap_or_else(|| format!("StoreKit bridge returned unexpected status {status}")),
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
                FrameworkErrorPayload::into_storekit_error,
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
                |payload| StoreKitError::Verification(VerificationFailure::from_payload(payload)),
            )
        },
    )
}

fn typed_framework_error(
    domain: &str,
    code: i64,
    localized_description: String,
    typed_error: TypedStoreKitError,
) -> StoreKitError {
    let framework_error = StoreKitFrameworkError {
        domain: domain.to_owned(),
        code,
        localized_description,
    };
    register_typed_framework_error(&framework_error, typed_error);
    StoreKitError::Framework(framework_error)
}

fn register_typed_framework_error(error: &StoreKitFrameworkError, typed_error: TypedStoreKitError) {
    typed_framework_error_registry()
        .lock()
        .expect("typed StoreKit error registry poisoned")
        .insert(framework_error_key(error), typed_error);
}

fn lookup_typed_framework_error(error: &StoreKitFrameworkError) -> Option<TypedStoreKitError> {
    typed_framework_error_registry()
        .lock()
        .expect("typed StoreKit error registry poisoned")
        .get(&framework_error_key(error))
        .cloned()
}

fn framework_error_key(error: &StoreKitFrameworkError) -> String {
    format!("{}\u{0}{}\u{0}", error.domain, error.code) + &error.localized_description
}

fn typed_framework_error_registry() -> &'static Mutex<HashMap<String, TypedStoreKitError>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, TypedStoreKitError>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_product_purchase_errors_into_typed_details() {
        let error = parse_framework_error(Some(
            r#"{"kind":"purchaseError","code":"invalidQuantity","errorDescription":"bad quantity","failureReason":"quantity must be positive","recoverySuggestion":"choose a quantity greater than zero"}"#
                .to_owned(),
        ));
        let typed = error
            .product_purchase_error()
            .expect("typed product purchase error");
        assert_eq!(typed.code, ProductPurchaseErrorCode::InvalidQuantity);
        assert_eq!(typed.error_description.as_deref(), Some("bad quantity"));
    }

    #[test]
    fn parses_storekit_errors_into_typed_details() {
        let error = parse_framework_error(Some(
            r#"{"kind":"storekitError","code":"unsupported","errorDescription":"unsupported","failureReason":"not available here","recoverySuggestion":"try a supported storefront"}"#
                .to_owned(),
        ));
        let typed = error.storekit_api_error().expect("typed StoreKit API error");
        assert_eq!(typed.code, StoreKitApiErrorCode::Unsupported);
        assert_eq!(typed.failure_reason.as_deref(), Some("not available here"));
    }

    #[test]
    fn parses_invalid_request_errors_into_typed_details() {
        let error = parse_framework_error(Some(
            r#"{"kind":"invalidRequestError","code":47,"message":"bad request"}"#
                .to_owned(),
        ));
        let typed = error
            .invalid_request_error()
            .expect("typed invalid request error");
        assert_eq!(typed.code, 47);
        assert_eq!(typed.message, "bad request");
    }
}
