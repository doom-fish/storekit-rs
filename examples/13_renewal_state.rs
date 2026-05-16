use storekit::RenewalState;

fn main() {
    let state = RenewalState::from_raw("inGracePeriod");
    println!("renewal state: {}", state.as_str());
}
