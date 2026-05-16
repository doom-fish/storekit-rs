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

func skKeyWindowController() -> NSViewController? {
    let windows = NSApplication.shared.windows
    return windows.first(where: { $0.isKeyWindow })?.contentViewController
        ?? windows.first(where: { $0.isVisible })?.contentViewController
        ?? windows.first?.contentViewController
}
