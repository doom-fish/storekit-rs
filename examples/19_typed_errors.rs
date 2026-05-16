use storekit::{
    InvalidRequestError, ProductPurchaseErrorCode, RefundRequestErrorCode, StoreKitApiErrorCode,
    TypedStoreKitError,
};

fn main() {
    println!("StoreKit API code: {}", StoreKitApiErrorCode::Unsupported.as_str());
    println!(
        "Purchase error code: {}",
        ProductPurchaseErrorCode::InvalidQuantity.as_str()
    );
    println!(
        "Refund error code: {}",
        RefundRequestErrorCode::Failed.as_str()
    );
    println!(
        "Typed error sample: {:?}",
        TypedStoreKitError::InvalidRequest(InvalidRequestError {
            code: 17,
            message: "invalid request".to_owned(),
        })
    );
}
