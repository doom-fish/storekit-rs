use storekit::{
    VerificationErrorCode, VerificationFailure, VerificationMetadata, VerificationResult,
};

#[test]
fn verification_result_helpers_work() {
    let metadata = VerificationMetadata {
        jws_representation: "header.payload.signature".to_owned(),
        header_data: vec![1],
        payload_data: vec![2],
        signature_data: vec![3],
        signed_data: vec![4],
        signed_date: "2026-05-16T00:00:00Z".to_owned(),
        device_verification: vec![5],
        device_verification_nonce: "nonce".to_owned(),
    };
    let failure = VerificationFailure {
        code: VerificationErrorCode::InvalidSignature,
        localized_description: "invalid signature".to_owned(),
    };
    let result = VerificationResult::Unverified {
        payload: 7_u8,
        metadata,
        failure,
    };
    assert!(!result.is_verified());
    assert_eq!(*result.payload(), 7);
    assert_eq!(result.jws_representation(), "header.payload.signature");
    assert!(result.verification_failure().is_some());
}
