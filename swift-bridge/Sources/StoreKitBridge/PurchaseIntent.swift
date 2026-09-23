import Foundation
import StoreKit

struct SKPurchaseIntentPayload: Codable {
    let product: SKProductPayload
    let offer: SKSubscriptionOfferPayload?
}

@available(macOS 14.4, *)
func skPurchaseIntentPayload(from intent: PurchaseIntent) -> SKPurchaseIntentPayload {
    let offer: SKSubscriptionOfferPayload?
    if #available(macOS 15.0, *) {
        offer = intent.offer.map(skSubscriptionOfferPayload(from:))
    } else {
        offer = nil
    }
    return SKPurchaseIntentPayload(product: skProductPayload(from: intent.product), offer: offer)
}

@_cdecl("sk_purchase_intent_stream_create")
public func sk_purchase_intent_stream_create(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 14.4, *) else {
        skPopulateError(outError, with: SKBridgeError.notSupported("PurchaseIntent.intents requires macOS 14.4+"))
        return nil
    }
    return sk_retain(SKStreamBox<SKPurchaseIntentPayload> { queue in
        var iterator = PurchaseIntent.intents.makeAsyncIterator()
        while !Task.isCancelled, let next = await iterator.next() {
            queue.push(skPurchaseIntentPayload(from: next))
        }
    })
}

@_cdecl("sk_purchase_intent_stream_release")
public func sk_purchase_intent_stream_release(_ stream: UnsafeMutableRawPointer?) {
    guard let stream else {
        return
    }
    sk_release(stream)
}

@_cdecl("sk_purchase_intent_stream_next")
public func sk_purchase_intent_stream_next(
    _ stream: UnsafeMutableRawPointer?,
    _ timeoutMilliseconds: Int64,
    _ outPayloadJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let stream else {
        let error = SKBridgeError.invalidArgument("missing purchase intent stream")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let box: SKStreamBox<SKPurchaseIntentPayload> = sk_borrow(stream)
    switch box.next(timeoutMilliseconds: timeoutMilliseconds) {
    case .item(let payload):
        if let json = try? skEncodeJSON(payload) {
            outPayloadJSON?.pointee = skCString(json)
            return SK_OK
        }
        let error = SKBridgeError.unknown("failed to encode purchase intent payload")
        skPopulateError(outError, with: error)
        return error.statusCode
    case .end:
        return SK_END_OF_STREAM
    case .timedOut:
        return SK_TIMED_OUT
    }
}
