use storekit::Storefront;

#[test]
fn storefront_query_is_safe() {
    match Storefront::current() {
        Ok(Some(storefront)) => assert!(!storefront.id.is_empty()),
        Ok(None) => {}
        Err(error) => assert!(!error.to_string().is_empty()),
    }
}
