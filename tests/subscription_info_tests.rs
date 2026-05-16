use storekit::SubscriptionInfo;

#[test]
fn subscription_info_helpers_are_callable() {
    let eligibility =
        SubscriptionInfo::is_eligible_for_intro_offer_for("com.example.subscription-group");
    match eligibility {
        Ok(_) => {}
        Err(error) => assert!(!error.to_string().is_empty()),
    }
}
