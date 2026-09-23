use storekit::{
    StoreKitError, VerificationErrorCode, VerificationFailure, VerificationMetadata,
    VerificationResult,
};

fn metadata() -> VerificationMetadata {
    VerificationMetadata {
        jws_representation: "header.payload.signature".to_owned(),
        header_data: vec![1],
        payload_data: vec![2],
        signature_data: vec![3],
        signed_data: vec![4],
        signed_date: "2026-05-16T00:00:00Z".to_owned(),
        device_verification: vec![5],
        device_verification_nonce: "nonce".to_owned(),
    }
}

fn failure() -> VerificationFailure {
    VerificationFailure {
        code: VerificationErrorCode::InvalidSignature,
        localized_description: "invalid signature".to_owned(),
    }
}

#[test]
fn verification_result_helpers_work() {
    let result = VerificationResult::Unverified {
        payload: 7_u8,
        metadata: metadata(),
        failure: failure(),
    };
    assert!(!result.is_verified());
    assert_eq!(*result.unverified_payload(), 7);
    assert_eq!(result.jws_representation(), "header.payload.signature");
    assert!(result.verification_failure().is_some());
}

#[test]
fn checked_payload_accessors_refuse_unverified_results() {
    let result = VerificationResult::Unverified {
        payload: "tampered",
        metadata: metadata(),
        failure: failure(),
    };
    match result.payload_value() {
        Err(StoreKitError::Verification(reported)) => assert_eq!(reported, failure()),
        other => panic!("expected a verification error, got {other:?}"),
    }
    match result.into_payload_value() {
        Err(StoreKitError::Verification(reported)) => {
            assert_eq!(reported.code, VerificationErrorCode::InvalidSignature);
        }
        other => panic!("expected a verification error, got {other:?}"),
    }
}

#[test]
fn checked_payload_accessors_return_verified_payloads() {
    let result = VerificationResult::Verified {
        payload: "genuine",
        metadata: metadata(),
    };
    assert_eq!(result.payload_value().ok(), Some(&"genuine"));
    assert_eq!(*result.unverified_payload(), "genuine");
    assert_eq!(result.into_payload_value().ok(), Some("genuine"));
}
