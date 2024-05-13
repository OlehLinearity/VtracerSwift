// swift-tools-version:5.6
import PackageDescription

let package = Package(
    name: "VtracerSwift",
    products: [
        .library(
            name: "VtracerSwift",
            targets: ["VtracerSwift"])
    ],
    targets: [
        .binaryTarget(
            name: "VtracerSwift",
            path: "./VtracerSwift.xcframework")
    ]
)