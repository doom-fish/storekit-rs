import AppKit
import Foundation
import StoreKit

struct SKAppStoreMerchandisingKindPayload: Decodable {
    let kind: String
    let groupID: String?
}

struct SKAppStoreMerchandisingPresentationResultPayload: Codable {
    let kind: String
    let purchaseResult: SKPurchaseResultPayload?
}

struct SKAdvancedCommercePurchaseOptionPayload: Decodable {
    let kind: String
    let shouldContinuePurchase: Bool?
}

struct SKAdvancedCommerceProductPayload: Codable {
    let id: String
    let type: String
}

struct SKTransactionAdvancedCommerceInfoPayload: Codable {
    let requestReferenceID: String
    let estimatedTax: String
    let taxRate: String
    let taxCode: String
    let taxExclusivePrice: String
    let description: String?
    let displayName: String?
    let period: SKSubscriptionPeriodPayload?
    let items: [SKTransactionAdvancedCommerceItemPayload]
}

struct SKTransactionAdvancedCommerceItemPayload: Codable {
    let details: SKTransactionAdvancedCommerceItemDetailsPayload
    let refunds: [SKTransactionAdvancedCommerceRefundPayload]?
    let revocationDate: String?
}

struct SKTransactionAdvancedCommerceItemDetailsPayload: Codable {
    let sku: String
    let displayName: String
    let description: String
    let offer: SKTransactionAdvancedCommerceOfferPayload?
    let price: String
}

struct SKTransactionAdvancedCommerceOfferPayload: Codable {
    let price: String
    let period: SKSubscriptionPeriodPayload
    let periodCount: Int
    let reason: String
}

struct SKTransactionAdvancedCommerceRefundPayload: Codable {
    let reason: String
    let type: String
    let date: String
    let amount: String
}

@available(macOS 26.2, *)
func skBuildAppStoreMerchandisingKind(
    from payload: SKAppStoreMerchandisingKindPayload
) throws -> AppStoreMerchandisingKind {
    switch payload.kind {
    case "subscriptionBundle":
        guard let groupID = payload.groupID else {
            throw SKBridgeError.invalidArgument("subscriptionBundle merchandising requires groupID")
        }
        return .subscriptionBundle(groupID)
    default:
        throw SKBridgeError.invalidArgument("unsupported App Store merchandising kind '\(payload.kind)'")
    }
}

@available(macOS 15.4, *)
func skBuildAdvancedCommercePurchaseOptions(
    from payloads: [SKAdvancedCommercePurchaseOptionPayload]
) throws -> Set<AdvancedCommerceProduct.PurchaseOption> {
    var options = Set<AdvancedCommerceProduct.PurchaseOption>()
    for payload in payloads {
        switch payload.kind {
        case "onStorefrontChange":
            options.insert(.onStorefrontChange { _ in payload.shouldContinuePurchase ?? true })
        default:
            throw SKBridgeError.invalidArgument(
                "unsupported advanced-commerce purchase option kind '\(payload.kind)'"
            )
        }
    }
    return options
}

@available(macOS 15.4, *)
func skAdvancedCommerceProductPayload(from product: AdvancedCommerceProduct) -> SKAdvancedCommerceProductPayload {
    SKAdvancedCommerceProductPayload(id: product.id, type: skProductTypeName(product.type))
}

@available(macOS 15.4, *)
func skTransactionAdvancedCommerceInfoPayload(
    from info: Transaction.AdvancedCommerceInfo
) -> SKTransactionAdvancedCommerceInfoPayload {
    SKTransactionAdvancedCommerceInfoPayload(
        requestReferenceID: info.requestReferenceID,
        estimatedTax: NSDecimalNumber(decimal: info.estimatedTax).stringValue,
        taxRate: NSDecimalNumber(decimal: info.taxRate).stringValue,
        taxCode: info.taxCode,
        taxExclusivePrice: NSDecimalNumber(decimal: info.taxExclusivePrice).stringValue,
        description: info.description,
        displayName: info.displayName,
        period: info.period.map(skSubscriptionPeriodPayload(from:)),
        items: info.items.map(skTransactionAdvancedCommerceItemPayload(from:))
    )
}

@available(macOS 15.4, *)
func skTransactionAdvancedCommerceItemPayload(
    from item: Transaction.AdvancedCommerceInfo.Item
) -> SKTransactionAdvancedCommerceItemPayload {
    SKTransactionAdvancedCommerceItemPayload(
        details: skTransactionAdvancedCommerceItemDetailsPayload(from: item.details),
        refunds: item.refunds?.map(skTransactionAdvancedCommerceRefundPayload(from:)),
        revocationDate: item.revocationDate.map(skFormatDate)
    )
}

@available(macOS 15.4, *)
func skTransactionAdvancedCommerceItemDetailsPayload(
    from details: Transaction.AdvancedCommerceInfo.Item.Details
) -> SKTransactionAdvancedCommerceItemDetailsPayload {
    SKTransactionAdvancedCommerceItemDetailsPayload(
        sku: details.sku,
        displayName: details.displayName,
        description: details.description,
        offer: details.offer.map(skTransactionAdvancedCommerceOfferPayload(from:)),
        price: NSDecimalNumber(decimal: details.price).stringValue
    )
}

@available(macOS 15.4, *)
func skTransactionAdvancedCommerceOfferPayload(
    from offer: Transaction.AdvancedCommerceInfo.Offer
) -> SKTransactionAdvancedCommerceOfferPayload {
    SKTransactionAdvancedCommerceOfferPayload(
        price: NSDecimalNumber(decimal: offer.price).stringValue,
        period: skSubscriptionPeriodPayload(from: offer.period),
        periodCount: offer.periodCount,
        reason: offer.reason.rawValue
    )
}

@available(macOS 15.4, *)
func skTransactionAdvancedCommerceRefundPayload(
    from refund: Transaction.AdvancedCommerceInfo.Refund
) -> SKTransactionAdvancedCommerceRefundPayload {
    SKTransactionAdvancedCommerceRefundPayload(
        reason: refund.reason.rawValue,
        type: refund.type.rawValue,
        date: skFormatDate(refund.date),
        amount: NSDecimalNumber(decimal: refund.amount).stringValue
    )
}

@_cdecl("sk_app_store_age_rating_code")
public func sk_app_store_age_rating_code(
    _ outValue: UnsafeMutablePointer<Int64>?,
    _ outHasValue: UnsafeMutablePointer<Int32>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            guard #available(macOS 26.2, *) else {
                throw SKBridgeError.notSupported("AppStore.ageRatingCode requires macOS 26.2+")
            }
            return await AppStore.ageRatingCode
        },
        onSuccess: { value in
            if let value {
                outValue?.pointee = Int64(value)
                outHasValue?.pointee = 1
            } else {
                outHasValue?.pointee = 0
            }
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_app_store_present_merchandising")
public func sk_app_store_present_merchandising(
    _ kindJSON: UnsafePointer<CChar>?,
    _ window: UnsafeMutableRawPointer?,
    _ outTransaction: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let payload: SKAppStoreMerchandisingKindPayload
    do {
        payload = try skDecodeJSON(kindJSON, as: SKAppStoreMerchandisingKindPayload.self)
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }

    return skBlockOnMainActorAsync(
        work: {
            guard #available(macOS 26.2, *) else {
                throw SKBridgeError.notSupported(
                    "AppStore.presentMerchandising(_:from:) requires macOS 26.2+"
                )
            }
            let merchandisingKind = try skBuildAppStoreMerchandisingKind(from: payload)
            let confirmedWindow: NSWindow = try skBorrowWindow(
                window,
                context: "AppStore.presentMerchandising(_:from:)"
            )
            let result = try await AppStore.presentMerchandising(merchandisingKind, from: confirmedWindow)
            switch result {
            case .dismissed:
                return try skEncodeJSON(
                    SKAppStoreMerchandisingPresentationResultPayload(
                        kind: "dismissed",
                        purchaseResult: nil
                    )
                )
            case .purchaseCompleted(let purchaseResult):
                return try skEncodeJSON(
                    SKAppStoreMerchandisingPresentationResultPayload(
                        kind: "purchaseCompleted",
                        purchaseResult: try skPurchaseResultPayload(
                            from: purchaseResult,
                            outTransaction: outTransaction
                        )
                    )
                )
            @unknown default:
                throw SKBridgeError.unknown("StoreKit returned an unknown merchandising result")
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

@_cdecl("sk_advanced_commerce_product_json")
public func sk_advanced_commerce_product_json(
    _ productID: UnsafePointer<CChar>?,
    _ outProductJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let productID else {
        let error = SKBridgeError.invalidArgument("missing advanced-commerce product identifier")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    let productIDString = String(cString: productID)

    return skBlockOnAsync(
        work: {
            guard #available(macOS 15.4, *) else {
                throw SKBridgeError.notSupported("AdvancedCommerceProduct requires macOS 15.4+")
            }
            let product = try await AdvancedCommerceProduct(id: productIDString)
            return try skEncodeJSON(skAdvancedCommerceProductPayload(from: product))
        },
        onSuccess: { json in
            outProductJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_advanced_commerce_product_purchase")
public func sk_advanced_commerce_product_purchase(
    _ productID: UnsafePointer<CChar>?,
    _ compactJWS: UnsafePointer<CChar>?,
    _ window: UnsafeMutableRawPointer?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ outTransaction: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let productID, let compactJWS else {
        let error = SKBridgeError.invalidArgument(
            "missing advanced-commerce purchase arguments"
        )
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let optionPayloads: [SKAdvancedCommercePurchaseOptionPayload]
    do {
        optionPayloads = try skDecodeJSONIfPresent(
            optionsJSON,
            as: [SKAdvancedCommercePurchaseOptionPayload].self
        ) ?? []
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }

    let productIDString = String(cString: productID)
    let compactJWSString = String(cString: compactJWS)
    return skBlockOnMainActorAsync(
        timeoutSeconds: 60,
        work: {
            guard #available(macOS 15.4, *) else {
                throw SKBridgeError.notSupported(
                    "AdvancedCommerceProduct.purchase(compactJWS:confirmIn:options:) requires macOS 15.4+"
                )
            }
            let confirmedWindow: NSWindow = try skBorrowWindow(
                window,
                context: "AdvancedCommerceProduct.purchase(compactJWS:confirmIn:options:)"
            )
            let product = try await AdvancedCommerceProduct(id: productIDString)
            let options = try skBuildAdvancedCommercePurchaseOptions(from: optionPayloads)
            let result = try await product.purchase(
                compactJWS: compactJWSString,
                confirmIn: confirmedWindow,
                options: options
            )
            return try skEncodeJSON(
                try skPurchaseResultPayload(from: result, outTransaction: outTransaction)
            )
        },
        onSuccess: { json in
            outResultJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}
