// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "StoreKitBridge",
    platforms: [
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "StoreKitBridge",
            type: .static,
            targets: ["StoreKitBridge"])
    ],
    targets: [
        .target(
            name: "StoreKitBridge",
            path: "Sources/StoreKitBridge",
            publicHeadersPath: "include")
    ]
)
