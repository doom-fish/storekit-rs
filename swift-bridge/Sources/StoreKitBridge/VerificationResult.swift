import Foundation
import StoreKit

struct SKVerificationMetadataPayload: Codable {
    let jwsRepresentation: String
    let headerDataBase64: String
    let payloadDataBase64: String
    let signatureDataBase64: String
    let signedDataBase64: String
    let signedDate: String
    let deviceVerificationBase64: String
    let deviceVerificationNonce: String
}

struct SKVerificationResultPayload<Payload: Codable>: Codable {
    let kind: String
    let payload: Payload
    let metadata: SKVerificationMetadataPayload
    let verificationError: SKVerificationErrorPayload?
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

@available(macOS 13.0, *)
func skVerificationErrorCode(_ error: StoreKit.VerificationResult<AppTransaction>.VerificationError) -> String {
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

func skVerificationErrorCode(_ error: StoreKit.VerificationResult<Product.SubscriptionInfo.RenewalInfo>.VerificationError) -> String {
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

func skVerificationMetadata(from result: VerificationResult<Transaction>) -> SKVerificationMetadataPayload {
    SKVerificationMetadataPayload(
        jwsRepresentation: result.jwsRepresentation,
        headerDataBase64: skDataBase64(result.headerData),
        payloadDataBase64: skDataBase64(result.payloadData),
        signatureDataBase64: skDataBase64(result.signatureData),
        signedDataBase64: skDataBase64(result.signedData),
        signedDate: skFormatDate(result.signedDate),
        deviceVerificationBase64: skDataBase64(result.deviceVerification),
        deviceVerificationNonce: result.deviceVerificationNonce.uuidString
    )
}

@available(macOS 13.0, *)
func skVerificationMetadata(from result: VerificationResult<AppTransaction>) -> SKVerificationMetadataPayload {
    SKVerificationMetadataPayload(
        jwsRepresentation: result.jwsRepresentation,
        headerDataBase64: skDataBase64(result.headerData),
        payloadDataBase64: skDataBase64(result.payloadData),
        signatureDataBase64: skDataBase64(result.signatureData),
        signedDataBase64: skDataBase64(result.signedData),
        signedDate: skFormatDate(result.signedDate),
        deviceVerificationBase64: skDataBase64(result.deviceVerification),
        deviceVerificationNonce: result.deviceVerificationNonce.uuidString
    )
}

func skVerificationMetadata(from result: VerificationResult<Product.SubscriptionInfo.RenewalInfo>) -> SKVerificationMetadataPayload {
    SKVerificationMetadataPayload(
        jwsRepresentation: result.jwsRepresentation,
        headerDataBase64: skDataBase64(result.headerData),
        payloadDataBase64: skDataBase64(result.payloadData),
        signatureDataBase64: skDataBase64(result.signatureData),
        signedDataBase64: skDataBase64(result.signedData),
        signedDate: skFormatDate(result.signedDate),
        deviceVerificationBase64: skDataBase64(result.deviceVerification),
        deviceVerificationNonce: result.deviceVerificationNonce.uuidString
    )
}

func skVerificationKind<T>(_ result: VerificationResult<T>) -> String {
    switch result {
    case .verified(_):
        return "verified"
    case .unverified(_, _):
        return "unverified"
    }
}

func skTransactionVerificationResultPayload(
    from result: VerificationResult<Transaction>
) -> SKVerificationResultPayload<SKTransactionPayload> {
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
    return SKVerificationResultPayload(
        kind: skVerificationKind(result),
        payload: skTransactionPayload(from: result),
        metadata: skVerificationMetadata(from: result),
        verificationError: verificationError
    )
}

@available(macOS 13.0, *)
func skAppTransactionVerificationResultPayload(
    from result: VerificationResult<AppTransaction>
) -> SKVerificationResultPayload<SKAppTransactionPayload> {
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
    return SKVerificationResultPayload(
        kind: skVerificationKind(result),
        payload: skAppTransactionPayload(from: result),
        metadata: skVerificationMetadata(from: result),
        verificationError: verificationError
    )
}

func skRenewalInfoVerificationResultPayload(
    from result: VerificationResult<Product.SubscriptionInfo.RenewalInfo>
) -> SKVerificationResultPayload<SKRenewalInfoPayload> {
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
    return SKVerificationResultPayload(
        kind: skVerificationKind(result),
        payload: skRenewalInfoPayload(from: result.unsafePayloadValue),
        metadata: skVerificationMetadata(from: result),
        verificationError: verificationError
    )
}
