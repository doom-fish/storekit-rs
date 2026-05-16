import Foundation
import StoreKit

struct SKPurchaseOptionPayload: Codable {
    let kind: String
    let appAccountToken: String?
    let quantity: Int?
    let simulateAskToBuyInSandbox: Bool?
    let key: String?
    let value: String?
    let doubleValue: Double?
    let boolValue: Bool?
    let valueBase64: String?
    let offerId: String?
    let keyId: String?
    let nonce: String?
    let signatureBase64: String?
    let timestamp: Int?
    let compactJws: String?
    let shouldContinuePurchase: Bool?
}

func skBuildPurchaseOptions(
    from payloads: [SKPurchaseOptionPayload],
    product: Product
) throws -> Set<Product.PurchaseOption> {
    var options = Set<Product.PurchaseOption>()
    for payload in payloads {
        switch payload.kind {
        case "appAccountToken":
            guard let token = payload.appAccountToken,
                  let uuid = UUID(uuidString: token)
            else {
                throw SKBridgeError.invalidArgument("purchase option appAccountToken requires a valid UUID")
            }
            options.insert(.appAccountToken(uuid))
        case "quantity":
            guard let quantity = payload.quantity else {
                throw SKBridgeError.invalidArgument("purchase option quantity requires an integer value")
            }
            options.insert(.quantity(quantity))
        case "simulatesAskToBuyInSandbox":
            options.insert(.simulatesAskToBuyInSandbox(payload.simulateAskToBuyInSandbox ?? false))
        case "customString":
            guard let key = payload.key, let value = payload.value else {
                throw SKBridgeError.invalidArgument("customString requires key and value")
            }
            options.insert(.custom(key: key, value: value))
        case "customNumber":
            guard let key = payload.key, let value = payload.doubleValue else {
                throw SKBridgeError.invalidArgument("customNumber requires key and doubleValue")
            }
            options.insert(.custom(key: key, value: value))
        case "customBool":
            guard let key = payload.key, let value = payload.boolValue else {
                throw SKBridgeError.invalidArgument("customBool requires key and boolValue")
            }
            options.insert(.custom(key: key, value: value))
        case "customData":
            guard let key = payload.key,
                  let encoded = payload.valueBase64,
                  let data = Data(base64Encoded: encoded)
            else {
                throw SKBridgeError.invalidArgument("customData requires key and valid base64 data")
            }
            options.insert(.custom(key: key, value: data))
        case "promotionalOfferSignature":
            guard let offerID = payload.offerId,
                  let keyID = payload.keyId,
                  let nonceString = payload.nonce,
                  let nonce = UUID(uuidString: nonceString),
                  let signatureBase64 = payload.signatureBase64,
                  let signature = Data(base64Encoded: signatureBase64),
                  let timestamp = payload.timestamp
            else {
                throw SKBridgeError.invalidArgument(
                    "promotionalOfferSignature requires offerID, keyID, nonce, signatureBase64, and timestamp"
                )
            }
            options.insert(
                .promotionalOffer(
                    offerID: offerID,
                    keyID: keyID,
                    nonce: nonce,
                    signature: signature,
                    timestamp: timestamp
                )
            )
        case "promotionalOfferCompactJws":
            guard let offerID = payload.offerId,
                  let compactJWS = payload.compactJws
            else {
                throw SKBridgeError.invalidArgument(
                    "promotionalOfferCompactJws requires offerID and compactJWS"
                )
            }
            for option in Product.PurchaseOption.promotionalOffer(offerID, compactJWS: compactJWS) {
                options.insert(option)
            }
        case "introductoryOfferEligibility":
            guard let compactJWS = payload.compactJws else {
                throw SKBridgeError.invalidArgument(
                    "introductoryOfferEligibility requires compactJWS"
                )
            }
            options.insert(.introductoryOfferEligibility(compactJWS: compactJWS))
        case "winBackOffer":
            guard #available(macOS 15.0, *) else {
                throw SKBridgeError.notSupported("win-back offers require macOS 15.0+")
            }
            guard let offerID = payload.offerId else {
                throw SKBridgeError.invalidArgument("winBackOffer requires offerID")
            }
            guard let offer = product.subscription?.winBackOffers.first(where: { $0.id == offerID }) else {
                throw SKBridgeError.invalidArgument("win-back offer '\(offerID)' was not found on the product")
            }
            options.insert(.winBackOffer(offer))
        case "onStorefrontChange":
            options.insert(.onStorefrontChange { _ in payload.shouldContinuePurchase ?? true })
        default:
            throw SKBridgeError.invalidArgument("unsupported purchase option kind '\(payload.kind)'")
        }
    }
    return options
}
