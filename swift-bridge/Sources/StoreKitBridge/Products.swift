import Foundation
import StoreKit

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
            let options = try skBuildPurchaseOptions(from: optionPayloads)
            let result = try await product.purchase(options: options)
            switch result {
            case .success(let verificationResult):
                let box = SKTransactionBox(result: verificationResult)
                outTransaction?.pointee = sk_retain(box)
                let payload = SKPurchaseResultPayload(
                    kind: "success",
                    transaction: box.payload
                )
                return try skEncodeJSON(payload)
            case .userCancelled:
                return try skEncodeJSON(SKPurchaseResultPayload(kind: "userCancelled", transaction: nil))
            case .pending:
                return try skEncodeJSON(SKPurchaseResultPayload(kind: "pending", transaction: nil))
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
