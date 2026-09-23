import AppKit
import Foundation
import StoreKit

let SK_OK: Int32 = 0
let SK_END_OF_STREAM: Int32 = 1
let SK_INVALID_ARGUMENT: Int32 = -1
let SK_TIMED_OUT: Int32 = -2
let SK_NOT_SUPPORTED: Int32 = -3
let SK_FRAMEWORK_ERROR: Int32 = -4
let SK_VERIFICATION_ERROR: Int32 = -5
let SK_UNKNOWN: Int32 = -99

private let skDateFormatter: ISO8601DateFormatter = {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return formatter
}()

@inline(__always)
func skCString(_ string: String) -> UnsafeMutablePointer<CChar>? {
    string.withCString { strdup($0) }
}

@inline(__always)
func skDataBase64(_ data: Data) -> String {
    data.base64EncodedString()
}

@inline(__always)
func sk_retain<T: AnyObject>(_ object: T) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(object).toOpaque()
}

@inline(__always)
func sk_borrow<T: AnyObject>(_ ptr: UnsafeMutableRawPointer, as _: T.Type = T.self) -> T {
    Unmanaged<T>.fromOpaque(ptr).takeUnretainedValue()
}

@inline(__always)
func sk_release(_ ptr: UnsafeMutableRawPointer) {
    Unmanaged<AnyObject>.fromOpaque(ptr).release()
}

@_cdecl("sk_string_free")
public func sk_string_free(_ ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}

enum SKBridgeError: Error, CustomStringConvertible {
    case invalidArgument(String)
    case timedOut(String)
    case notSupported(String)
    case unknown(String)

    var statusCode: Int32 {
        switch self {
        case .invalidArgument:
            return SK_INVALID_ARGUMENT
        case .timedOut:
            return SK_TIMED_OUT
        case .notSupported:
            return SK_NOT_SUPPORTED
        case .unknown:
            return SK_UNKNOWN
        }
    }

    var description: String {
        switch self {
        case .invalidArgument(let message),
             .timedOut(let message),
             .notSupported(let message),
             .unknown(let message):
            return message
        }
    }
}

struct SKFrameworkErrorPayload: Encodable {
    let kind = "framework"
    let domain: String
    let code: Int
    let localizedDescription: String
}

struct SKTypedStoreKitErrorPayload: Encodable {
    let kind = "storekitError"
    let code: String
    let errorDescription: String?
    let failureReason: String?
    let recoverySuggestion: String?
    let underlyingDomain: String?
    let underlyingCode: Int?
    let underlyingDescription: String?
}

struct SKProductPurchaseErrorPayload: Encodable {
    let kind = "purchaseError"
    let code: String
    let errorDescription: String?
    let failureReason: String?
    let recoverySuggestion: String?
}

struct SKRefundRequestErrorPayload: Encodable {
    let kind = "refundRequestError"
    let code: String
    let errorDescription: String?
    let failureReason: String?
    let recoverySuggestion: String?
}

struct SKInvalidRequestErrorPayload: Encodable {
    let kind = "invalidRequestError"
    let code: Int64
    let message: String
}

struct SKVerificationErrorPayload: Codable {
    let kind: String
    let code: String
    let localizedDescription: String

    init(code: String, localizedDescription: String) {
        kind = "verification"
        self.code = code
        self.localizedDescription = localizedDescription
    }
}

func skStoreKitErrorCode(_ error: StoreKit.StoreKitError) -> String {
    switch error {
    case .unknown:
        return "unknown"
    case .userCancelled:
        return "userCancelled"
    case .networkError:
        return "networkError"
    case .systemError:
        return "systemError"
    case .notAvailableInStorefront:
        return "notAvailableInStorefront"
    @unknown default:
        if #available(macOS 15.4, *), case .unsupported = error {
            return "unsupported"
        }
        if #available(macOS 12.3, *), case .notEntitled = error {
            return "notEntitled"
        }
        return "unknown"
    }
}

func skProductPurchaseErrorCode(_ error: Product.PurchaseError) -> String {
    switch error {
    case .invalidQuantity:
        return "invalidQuantity"
    case .productUnavailable:
        return "productUnavailable"
    case .purchaseNotAllowed:
        return "purchaseNotAllowed"
    case .ineligibleForOffer:
        return "ineligibleForOffer"
    case .invalidOfferIdentifier:
        return "invalidOfferIdentifier"
    case .invalidOfferPrice:
        return "invalidOfferPrice"
    case .invalidOfferSignature:
        return "invalidOfferSignature"
    case .missingOfferParameters:
        return "missingOfferParameters"
    @unknown default:
        return "unknown"
    }
}

func skRefundRequestErrorCode(_ error: Transaction.RefundRequestError) -> String {
    switch error {
    case .duplicateRequest:
        return "duplicateRequest"
    case .failed:
        return "failed"
    @unknown default:
        return "unknown"
    }
}

func skLocalizedErrorParts(_ error: any LocalizedError) -> (String?, String?, String?) {
    (error.errorDescription, error.failureReason, error.recoverySuggestion)
}

func skStatus(for error: Error) -> Int32 {
    if let bridgeError = error as? SKBridgeError {
        return bridgeError.statusCode
    }
    if error is StoreKit.VerificationResult<Transaction>.VerificationError
        || error is StoreKit.VerificationResult<Product.SubscriptionInfo.RenewalInfo>.VerificationError
    {
        return SK_VERIFICATION_ERROR
    }
    if #available(macOS 13.0, *), error is StoreKit.VerificationResult<AppTransaction>.VerificationError {
        return SK_VERIFICATION_ERROR
    }
    return SK_FRAMEWORK_ERROR
}

func skEncodeJSON<T: Encodable>(_ value: T) throws -> String {
    let encoder = JSONEncoder()
    let data = try encoder.encode(value)
    guard let string = String(data: data, encoding: .utf8) else {
        throw SKBridgeError.unknown("failed to encode JSON as UTF-8")
    }
    return string
}

func skDecodeJSON<T: Decodable>(_ cString: UnsafePointer<CChar>?, as type: T.Type) throws -> T {
    guard let cString else {
        throw SKBridgeError.invalidArgument("missing JSON payload")
    }
    let data = Data(String(cString: cString).utf8)
    do {
        return try JSONDecoder().decode(T.self, from: data)
    } catch {
        throw SKBridgeError.invalidArgument("invalid JSON payload: \(error.localizedDescription)")
    }
}

func skDecodeJSONIfPresent<T: Decodable>(_ cString: UnsafePointer<CChar>?, as type: T.Type) throws -> T? {
    guard cString != nil else {
        return nil
    }
    return try skDecodeJSON(cString, as: type)
}

func skPopulateError(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    with error: Error
) {
    outError?.pointee = skCString(skErrorMessage(for: error))
}

func skErrorMessage(for error: Error) -> String {
    let message: String
    if let bridgeError = error as? SKBridgeError {
        message = bridgeError.description
    } else if let verificationError = error as? StoreKit.VerificationResult<Transaction>.VerificationError {
        let payload = SKVerificationErrorPayload(
            code: skVerificationErrorCode(verificationError),
            localizedDescription: verificationError.localizedDescription
        )
        message = (try? skEncodeJSON(payload)) ?? verificationError.localizedDescription
    } else if #available(macOS 13.0, *), let verificationError = error as? StoreKit.VerificationResult<AppTransaction>.VerificationError {
        let payload = SKVerificationErrorPayload(
            code: skVerificationErrorCode(verificationError),
            localizedDescription: verificationError.localizedDescription
        )
        message = (try? skEncodeJSON(payload)) ?? verificationError.localizedDescription
    } else if let verificationError = error as? StoreKit.VerificationResult<Product.SubscriptionInfo.RenewalInfo>.VerificationError {
        let payload = SKVerificationErrorPayload(
            code: skVerificationErrorCode(verificationError),
            localizedDescription: verificationError.localizedDescription
        )
        message = (try? skEncodeJSON(payload)) ?? verificationError.localizedDescription
    } else if #available(macOS 12.3, *), let storeKitError = error as? StoreKit.StoreKitError {
        let localized = skLocalizedErrorParts(storeKitError)
        let nsError = error as NSError
        let underlyingError = nsError.userInfo[NSUnderlyingErrorKey] as? NSError
        let payload = SKTypedStoreKitErrorPayload(
            code: skStoreKitErrorCode(storeKitError),
            errorDescription: localized.0,
            failureReason: localized.1,
            recoverySuggestion: localized.2,
            underlyingDomain: underlyingError?.domain,
            underlyingCode: underlyingError?.code,
            underlyingDescription: underlyingError?.localizedDescription
        )
        message = (try? skEncodeJSON(payload)) ?? nsError.localizedDescription
    } else if #available(macOS 12.3, *), let purchaseError = error as? Product.PurchaseError {
        let localized = skLocalizedErrorParts(purchaseError)
        let payload = SKProductPurchaseErrorPayload(
            code: skProductPurchaseErrorCode(purchaseError),
            errorDescription: localized.0,
            failureReason: localized.1,
            recoverySuggestion: localized.2
        )
        message = (try? skEncodeJSON(payload)) ?? purchaseError.localizedDescription
    } else if #available(macOS 12.3, *), let refundError = error as? Transaction.RefundRequestError {
        let localized = skLocalizedErrorParts(refundError)
        let payload = SKRefundRequestErrorPayload(
            code: skRefundRequestErrorCode(refundError),
            errorDescription: localized.0,
            failureReason: localized.1,
            recoverySuggestion: localized.2
        )
        message = (try? skEncodeJSON(payload)) ?? refundError.localizedDescription
    } else if #available(macOS 15.4, *), let invalidRequestError = error as? InvalidRequestError {
        let payload = SKInvalidRequestErrorPayload(
            code: invalidRequestError.code,
            message: invalidRequestError.message
        )
        message = (try? skEncodeJSON(payload)) ?? invalidRequestError.localizedDescription
    } else {
        let nsError = error as NSError
        let payload = SKFrameworkErrorPayload(
            domain: nsError.domain,
            code: nsError.code,
            localizedDescription: nsError.localizedDescription
        )
        message = (try? skEncodeJSON(payload)) ?? nsError.localizedDescription
    }
    return message
}

let SK_TIMEOUT_SECONDS = 30
let SK_UI_START_TIMEOUT_SECONDS = 10
let SK_UI_TIMEOUT_SECONDS = 600

enum SKAwaitOutcome<Value> {
    case finished(Result<Value, Error>)
    case notStarted
    case timedOut
}

final class SKAwaitState<Value> {
    private let condition = NSCondition()
    private var started = false
    private var abandoned = false
    private var result: Result<Value, Error>?

    func begin() -> Bool {
        condition.lock()
        defer { condition.unlock() }
        if abandoned {
            return false
        }
        started = true
        condition.broadcast()
        return true
    }

    func complete(_ value: Result<Value, Error>) {
        condition.lock()
        if result == nil {
            result = value
        }
        condition.broadcast()
        condition.unlock()
    }

    func wait(startDeadline: Date?, deadline: Date) -> SKAwaitOutcome<Value> {
        condition.lock()
        defer { condition.unlock() }
        while true {
            if let result {
                return .finished(result)
            }
            let now = Date()
            if !started, let startDeadline, now >= startDeadline {
                abandoned = true
                return .notStarted
            }
            if now >= deadline {
                abandoned = true
                return .timedOut
            }
            if !started, let startDeadline {
                _ = condition.wait(until: min(startDeadline, deadline))
            } else {
                _ = condition.wait(until: deadline)
            }
        }
    }
}

func skAwait<T>(
    label: String,
    timeoutSeconds: Int,
    work: @escaping () async throws -> T
) -> Result<T, Error> {
    let state = SKAwaitState<T>()
    let task = Task {
        guard state.begin() else {
            return
        }
        do {
            state.complete(.success(try await work()))
        } catch {
            state.complete(.failure(error))
        }
    }
    switch state.wait(startDeadline: nil, deadline: Date(timeIntervalSinceNow: TimeInterval(timeoutSeconds))) {
    case .finished(let result):
        return result
    case .notStarted, .timedOut:
        task.cancel()
        return .failure(SKBridgeError.timedOut("\(label) timed out after \(timeoutSeconds) seconds and was cancelled"))
    }
}

func skAwaitMainActor<T>(
    label: String,
    work: @escaping @MainActor () async throws -> T
) -> Result<T, Error> {
    guard !Thread.isMainThread else {
        return .failure(
            SKBridgeError.notSupported(
                "\(label) presents StoreKit UI on the main actor and would block the main thread; call it from another thread or use the async API"
            )
        )
    }
    let state = SKAwaitState<T>()
    let task = Task { @MainActor in
        guard state.begin() else {
            return
        }
        do {
            state.complete(.success(try await work()))
        } catch {
            state.complete(.failure(error))
        }
    }
    let now = Date()
    let outcome = state.wait(
        startDeadline: now.addingTimeInterval(TimeInterval(SK_UI_START_TIMEOUT_SECONDS)),
        deadline: now.addingTimeInterval(TimeInterval(SK_UI_TIMEOUT_SECONDS))
    )
    switch outcome {
    case .finished(let result):
        return result
    case .notStarted:
        task.cancel()
        return .failure(
            SKBridgeError.timedOut(
                "\(label) did not start within \(SK_UI_START_TIMEOUT_SECONDS) seconds because the main thread is not running its run loop; no StoreKit UI was presented"
            )
        )
    case .timedOut:
        task.cancel()
        return .failure(
            SKBridgeError.timedOut(
                "\(label) did not finish within \(SK_UI_TIMEOUT_SECONDS) seconds and was cancelled; a purchase that completes later is delivered through Transaction.updates"
            )
        )
    }
}

func skComplete<T>(
    _ result: Result<T, Error>,
    onSuccess: (T) -> Void,
    onError: (Error) -> Void
) -> Int32 {
    switch result {
    case .success(let value):
        onSuccess(value)
        return SK_OK
    case .failure(let error):
        onError(error)
        return skStatus(for: error)
    }
}

func skBlockOnAsync<T>(
    label: String = "StoreKit operation",
    timeoutSeconds: Int = SK_TIMEOUT_SECONDS,
    work: @escaping () async throws -> T,
    onSuccess: (T) -> Void,
    onError: (Error) -> Void
) -> Int32 {
    skComplete(
        skAwait(label: label, timeoutSeconds: timeoutSeconds, work: work),
        onSuccess: onSuccess,
        onError: onError
    )
}

func skBlockOnMainActorAsync<T>(
    label: String,
    work: @escaping @MainActor () async throws -> T,
    onSuccess: (T) -> Void,
    onError: (Error) -> Void
) -> Int32 {
    skComplete(
        skAwaitMainActor(label: label, work: work),
        onSuccess: onSuccess,
        onError: onError
    )
}

enum SKStreamNext<Element> {
    case item(Element)
    case end
    case timedOut
}

final class SKStreamQueue<Element> {
    private let condition = NSCondition()
    private var items: [Element] = []
    private var finished = false

    func push(_ element: Element) {
        condition.lock()
        if !finished {
            items.append(element)
        }
        condition.broadcast()
        condition.unlock()
    }

    func finish() {
        condition.lock()
        finished = true
        condition.broadcast()
        condition.unlock()
    }

    func next(timeoutMilliseconds: Int64) -> SKStreamNext<Element> {
        let deadline = timeoutMilliseconds < 0
            ? nil
            : min(Date(timeIntervalSinceNow: TimeInterval(timeoutMilliseconds) / 1000), Date.distantFuture)
        condition.lock()
        defer { condition.unlock() }
        while true {
            if !items.isEmpty {
                return .item(items.removeFirst())
            }
            if finished {
                return .end
            }
            if let deadline {
                if Date() >= deadline {
                    return .timedOut
                }
                _ = condition.wait(until: deadline)
            } else {
                condition.wait()
            }
        }
    }
}

final class SKStreamBox<Element> {
    private let queue = SKStreamQueue<Element>()
    private var task: Task<Void, Never>?

    init(produce: @escaping (SKStreamQueue<Element>) async -> Void) {
        let queue = self.queue
        task = Task {
            await produce(queue)
            queue.finish()
        }
    }

    deinit {
        task?.cancel()
        queue.finish()
    }

    func next(timeoutMilliseconds: Int64) -> SKStreamNext<Element> {
        queue.next(timeoutMilliseconds: timeoutMilliseconds)
    }
}

func skFormatDate(_ date: Date) -> String {
    skDateFormatter.string(from: date)
}

@MainActor
func skKeyWindowController() -> NSViewController? {
    let windows = NSApplication.shared.windows
    return windows.first(where: { $0.isKeyWindow })?.contentViewController
        ?? windows.first(where: { $0.isVisible })?.contentViewController
        ?? windows.first?.contentViewController
}

func skBorrowWindow(_ ptr: UnsafeMutableRawPointer?, context: String) throws -> NSWindow {
    guard let ptr else {
        throw SKBridgeError.invalidArgument("missing NSWindow for \(context)")
    }
    return Unmanaged<NSWindow>.fromOpaque(ptr).takeUnretainedValue()
}
