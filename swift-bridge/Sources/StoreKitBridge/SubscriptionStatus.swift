import Foundation
import StoreKit

struct SKSubscriptionGroupStatusesPayload: Codable {
    let groupID: String
    let statuses: [SKSubscriptionStatusPayload]
}

@_cdecl("sk_subscription_status_stream_create")
public func sk_subscription_status_stream_create(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    _ = outError
    return sk_retain(SKStreamBox<SKSubscriptionStatusPayload> { queue in
        var iterator = Product.SubscriptionInfo.Status.updates.makeAsyncIterator()
        while !Task.isCancelled, let next = await iterator.next() {
            queue.push(skSubscriptionStatusPayload(from: next))
        }
    })
}

@_cdecl("sk_subscription_status_stream_release")
public func sk_subscription_status_stream_release(_ stream: UnsafeMutableRawPointer?) {
    guard let stream else {
        return
    }
    sk_release(stream)
}

@_cdecl("sk_subscription_status_stream_next")
public func sk_subscription_status_stream_next(
    _ stream: UnsafeMutableRawPointer?,
    _ timeoutMilliseconds: Int64,
    _ outStatusJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let stream else {
        let error = SKBridgeError.invalidArgument("missing subscription status stream")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let box: SKStreamBox<SKSubscriptionStatusPayload> = sk_borrow(stream)
    switch box.queue.next(timeoutMilliseconds: timeoutMilliseconds) {
    case .item(let payload):
        if let json = try? skEncodeJSON(payload) {
            outStatusJSON?.pointee = skCString(json)
            return SK_OK
        }
        let error = SKBridgeError.unknown("failed to encode subscription status payload")
        skPopulateError(outError, with: error)
        return error.statusCode
    case .end:
        return SK_END_OF_STREAM
    case .timedOut:
        return SK_TIMED_OUT
    }
}

@_cdecl("sk_subscription_group_status_stream_create")
public func sk_subscription_group_status_stream_create(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 14.0, *) else {
        skPopulateError(
            outError,
            with: SKBridgeError.notSupported("Product.SubscriptionInfo.Status.all requires macOS 14.0+")
        )
        return nil
    }
    return sk_retain(SKStreamBox<SKSubscriptionGroupStatusesPayload> { queue in
        var iterator = Product.SubscriptionInfo.Status.all.makeAsyncIterator()
        while !Task.isCancelled, let next = await iterator.next() {
            queue.push(
                SKSubscriptionGroupStatusesPayload(
                    groupID: next.groupID,
                    statuses: next.statuses.map(skSubscriptionStatusPayload(from:))
                )
            )
        }
    })
}

@_cdecl("sk_subscription_group_status_stream_release")
public func sk_subscription_group_status_stream_release(_ stream: UnsafeMutableRawPointer?) {
    guard let stream else {
        return
    }
    sk_release(stream)
}

@_cdecl("sk_subscription_group_status_stream_next")
public func sk_subscription_group_status_stream_next(
    _ stream: UnsafeMutableRawPointer?,
    _ timeoutMilliseconds: Int64,
    _ outPayloadJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let stream else {
        let error = SKBridgeError.invalidArgument("missing subscription group status stream")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let box: SKStreamBox<SKSubscriptionGroupStatusesPayload> = sk_borrow(stream)
    switch box.queue.next(timeoutMilliseconds: timeoutMilliseconds) {
    case .item(let payload):
        if let json = try? skEncodeJSON(payload) {
            outPayloadJSON?.pointee = skCString(json)
            return SK_OK
        }
        let error = SKBridgeError.unknown("failed to encode subscription group status payload")
        skPopulateError(outError, with: error)
        return error.statusCode
    case .end:
        return SK_END_OF_STREAM
    case .timedOut:
        return SK_TIMED_OUT
    }
}
