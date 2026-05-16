use storekit::AppTransaction;

fn main() {
    println!("shared app transaction: {:?}", AppTransaction::shared());
    println!("refreshed app transaction: {:?}", AppTransaction::refresh());
}
