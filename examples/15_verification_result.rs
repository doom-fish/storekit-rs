use storekit::{
    VerificationErrorCode, VerificationFailure, VerificationMetadata, VerificationResult,
};

fn main() {
    let result = VerificationResult::Unverified {
        payload: "demo-payload",
        metadata: VerificationMetadata {
            jws_representation: "header.payload.signature".to_owned(),
            header_data: vec![1],
            payload_data: vec![2],
            signature_data: vec![3],
            signed_data: vec![4],
            signed_date: "2026-05-16T00:00:00Z".to_owned(),
            device_verification: vec![5],
            device_verification_nonce: "nonce".to_owned(),
        },
        failure: VerificationFailure {
            code: VerificationErrorCode::InvalidSignature,
            localized_description: "invalid signature".to_owned(),
        },
    };

    println!("verified: {}", result.is_verified());
    println!("payload: {:?}", result.payload());
    println!("jws: {}", result.jws_representation());
    println!("failure: {:?}", result.verification_failure());
}
