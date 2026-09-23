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
            guard let shouldContinuePurchase = payload.shouldContinuePurchase else {
                throw SKBridgeError.invalidArgument("onStorefrontChange requires shouldContinuePurchase")
            }
            options.insert(.onStorefrontChange { _ in shouldContinuePurchase })
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

@available(macOS 26.2, *)
func skMerchandisingOutcome(
    from result: AppStoreMerchandisingKind.PresentationResult
) throws -> SKTransactionOutcome {
    switch result {
    case .dismissed:
        return SKTransactionOutcome(
            json: try skEncodeJSON(
                SKAppStoreMerchandisingPresentationResultPayload(kind: "dismissed", purchaseResult: nil)
            ),
            transaction: nil
        )
    case .purchaseCompleted(let purchaseResult):
        let parts = try skPurchaseResultParts(from: purchaseResult)
        return SKTransactionOutcome(
            json: try skEncodeJSON(
                SKAppStoreMerchandisingPresentationResultPayload(
                    kind: "purchaseCompleted",
                    purchaseResult: parts.payload
                )
            ),
            transaction: parts.transaction
        )
    @unknown default:
        throw SKBridgeError.unknown("StoreKit returned an unknown merchandising result")
    }
}

@_cdecl("sk_app_store_present_merchandising")
public func sk_app_store_present_merchandising(
    _ kindJSON: UnsafePointer<CChar>?,
    _ window: UnsafeMutableRawPointer?,
    _ outTransaction: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.2, *) else {
        let error = SKBridgeError.notSupported("AppStore.presentMerchandising(_:from:) requires macOS 26.2+")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let merchandisingKind: AppStoreMerchandisingKind
    let confirmedWindow: NSWindow
    do {
        let payload = try skDecodeJSON(kindJSON, as: SKAppStoreMerchandisingKindPayload.self)
        merchandisingKind = try skBuildAppStoreMerchandisingKind(from: payload)
        confirmedWindow = try skBorrowWindow(window, context: "AppStore.presentMerchandising(_:from:)")
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }

    return skBlockOnMainActorAsync(
        label: "AppStore.presentMerchandising(_:from:)",
        work: {
            try skMerchandisingOutcome(
                from: try await AppStore.presentMerchandising(merchandisingKind, from: confirmedWindow)
            )
        },
        onSuccess: { outcome in
            skWriteTransactionOutcome(outcome, outTransaction: outTransaction, outResultJSON: outResultJSON)
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
    guard #available(macOS 15.4, *) else {
        let error = SKBridgeError.notSupported(
            "AdvancedCommerceProduct.purchase(compactJWS:confirmIn:options:) requires macOS 15.4+"
        )
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    guard let productID, let compactJWS else {
        let error = SKBridgeError.invalidArgument(
            "missing advanced-commerce purchase arguments"
        )
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    let productIDString = String(cString: productID)
    let compactJWSString = String(cString: compactJWS)

    let options: Set<AdvancedCommerceProduct.PurchaseOption>
    let confirmedWindow: NSWindow
    do {
        let optionPayloads = try skDecodeJSONIfPresent(
            optionsJSON,
            as: [SKAdvancedCommercePurchaseOptionPayload].self
        ) ?? []
        options = try skBuildAdvancedCommercePurchaseOptions(from: optionPayloads)
        confirmedWindow = try skBorrowWindow(
            window,
            context: "AdvancedCommerceProduct.purchase(compactJWS:confirmIn:options:)"
        )
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }

    return skBlockOnMainActorAsync(
        label: "AdvancedCommerceProduct.purchase(compactJWS:confirmIn:options:)",
        work: {
            let product = try await AdvancedCommerceProduct(id: productIDString)
            let result = try await product.purchase(
                compactJWS: compactJWSString,
                confirmIn: confirmedWindow,
                options: options
            )
            return try skPurchaseOutcome(from: result)
        },
        onSuccess: { outcome in
            skWriteTransactionOutcome(outcome, outTransaction: outTransaction, outResultJSON: outResultJSON)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}
