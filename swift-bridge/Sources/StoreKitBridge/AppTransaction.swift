import Foundation
import StoreKit

@available(macOS 13.0, *)
struct SKAppTransactionPayload: Codable {
    let appID: UInt64?
    let appTransactionID: String
    let appVersion: String
    let appVersionID: UInt64?
    let bundleID: String
    let environment: String
    let originalAppVersion: String
    let originalPurchaseDate: String
    let originalPlatform: String?
    let preorderDate: String?
    let jsonRepresentationBase64: String
}

@available(macOS 13.0, *)
func skAppTransactionPayload(from result: VerificationResult<AppTransaction>) -> SKAppTransactionPayload {
    let transaction = result.unsafePayloadValue
    let originalPlatform: String?
    if #available(macOS 15.4, *) {
        originalPlatform = transaction.originalPlatform.rawValue
    } else {
        originalPlatform = transaction.originalPlatformStringRepresentation
    }
    return SKAppTransactionPayload(
        appID: transaction.appID,
        appTransactionID: transaction.appTransactionID,
        appVersion: transaction.appVersion,
        appVersionID: transaction.appVersionID,
        bundleID: transaction.bundleID,
        environment: transaction.environment.rawValue,
        originalAppVersion: transaction.originalAppVersion,
        originalPurchaseDate: skFormatDate(transaction.originalPurchaseDate),
        originalPlatform: originalPlatform,
        preorderDate: transaction.preorderDate.map(skFormatDate),
        jsonRepresentationBase64: skDataBase64(transaction.jsonRepresentation)
    )
}

@_cdecl("sk_app_transaction_shared")
public func sk_app_transaction_shared(
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            guard #available(macOS 13.0, *) else {
                throw SKBridgeError.notSupported("AppTransaction.shared requires macOS 13.0+")
            }
            let shared = try await AppTransaction.shared
            return try skEncodeJSON(skAppTransactionVerificationResultPayload(from: shared))
        },
        onSuccess: { json in
            outResultJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_app_transaction_refresh")
public func sk_app_transaction_refresh(
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            guard #available(macOS 13.0, *) else {
                throw SKBridgeError.notSupported("AppTransaction.refresh() requires macOS 13.0+")
            }
            let refreshed = try await AppTransaction.refresh()
            return try skEncodeJSON(skAppTransactionVerificationResultPayload(from: refreshed))
        },
        onSuccess: { json in
            outResultJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}
