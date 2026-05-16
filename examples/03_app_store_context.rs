use storekit::AppStore;

fn main() {
    println!("can make payments: {:?}", AppStore::can_make_payments());
    println!(
        "device verification id: {:?}",
        AppStore::device_verification_id()
    );
    println!("request review: {:?}", AppStore::request_review());
}
