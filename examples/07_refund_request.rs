use storekit::Refund;

fn main() {
    println!(
        "refund request result: {:?}",
        Refund::begin_for_transaction_id(0)
    );
}
