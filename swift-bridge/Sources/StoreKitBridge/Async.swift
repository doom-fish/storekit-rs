// Async.swift — Non-blocking callback-based thunks for StoreKit async APIs.
//
// Each thunk fires a C callback of the form:
//   cb(result_ptr, error_cstr, ctx)
// where exactly one of result_ptr/error_cstr is non-nil:
//   - success: result_ptr is non-nil (a JSON CString cast to UnsafeRawPointer,
//              or a retained opaque Swift object, or a sentinel 0x1 for void),
//              error_cstr is nil.
//   - failure: result_ptr is nil, error_cstr is a UTF-8 C string.

import AppKit
import Foundation
import StoreKit

// MARK: - Sentinel for void-returning async APIs
private let SK_ASYNC_VOID_SENTINEL = UnsafeRawPointer(bitPattern: 1)!

// MARK: - Product.products(for:) async throws -> [Product]

@_cdecl("sk_products_async")
public func sk_products_async(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let identifiers: [String]
    do {
        identifiers = try skDecodeJSON(identifiersJSON, as: [String].self)
    } catch {
        error.localizedDescription.withCString { ptr in cb(nil, ptr, ctx) }
        return
    }
    Task {
        do {
            let products = try await Product.products(for: identifiers)
            let json = try skEncodeJSON(products.map(skProductPayload(from:)))
            json.withCString { cb(UnsafeRawPointer($0), nil, ctx) }
        } catch {
            error.localizedDescription.withCString { ptr in cb(nil, ptr, ctx) }
        }
    }
}

// MARK: - Product.purchase(options:) async throws -> Product.PurchaseResult

private final class SKPurchaseAsyncResult {
    let json: String
    var handle: UnsafeMutableRawPointer?

    init(json: String, handle: UnsafeMutableRawPointer?) {
        self.json = json
        self.handle = handle
    }

    deinit {
        if let h = handle {
            sk_release(h)
        }
    }
}

@_cdecl("sk_purchase_async_result_json")
public func sk_purchase_async_result_json(
    _ ptr: UnsafeMutableRawPointer
) -> UnsafeMutablePointer<CChar>? {
    let r: SKPurchaseAsyncResult = sk_borrow(ptr)
    return skCString(r.json)
}

@_cdecl("sk_purchase_async_result_take_handle")
public func sk_purchase_async_result_take_handle(
    _ ptr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let r: SKPurchaseAsyncResult = sk_borrow(ptr)
    let h = r.handle
    r.handle = nil
    return h
}

@_cdecl("sk_purchase_async_result_release")
public func sk_purchase_async_result_release(_ ptr: UnsafeMutableRawPointer) {
    sk_release(ptr)
}

// Note: we do NOT annotate this Task with @MainActor so that the product
// lookup (skSingleProduct / Product.products) can run on any actor.
// The purchase sheet presentation (product.purchase) will internally dispatch
// to the main thread when it needs to show UI.
@_cdecl("sk_product_purchase_async")
public func sk_product_purchase_async(
    _ productID: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    guard let productID else {
        "missing product identifier".withCString { ptr in cb(nil, ptr, ctx) }
        return
    }
    let idStr = String(cString: productID)
    let optionPayloads: [SKPurchaseOptionPayload]
    do {
        optionPayloads = try skDecodeJSONIfPresent(optionsJSON, as: [SKPurchaseOptionPayload].self) ?? []
    } catch {
        error.localizedDescription.withCString { ptr in cb(nil, ptr, ctx) }
        return
    }
    Task {
        do {
            let product = try await skSingleProduct(for: idStr)
            let options = try skBuildPurchaseOptions(from: optionPayloads, product: product)
            var transactionHandle: UnsafeMutableRawPointer? = nil
            let result = try await product.purchase(options: options)
            let payload = try skPurchaseResultPayload(
                from: result,
                outTransaction: &transactionHandle
            )
            let json = try skEncodeJSON(payload)
            let box = SKPurchaseAsyncResult(json: json, handle: transactionHandle)
            cb(Unmanaged.passRetained(box).toOpaque(), nil, ctx)
        } catch {
            error.localizedDescription.withCString { ptr in cb(nil, ptr, ctx) }
        }
    }
}

// MARK: - AppStore.requestReview() async
//
// Note: AppStore.requestReview(in:) requires a live NSViewController-backed
// window and the @MainActor context. The checks that don't require UI
// (availability guard, window-controller lookup) are performed before
// spawning the @MainActor task so that headless environments fail-fast
// without needing the main run loop.

@_cdecl("sk_app_store_request_review_async")
public func sk_app_store_request_review_async(
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    guard #available(macOS 13.0, *) else {
        "AppStore.requestReview(in:) requires macOS 13.0+".withCString { ptr in cb(nil, ptr, ctx) }
        return
    }
    Task { @MainActor in
        guard let controller = skKeyWindowController() else {
            "AppStore.requestReview(in:) requires an NSViewController-backed window"
                .withCString { ptr in cb(nil, ptr, ctx) }
            return
        }
        AppStore.requestReview(in: controller)
        cb(SK_ASYNC_VOID_SENTINEL, nil, ctx)
    }
}

// MARK: - AppStore.showManageSubscriptions(in:) async

@_cdecl("sk_app_store_show_manage_subscriptions_async")
public func sk_app_store_show_manage_subscriptions_async(
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    let msg = "AppStore.showManageSubscriptions(in:) is scene-based and unavailable in the macOS StoreKit SDK"
    msg.withCString { ptr in cb(nil, ptr, ctx) }
}

// MARK: - AppTransaction.shared async throws

@_cdecl("sk_app_transaction_shared_async")
public func sk_app_transaction_shared_async(
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    guard #available(macOS 13.0, *) else {
        "AppTransaction.shared requires macOS 13.0+".withCString { ptr in cb(nil, ptr, ctx) }
        return
    }
    Task {
        do {
            let shared = try await AppTransaction.shared
            let json = try skEncodeJSON(skAppTransactionVerificationResultPayload(from: shared))
            json.withCString { cb(UnsafeRawPointer($0), nil, ctx) }
        } catch {
            error.localizedDescription.withCString { ptr in cb(nil, ptr, ctx) }
        }
    }
}

// MARK: - Storefront.current async

private struct SKStorefrontCurrentResult: Encodable {
    let storefront: SKStorefrontPayload?
}

@_cdecl("sk_storefront_current_async")
public func sk_storefront_current_async(
    _ cb: @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void,
    _ ctx: UnsafeMutableRawPointer
) {
    Task {
        let current = await Storefront.current
        do {
            let result = SKStorefrontCurrentResult(
                storefront: current.map(skStorefrontPayload(from:))
            )
            let json = try skEncodeJSON(result)
            json.withCString { cb(UnsafeRawPointer($0), nil, ctx) }
        } catch {
            error.localizedDescription.withCString { ptr in cb(nil, ptr, ctx) }
        }
    }
}
