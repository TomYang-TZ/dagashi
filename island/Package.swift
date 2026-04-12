// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "DagashiIsland",
    platforms: [.macOS(.v14)],
    targets: [
        .executableTarget(
            name: "DagashiIsland",
            path: "Sources/DagashiIsland",
            resources: [.copy("Resources")]
        )
    ]
)
