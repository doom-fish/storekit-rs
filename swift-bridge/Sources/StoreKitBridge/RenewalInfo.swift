import Foundation
import StoreKit

struct SKRenewalCommitmentInfoPayload: Codable {
    let autoRenewPreference: String
    let renewalBillingPlanType: String
    let renewalDate: String
    let renewalPrice: String
    let willAutoRenew: Bool
}

struct SKRenewalInfoPayload: Codable {
    let originalTransactionID: UInt64
    let currentProductID: String
    let willAutoRenew: Bool
    let autoRenewPreference: String?
    let expirationReason: String?
    let priceIncreaseStatus: String
    let isInBillingRetry: Bool
    let gracePeriodExpirationDate: String?
    let offer: SKTransactionOfferPayload?
    let environment: String?
    let recentSubscriptionStartDate: String
    let renewalDate: String?
    let renewalPrice: String?
    let commitmentInfo: SKRenewalCommitmentInfoPayload?
    let renewalBillingPlanType: String?
    let currencyCode: String?
    let eligibleWinBackOfferIDs: [String]
    let appAccountToken: String?
    let appTransactionID: String?
}

@available(macOS 26.4, *)
func skRenewalCommitmentInfoPayload(
    from commitmentInfo: Product.SubscriptionInfo.RenewalInfo.CommitmentInfo
) -> SKRenewalCommitmentInfoPayload {
    SKRenewalCommitmentInfoPayload(
        autoRenewPreference: commitmentInfo.autoRenewPreference,
        renewalBillingPlanType: skBillingPlanTypeName(commitmentInfo.renewalBillingPlanType),
        renewalDate: skFormatDate(commitmentInfo.renewalDate),
        renewalPrice: NSDecimalNumber(decimal: commitmentInfo.renewalPrice).stringValue,
        willAutoRenew: commitmentInfo.willAutoRenew
    )
}

func skRenewalInfoPayload(from info: Product.SubscriptionInfo.RenewalInfo) -> SKRenewalInfoPayload {
    let offer: SKTransactionOfferPayload?
    if #available(macOS 15.0, *) {
        offer = info.offer.map(skTransactionOfferPayload(from:))
    } else {
        let type = info.offerType.map(skTransactionOfferTypeName(_:))
        offer = info.offerID.map { SKTransactionOfferPayload(id: $0, type: type ?? "unknown", paymentMode: info.offerPaymentModeStringRepresentation, period: nil) }
    }

    let environment: String?
    if #available(macOS 13.0, *) {
        environment = info.environment.rawValue
    } else {
        environment = info.environmentStringRepresentation
    }

    let renewalDate: String?
    if #available(macOS 14.0, *) {
        renewalDate = info.renewalDate.map(skFormatDate)
    } else {
        renewalDate = nil
    }

    let renewalPrice: String?
    if #available(macOS 15.0, *) {
        renewalPrice = info.renewalPrice.map { NSDecimalNumber(decimal: $0).stringValue }
    } else {
        renewalPrice = nil
    }

    let commitmentInfo: SKRenewalCommitmentInfoPayload?
    let renewalBillingPlanType: String?
    if #available(macOS 26.4, *) {
        commitmentInfo = info.commitmentInfo.map(skRenewalCommitmentInfoPayload(from:))
        renewalBillingPlanType = info.renewalBillingPlanType.map(skBillingPlanTypeName(_:))
    } else {
        commitmentInfo = nil
        renewalBillingPlanType = nil
    }

    let currencyCode: String?
    if #available(macOS 15.0, *) {
        currencyCode = info.currency?.identifier
    } else {
        currencyCode = nil
    }

    let eligibleWinBackOfferIDs: [String]
    if #available(macOS 15.0, *) {
        eligibleWinBackOfferIDs = info.eligibleWinBackOfferIDs
    } else {
        eligibleWinBackOfferIDs = []
    }

    return SKRenewalInfoPayload(
        originalTransactionID: info.originalTransactionID,
        currentProductID: info.currentProductID,
        willAutoRenew: info.willAutoRenew,
        autoRenewPreference: info.autoRenewPreference,
        expirationReason: info.expirationReason.map(skExpirationReasonName(_:)),
        priceIncreaseStatus: skPriceIncreaseStatusName(info.priceIncreaseStatus),
        isInBillingRetry: info.isInBillingRetry,
        gracePeriodExpirationDate: info.gracePeriodExpirationDate.map(skFormatDate),
        offer: offer,
        environment: environment,
        recentSubscriptionStartDate: skFormatDate(info.recentSubscriptionStartDate),
        renewalDate: renewalDate,
        renewalPrice: renewalPrice,
        commitmentInfo: commitmentInfo,
        renewalBillingPlanType: renewalBillingPlanType,
        currencyCode: currencyCode,
        eligibleWinBackOfferIDs: eligibleWinBackOfferIDs,
        appAccountToken: info.appAccountToken?.uuidString,
        appTransactionID: info.appTransactionID
    )
}
