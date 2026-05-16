use storekit::RenewalState;

#[test]
fn renewal_state_round_trips_raw_names() {
    let state = RenewalState::from_raw("expired");
    assert_eq!(state.as_str(), "expired");
}
