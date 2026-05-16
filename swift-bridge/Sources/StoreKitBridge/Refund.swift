import AppKit
import StoreKit

func skRefundRequestStatusName(_ status: Transaction.RefundRequestStatus) -> String {
    switch status {
    case .success:
        return "success"
    case .userCancelled:
        return "userCancelled"
    @unknown default:
        return "unknown"
    }
}

@_cdecl("sk_refund_begin_request_for_transaction_id")
public func sk_refund_begin_request_for_transaction_id(
    _ transactionID: UnsafePointer<CChar>?,
    _ outStatus: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let transactionID else {
        let error = SKBridgeError.invalidArgument("missing transaction id")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    guard let parsedTransactionID = UInt64(String(cString: transactionID)) else {
        let error = SKBridgeError.invalidArgument("transaction id must be an unsigned integer")
        skPopulateError(outError, with: error)
        return error.statusCode
    }

    return skBlockOnMainActorAsync(
        work: {
            guard let controller = skKeyWindowController() else {
                throw SKBridgeError.notSupported(
                    "refund requests require an NSViewController-backed window"
                )
            }
            let status = try await Transaction.beginRefundRequest(
                for: parsedTransactionID,
                in: controller
            )
            return skRefundRequestStatusName(status)
        },
        onSuccess: { status in
            outStatus?.pointee = skCString(status)
        },
        onError: { error in
            skPopulateError(outError, with: error)
        }
    )
}
