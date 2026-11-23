// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "BNOSPrintTray",
    platforms: [.macOS(.v13)],
    products: [
        .executable(name: "BNOSPrintTray", targets: ["BNOSPrintTray"])
    ],
    targets: [
        .executableTarget(
            name: "BNOSPrintTray",
            path: "Sources"
        )
    ]
)
