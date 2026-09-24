pub fn live_tests_enabled(test: &str) -> bool {
    let enabled = std::env::var("STOREKIT_LIVE_TESTS").as_deref() == Ok("1");
    if !enabled {
        eprintln!("{test}: skipped; set STOREKIT_LIVE_TESTS=1 to query StoreKit");
    }
    enabled
}
