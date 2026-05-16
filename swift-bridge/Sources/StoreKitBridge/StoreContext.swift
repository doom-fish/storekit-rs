import Foundation
import StoreKit

struct SKStoreContextPayload: Codable {
    let bundleIdentifier: String?
    let bundleName: String?
    let bundleVersion: String?
    let receiptURL: String?
    let canMakePayments: Bool
    let deviceVerificationID: String?
    let isBundled: Bool
    let executablePath: String?
}

@_cdecl("sk_store_context_json")
public func sk_store_context_json(
    _ outContextJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let bundle = Bundle.main
    let payload = SKStoreContextPayload(
        bundleIdentifier: bundle.bundleIdentifier,
        bundleName: bundle.object(forInfoDictionaryKey: "CFBundleName") as? String,
        bundleVersion: bundle.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
            ?? bundle.object(forInfoDictionaryKey: "CFBundleVersion") as? String,
        receiptURL: bundle.appStoreReceiptURL?.path,
        canMakePayments: AppStore.canMakePayments,
        deviceVerificationID: AppStore.deviceVerificationID?.uuidString,
        isBundled: bundle.bundleURL.pathExtension == "app" || (bundle.executablePath?.contains(".app/") ?? false),
        executablePath: bundle.executablePath
    )
    do {
        outContextJSON?.pointee = skCString(try skEncodeJSON(payload))
        return SK_OK
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }
}
