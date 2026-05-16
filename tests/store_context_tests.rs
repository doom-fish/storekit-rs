use storekit::StoreContext;

#[test]
fn store_context_current_returns_environment_info() {
    let context = StoreContext::current().expect("store context should be readable");
    assert!(context.bundle_name.is_some() || context.executable_path.is_some());
}
