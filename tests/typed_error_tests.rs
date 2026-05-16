use storekit::{
    InvalidRequestError, ProductPurchaseErrorCode, RefundRequestErrorCode, StoreKitApiErrorCode,
    StoreKitError, StoreKitFrameworkError, TypedStoreKitError,
};

#[test]
fn typed_error_helpers_are_constructible() {
    assert_eq!(StoreKitApiErrorCode::Unsupported.as_str(), "unsupported");
    assert_eq!(ProductPurchaseErrorCode::InvalidQuantity.as_str(), "invalidQuantity");
    assert_eq!(RefundRequestErrorCode::Failed.as_str(), "failed");

    let typed = TypedStoreKitError::InvalidRequest(InvalidRequestError {
        code: 17,
        message: "invalid request".to_owned(),
    });
    match typed {
        TypedStoreKitError::InvalidRequest(error) => assert_eq!(error.code, 17),
        _ => unreachable!("expected invalid-request typed error"),
    }

    let framework = StoreKitError::Framework(StoreKitFrameworkError {
        domain: "StoreKit".to_owned(),
        code: -4,
        localized_description: "framework failure".to_owned(),
    });
    assert!(framework.typed().is_none());
}
