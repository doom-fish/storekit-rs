import AppKit
import Foundation
import StoreKit

@_cdecl("sk_app_store_can_make_payments")
public func sk_app_store_can_make_payments(
    _ outValue: UnsafeMutablePointer<Int32>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outValue?.pointee = AppStore.canMakePayments ? 1 : 0
    return SK_OK
}

@_cdecl("sk_app_store_device_verification_id")
public func sk_app_store_device_verification_id(
    _ outUUID: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if let uuid = AppStore.deviceVerificationID {
        outUUID?.pointee = skCString(uuid.uuidString)
    } else {
        outUUID?.pointee = nil
    }
    return SK_OK
}

@_cdecl("sk_app_store_sync")
public func sk_app_store_sync(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            try await AppStore.sync()
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_app_store_show_manage_subscriptions")
public func sk_app_store_show_manage_subscriptions(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let error = SKBridgeError.notSupported(
        "AppStore.showManageSubscriptions(in:) is scene-based and unavailable in the macOS StoreKit SDK"
    )
    skPopulateError(outError, with: error)
    return error.statusCode
}

@_cdecl("sk_app_store_request_review")
public func sk_app_store_request_review(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnMainActorAsync(
        work: {
            guard #available(macOS 13.0, *) else {
                throw SKBridgeError.notSupported("AppStore.requestReview(in:) requires macOS 13.0+")
            }
            guard let controller = skKeyWindowController() else {
                throw SKBridgeError.notSupported(
                    "AppStore.requestReview(in:) requires an NSViewController-backed window"
                )
            }
            AppStore.requestReview(in: controller)
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_app_store_present_offer_code_redeem_sheet")
public func sk_app_store_present_offer_code_redeem_sheet(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnMainActorAsync(
        work: {
            guard #available(macOS 15.0, *) else {
                throw SKBridgeError.notSupported(
                    "AppStore.presentOfferCodeRedeemSheet(from:) requires macOS 15.0+"
                )
            }
            guard let controller = skKeyWindowController() else {
                throw SKBridgeError.notSupported(
                    "AppStore.presentOfferCodeRedeemSheet(from:) requires an NSViewController-backed window"
                )
            }
            try await AppStore.presentOfferCodeRedeemSheet(from: controller)
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}
