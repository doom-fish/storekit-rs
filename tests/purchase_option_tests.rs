use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::json;
use storekit::{BillingPlanType, PurchaseOption};

#[test]
fn purchase_options_serialize_with_expected_tags() {
    let option = PurchaseOption::CustomString {
        key: "source".to_owned(),
        value: "tests".to_owned(),
    };
    let json = serde_json::to_string(&option).expect("purchase option should serialize");
    assert!(json.contains("customString"));
    assert!(json.contains("source"));
}

#[test]
fn billing_plan_type_purchase_option_serializes_raw_value() {
    let option = PurchaseOption::BillingPlanType {
        billing_plan_type: BillingPlanType::UpFront,
    };
    let json = serde_json::to_value(&option).expect("billing plan type should serialize");
    assert_eq!(json["kind"], "billingPlanType");
    assert_eq!(json["billingPlanType"], "upFront");
}

fn samples() -> Vec<(PurchaseOption, serde_json::Value)> {
    vec![
        (
            PurchaseOption::AppAccountToken {
                app_account_token: "8C2F59B0-5C6E-4D4F-9F55-0D3D6F1F7A11".to_owned(),
            },
            json!({"kind": "appAccountToken", "appAccountToken": "8C2F59B0-5C6E-4D4F-9F55-0D3D6F1F7A11"}),
        ),
        (
            PurchaseOption::BillingPlanType {
                billing_plan_type: BillingPlanType::Monthly,
            },
            json!({"kind": "billingPlanType", "billingPlanType": "monthly"}),
        ),
        (
            PurchaseOption::Quantity { quantity: 3 },
            json!({"kind": "quantity", "quantity": 3}),
        ),
        (
            PurchaseOption::SimulatesAskToBuyInSandbox {
                simulate_ask_to_buy_in_sandbox: true,
            },
            json!({"kind": "simulatesAskToBuyInSandbox", "simulateAskToBuyInSandbox": true}),
        ),
        (
            PurchaseOption::CustomString {
                key: "source".to_owned(),
                value: "tests".to_owned(),
            },
            json!({"kind": "customString", "key": "source", "value": "tests"}),
        ),
        (
            PurchaseOption::CustomNumber {
                key: "score".to_owned(),
                value: 1.5,
            },
            json!({"kind": "customNumber", "key": "score", "doubleValue": 1.5}),
        ),
        (
            PurchaseOption::CustomBool {
                key: "trial".to_owned(),
                value: true,
            },
            json!({"kind": "customBool", "key": "trial", "boolValue": true}),
        ),
        (
            PurchaseOption::CustomData {
                key: "blob".to_owned(),
                value_base64: "AQID".to_owned(),
            },
            json!({"kind": "customData", "key": "blob", "valueBase64": "AQID"}),
        ),
        (
            PurchaseOption::PromotionalOfferSignature {
                offer_id: "promo".to_owned(),
                key_id: "KEY123".to_owned(),
                nonce: "4E5F6A7B-8C9D-4E0F-A1B2-C3D4E5F6A7B8".to_owned(),
                signature_base64: "c2ln".to_owned(),
                timestamp: 1_700_000_000_000,
            },
            json!({
                "kind": "promotionalOfferSignature",
                "offerId": "promo",
                "keyId": "KEY123",
                "nonce": "4E5F6A7B-8C9D-4E0F-A1B2-C3D4E5F6A7B8",
                "signatureBase64": "c2ln",
                "timestamp": 1_700_000_000_000_i64,
            }),
        ),
        (
            PurchaseOption::PromotionalOfferCompactJws {
                offer_id: "promo".to_owned(),
                compact_jws: "header.payload.signature".to_owned(),
            },
            json!({"kind": "promotionalOfferCompactJws", "offerId": "promo", "compactJws": "header.payload.signature"}),
        ),
        (
            PurchaseOption::IntroductoryOfferEligibility {
                compact_jws: "header.payload.signature".to_owned(),
            },
            json!({"kind": "introductoryOfferEligibility", "compactJws": "header.payload.signature"}),
        ),
        (
            PurchaseOption::WinBackOffer {
                offer_id: "winback".to_owned(),
            },
            json!({"kind": "winBackOffer", "offerId": "winback"}),
        ),
        (
            PurchaseOption::OnStorefrontChange {
                should_continue_purchase: false,
            },
            json!({"kind": "onStorefrontChange", "shouldContinuePurchase": false}),
        ),
    ]
}

const fn variant_is_sampled(option: &PurchaseOption) -> bool {
    match option {
        PurchaseOption::AppAccountToken { .. }
        | PurchaseOption::BillingPlanType { .. }
        | PurchaseOption::Quantity { .. }
        | PurchaseOption::SimulatesAskToBuyInSandbox { .. }
        | PurchaseOption::CustomString { .. }
        | PurchaseOption::CustomNumber { .. }
        | PurchaseOption::CustomBool { .. }
        | PurchaseOption::CustomData { .. }
        | PurchaseOption::PromotionalOfferSignature { .. }
        | PurchaseOption::PromotionalOfferCompactJws { .. }
        | PurchaseOption::IntroductoryOfferEligibility { .. }
        | PurchaseOption::WinBackOffer { .. }
        | PurchaseOption::OnStorefrontChange { .. } => true,
    }
}

#[test]
fn every_purchase_option_round_trips_through_the_bridge_json() {
    let samples = samples();
    assert_eq!(samples.len(), 13);
    for (option, expected) in samples {
        assert!(variant_is_sampled(&option));
        let encoded = serde_json::to_value(&option).expect("purchase option serializes");
        assert_eq!(encoded, expected, "{option:?}");
        let decoded: PurchaseOption =
            serde_json::from_value(expected).expect("bridge JSON deserializes");
        assert_eq!(decoded, option);
    }
}

#[test]
fn boolean_options_keep_their_value() {
    for value in [false, true] {
        let ask_to_buy = serde_json::to_value(PurchaseOption::SimulatesAskToBuyInSandbox {
            simulate_ask_to_buy_in_sandbox: value,
        })
        .expect("serializes");
        assert_eq!(ask_to_buy["simulateAskToBuyInSandbox"], value);

        let storefront = serde_json::to_value(PurchaseOption::OnStorefrontChange {
            should_continue_purchase: value,
        })
        .expect("serializes");
        assert_eq!(storefront["shouldContinuePurchase"], value);
    }
}

fn swift_purchase_option_source() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("swift-bridge/Sources/StoreKitBridge/PurchaseOption.swift");
    fs::read_to_string(path).expect("PurchaseOption.swift is readable")
}

fn swift_payload_fields(source: &str) -> BTreeMap<String, String> {
    let start = source
        .find("struct SKPurchaseOptionPayload")
        .expect("SKPurchaseOptionPayload is declared");
    let body = &source[start..];
    let body = &body[..body.find("\n}").expect("SKPurchaseOptionPayload ends")];
    body.lines()
        .filter_map(|line| {
            let declaration = line.trim().strip_prefix("let ")?;
            let (name, swift_type) = declaration.split_once(':')?;
            Some((
                name.trim().to_owned(),
                swift_type.trim().trim_end_matches('?').to_owned(),
            ))
        })
        .collect()
}

fn swift_option_kinds(source: &str) -> BTreeSet<String> {
    let start = source
        .find("func skBuildPurchaseOptions")
        .expect("skBuildPurchaseOptions is declared");
    source[start..]
        .lines()
        .filter_map(|line| {
            let label = line.trim().strip_prefix("case \"")?;
            Some(label[..label.find('"')?].to_owned())
        })
        .collect()
}

fn json_matches_swift_type(value: &serde_json::Value, swift_type: &str) -> bool {
    match swift_type {
        "String" => value.is_string(),
        "Int" => value.is_i64(),
        "Double" => value.is_number(),
        "Bool" => value.is_boolean(),
        _ => false,
    }
}

#[test]
fn purchase_option_json_matches_the_swift_decoder() {
    let source = swift_purchase_option_source();
    let fields = swift_payload_fields(&source);
    let kinds = swift_option_kinds(&source);
    assert_eq!(fields.get("kind").map(String::as_str), Some("String"));

    let mut sampled_kinds = BTreeSet::new();
    for (option, _) in samples() {
        let encoded = serde_json::to_value(&option).expect("purchase option serializes");
        let object = encoded
            .as_object()
            .expect("purchase options encode as objects");
        for (key, value) in object {
            let swift_type = fields
                .get(key)
                .unwrap_or_else(|| panic!("{option:?}: Swift does not decode `{key}`"));
            assert!(
                json_matches_swift_type(value, swift_type),
                "{option:?}: `{key}` is {value} but Swift decodes {swift_type}"
            );
        }
        sampled_kinds.insert(object["kind"].as_str().expect("kind").to_owned());
    }
    assert_eq!(sampled_kinds, kinds);
}
