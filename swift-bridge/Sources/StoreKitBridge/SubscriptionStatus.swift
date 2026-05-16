import Foundation
import StoreKit

struct SKSubscriptionGroupStatusesPayload: Codable {
    let groupID: String
    let statuses: [SKSubscriptionStatusPayload]
}

final class SKSubscriptionStatusStreamBox {
    private let stateQueue = DispatchQueue(label: "storekit.subscription-status-stream")
    private let semaphore = DispatchSemaphore(value: 0)
    private var queue: [SKSubscriptionStatusPayload] = []
    private var finished = false
    private var task: Task<Void, Never>?

    init() {
        task = Task { [weak self] in
            guard let self else {
                return
            }
            var iterator = Product.SubscriptionInfo.Status.updates.makeAsyncIterator()
            while !Task.isCancelled, let next = await iterator.next() {
                let payload = skSubscriptionStatusPayload(from: next)
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

    func next(timeoutMilliseconds: UInt32) -> SKSubscriptionStatusNextState {
        while true {
            let state = stateQueue.sync { () -> (SKSubscriptionStatusPayload?, Bool) in
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

enum SKSubscriptionStatusNextState {
    case item(SKSubscriptionStatusPayload)
    case end
    case timedOut
}

final class SKSubscriptionGroupStatusStreamBox {
    private let stateQueue = DispatchQueue(label: "storekit.subscription-group-status-stream")
    private let semaphore = DispatchSemaphore(value: 0)
    private var queue: [SKSubscriptionGroupStatusesPayload] = []
    private var finished = false
    private var task: Task<Void, Never>?

    init() throws {
        guard #available(macOS 14.0, *) else {
            throw SKBridgeError.notSupported("Product.SubscriptionInfo.Status.all requires macOS 14.0+")
        }
        task = Task { [weak self] in
            guard let self else {
                return
            }
            for await next in Product.SubscriptionInfo.Status.all {
                let payload = SKSubscriptionGroupStatusesPayload(
                    groupID: next.groupID,
                    statuses: next.statuses.map(skSubscriptionStatusPayload(from:))
                )
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

    func next(timeoutMilliseconds: UInt32) -> SKSubscriptionGroupStatusNextState {
        while true {
            let state = stateQueue.sync { () -> (SKSubscriptionGroupStatusesPayload?, Bool) in
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

enum SKSubscriptionGroupStatusNextState {
    case item(SKSubscriptionGroupStatusesPayload)
    case end
    case timedOut
}

@_cdecl("sk_subscription_status_stream_create")
public func sk_subscription_status_stream_create(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    _ = outError
    return sk_retain(SKSubscriptionStatusStreamBox())
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
    _ timeoutMilliseconds: UInt32,
    _ outStatusJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let stream else {
        let error = SKBridgeError.invalidArgument("missing subscription status stream")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let box: SKSubscriptionStatusStreamBox = sk_borrow(stream)
    switch box.next(timeoutMilliseconds: timeoutMilliseconds) {
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
    do {
        return sk_retain(try SKSubscriptionGroupStatusStreamBox())
    } catch {
        skPopulateError(outError, with: error)
        return nil
    }
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
    _ timeoutMilliseconds: UInt32,
    _ outPayloadJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let stream else {
        let error = SKBridgeError.invalidArgument("missing subscription group status stream")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let box: SKSubscriptionGroupStatusStreamBox = sk_borrow(stream)
    switch box.next(timeoutMilliseconds: timeoutMilliseconds) {
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
