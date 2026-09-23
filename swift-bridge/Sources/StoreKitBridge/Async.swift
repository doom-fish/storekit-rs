// Async.swift — Non-blocking callback-based thunks for StoreKit async APIs.
//
// Each thunk fires a C callback exactly once, of the form:
//   cb(ctx, status, json_cstr, transaction, error_cstr)
//   - success: status is SK_OK, json_cstr is the result JSON (nil for void
//              results), transaction is a +1 SKTransactionBox or nil, and
//              error_cstr is nil. The callee owns the transaction reference.
//   - failure: status is the SK_* error status, json_cstr and transaction are
//              nil, and error_cstr carries the same error payload that the
//              synchronous bridge writes to out_error.
// The C strings are only valid for the duration of the callback.

import AppKit
import Foundation
import StoreKit

public typealias SKAsyncCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    UnsafePointer<CChar>?,
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?
) -> Void

func skDeliverAsync(
    _ cb: SKAsyncCallback,
    _ ctx: UnsafeMutableRawPointer?,
    _ result: Result<SKTransactionOutcome?, Error>
) {
    switch result {
    case .success(let outcome):
        let transaction = outcome?.transaction.map { sk_retain($0) }
        if let json = outcome?.json {
            json.withCString { cb(ctx, SK_OK, $0, transaction, nil) }
        } else {
            cb(ctx, SK_OK, nil, transaction, nil)
        }
    case .failure(let error):
        skErrorMessage(for: error).withCString { cb(ctx, skStatus(for: error), nil, nil, $0) }
    }
}

func skRunAsync(
    _ cb: SKAsyncCallback,
    _ ctx: UnsafeMutableRawPointer?,
    work: @escaping () async throws -> SKTransactionOutcome?
) {
    Task {
        let result: Result<SKTransactionOutcome?, Error>
        do {
            result = .success(try await work())
        } catch {
            result = .failure(error)
        }
        skDeliverAsync(cb, ctx, result)
    }
}

func skRunMainActorAsync(
    _ cb: SKAsyncCallback,
    _ ctx: UnsafeMutableRawPointer?,
    work: @escaping @MainActor () async throws -> SKTransactionOutcome?
) {
    Task { @MainActor in
        let result: Result<SKTransactionOutcome?, Error>
        do {
            result = .success(try await work())
        } catch {
            result = .failure(error)
        }
        skDeliverAsync(cb, ctx, result)
    }
}

func skJSONOutcome(_ json: String) -> SKTransactionOutcome {
    SKTransactionOutcome(json: json, transaction: nil)
}

// MARK: - Product.products(for:) async throws -> [Product]

@_cdecl("sk_products_async")
public func sk_products_async(
    _ identifiersJSON: UnsafePointer<CChar>?,
    _ cb: SKAsyncCallback,
    _ ctx: UnsafeMutableRawPointer?
) {
    let identifiers: [String]
    do {
        identifiers = try skDecodeJSON(identifiersJSON, as: [String].self)
    } catch {
        skDeliverAsync(cb, ctx, .failure(error))
        return
    }
    skRunAsync(cb, ctx) {
        let products = try await Product.products(for: identifiers)
        return skJSONOutcome(try skEncodeJSON(products.map(skProductPayload(from:))))
    }
}

// MARK: - Product.purchase(options:) async throws -> Product.PurchaseResult

// Note: we do NOT annotate this Task with @MainActor so that the product
// lookup (skSingleProduct / Product.products) can run on any actor.
// The purchase sheet presentation (product.purchase) will internally dispatch
// to the main thread when it needs to show UI.
@_cdecl("sk_product_purchase_async")
public func sk_product_purchase_async(
    _ productID: UnsafePointer<CChar>?,
    _ optionsJSON: UnsafePointer<CChar>?,
    _ cb: SKAsyncCallback,
    _ ctx: UnsafeMutableRawPointer?
) {
    guard let productID else {
        skDeliverAsync(cb, ctx, .failure(SKBridgeError.invalidArgument("missing product identifier")))
        return
    }
    let idStr = String(cString: productID)
    let optionPayloads: [SKPurchaseOptionPayload]
    do {
        optionPayloads = try skDecodeJSONIfPresent(optionsJSON, as: [SKPurchaseOptionPayload].self) ?? []
    } catch {
        skDeliverAsync(cb, ctx, .failure(error))
        return
    }
    skRunAsync(cb, ctx) {
        let product = try await skSingleProduct(for: idStr)
        let options = try skBuildPurchaseOptions(from: optionPayloads, product: product)
        return try skPurchaseOutcome(from: try await product.purchase(options: options))
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
    _ cb: SKAsyncCallback,
    _ ctx: UnsafeMutableRawPointer?
) {
    guard #available(macOS 13.0, *) else {
        skDeliverAsync(
            cb,
            ctx,
            .failure(SKBridgeError.notSupported("AppStore.requestReview(in:) requires macOS 13.0+"))
        )
        return
    }
    skRunMainActorAsync(cb, ctx) {
        try skRequestReview()
        return nil
    }
}

// MARK: - AppStore.showManageSubscriptions(in:) async

@_cdecl("sk_app_store_show_manage_subscriptions_async")
public func sk_app_store_show_manage_subscriptions_async(
    _ cb: SKAsyncCallback,
    _ ctx: UnsafeMutableRawPointer?
) {
    skDeliverAsync(
        cb,
        ctx,
        .failure(
            SKBridgeError.notSupported(
                "AppStore.showManageSubscriptions(in:) is scene-based and unavailable in the macOS StoreKit SDK"
            )
        )
    )
}

// MARK: - AppTransaction.shared async throws

@_cdecl("sk_app_transaction_shared_async")
public func sk_app_transaction_shared_async(
    _ cb: SKAsyncCallback,
    _ ctx: UnsafeMutableRawPointer?
) {
    guard #available(macOS 13.0, *) else {
        skDeliverAsync(cb, ctx, .failure(SKBridgeError.notSupported("AppTransaction.shared requires macOS 13.0+")))
        return
    }
    skRunAsync(cb, ctx) {
        let shared = try await AppTransaction.shared
        return skJSONOutcome(try skEncodeJSON(skAppTransactionVerificationResultPayload(from: shared)))
    }
}

// MARK: - Storefront.current async

private struct SKStorefrontCurrentResult: Encodable {
    let storefront: SKStorefrontPayload?
}

@_cdecl("sk_storefront_current_async")
public func sk_storefront_current_async(
    _ cb: SKAsyncCallback,
    _ ctx: UnsafeMutableRawPointer?
) {
    skRunAsync(cb, ctx) {
        let current = await Storefront.current
        return skJSONOutcome(
            try skEncodeJSON(SKStorefrontCurrentResult(storefront: current.map(skStorefrontPayload(from:))))
        )
    }
}
