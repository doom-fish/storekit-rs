import StoreKit

func skRenewalStateName(_ state: Product.SubscriptionInfo.RenewalState) -> String {
    switch state {
    case .subscribed:
        return "subscribed"
    case .expired:
        return "expired"
    case .inBillingRetryPeriod:
        return "inBillingRetryPeriod"
    case .inGracePeriod:
        return "inGracePeriod"
    case .revoked:
        return "revoked"
    default:
        return "unknown"
    }
}

func skExpirationReasonName(_ reason: Product.SubscriptionInfo.RenewalInfo.ExpirationReason) -> String {
    switch reason {
    case .autoRenewDisabled:
        return "autoRenewDisabled"
    case .billingError:
        return "billingError"
    case .didNotConsentToPriceIncrease:
        return "didNotConsentToPriceIncrease"
    case .productUnavailable:
        return "productUnavailable"
    case .unknown:
        return "unknown"
    default:
        return "unknown"
    }
}

func skPriceIncreaseStatusName(_ status: Product.SubscriptionInfo.RenewalInfo.PriceIncreaseStatus) -> String {
    switch status {
    case .noIncreasePending:
        return "noIncreasePending"
    case .pending:
        return "pending"
    case .agreed:
        return "agreed"
    default:
        return "unknown"
    }
}
