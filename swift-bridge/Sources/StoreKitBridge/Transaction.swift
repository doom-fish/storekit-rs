import Foundation
import StoreKit

struct SKTransactionStreamConfig: Decodable {
    let kind: String
    let productID: String?
}

struct SKTransactionOfferPayload: Codable {
    let id: String?
    let type: String
    let paymentMode: String?
    let period: SKSubscriptionPeriodPayload?
}

struct SKTransactionPayload: Codable {
    let id: UInt64
    let originalID: UInt64
    let webOrderLineItemID: String?
    let productID: String
    let subscriptionGroupID: String?
    let appBundleID: String
    let purchaseDate: String
    let originalPurchaseDate: String
    let expirationDate: String?
    let purchasedQuantity: UInt64
    let isUpgraded: Bool
    let ownershipType: String
    let signedDate: String
    let jwsRepresentation: String
    let verificationError: SKVerificationErrorPayload?
    let revocationDate: String?
    let revocationReason: String?
    let productType: String?
    let appAccountToken: String?
    let environment: String?
    let reason: String?
    let storefront: SKStorefrontPayload?
    let price: String?
    let currencyCode: String?
    let appTransactionID: String?
    let offer: SKTransactionOfferPayload?
    let jsonRepresentationBase64: String
}

func skOwnershipTypeName(_ type: Transaction.OwnershipType) -> String {
    switch type {
    case .purchased:
        return "purchased"
    case .familyShared:
        return "familyShared"
    default:
        return type.rawValue
    }
}

@available(macOS 14.0, *)
func skTransactionReasonName(_ reason: Transaction.Reason) -> String {
    switch reason {
    case .purchase:
        return "purchase"
    case .renewal:
        return "renewal"
    default:
        return reason.rawValue
    }
}

func skTransactionRevocationReasonName(_ reason: Transaction.RevocationReason) -> String {
    switch reason {
    case .developerIssue:
        return "developerIssue"
    case .other:
        return "other"
    default:
        return "other"
    }
}

func skTransactionOfferTypeName(_ type: Transaction.OfferType) -> String {
    if #available(macOS 15.0, *), type == .winBack {
        return "winBack"
    }
    switch type {
    case .introductory:
        return "introductory"
    case .promotional:
        return "promotional"
    case .code:
        return "code"
    default:
        return "unknown"
    }
}

@available(macOS 14.2, *)
func skTransactionOfferPaymentModeName(_ mode: Transaction.Offer.PaymentMode) -> String {
    if #available(macOS 26.0, *), mode == .oneTime {
        return "oneTime"
    }
    switch mode {
    case .freeTrial:
        return "freeTrial"
    case .payAsYouGo:
        return "payAsYouGo"
    case .payUpFront:
        return "payUpFront"
    default:
        return mode.rawValue
    }
}

@available(macOS 14.2, *)
func skTransactionOfferPayload(from offer: Transaction.Offer) -> SKTransactionOfferPayload {
    let period: SKSubscriptionPeriodPayload?
    if #available(macOS 15.4, *) {
        period = offer.period.map(skSubscriptionPeriodPayload(from:))
    } else {
        period = nil
    }
    return SKTransactionOfferPayload(
        id: offer.id,
        type: skTransactionOfferTypeName(offer.type),
        paymentMode: offer.paymentMode.map(skTransactionOfferPaymentModeName(_:)),
        period: period
    )
}

func skLegacyTransactionOfferPayload(from transaction: Transaction) -> SKTransactionOfferPayload? {
    guard let offerID = transaction.offerID else {
        return nil
    }
    let type = transaction.offerType.map(skTransactionOfferTypeName(_:)) ?? "unknown"
    let paymentMode = transaction.offerPaymentModeStringRepresentation
    return SKTransactionOfferPayload(id: offerID, type: type, paymentMode: paymentMode, period: nil)
}

func skTransactionPayload(from result: VerificationResult<Transaction>) -> SKTransactionPayload {
    let transaction = result.unsafePayloadValue
    let verificationError: SKVerificationErrorPayload?
    switch result {
    case .verified(_):
        verificationError = nil
    case .unverified(_, let error):
        verificationError = SKVerificationErrorPayload(
            code: skVerificationErrorCode(error),
            localizedDescription: error.localizedDescription
        )
    }

    let environment: String?
    if #available(macOS 13.0, *) {
        environment = transaction.environment.rawValue
    } else {
        environment = transaction.environmentStringRepresentation
    }

    let reason: String?
    if #available(macOS 14.0, *) {
        reason = skTransactionReasonName(transaction.reason)
    } else {
        reason = transaction.reasonStringRepresentation
    }

    let storefront: SKStorefrontPayload?
    if #available(macOS 14.0, *) {
        storefront = skStorefrontPayload(from: transaction.storefront)
    } else {
        storefront = nil
    }

    let offer: SKTransactionOfferPayload?
    if #available(macOS 14.2, *) {
        offer = transaction.offer.map(skTransactionOfferPayload(from:))
    } else {
        offer = skLegacyTransactionOfferPayload(from: transaction)
    }

    let price: String?
    if #available(macOS 14.2, *) {
        price = transaction.price.map { NSDecimalNumber(decimal: $0).stringValue }
    } else {
        price = nil
    }

    let currencyCode: String?
    if #available(macOS 13.0, *) {
        currencyCode = transaction.currencyCode
    } else {
        currencyCode = nil
    }

    return SKTransactionPayload(
        id: transaction.id,
        originalID: transaction.originalID,
        webOrderLineItemID: transaction.webOrderLineItemID,
        productID: transaction.productID,
        subscriptionGroupID: transaction.subscriptionGroupID,
        appBundleID: transaction.appBundleID,
        purchaseDate: skFormatDate(transaction.purchaseDate),
        originalPurchaseDate: skFormatDate(transaction.originalPurchaseDate),
        expirationDate: transaction.expirationDate.map(skFormatDate),
        purchasedQuantity: UInt64(transaction.purchasedQuantity),
        isUpgraded: transaction.isUpgraded,
        ownershipType: skOwnershipTypeName(transaction.ownershipType),
        signedDate: skFormatDate(result.signedDate),
        jwsRepresentation: result.jwsRepresentation,
        verificationError: verificationError,
        revocationDate: transaction.revocationDate.map(skFormatDate),
        revocationReason: transaction.revocationReason.map(skTransactionRevocationReasonName(_:)),
        productType: skProductTypeName(transaction.productType),
        appAccountToken: transaction.appAccountToken?.uuidString,
        environment: environment,
        reason: reason,
        storefront: storefront,
        price: price,
        currencyCode: currencyCode,
        appTransactionID: transaction.appTransactionID,
        offer: offer,
        jsonRepresentationBase64: skDataBase64(transaction.jsonRepresentation)
    )
}

final class SKTransactionBox {
    let result: VerificationResult<Transaction>

    init(result: VerificationResult<Transaction>) {
        self.result = result
    }

    func verifiedTransaction() throws -> Transaction {
        try result.payloadValue
    }
}

final class SKTransactionStreamBox {
    private let stateQueue = DispatchQueue(label: "storekit.transaction-stream")
    private let semaphore = DispatchSemaphore(value: 0)
    private var queue: [SKTransactionBox] = []
    private var finished = false
    private var task: Task<Void, Never>?

    init(config: SKTransactionStreamConfig) throws {
        guard ["all", "currentEntitlements", "updates", "unfinished", "allFor", "currentEntitlementsFor"].contains(config.kind) else {
            throw SKBridgeError.invalidArgument("unknown transaction stream kind '\(config.kind)'")
        }
        task = Task { [weak self] in
            guard let self else {
                return
            }
            switch config.kind {
            case "all":
                await self.consume(sequence: Transaction.all, filterProductID: nil)
            case "currentEntitlements":
                await self.consume(sequence: Transaction.currentEntitlements, filterProductID: nil)
            case "updates":
                await self.consume(sequence: Transaction.updates, filterProductID: nil)
            case "unfinished":
                await self.consume(sequence: Transaction.unfinished, filterProductID: nil)
            case "allFor":
                await self.consume(sequence: Transaction.all, filterProductID: config.productID)
            case "currentEntitlementsFor":
                await self.consume(sequence: Transaction.currentEntitlements, filterProductID: config.productID)
            default:
                self.finishStream()
            }
        }
    }

    deinit {
        task?.cancel()
        semaphore.signal()
    }

    private func consume(
        sequence: Transaction.Transactions,
        filterProductID: String?
    ) async {
        var iterator = sequence.makeAsyncIterator()
        while !Task.isCancelled, let next = await iterator.next() {
            if let filterProductID, next.unsafePayloadValue.productID != filterProductID {
                continue
            }
            let box = SKTransactionBox(result: next)
            stateQueue.sync {
                queue.append(box)
            }
            semaphore.signal()
        }
        finishStream()
    }

    private func finishStream() {
        stateQueue.sync {
            finished = true
        }
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

enum SKTransactionStreamNextState {
    case item(SKTransactionBox)
    case end
    case timedOut
}

@_cdecl("sk_transaction_stream_create")
public func sk_transaction_stream_create(
    _ configJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let config = try skDecodeJSON(configJSON, as: SKTransactionStreamConfig.self)
        let stream = try SKTransactionStreamBox(config: config)
        return sk_retain(stream)
    } catch {
        skPopulateError(outError, with: error)
        return nil
    }
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
    _ outVerificationJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
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
        if let json = try? skEncodeJSON(skTransactionVerificationResultPayload(from: transactionBox.result)) {
            outVerificationJSON?.pointee = skCString(json)
            return SK_OK
        }
        let error = SKBridgeError.unknown("failed to encode transaction verification payload")
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

@_cdecl("sk_transaction_latest_for")
public func sk_transaction_latest_for(
    _ productID: UnsafePointer<CChar>?,
    _ outTransaction: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let productID else {
        let error = SKBridgeError.invalidArgument("missing product identifier")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let productIDString = String(cString: productID)
    return skBlockOnAsync(
        work: {
            guard let result = await Transaction.latest(for: productIDString) else {
                return nil as String?
            }
            let box = SKTransactionBox(result: result)
            outTransaction?.pointee = sk_retain(box)
            return try skEncodeJSON(skTransactionVerificationResultPayload(from: result))
        },
        onSuccess: { json in
            if let json {
                outResultJSON?.pointee = skCString(json)
            }
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}

@_cdecl("sk_transaction_current_entitlement_for")
public func sk_transaction_current_entitlement_for(
    _ productID: UnsafePointer<CChar>?,
    _ outTransaction: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outResultJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let productID else {
        let error = SKBridgeError.invalidArgument("missing product identifier")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    let productIDString = String(cString: productID)
    return skBlockOnAsync(
        work: {
            guard let result = await Transaction.currentEntitlement(for: productIDString) else {
                return nil as String?
            }
            let box = SKTransactionBox(result: result)
            outTransaction?.pointee = sk_retain(box)
            return try skEncodeJSON(skTransactionVerificationResultPayload(from: result))
        },
        onSuccess: { json in
            if let json {
                outResultJSON?.pointee = skCString(json)
            }
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}
