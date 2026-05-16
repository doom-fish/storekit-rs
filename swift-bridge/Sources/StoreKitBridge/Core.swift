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

struct SKSubscriptionPeriodPayload: Codable {
    let unit: String
    let value: Int
}

struct SKSubscriptionInfoPayload: Codable {
    let subscriptionGroupID: String
    let subscriptionPeriod: SKSubscriptionPeriodPayload
}

struct SKProductPayload: Codable {
    let id: String
    let displayName: String
    let description: String
    let price: String
    let displayPrice: String
    let type: String
    let subscription: SKSubscriptionInfoPayload?
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
}

struct SKPurchaseOptionPayload: Codable {
    let kind: String
    let appAccountToken: String?
    let quantity: Int?
    let simulateAskToBuyInSandbox: Bool?
}

struct SKPurchaseResultPayload: Codable {
    let kind: String
    let transaction: SKTransactionPayload?
}

func skStatus(for error: Error) -> Int32 {
    if let bridgeError = error as? SKBridgeError {
        return bridgeError.statusCode
    }
    if error is StoreKit.VerificationResult<Transaction>.VerificationError {
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
    let message: String
    if let bridgeError = error as? SKBridgeError {
        message = bridgeError.description
    } else if let verificationError = error as? StoreKit.VerificationResult<Transaction>.VerificationError {
        let payload = SKVerificationErrorPayload(
            code: skVerificationErrorCode(verificationError),
            localizedDescription: verificationError.localizedDescription
        )
        message = (try? skEncodeJSON(payload)) ?? verificationError.localizedDescription
    } else {
        let nsError = error as NSError
        let payload = SKFrameworkErrorPayload(
            domain: nsError.domain,
            code: nsError.code,
            localizedDescription: nsError.localizedDescription
        )
        message = (try? skEncodeJSON(payload)) ?? nsError.localizedDescription
    }
    outError?.pointee = skCString(message)
}

func skBlockOnAsync<T>(
    timeoutSeconds: Int = 30,
    work: @escaping () async throws -> T,
    onSuccess: @escaping (T) -> Void,
    onError: @escaping (Error) -> Void
) -> Int32 {
    let semaphore = DispatchSemaphore(value: 0)
    var result: Result<T, Error>?

    Task {
        do {
            result = .success(try await work())
        } catch {
            result = .failure(error)
        }
        semaphore.signal()
    }

    guard semaphore.wait(timeout: .now() + .seconds(timeoutSeconds)) == .success else {
        onError(SKBridgeError.timedOut("StoreKit operation timed out after \(timeoutSeconds) seconds"))
        return SK_TIMED_OUT
    }

    switch result {
    case .success(let value):
        onSuccess(value)
        return SK_OK
    case .failure(let error):
        onError(error)
        return skStatus(for: error)
    case .none:
        let error = SKBridgeError.unknown("StoreKit operation completed without a result")
        onError(error)
        return error.statusCode
    }
}

func skBlockOnMainActorAsync<T>(
    timeoutSeconds: Int = 30,
    work: @escaping @MainActor () async throws -> T,
    onSuccess: @escaping (T) -> Void,
    onError: @escaping (Error) -> Void
) -> Int32 {
    let semaphore = DispatchSemaphore(value: 0)
    var result: Result<T, Error>?

    Task { @MainActor in
        do {
            result = .success(try await work())
        } catch {
            result = .failure(error)
        }
        semaphore.signal()
    }

    guard semaphore.wait(timeout: .now() + .seconds(timeoutSeconds)) == .success else {
        onError(SKBridgeError.timedOut("StoreKit operation timed out after \(timeoutSeconds) seconds"))
        return SK_TIMED_OUT
    }

    switch result {
    case .success(let value):
        onSuccess(value)
        return SK_OK
    case .failure(let error):
        onError(error)
        return skStatus(for: error)
    case .none:
        let error = SKBridgeError.unknown("StoreKit operation completed without a result")
        onError(error)
        return error.statusCode
    }
}

func skFormatDate(_ date: Date) -> String {
    skDateFormatter.string(from: date)
}

func skProductTypeName(_ type: Product.ProductType) -> String {
    switch type {
    case .consumable:
        return "consumable"
    case .nonConsumable:
        return "nonConsumable"
    case .autoRenewable:
        return "autoRenewable"
    case .nonRenewable:
        return "nonRenewing"
    default:
        return type.rawValue
    }
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

func skSubscriptionPeriodPayload(from period: Product.SubscriptionPeriod) -> SKSubscriptionPeriodPayload {
    let unit: String
    switch period.unit {
    case .day:
        unit = "day"
    case .week:
        unit = "week"
    case .month:
        unit = "month"
    case .year:
        unit = "year"
    @unknown default:
        unit = "unknown"
    }
    return SKSubscriptionPeriodPayload(unit: unit, value: period.value)
}

func skSubscriptionInfoPayload(from info: Product.SubscriptionInfo) -> SKSubscriptionInfoPayload {
    SKSubscriptionInfoPayload(
        subscriptionGroupID: info.subscriptionGroupID,
        subscriptionPeriod: skSubscriptionPeriodPayload(from: info.subscriptionPeriod)
    )
}

func skProductPayload(from product: Product) -> SKProductPayload {
    SKProductPayload(
        id: product.id,
        displayName: product.displayName,
        description: product.description,
        price: NSDecimalNumber(decimal: product.price).stringValue,
        displayPrice: product.displayPrice,
        type: skProductTypeName(product.type),
        subscription: product.subscription.map(skSubscriptionInfoPayload(from:))
    )
}

func skVerificationErrorCode(_ error: StoreKit.VerificationResult<Transaction>.VerificationError) -> String {
    switch error {
    case .revokedCertificate:
        return "revokedCertificate"
    case .invalidCertificateChain:
        return "invalidCertificateChain"
    case .invalidDeviceVerification:
        return "invalidDeviceVerification"
    case .invalidEncoding:
        return "invalidEncoding"
    case .invalidSignature:
        return "invalidSignature"
    case .missingRequiredProperties:
        return "missingRequiredProperties"
    @unknown default:
        return "unknown"
    }
}

func skTransactionPayload(from result: VerificationResult<Transaction>) -> SKTransactionPayload {
    let transaction = result.unsafePayloadValue
    let verificationError: SKVerificationErrorPayload?
    switch result {
    case .verified:
        verificationError = nil
    case .unverified(_, let error):
        verificationError = SKVerificationErrorPayload(
            code: skVerificationErrorCode(error),
            localizedDescription: error.localizedDescription
        )
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
        verificationError: verificationError
    )
}

func skBuildPurchaseOptions(from payloads: [SKPurchaseOptionPayload]) throws -> Set<Product.PurchaseOption> {
    var options = Set<Product.PurchaseOption>()
    for payload in payloads {
        switch payload.kind {
        case "appAccountToken":
            guard let token = payload.appAccountToken,
                  let uuid = UUID(uuidString: token)
            else {
                throw SKBridgeError.invalidArgument("purchase option appAccountToken requires a valid UUID")
            }
            options.insert(.appAccountToken(uuid))
        case "quantity":
            guard let quantity = payload.quantity else {
                throw SKBridgeError.invalidArgument("purchase option quantity requires an integer value")
            }
            options.insert(.quantity(quantity))
        case "simulatesAskToBuyInSandbox":
            options.insert(.simulatesAskToBuyInSandbox(payload.simulateAskToBuyInSandbox ?? false))
        default:
            throw SKBridgeError.invalidArgument("unsupported purchase option kind '\(payload.kind)'")
        }
    }
    return options
}
