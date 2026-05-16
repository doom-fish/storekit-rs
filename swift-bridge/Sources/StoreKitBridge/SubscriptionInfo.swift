import Foundation
import StoreKit

struct SKSubscriptionInfoPayload: Codable {
    let introductoryOffer: SKSubscriptionOfferPayload?
    let promotionalOffers: [SKSubscriptionOfferPayload]
    let winBackOffers: [SKSubscriptionOfferPayload]
    let subscriptionGroupID: String
    let subscriptionPeriod: SKSubscriptionPeriodPayload
    let groupLevel: Int?
    let groupDisplayName: String?
}

struct SKSubscriptionStatusPayload: Codable {
    let state: String
    let transaction: SKVerificationResultPayload<SKTransactionPayload>
    let renewalInfo: SKVerificationResultPayload<SKRenewalInfoPayload>
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

    return SKSubscriptionInfoPayload(
        introductoryOffer: info.introductoryOffer.map(skSubscriptionOfferPayload(from:)),
        promotionalOffers: info.promotionalOffers.map(skSubscriptionOfferPayload(from:)),
        winBackOffers: winBackOffers,
        subscriptionGroupID: info.subscriptionGroupID,
        subscriptionPeriod: skSubscriptionPeriodPayload(from: info.subscriptionPeriod),
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
