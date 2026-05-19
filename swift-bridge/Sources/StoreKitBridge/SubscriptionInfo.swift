import Foundation
import StoreKit

@available(macOS 26.4, *)
func skBillingPlanTypeName(_ type: Product.SubscriptionInfo.BillingPlanType) -> String {
    switch type {
    case .monthly:
        return "monthly"
    case .upFront:
        return "upFront"
    default:
        return type.rawValue
    }
}

struct SKSubscriptionCommitmentInfoPayload: Codable {
    let price: String
    let displayPrice: String
    let period: SKSubscriptionPeriodPayload
}

struct SKSubscriptionPricingTermsPayload: Codable {
    let billingPrice: String
    let billingDisplayPrice: String
    let billingPeriod: SKSubscriptionPeriodPayload
    let billingPlanType: String
    let commitmentInfo: SKSubscriptionCommitmentInfoPayload
    let subscriptionOffers: [SKSubscriptionOfferPayload]
}

struct SKSubscriptionInfoPayload: Codable {
    let introductoryOffer: SKSubscriptionOfferPayload?
    let promotionalOffers: [SKSubscriptionOfferPayload]
    let winBackOffers: [SKSubscriptionOfferPayload]
    let subscriptionGroupID: String
    let subscriptionPeriod: SKSubscriptionPeriodPayload
    let pricingTerms: [SKSubscriptionPricingTermsPayload]
    let groupLevel: Int?
    let groupDisplayName: String?
}

struct SKSubscriptionStatusPayload: Codable {
    let state: String
    let transaction: SKVerificationResultPayload<SKTransactionPayload>
    let renewalInfo: SKVerificationResultPayload<SKRenewalInfoPayload>
}

@available(macOS 26.4, *)
func skSubscriptionCommitmentInfoPayload(
    from commitmentInfo: Product.SubscriptionInfo.CommitmentInfo
) -> SKSubscriptionCommitmentInfoPayload {
    SKSubscriptionCommitmentInfoPayload(
        price: NSDecimalNumber(decimal: commitmentInfo.price).stringValue,
        displayPrice: commitmentInfo.displayPrice,
        period: skSubscriptionPeriodPayload(from: commitmentInfo.period)
    )
}

@available(macOS 26.4, *)
func skSubscriptionPricingTermsPayload(
    from pricingTerms: Product.SubscriptionInfo.PricingTerms
) -> SKSubscriptionPricingTermsPayload {
    SKSubscriptionPricingTermsPayload(
        billingPrice: NSDecimalNumber(decimal: pricingTerms.billingPrice).stringValue,
        billingDisplayPrice: pricingTerms.billingDisplayPrice,
        billingPeriod: skSubscriptionPeriodPayload(from: pricingTerms.billingPeriod),
        billingPlanType: skBillingPlanTypeName(pricingTerms.billingPlanType),
        commitmentInfo: skSubscriptionCommitmentInfoPayload(from: pricingTerms.commitmentInfo),
        subscriptionOffers: pricingTerms.subscriptionOffers.map(skSubscriptionOfferPayload(from:))
    )
}

func skSubscriptionInfoPayload(from info: Product.SubscriptionInfo) -> SKSubscriptionInfoPayload {
    let groupLevel: Int?
    let groupDisplayName: String?
    if #available(macOS 13.3, *) {
        groupLevel = info.groupLevel
        groupDisplayName = info.groupDisplayName
    } else {
        groupLevel = nil
        groupDisplayName = nil
    }

    let winBackOffers: [SKSubscriptionOfferPayload]
    if #available(macOS 15.0, *) {
        winBackOffers = info.winBackOffers.map(skSubscriptionOfferPayload(from:))
    } else {
        winBackOffers = []
    }

    let pricingTerms: [SKSubscriptionPricingTermsPayload]
    if #available(macOS 26.4, *) {
        pricingTerms = info.pricingTerms.map(skSubscriptionPricingTermsPayload(from:))
    } else {
        pricingTerms = []
    }

    return SKSubscriptionInfoPayload(
        introductoryOffer: info.introductoryOffer.map(skSubscriptionOfferPayload(from:)),
        promotionalOffers: info.promotionalOffers.map(skSubscriptionOfferPayload(from:)),
        winBackOffers: winBackOffers,
        subscriptionGroupID: info.subscriptionGroupID,
        subscriptionPeriod: skSubscriptionPeriodPayload(from: info.subscriptionPeriod),
        pricingTerms: pricingTerms,
        groupLevel: groupLevel,
        groupDisplayName: groupDisplayName
    )
}

func skSubscriptionStatusPayload(from status: Product.SubscriptionInfo.Status) -> SKSubscriptionStatusPayload {
    SKSubscriptionStatusPayload(
        state: skRenewalStateName(status.state),
        transaction: skTransactionVerificationResultPayload(from: status.transaction),
        renewalInfo: skRenewalInfoVerificationResultPayload(from: status.renewalInfo)
    )
}

@_cdecl("sk_subscription_info_is_eligible_for_intro_offer")
public func sk_subscription_info_is_eligible_for_intro_offer(
    _ groupID: UnsafePointer<CChar>?,
    _ outValue: UnsafeMutablePointer<Int32>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let groupID else {
        let error = SKBridgeError.invalidArgument("missing subscription group id")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    let groupIDString = String(cString: groupID)
    return skBlockOnAsync(
        work: {
            await Product.SubscriptionInfo.isEligibleForIntroOffer(for: groupIDString)
        },
        onSuccess: { value in
            outValue?.pointee = value ? 1 : 0
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_subscription_info_statuses_json")
public func sk_subscription_info_statuses_json(
    _ groupID: UnsafePointer<CChar>?,
    _ outStatusesJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let groupID else {
        let error = SKBridgeError.invalidArgument("missing subscription group id")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    let groupIDString = String(cString: groupID)
    return skBlockOnAsync(
        work: {
            let statuses = try await Product.SubscriptionInfo.status(for: groupIDString)
            return try skEncodeJSON(statuses.map(skSubscriptionStatusPayload(from:)))
        },
        onSuccess: { json in
            outStatusesJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_subscription_info_status_for_transaction")
public func sk_subscription_info_status_for_transaction(
    _ transactionID: UnsafePointer<CChar>?,
    _ outStatusJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let transactionID else {
        let error = SKBridgeError.invalidArgument("missing transaction id")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    guard let parsedTransactionID = UInt64(String(cString: transactionID)) else {
        let error = SKBridgeError.invalidArgument("transaction id must be an unsigned integer")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    return skBlockOnAsync(
        work: { () async throws -> String? in
            guard #available(macOS 15.4, *) else {
                throw SKBridgeError.notSupported(
                    "Product.SubscriptionInfo.status(transactionID:) requires macOS 15.4+"
                )
            }
            if let status = try await Product.SubscriptionInfo.status(transactionID: parsedTransactionID) {
                return try skEncodeJSON(skSubscriptionStatusPayload(from: status))
            }
            return nil as String?
        },
        onSuccess: { (json: String?) in
            outStatusJSON?.pointee = json.flatMap(skCString(_:))
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}
