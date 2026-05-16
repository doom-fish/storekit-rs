import Foundation
import StoreKit

struct SKProductFormattingPayload: Codable {
    let formattedPrice: String
    let formattedSubscriptionPeriod: String?
    let formattedSubscriptionPeriodUnit: String?
}

func skSingleProduct(for productID: String) async throws -> Product {
    let products = try await Product.products(for: [productID])
    guard let product = products.first else {
        throw SKBridgeError.invalidArgument("product not found for identifier \(productID)")
    }
    return product
}

func skProductFormattingPayload(from product: Product) -> SKProductFormattingPayload {
    let formattedSubscriptionPeriod: String?
    let formattedSubscriptionPeriodUnit: String?
    if let subscription = product.subscription {
        formattedSubscriptionPeriod = subscription.subscriptionPeriod.formatted(product.subscriptionPeriodFormatStyle)
        if #available(macOS 13.0, *) {
            formattedSubscriptionPeriodUnit = subscription.subscriptionPeriod.unit.formatted(product.subscriptionPeriodUnitFormatStyle)
        } else {
            formattedSubscriptionPeriodUnit = nil
        }
    } else {
        formattedSubscriptionPeriod = nil
        formattedSubscriptionPeriodUnit = nil
    }

    return SKProductFormattingPayload(
        formattedPrice: product.price.formatted(product.priceFormatStyle),
        formattedSubscriptionPeriod: formattedSubscriptionPeriod,
        formattedSubscriptionPeriodUnit: formattedSubscriptionPeriodUnit
    )
}

@_cdecl("sk_product_formatting_json")
public func sk_product_formatting_json(
    _ productID: UnsafePointer<CChar>?,
    _ outFormattingJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let productID else {
        let error = SKBridgeError.invalidArgument("missing product identifier")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let productIDString = String(cString: productID)
    return skBlockOnAsync(
        work: {
            let product = try await skSingleProduct(for: productIDString)
            return try skEncodeJSON(skProductFormattingPayload(from: product))
        },
        onSuccess: { json in
            outFormattingJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_localized_description")
public func sk_localized_description(
    _ kind: UnsafePointer<CChar>?,
    _ rawValue: UnsafePointer<CChar>?,
    _ outLocalizedDescription: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let kind, let rawValue else {
        let error = SKBridgeError.invalidArgument("missing localized-description arguments")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    do {
        let description = try skLocalizedDescription(kind: String(cString: kind), rawValue: String(cString: rawValue))
        outLocalizedDescription?.pointee = skCString(description)
        return SK_OK
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }
}

func skLocalizedDescription(kind: String, rawValue: String) throws -> String {
    guard #available(macOS 12.3, *) else {
        throw SKBridgeError.notSupported("localized StoreKit descriptions require macOS 12.3+")
    }

    switch kind {
    case "productType":
        return Product.ProductType(rawValue: rawValue).localizedDescription
    case "renewalState":
        return try skRenewalState(from: rawValue).localizedDescription
    case "expirationReason":
        return try skExpirationReason(from: rawValue).localizedDescription
    case "priceIncreaseStatus":
        return try skPriceIncreaseStatus(from: rawValue).localizedDescription
    case "subscriptionOfferType":
        return try skSubscriptionOfferType(from: rawValue).localizedDescription
    case "transactionOfferType":
        return try skTransactionOfferType(from: rawValue).localizedDescription
    case "subscriptionPaymentMode":
        return try skSubscriptionPaymentMode(from: rawValue).localizedDescription
    case "subscriptionPeriodUnit":
        return try skSubscriptionPeriodUnit(from: rawValue).localizedDescription
    case "revocationReason":
        return try skTransactionRevocationReason(from: rawValue).localizedDescription
    case "ownershipType":
        return Transaction.OwnershipType(rawValue: rawValue).localizedDescription
    default:
        throw SKBridgeError.invalidArgument("unsupported localized-description kind '\(kind)'")
    }
}

@available(macOS 12.3, *)
func skRenewalState(from rawValue: String) throws -> Product.SubscriptionInfo.RenewalState {
    switch rawValue {
    case "subscribed":
        return .subscribed
    case "expired":
        return .expired
    case "inBillingRetryPeriod":
        return .inBillingRetryPeriod
    case "inGracePeriod":
        return .inGracePeriod
    case "revoked":
        return .revoked
    default:
        throw SKBridgeError.invalidArgument("unknown renewal state '\(rawValue)'")
    }
}

@available(macOS 12.3, *)
func skExpirationReason(from rawValue: String) throws -> Product.SubscriptionInfo.RenewalInfo.ExpirationReason {
    switch rawValue {
    case "autoRenewDisabled":
        return .autoRenewDisabled
    case "billingError":
        return .billingError
    case "didNotConsentToPriceIncrease":
        return .didNotConsentToPriceIncrease
    case "productUnavailable":
        return .productUnavailable
    case "unknown":
        return .unknown
    default:
        throw SKBridgeError.invalidArgument("unknown expiration reason '\(rawValue)'")
    }
}

@available(macOS 12.3, *)
func skPriceIncreaseStatus(from rawValue: String) throws -> Product.SubscriptionInfo.RenewalInfo.PriceIncreaseStatus {
    switch rawValue {
    case "noIncreasePending":
        return .noIncreasePending
    case "pending":
        return .pending
    case "agreed":
        return .agreed
    default:
        throw SKBridgeError.invalidArgument("unknown price increase status '\(rawValue)'")
    }
}

@available(macOS 12.3, *)
func skSubscriptionOfferType(from rawValue: String) throws -> Product.SubscriptionOffer.OfferType {
    switch rawValue {
    case "introductory":
        return .introductory
    case "promotional":
        return .promotional
    case "winBack":
        guard #available(macOS 15.0, *) else {
            throw SKBridgeError.notSupported("win-back offer descriptions require macOS 15.0+")
        }
        return .winBack
    default:
        throw SKBridgeError.invalidArgument("unknown subscription offer type '\(rawValue)'")
    }
}

@available(macOS 12.3, *)
func skTransactionOfferType(from rawValue: String) throws -> Transaction.OfferType {
    switch rawValue {
    case "introductory":
        return .introductory
    case "promotional":
        return .promotional
    case "code":
        return .code
    case "winBack":
        guard #available(macOS 15.0, *) else {
            throw SKBridgeError.notSupported("win-back transaction offer descriptions require macOS 15.0+")
        }
        return .winBack
    default:
        throw SKBridgeError.invalidArgument("unknown transaction offer type '\(rawValue)'")
    }
}

@available(macOS 12.3, *)
func skSubscriptionPaymentMode(from rawValue: String) throws -> Product.SubscriptionOffer.PaymentMode {
    switch rawValue {
    case "payAsYouGo":
        return .payAsYouGo
    case "payUpFront":
        return .payUpFront
    case "freeTrial":
        return .freeTrial
    default:
        throw SKBridgeError.invalidArgument("unknown subscription payment mode '\(rawValue)'")
    }
}

@available(macOS 12.3, *)
func skSubscriptionPeriodUnit(from rawValue: String) throws -> Product.SubscriptionPeriod.Unit {
    switch rawValue {
    case "day":
        return .day
    case "week":
        return .week
    case "month":
        return .month
    case "year":
        return .year
    default:
        throw SKBridgeError.invalidArgument("unknown subscription period unit '\(rawValue)'")
    }
}

@available(macOS 12.3, *)
func skTransactionRevocationReason(from rawValue: String) throws -> Transaction.RevocationReason {
    switch rawValue {
    case "developerIssue":
        return .developerIssue
    case "other":
        return .other
    default:
        throw SKBridgeError.invalidArgument("unknown revocation reason '\(rawValue)'")
    }
}
