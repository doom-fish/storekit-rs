import Foundation
import StoreKit

final class SKTransactionBox {
    let result: VerificationResult<Transaction>
    let payload: SKTransactionPayload

    init(result: VerificationResult<Transaction>) {
        self.result = result
        self.payload = skTransactionPayload(from: result)
    }

    func verifiedTransaction() throws -> Transaction {
        try result.payloadValue
    }
}

enum SKTransactionStreamKind: Int32 {
    case all = 0
    case currentEntitlements = 1
    case updates = 2
}

enum SKTransactionStreamNextState {
    case item(SKTransactionBox)
    case end
    case timedOut
}

final class SKTransactionStreamBox {
    private let stateQueue = DispatchQueue(label: "storekit.transaction-stream")
    private let semaphore = DispatchSemaphore(value: 0)
    private var queue: [SKTransactionBox] = []
    private var finished = false
    private var task: Task<Void, Never>?

    init(kind: SKTransactionStreamKind) {
        let sequence: Transaction.Transactions
        switch kind {
        case .all:
            sequence = Transaction.all
        case .currentEntitlements:
            sequence = Transaction.currentEntitlements
        case .updates:
            sequence = Transaction.updates
        }

        task = Task { [weak self] in
            guard let self else {
                return
            }
            var iterator = sequence.makeAsyncIterator()
            while !Task.isCancelled, let next = await iterator.next() {
                let box = SKTransactionBox(result: next)
                self.stateQueue.sync {
                    self.queue.append(box)
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

    func next(timeoutMilliseconds: UInt32) -> SKTransactionStreamNextState {
        while true {
            let state = stateQueue.sync { () -> (SKTransactionBox?, Bool) in
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

@_cdecl("sk_transaction_stream_create")
public func sk_transaction_stream_create(
    _ rawKind: Int32,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let kind = SKTransactionStreamKind(rawValue: rawKind) else {
        let error = SKBridgeError.invalidArgument("unknown StoreKit transaction stream kind \(rawKind)")
        skPopulateError(outError, with: error)
        return nil
    }
    let stream = SKTransactionStreamBox(kind: kind)
    return sk_retain(stream)
}

@_cdecl("sk_transaction_stream_release")
public func sk_transaction_stream_release(_ stream: UnsafeMutableRawPointer?) {
    guard let stream else {
        return
    }
    sk_release(stream)
}

@_cdecl("sk_transaction_stream_next")
public func sk_transaction_stream_next(
    _ stream: UnsafeMutableRawPointer?,
    _ timeoutMilliseconds: UInt32,
    _ outTransaction: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outTransactionJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let stream else {
        let error = SKBridgeError.invalidArgument("missing transaction stream")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let box: SKTransactionStreamBox = sk_borrow(stream)
    switch box.next(timeoutMilliseconds: timeoutMilliseconds) {
    case .item(let transactionBox):
        outTransaction?.pointee = sk_retain(transactionBox)
        if let json = try? skEncodeJSON(transactionBox.payload) {
            outTransactionJSON?.pointee = skCString(json)
            return SK_OK
        }
        let error = SKBridgeError.unknown("failed to encode transaction payload")
        skPopulateError(outError, with: error)
        return error.statusCode
    case .end:
        return SK_END_OF_STREAM
    case .timedOut:
        return SK_TIMED_OUT
    }
}

@_cdecl("sk_transaction_retain")
public func sk_transaction_retain(_ transaction: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let transaction else {
        return nil
    }
    let box: SKTransactionBox = sk_borrow(transaction)
    return sk_retain(box)
}

@_cdecl("sk_transaction_release")
public func sk_transaction_release(_ transaction: UnsafeMutableRawPointer?) {
    guard let transaction else {
        return
    }
    sk_release(transaction)
}

@_cdecl("sk_transaction_verify")
public func sk_transaction_verify(
    _ transaction: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let transaction else {
        let error = SKBridgeError.invalidArgument("missing transaction handle")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let box: SKTransactionBox = sk_borrow(transaction)
    do {
        _ = try box.verifiedTransaction()
        return SK_OK
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }
}

@_cdecl("sk_transaction_finish")
public func sk_transaction_finish(
    _ transaction: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let transaction else {
        let error = SKBridgeError.invalidArgument("missing transaction handle")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let box: SKTransactionBox = sk_borrow(transaction)
    return skBlockOnAsync(
        work: {
            let verifiedTransaction = try box.verifiedTransaction()
            await verifiedTransaction.finish()
        },
        onSuccess: { (_: Void) in },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
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
