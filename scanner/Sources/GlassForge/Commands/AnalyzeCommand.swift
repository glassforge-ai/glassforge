import ArgumentParser
import Foundation

struct AnalyzeCommand: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "analyze",
        abstract: "Analyze an iOS repository and emit a JSON artifact."
    )

    @Argument(help: "Path to the repository root (directory)")
    var path: String = "."

    @Option(name: [.short, .long], help: "Directory to write artifact and reports")
    var outputDir: String?

    @Option(name: [.long], help: "Artifact filename")
    var artifactName: String = "glassforge-artifact.json"

    @Flag(name: [.long], help: "Verbose output")
    var verbose: Bool = false

    @Flag(name: [.long], help: "Also generate Markdown report")
    var report: Bool = false

    @Flag(name: [.long], help: "Quiet (only errors)")
    var quiet: Bool = false

    @Option(name: [.long], help: "Exclude paths (comma-separated, e.g. Pods,Generated)")
    var exclude: String?

    @Option(name: [.long], help: "Fail if readiness score is below this threshold (for CI)")
    var failUnder: Int?

    func run() throws {
        let startTime = DispatchTime.now()

        let repoURL: URL
        if path == "." {
            repoURL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
        } else {
            repoURL = URL(fileURLWithPath: (path as NSString).expandingTildeInPath)
        }

        var isDir: ObjCBool = false
        guard FileManager.default.fileExists(atPath: repoURL.path, isDirectory: &isDir), isDir.boolValue else {
            throw AnalyzeError.pathNotDirectory(repoURL.path)
        }

        let outDir: URL
        if let outputDir = outputDir {
            outDir = URL(fileURLWithPath: (outputDir as NSString).expandingTildeInPath)
            try FileManager.default.createDirectory(at: outDir, withIntermediateDirectories: true)
        } else {
            outDir = repoURL
        }

        if !quiet { print("Scanning \(repoURL.path)...") }

        let excludePatterns = exclude?.components(separatedBy: ",").map { $0.trimmingCharacters(in: .whitespaces) } ?? []
        let scanner = DirectoryScanner(excludePatterns: excludePatterns)
        let scanResult = try scanner.scan(at: repoURL)

        if !quiet {
            print("  Swift: \(scanResult.swiftFiles.count), Obj-C: \(scanResult.objcFiles.count), Storyboards: \(scanResult.storyboards.count), XIBs: \(scanResult.xibs.count)")
            if scanResult.xcodeProjectDetected {
                print("  Xcode project detected")
            }
        }

        let depParser = DependencyParser()
        let depInfo = depParser.parse(repoRoot: repoURL)

        let deprecatedRules = RulesLoader.loadDeprecatedAPIRules()
        let swiftAnalyzer = SwiftAnalyzer()
        let swiftResult = swiftAnalyzer.analyze(
            swiftFiles: scanResult.swiftFiles,
            objcFiles: scanResult.objcFiles,
            repoRoot: repoURL,
            deprecatedRules: deprecatedRules
        )

        var pods: [PodInfo] = []
        var dependencyRiskScore: Double?
        let podfileURL = repoURL.appendingPathComponent("Podfile")
        if FileManager.default.fileExists(atPath: podfileURL.path) {
            let depRules = RulesLoader.loadDependencyRules()
            let podNames = extractPodNames(from: podfileURL)
            pods = lookupPods(podNames: podNames, rules: depRules)
            dependencyRiskScore = computeDependencyRisk(manager: depInfo.manager, pods: pods, hasSwiftPackage: depInfo.hasSwiftPackage)
        }

        // Liquid Glass analysis
        var liquidGlassResult: LiquidGlassResult?
        if let lgRules = RulesLoader.loadLiquidGlassRules() {
            let lgAnalyzer = LiquidGlassAnalyzer(rules: lgRules)
            liquidGlassResult = lgAnalyzer.analyze(swiftFiles: scanResult.swiftFiles, repoRoot: repoURL)
            if !quiet && verbose {
                let lg = liquidGlassResult!
                print("  Liquid Glass: \(lg.readinessLevel) (positive: \(lg.positiveSignalCount), migration: \(lg.migrationTargetCount), anti-patterns: \(lg.antiPatternCount))")
            }
        }

        let mixedModules = countMixedModules(swiftFiles: scanResult.swiftFiles, objcFiles: scanResult.objcFiles)
        let testDetector = TestDetector()
        let (hasTests, testFileCount, coverageSignal) = testDetector.detect(swiftFiles: scanResult.swiftFiles, repoRoot: repoURL)

        // Git metadata
        let gitBranch = shellOutput("git", "-C", repoURL.path, "branch", "--show-current")
        let gitCommit = shellOutput("git", "-C", repoURL.path, "rev-parse", "--short", "HEAD")

        let endTime = DispatchTime.now()
        let durationMs = Int((endTime.uptimeNanoseconds - startTime.uptimeNanoseconds) / 1_000_000)

        let repoName = repoURL.lastPathComponent
        let meta = ArtifactMeta(
            repo_name: repoName,
            branch: gitBranch,
            commit: gitCommit,
            target_sdk: 26,
            scanDurationMs: durationMs,
            xcodeProjectDetected: scanResult.xcodeProjectDetected,
            cliVersion: "0.2.0"
        )
        let source = ArtifactSource(type: "local", path: repoURL.path, revision: gitCommit)
        let summary = ArtifactSummary(
            swiftFiles: scanResult.swiftFiles.count,
            objcFiles: scanResult.objcFiles.count,
            totalLines: scanResult.totalLines,
            dependencyManager: depInfo.manager,
            hasSwiftPackage: depInfo.hasSwiftPackage
        )
        let ui = ArtifactUI(
            usesSwiftUI: swiftResult.usesSwiftUI,
            usesUIKit: swiftResult.usesUIKit,
            storyboardCount: scanResult.storyboards.count,
            xibCount: scanResult.xibs.count,
            swiftUIFileCount: swiftResult.swiftUIFileCount,
            uiKitFileCount: swiftResult.uiKitFileCount
        )
        let findings = ArtifactFindings(
            deprecatedApis: swiftResult.deprecatedApis,
            deprecated_apis: swiftResult.deprecated_apis,
            layoutRisks: swiftResult.layoutRisks,
            objcFootprint: ObjCFootprint(fileCount: scanResult.objcFiles.count, mixedModules: mixedModules)
        )
        let dependencies = ArtifactDependencies(
            manager: depInfo.manager,
            count: depInfo.count,
            outdatedCount: depInfo.outdatedCount,
            spmMigratable: depInfo.spmMigratable,
            pods: pods,
            risk_score: dependencyRiskScore.map { $0 * 100 }
        )
        let testing = ArtifactTesting(hasTests: hasTests, testFileCount: testFileCount, coverageSignal: coverageSignal)

        let deprecatedRisk = computeDeprecatedRisk(deprecatedApis: swiftResult.deprecated_apis)
        let depRisk = dependencyRiskScore ?? computeDependencyRisk(manager: depInfo.manager, pods: pods, hasSwiftPackage: depInfo.hasSwiftPackage)
        let signals = ArtifactSignals(deprecated_risk: deprecatedRisk, dependency_risk: depRisk)
        let readiness = computeReadiness(summary: summary, ui: ui, findings: findings, dependencies: dependencies, testing: testing, signals: signals, liquidGlass: liquidGlassResult)

        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        let generatedAt = formatter.string(from: Date())

        let artifact = GlassForgeArtifact(
            schemaVersion: ArtifactSchema.version,
            generatedAt: generatedAt,
            source: source,
            meta: meta,
            summary: summary,
            ui: ui,
            findings: findings,
            dependencies: dependencies,
            testing: testing,
            signals: signals,
            readiness: readiness,
            liquidGlass: liquidGlassResult
        )

        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try encoder.encode(artifact)

        let artifactURL = outDir.appendingPathComponent(artifactName)
        try data.write(to: artifactURL)

        if !quiet {
            print("Artifact written to \(artifactURL.path)")
            print("  Readiness: \(readiness.score)/100 (\(readiness.band))")
            print("  Deprecated API findings: \(findings.deprecated_apis.count)")
            print("  Layout risks: \(findings.layoutRisks.count)")
            if let lg = liquidGlassResult {
                print("  Liquid Glass: \(lg.readinessLevel) (migration targets: \(lg.migrationTargetCount), anti-patterns: \(lg.antiPatternCount))")
            }
            print("  Scan duration: \(durationMs)ms")
        }

        if report {
            let markdown = generateMarkdownReport(artifact: artifact)
            let reportURL = outDir.appendingPathComponent("glassforge_report.md")
            try markdown.write(to: reportURL, atomically: true, encoding: .utf8)
            if !quiet { print("Report written to \(reportURL.path)") }
        }

        // CI gating: exit 1 if score below threshold
        if let threshold = failUnder, readiness.score < threshold {
            throw AnalyzeError.readinessBelowThreshold(score: readiness.score, threshold: threshold)
        }
    }
}

enum AnalyzeError: Error, CustomStringConvertible {
    case pathNotDirectory(String)
    case readinessBelowThreshold(score: Int, threshold: Int)

    var description: String {
        switch self {
        case .pathNotDirectory(let path):
            return "Path is not a directory: \(path)"
        case .readinessBelowThreshold(let score, let threshold):
            return "Readiness score \(score) is below threshold \(threshold)"
        }
    }
}

/// Run a shell command and return trimmed stdout, or nil on failure.
private func shellOutput(_ args: String...) -> String? {
    let process = Process()
    let pipe = Pipe()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    process.arguments = args
    process.standardOutput = pipe
    process.standardError = FileHandle.nullDevice
    do {
        try process.run()
        process.waitUntilExit()
        guard process.terminationStatus == 0 else { return nil }
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        return String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines)
    } catch {
        return nil
    }
}
