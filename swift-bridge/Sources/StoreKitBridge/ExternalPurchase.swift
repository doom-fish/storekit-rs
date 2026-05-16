import Foundation
import StoreKit

struct SKExternalPurchaseNoticeResultPayload: Codable {
    let kind: String
    let token: String?
}

struct SKExternalPurchaseCustomLinkNoticeResultPayload: Codable {
    let kind: String
}

struct SKExternalPurchaseCustomLinkTokenPayload: Codable {
    let value: String
}

@available(macOS 14.4, *)
func skExternalPurchaseNoticeResultPayload(from result: ExternalPurchase.NoticeResult) -> SKExternalPurchaseNoticeResultPayload {
    switch result {
    case .cancelled:
        return SKExternalPurchaseNoticeResultPayload(kind: "cancelled", token: nil)
    case .continuedWithExternalPurchaseToken(let token):
        return SKExternalPurchaseNoticeResultPayload(
            kind: "continuedWithExternalPurchaseToken",
            token: token
        )
    @unknown default:
        return SKExternalPurchaseNoticeResultPayload(kind: "unknown", token: nil)
    }
}

@available(macOS 15.1, *)
func skExternalPurchaseCustomLinkNoticeResultPayload(
    from result: ExternalPurchaseCustomLink.NoticeResult
) -> SKExternalPurchaseCustomLinkNoticeResultPayload {
    switch result {
    case .cancelled:
        return SKExternalPurchaseCustomLinkNoticeResultPayload(kind: "cancelled")
    case .continued:
        return SKExternalPurchaseCustomLinkNoticeResultPayload(kind: "continued")
    @unknown default:
        return SKExternalPurchaseCustomLinkNoticeResultPayload(kind: "unknown")
    }
}

@_cdecl("sk_external_purchase_can_present")
public func sk_external_purchase_can_present(
    _ outValue: UnsafeMutablePointer<Int32>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            guard #available(macOS 14.4, *) else {
                throw SKBridgeError.notSupported("ExternalPurchase.canPresent requires macOS 14.4+")
            }
            return await ExternalPurchase.canPresent
        },
        onSuccess: { value in
            outValue?.pointee = value ? 1 : 0
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_external_purchase_present_notice_result_json")
public func sk_external_purchase_present_notice_result_json(
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            guard #available(macOS 14.4, *) else {
                throw SKBridgeError.notSupported("ExternalPurchase.presentNoticeSheet() requires macOS 14.4+")
            }
            let result = try await ExternalPurchase.presentNoticeSheet()
            return try skEncodeJSON(skExternalPurchaseNoticeResultPayload(from: result))
        },
        onSuccess: { json in
            outResultJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_external_purchase_link_can_open")
public func sk_external_purchase_link_can_open(
    _ outValue: UnsafeMutablePointer<Int32>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            guard #available(macOS 14.4, *) else {
                throw SKBridgeError.notSupported("ExternalPurchaseLink.canOpen requires macOS 14.4+")
            }
            return await ExternalPurchaseLink.canOpen
        },
        onSuccess: { value in
            outValue?.pointee = value ? 1 : 0
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_external_purchase_link_eligible_urls_json")
public func sk_external_purchase_link_eligible_urls_json(
    _ outURLsJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: { () async throws -> String? in
            guard #available(macOS 14.5, *) else {
                throw SKBridgeError.notSupported("ExternalPurchaseLink.eligibleURLs requires macOS 14.5+")
            }
            guard let urls = await ExternalPurchaseLink.eligibleURLs else {
                return nil
            }
            return try skEncodeJSON(urls.map(\.absoluteString))
        },
        onSuccess: { json in
            outURLsJSON?.pointee = json.flatMap(skCString(_:))
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_external_purchase_link_open")
public func sk_external_purchase_link_open(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            guard #available(macOS 14.4, *) else {
                throw SKBridgeError.notSupported("ExternalPurchaseLink.open() requires macOS 14.4+")
            }
            try await ExternalPurchaseLink.open()
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_external_purchase_link_open_url")
public func sk_external_purchase_link_open_url(
    _ url: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let url else {
        let error = SKBridgeError.invalidArgument("missing external purchase URL")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    let urlString = String(cString: url)
    guard let parsedURL = URL(string: urlString) else {
        let error = SKBridgeError.invalidArgument("invalid external purchase URL '\(urlString)'")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    return skBlockOnAsync(
        work: {
            guard #available(macOS 14.5, *) else {
                throw SKBridgeError.notSupported("ExternalPurchaseLink.open(url:) requires macOS 14.5+")
            }
            try await ExternalPurchaseLink.open(url: parsedURL)
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_external_purchase_custom_link_is_eligible")
public func sk_external_purchase_custom_link_is_eligible(
    _ outValue: UnsafeMutablePointer<Int32>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            guard #available(macOS 15.1, *) else {
                throw SKBridgeError.notSupported(
                    "ExternalPurchaseCustomLink.isEligible requires macOS 15.1+"
                )
            }
            return await ExternalPurchaseCustomLink.isEligible
        },
        onSuccess: { value in
            outValue?.pointee = value ? 1 : 0
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_external_purchase_custom_link_show_notice_result_json")
public func sk_external_purchase_custom_link_show_notice_result_json(
    _ noticeType: Int32,
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    return skBlockOnAsync(
        work: {
            guard #available(macOS 15.1, *) else {
                throw SKBridgeError.notSupported(
                    "ExternalPurchaseCustomLink.showNotice(type:) requires macOS 15.1+"
                )
            }
            let resolvedNoticeType: ExternalPurchaseCustomLink.NoticeType
            switch noticeType {
            case 0:
                resolvedNoticeType = .withinApp
            case 1:
                resolvedNoticeType = .browser
            default:
                throw SKBridgeError.invalidArgument(
                    "unknown external purchase custom-link notice type \(noticeType)"
                )
            }
            let result = try await ExternalPurchaseCustomLink.showNotice(type: resolvedNoticeType)
            return try skEncodeJSON(skExternalPurchaseCustomLinkNoticeResultPayload(from: result))
        },
        onSuccess: { json in
            outResultJSON?.pointee = skCString(json)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_external_purchase_custom_link_token_json")
public func sk_external_purchase_custom_link_token_json(
    _ tokenType: UnsafePointer<CChar>?,
    _ outTokenJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let tokenType else {
        let error = SKBridgeError.invalidArgument("missing external purchase custom-link token type")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    let tokenTypeString = String(cString: tokenType)

    return skBlockOnAsync(
        work: { () async throws -> String? in
            guard #available(macOS 15.1, *) else {
                throw SKBridgeError.notSupported(
                    "ExternalPurchaseCustomLink.token(for:) requires macOS 15.1+"
                )
            }
            guard let token = try await ExternalPurchaseCustomLink.token(for: tokenTypeString) else {
                return nil
            }
            return try skEncodeJSON(SKExternalPurchaseCustomLinkTokenPayload(value: token.value))
        },
        onSuccess: { json in
            outTokenJSON?.pointee = json.flatMap(skCString(_:))
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}
