use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use storekit::ReceiptValidator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("current receipt: {:?}", ReceiptValidator::current_receipt());
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none"}"#);
    let payload = URL_SAFE_NO_PAD.encode(r#"{"hello":"world"}"#);
    let jws = format!("{header}.{payload}.signature");
    println!(
        "decoded payload: {:?}",
        ReceiptValidator::extract_unverified_payload(&jws)?
    );
    Ok(())
}
