import Foundation

/// Detected dependency manager and parsed counts.
public struct DependencyInfo: Sendable {
    public let manager: String
    public let count: Int
    public let outdatedCount: Int
    public let hasSwiftPackage: Bool
    public let spmMigratable: Bool

    public init(manager: String, count: Int, outdatedCount: Int, hasSwiftPackage: Bool, spmMigratable: Bool) {
        self.manager = manager
        self.count = count
        self.outdatedCount = outdatedCount
        self.hasSwiftPackage = hasSwiftPackage
        self.spmMigratable = spmMigratable
    }
}

/// Parses dependency manifests in the repo root.
public struct DependencyParser: Sendable {
    public init() {}

    public func parse(repoRoot: URL) -> DependencyInfo {
        let fm = FileManager.default
        var manager = "none"
        var count = 0
        var outdatedCount = 0
        var hasSwiftPackage = false
        var spmMigratable = false

        let packagePath = repoRoot.appendingPathComponent("Package.swift")
        let podfilePath = repoRoot.appendingPathComponent("Podfile")
        let podfileLockPath = repoRoot.appendingPathComponent("Podfile.lock")
        let cartfilePath = repoRoot.appendingPathComponent("Cartfile")
        let cartfileResolvedPath = repoRoot.appendingPathComponent("Cartfile.resolved")

        if fm.fileExists(atPath: packagePath.path) {
            hasSwiftPackage = true
            count = countPackages(in: packagePath) ?? 0
            manager = "swift"
            spmMigratable = true
        }

        if fm.fileExists(atPath: podfilePath.path) {
            let pods = countPods(lockPath: podfileLockPath) ?? countPodsFromPodfile(podfilePath)
            if manager == "none" {
                manager = "cocoapods"
                count = pods
                outdatedCount = 0
                spmMigratable = false
            }
        }

        if fm.fileExists(atPath: cartfilePath.path), manager == "none" {
            count = countCarthageDeps(resolvedPath: cartfileResolvedPath)
            manager = "carthage"
            spmMigratable = false
        }

        return DependencyInfo(
            manager: manager,
            count: count,
            outdatedCount: outdatedCount,
            hasSwiftPackage: hasSwiftPackage,
            spmMigratable: spmMigratable
        )
    }

    private func countPackages(in packageSwift: URL) -> Int? {
        guard let content = try? String(contentsOf: packageSwift, encoding: .utf8) else { return nil }
        let regex = try? NSRegularExpression(pattern: #"\.package\s*\("#)
        let range = NSRange(content.startIndex..., in: content)
        return regex?.numberOfMatches(in: content, range: range)
    }

    private func countPods(lockPath: URL) -> Int? {
        guard FileManager.default.fileExists(atPath: lockPath.path),
              let content = try? String(contentsOf: lockPath, encoding: .utf8) else { return nil }
        let lines = content.components(separatedBy: .newlines)
        var count = 0
        for line in lines {
            let t = line.trimmingCharacters(in: .whitespaces)
            if t.hasPrefix("- ") && (t.contains(":") || t.contains("(")) {
                count += 1
            }
        }
        return count > 0 ? count : nil
    }

    private func countPodsFromPodfile(_ podfile: URL) -> Int {
        guard let content = try? String(contentsOf: podfile, encoding: .utf8) else { return 0 }
        let pattern = #"pod\s+['\"]"#
        return content.ranges(of: pattern).count
    }

    private func countCarthageDeps(resolvedPath: URL) -> Int {
        guard FileManager.default.fileExists(atPath: resolvedPath.path),
              let content = try? String(contentsOf: resolvedPath, encoding: .utf8) else { return 0 }
        return content.components(separatedBy: .newlines).filter { $0.hasPrefix("git ") || $0.hasPrefix("github ") }.count
    }
}
