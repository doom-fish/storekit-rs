import Foundation
import StoreKit

struct SKSubscriptionPeriodPayload: Codable {
    let unit: String
    let value: Int
}

struct SKSubscriptionOfferPayload: Codable {
    let id: String?
    let type: String
    let price: String
    let displayPrice: String
    let period: SKSubscriptionPeriodPayload
    let periodCount: Int
    let paymentMode: String
}

func skSubscriptionPeriodPayload(from period: Product.SubscriptionPeriod) -> SKSubscriptionPeriodPayload {
    let unit: String
    switch period.unit {
    case .day:
        unit = "day"
    case .week:
        unit = "week"
    case .month:
        unit = "month"
    case .year:
        unit = "year"
    @unknown default:
        unit = "unknown"
    }
    return SKSubscriptionPeriodPayload(unit: unit, value: period.value)
}

func skSubscriptionOfferTypeName(_ type: Product.SubscriptionOffer.OfferType) -> String {
    if #available(macOS 15.0, *), type == .winBack {
        return "winBack"
    }
    switch type {
    case .introductory:
        return "introductory"
    case .promotional:
        return "promotional"
    default:
        return type.rawValue
    }
}

func skSubscriptionPaymentModeName(_ mode: Product.SubscriptionOffer.PaymentMode) -> String {
    switch mode {
    case .payAsYouGo:
        return "payAsYouGo"
    case .payUpFront:
        return "payUpFront"
    case .freeTrial:
        return "freeTrial"
    default:
        return mode.rawValue
    }
}

func skSubscriptionOfferPayload(from offer: Product.SubscriptionOffer) -> SKSubscriptionOfferPayload {
    SKSubscriptionOfferPayload(
        id: offer.id,
        type: skSubscriptionOfferTypeName(offer.type),
        price: NSDecimalNumber(decimal: offer.price).stringValue,
        displayPrice: offer.displayPrice,
        period: skSubscriptionPeriodPayload(from: offer.period),
        periodCount: offer.periodCount,
        paymentMode: skSubscriptionPaymentModeName(offer.paymentMode)
    )
}
