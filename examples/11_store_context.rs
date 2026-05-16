use storekit::StoreContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = StoreContext::current()?;
    println!("store context: {context:#?}");
    Ok(())
}
