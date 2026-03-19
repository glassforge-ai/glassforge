import Foundation

/// Detects test targets by file naming and path.
public struct TestDetector: Sendable {
    public init() {}

    public func detect(swiftFiles: [URL], repoRoot: URL) -> (hasTests: Bool, testFileCount: Int, coverageSignal: String) {
        var testCount = 0
        for url in swiftFiles {
            let name = url.lastPathComponent
            let path = url.path
            if path.contains("Tests") || path.contains("Test/") || name.hasPrefix("Tests") ||
               name.hasSuffix("Tests.swift") || name.hasSuffix("Spec.swift") || name.contains("TestCase") {
                testCount += 1
            }
        }
        let hasTests = testCount > 0
        let signal = hasTests ? "present" : "absent"
        return (hasTests, testCount, signal)
    }
}
