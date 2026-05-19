use storekit::{
    BillingPlanType, SubscriptionCommitmentInfo, SubscriptionInfo, SubscriptionOffer,
    SubscriptionOfferType, SubscriptionPaymentMode, SubscriptionPeriod, SubscriptionPeriodUnit,
    SubscriptionPricingTerms,
};

#[test]
fn subscription_info_helpers_are_callable() {
    let eligibility =
        SubscriptionInfo::is_eligible_for_intro_offer_for("com.example.subscription-group");
    match eligibility {
        Ok(_) => {}
        Err(error) => assert!(!error.to_string().is_empty()),
    }
}

#[test]
fn subscription_pricing_terms_types_are_constructible() {
    let monthly_period = SubscriptionPeriod {
        unit: SubscriptionPeriodUnit::Month,
        value: 1,
    };
    let offer = SubscriptionOffer {
        id: Some("intro".to_owned()),
        offer_type: SubscriptionOfferType::Introductory,
        price: "4.99".to_owned(),
        display_price: "$4.99".to_owned(),
        period: monthly_period.clone(),
        period_count: 1,
        payment_mode: SubscriptionPaymentMode::PayAsYouGo,
    };
    let commitment_info = SubscriptionCommitmentInfo {
        price: "59.99".to_owned(),
        display_price: "$59.99".to_owned(),
        period: SubscriptionPeriod {
            unit: SubscriptionPeriodUnit::Year,
            value: 1,
        },
    };
    let pricing_terms = SubscriptionPricingTerms {
        billing_price: "9.99".to_owned(),
        billing_display_price: "$9.99".to_owned(),
        billing_period: monthly_period.clone(),
        billing_plan_type: BillingPlanType::Monthly,
        commitment_info,
        subscription_offers: vec![offer.clone()],
    };
    let info = SubscriptionInfo {
        introductory_offer: Some(offer.clone()),
        promotional_offers: vec![offer.clone()],
        win_back_offers: vec![offer],
        subscription_group_id: "com.example.subscription-group".to_owned(),
        subscription_period: monthly_period,
        pricing_terms: vec![pricing_terms],
        group_level: Some(1),
        group_display_name: Some("Pro".to_owned()),
    };

    assert_eq!(info.pricing_terms[0].billing_plan_type.as_str(), "monthly");
    assert_eq!(
        info.pricing_terms[0].commitment_info.display_price,
        "$59.99"
    );
    assert_eq!(
        info.pricing_terms[0].subscription_offers[0]
            .offer_type
            .as_str(),
        "introductory"
    );
}
