// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "GlassForge",
    platforms: [
        .macOS(.v13),
    ],
    products: [
        .executable(name: "glassforge", targets: ["GlassForge"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-argument-parser.git", from: "1.2.0"),
    ],
    targets: [
        .executableTarget(
            name: "GlassForge",
            dependencies: [
                .product(name: "ArgumentParser", package: "swift-argument-parser"),
            ],
            path: "Sources/GlassForge",
            resources: [.process("Rules")],
            swiftSettings: [
                .enableUpcomingFeature("StrictConcurrency"),
                .unsafeFlags(["-parse-as-library"]),
            ]
        ),
    ]
)
