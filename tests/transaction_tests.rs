use storekit::{
    BillingPlanType, OfferPaymentMode, OfferType, OwnershipType, ProductType, RevocationType,
    TransactionCommitmentInfo, TransactionData, TransactionReason,
};

#[test]
fn transaction_enums_report_expected_raw_values() {
    assert_eq!(TransactionReason::Purchase.as_str(), "purchase");
    assert_eq!(OfferType::Code.as_str(), "code");
    assert_eq!(OfferPaymentMode::FreeTrial.as_str(), "freeTrial");
    assert_eq!(RevocationType::ProratedRefund.as_str(), "proratedRefund");
}

#[test]
fn transaction_commitment_fields_are_constructible() {
    let commitment_info = TransactionCommitmentInfo {
        billing_period_number: 1,
        total_billing_periods: 12,
        expiration_date: "2026-06-01T00:00:00Z".to_owned(),
        price: "9.99".to_owned(),
    };
    let data = TransactionData {
        id: 1,
        original_id: 1,
        web_order_line_item_id: None,
        product_id: "com.example.subscription.monthly".to_owned(),
        subscription_group_id: Some("com.example.subscription-group".to_owned()),
        app_bundle_id: "com.example.app".to_owned(),
        purchase_date: "2026-05-01T00:00:00Z".to_owned(),
        original_purchase_date: "2026-05-01T00:00:00Z".to_owned(),
        expiration_date: Some("2026-06-01T00:00:00Z".to_owned()),
        purchased_quantity: 1,
        is_upgraded: false,
        ownership_type: OwnershipType::Purchased,
        signed_date: "2026-05-01T00:00:00Z".to_owned(),
        jws_representation: "signed".to_owned(),
        verification_failure: None,
        revocation_date: None,
        revocation_reason: None,
        revocation_type: Some(RevocationType::FullRefund),
        product_type: Some(ProductType::AutoRenewable),
        app_account_token: None,
        environment: None,
        reason: Some(TransactionReason::Renewal),
        storefront: None,
        price: Some("9.99".to_owned()),
        currency_code: Some("USD".to_owned()),
        billing_plan_type: Some(BillingPlanType::Monthly),
        commitment_info: Some(commitment_info),
        app_transaction_id: None,
        offer: None,
        json_representation: vec![],
    };

    assert_eq!(
        data.revocation_type.as_ref().map(RevocationType::as_str),
        Some("fullRefund")
    );
    assert_eq!(
        data.billing_plan_type.as_ref().map(BillingPlanType::as_str),
        Some("monthly")
    );
    assert_eq!(
        data.commitment_info
            .as_ref()
            .expect("transaction commitment info should be present")
            .total_billing_periods,
        12
    );
}
