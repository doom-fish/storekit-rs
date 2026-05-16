use storekit::Message;

#[test]
fn messages_are_marked_unsupported_on_macos() {
    assert!(!Message::is_supported());
    assert!(Message::messages().is_err());
}
