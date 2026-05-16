use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use storekit::ReceiptValidator;

#[test]
fn receipt_validator_decodes_jws_payloads() {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none"}"#);
    let payload = URL_SAFE_NO_PAD.encode(r#"{"hello":"world"}"#);
    let jws = format!("{header}.{payload}.signature");
    let json = ReceiptValidator::extract_unverified_payload(&jws).expect("valid JWS payload");
    assert_eq!(json["hello"], "world");
}
