use storekit::{
    BillingPlanType, ExpirationReason, PriceIncreaseStatus, RenewalCommitmentInfo, RenewalInfo,
};

#[test]
fn renewal_info_enums_have_expected_names() {
    assert_eq!(ExpirationReason::BillingError.as_str(), "billingError");
    assert_eq!(PriceIncreaseStatus::Pending.as_str(), "pending");
}

#[test]
fn renewal_commitment_info_types_are_constructible() {
    let commitment_info = RenewalCommitmentInfo {
        auto_renew_preference: "com.example.subscription.annual".to_owned(),
        renewal_billing_plan_type: BillingPlanType::UpFront,
        renewal_date: "2026-06-01T00:00:00Z".to_owned(),
        renewal_price: "59.99".to_owned(),
        will_auto_renew: true,
    };
    let info = RenewalInfo {
        original_transaction_id: 1,
        current_product_id: "com.example.subscription.monthly".to_owned(),
        will_auto_renew: true,
        auto_renew_preference: Some("com.example.subscription.annual".to_owned()),
        expiration_reason: Some(ExpirationReason::BillingError),
        price_increase_status: PriceIncreaseStatus::Pending,
        is_in_billing_retry: false,
        grace_period_expiration_date: None,
        offer: None,
        environment: None,
        recent_subscription_start_date: "2026-05-01T00:00:00Z".to_owned(),
        renewal_date: Some(commitment_info.renewal_date.clone()),
        renewal_price: Some(commitment_info.renewal_price.clone()),
        commitment_info: Some(commitment_info),
        renewal_billing_plan_type: Some(BillingPlanType::UpFront),
        currency_code: Some("USD".to_owned()),
        eligible_win_back_offer_ids: vec![],
        app_account_token: None,
        app_transaction_id: None,
    };

    let stored_commitment_info = info
        .commitment_info
        .as_ref()
        .expect("renewal commitment info should be present");
    assert!(stored_commitment_info.will_auto_renew);
    assert_eq!(
        info.renewal_billing_plan_type
            .as_ref()
            .map(BillingPlanType::as_str),
        Some("upFront")
    );
}
