use storekit::Message;

fn main() {
    println!("messages supported on macOS: {}", Message::is_supported());
    println!("message stream result: {:?}", Message::messages());
}
