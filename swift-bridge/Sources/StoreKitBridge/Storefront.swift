import Foundation
import StoreKit

struct SKStorefrontPayload: Codable {
    let countryCode: String
    let id: String
    let currencyCode: String?
}

func skStorefrontPayload(from storefront: Storefront) -> SKStorefrontPayload {
    let currencyCode: String?
    if #available(macOS 14.0, *) {
        currencyCode = storefront.currency?.identifier
    } else {
        currencyCode = nil
    }
    return SKStorefrontPayload(
        countryCode: storefront.countryCode,
        id: storefront.id,
        currencyCode: currencyCode
    )
}

@_cdecl("sk_storefront_current_json")
public func sk_storefront_current_json(
    _ outStorefrontJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    skBlockOnAsync(
        work: {
            await Storefront.current.map(skStorefrontPayload(from:))
        },
        onSuccess: { payload in
            if let payload, let json = try? skEncodeJSON(payload) {
                outStorefrontJSON?.pointee = skCString(json)
            }
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_storefront_stream_create")
public func sk_storefront_stream_create(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    sk_retain(SKStreamBox<SKStorefrontPayload> { queue in
        var iterator = Storefront.updates.makeAsyncIterator()
        while !Task.isCancelled, let next = await iterator.next() {
            queue.push(skStorefrontPayload(from: next))
        }
    })
}

@_cdecl("sk_storefront_stream_release")
public func sk_storefront_stream_release(_ stream: UnsafeMutableRawPointer?) {
    guard let stream else {
        return
    }
    sk_release(stream)
}

@_cdecl("sk_storefront_stream_next")
public func sk_storefront_stream_next(
    _ stream: UnsafeMutableRawPointer?,
    _ timeoutMilliseconds: Int64,
    _ outStorefrontJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let stream else {
        let error = SKBridgeError.invalidArgument("missing storefront stream")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    let box: SKStreamBox<SKStorefrontPayload> = sk_borrow(stream)
    switch box.next(timeoutMilliseconds: timeoutMilliseconds) {
    case .item(let payload):
        if let json = try? skEncodeJSON(payload) {
            outStorefrontJSON?.pointee = skCString(json)
            return SK_OK
        }
        let error = SKBridgeError.unknown("failed to encode storefront payload")
        skPopulateError(outError, with: error)
        return error.statusCode
    case .end:
        return SK_END_OF_STREAM
    case .timedOut:
        return SK_TIMED_OUT
    }
}
