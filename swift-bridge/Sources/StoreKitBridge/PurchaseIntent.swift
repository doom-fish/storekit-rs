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

final class SKPurchaseIntentStreamBox {
    private let stateQueue = DispatchQueue(label: "storekit.purchase-intent-stream")
    private let semaphore = DispatchSemaphore(value: 0)
    private var queue: [SKPurchaseIntentPayload] = []
    private var finished = false
    private var task: Task<Void, Never>?

    init() throws {
        guard #available(macOS 14.4, *) else {
            throw SKBridgeError.notSupported("PurchaseIntent.intents requires macOS 14.4+")
        }
        task = Task { [weak self] in
            guard let self else {
                return
            }
            var iterator = PurchaseIntent.intents.makeAsyncIterator()
            while !Task.isCancelled, let next = await iterator.next() {
                let payload = skPurchaseIntentPayload(from: next)
                stateQueue.sync {
                    self.queue.append(payload)
                }
                semaphore.signal()
            }
            finishStream()
        }
    }

    deinit {
        task?.cancel()
        semaphore.signal()
    }

    private func finishStream() {
        stateQueue.sync {
            finished = true
        }
        semaphore.signal()
    }

    func next(timeoutMilliseconds: UInt32) -> SKPurchaseIntentNextState {
        while true {
            let state = stateQueue.sync { () -> (SKPurchaseIntentPayload?, Bool) in
                if queue.isEmpty {
                    return (nil, finished)
                }
                return (queue.removeFirst(), finished)
            }

            if let next = state.0 {
                return .item(next)
            }
            if state.1 {
                return .end
            }

            let timeout = DispatchTime.now() + .milliseconds(Int(timeoutMilliseconds))
            if semaphore.wait(timeout: timeout) == .timedOut {
                return .timedOut
            }
        }
    }
}

enum SKPurchaseIntentNextState {
    case item(SKPurchaseIntentPayload)
    case end
    case timedOut
}

@_cdecl("sk_purchase_intent_stream_create")
public func sk_purchase_intent_stream_create(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        return sk_retain(try SKPurchaseIntentStreamBox())
    } catch {
        skPopulateError(outError, with: error)
        return nil
    }
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
    _ timeoutMilliseconds: UInt32,
    _ outPayloadJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let stream else {
        let error = SKBridgeError.invalidArgument("missing purchase intent stream")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let box: SKPurchaseIntentStreamBox = sk_borrow(stream)
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
