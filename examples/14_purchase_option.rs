use storekit::PurchaseOption;

fn main() {
    let options = [
        PurchaseOption::Quantity { quantity: 1 },
        PurchaseOption::SimulatesAskToBuyInSandbox {
            simulate_ask_to_buy_in_sandbox: true,
        },
        PurchaseOption::CustomString {
            key: "source".to_owned(),
            value: "example".to_owned(),
        },
    ];
    println!("purchase options: {options:#?}");
}
