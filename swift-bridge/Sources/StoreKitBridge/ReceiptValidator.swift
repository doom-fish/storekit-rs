import Foundation

struct SKReceiptPayload: Codable {
    let path: String
    let dataBase64: String
}

@_cdecl("sk_receipt_json")
public func sk_receipt_json(
    _ outReceiptJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let receiptURL = Bundle.main.appStoreReceiptURL else {
        return SK_OK
    }
    guard let receiptData = try? Data(contentsOf: receiptURL) else {
        let error = SKBridgeError.notSupported("unable to load the app receipt from \(receiptURL.path)")
        skPopulateError(outError, with: error)
        return error.statusCode
    }
    let payload = SKReceiptPayload(path: receiptURL.path, dataBase64: skDataBase64(receiptData))
    do {
        outReceiptJSON?.pointee = skCString(try skEncodeJSON(payload))
        return SK_OK
    } catch {
        skPopulateError(outError, with: error)
        return skStatus(for: error)
    }
}
