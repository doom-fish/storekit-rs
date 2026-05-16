use storekit::SubscriptionInfo;

fn main() {
    println!(
        "intro offer eligibility: {:?}",
        SubscriptionInfo::is_eligible_for_intro_offer_for("com.example.subscription-group")
    );
    println!(
        "subscription statuses: {:?}",
        SubscriptionInfo::status_for("com.example.subscription-group")
    );
}
