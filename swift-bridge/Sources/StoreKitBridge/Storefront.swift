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

final class SKStorefrontStreamBox {
    private let stateQueue = DispatchQueue(label: "storekit.storefront-stream")
    private let semaphore = DispatchSemaphore(value: 0)
    private var queue: [SKStorefrontPayload] = []
    private var finished = false
    private var task: Task<Void, Never>?

    init() {
        task = Task { [weak self] in
            guard let self else {
                return
            }
            var iterator = Storefront.updates.makeAsyncIterator()
            while !Task.isCancelled, let next = await iterator.next() {
                let payload = skStorefrontPayload(from: next)
                self.stateQueue.sync {
                    self.queue.append(payload)
                }
                self.semaphore.signal()
            }
            self.stateQueue.sync {
                self.finished = true
            }
            self.semaphore.signal()
        }
    }

    deinit {
        task?.cancel()
        semaphore.signal()
    }

    func next(timeoutMilliseconds: UInt32) -> SKStorefrontStreamNextState {
        while true {
            let state = stateQueue.sync { () -> (SKStorefrontPayload?, Bool) in
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

enum SKStorefrontStreamNextState {
    case item(SKStorefrontPayload)
    case end
    case timedOut
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
    sk_retain(SKStorefrontStreamBox())
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
    _ timeoutMilliseconds: UInt32,
    _ outStorefrontJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let stream else {
        let error = SKBridgeError.invalidArgument("missing storefront stream")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    let box: SKStorefrontStreamBox = sk_borrow(stream)
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
