import Foundation
import StoreKit

struct SKProductPayload: Codable {
    let id: String
    let displayName: String
    let description: String
    let price: String
    let displayPrice: String
    let type: String
    let isFamilyShareable: Bool
    let subscription: SKSubscriptionInfoPayload?
    let currencyCode: String?
    let priceLocaleIdentifier: String?
    let jsonRepresentationBase64: String
}

struct SKPurchaseResultPayload: Codable {
    let kind: String
    let verificationResult: SKVerificationResultPayload<SKTransactionPayload>?
}

func skProductTypeName(_ type: Product.ProductType) -> String {
    switch type {
    case .consumable:
        return "consumable"
    case .nonConsumable:
        return "nonConsumable"
    case .autoRenewable:
        return "autoRenewable"
    case .nonRenewable:
        return "nonRenewing"
    default:
        return type.rawValue
    }
}

func skProductPayload(from product: Product) -> SKProductPayload {
    SKProductPayload(
        id: product.id,
        displayName: product.displayName,
        description: product.description,
        price: NSDecimalNumber(decimal: product.price).stringValue,
        displayPrice: product.displayPrice,
        type: skProductTypeName(product.type),
        isFamilyShareable: product.isFamilyShareable,
        subscription: product.subscription.map(skSubscriptionInfoPayload(from:)),
        currencyCode: nil,
        priceLocaleIdentifier: nil,
        jsonRepresentationBase64: skDataBase64(product.jsonRepresentation)
    )
}

@_cdecl("sk_products_json")
public func sk_products_json(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ outProductsJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let identifiers: [String]
    do {
        identifiers = try skDecodeJSON(identifiersJSON, as: [String].self)
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }

    return skBlockOnAsync(
        work: {
            let products = try await Product.products(for: identifiers)
            return try skEncodeJSON(products.map(skProductPayload(from:)))
        },
        onSuccess: { json in
            outProductsJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_product_purchase")
public func sk_product_purchase(
    _ productID: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ outTransaction: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let productID else {
        let error = SKBridgeError.invalidArgument("missing product identifier")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let optionPayloads: [SKPurchaseOptionPayload]
    do {
        optionPayloads = try skDecodeJSONIfPresent(optionsJSON, as: [SKPurchaseOptionPayload].self) ?? []
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }

    return skBlockOnMainActorAsync(
        timeoutSeconds: 60,
        work: {
            let products = try await Product.products(for: [String(cString: productID)])
            guard let product = products.first else {
                throw SKBridgeError.invalidArgument("product not found for identifier \(String(cString: productID))")
            }
            let options = try skBuildPurchaseOptions(from: optionPayloads, product: product)
            let result = try await product.purchase(options: options)
            switch result {
            case .success(let verificationResult):
                let box = SKTransactionBox(result: verificationResult)
                outTransaction?.pointee = sk_retain(box)
                return try skEncodeJSON(
                    SKPurchaseResultPayload(
                        kind: "success",
                        verificationResult: skTransactionVerificationResultPayload(from: verificationResult)
                    )
                )
            case .userCancelled:
                return try skEncodeJSON(
                    SKPurchaseResultPayload(kind: "userCancelled", verificationResult: nil)
                )
            case .pending:
                return try skEncodeJSON(
                    SKPurchaseResultPayload(kind: "pending", verificationResult: nil)
                )
            @unknown default:
                throw SKBridgeError.unknown("StoreKit returned an unknown purchase result")
            }
        },
        onSuccess: { json in
            outResultJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}
