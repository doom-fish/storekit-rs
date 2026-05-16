use storekit::{ExternalPurchase, ExternalPurchaseCustomLink, ExternalPurchaseLink};

fn main() {
    println!("external purchase notice available: {:?}", ExternalPurchase::can_present());
    println!("external purchase link can open: {:?}", ExternalPurchaseLink::can_open());
    println!(
        "external purchase custom link eligible: {:?}",
        ExternalPurchaseCustomLink::is_eligible()
    );
}
