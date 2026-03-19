import Foundation

/// Schema version for the artifact; bump when breaking changes occur.
public enum ArtifactSchema {
    public static let version = "2025.02.2"
}

/// Source of the analysis (local path, git URL, or zip).
public struct ArtifactSource: Codable, Sendable {
    public let type: String
    public let path: String?
    public let revision: String?

    public init(type: String, path: String? = nil, revision: String? = nil) {
        self.type = type
        self.path = path
        self.revision = revision
    }
}

/// High-level repo summary.
public struct ArtifactSummary: Codable, Sendable {
    public let swiftFiles: Int
    public let objcFiles: Int
    public let totalLines: Int
    public let dependencyManager: String
    public let hasSwiftPackage: Bool

    public init(swiftFiles: Int, objcFiles: Int, totalLines: Int, dependencyManager: String, hasSwiftPackage: Bool) {
        self.swiftFiles = swiftFiles
        self.objcFiles = objcFiles
        self.totalLines = totalLines
        self.dependencyManager = dependencyManager
        self.hasSwiftPackage = hasSwiftPackage
    }
}

/// UI stack signals.
public struct ArtifactUI: Codable, Sendable {
    public let usesSwiftUI: Bool
    public let usesUIKit: Bool
    public let storyboardCount: Int
    public let xibCount: Int
    public let swiftUIFileCount: Int
    public let uiKitFileCount: Int

    public init(usesSwiftUI: Bool, usesUIKit: Bool, storyboardCount: Int, xibCount: Int, swiftUIFileCount: Int = 0, uiKitFileCount: Int = 0) {
        self.usesSwiftUI = usesSwiftUI
        self.usesUIKit = usesUIKit
        self.storyboardCount = storyboardCount
        self.xibCount = xibCount
        self.swiftUIFileCount = swiftUIFileCount
        self.uiKitFileCount = uiKitFileCount
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        usesSwiftUI = try container.decode(Bool.self, forKey: .usesSwiftUI)
        usesUIKit = try container.decode(Bool.self, forKey: .usesUIKit)
        storyboardCount = try container.decode(Int.self, forKey: .storyboardCount)
        xibCount = try container.decode(Int.self, forKey: .xibCount)
        swiftUIFileCount = try container.decodeIfPresent(Int.self, forKey: .swiftUIFileCount) ?? 0
        uiKitFileCount = try container.decodeIfPresent(Int.self, forKey: .uiKitFileCount) ?? 0
    }
}

/// A single finding (deprecated API, layout risk, etc.).
public struct Finding: Codable, Sendable {
    public let file: String
    public let line: Int
    public let symbol: String?
    public let kind: String
    public let severity: String

    public init(file: String, line: Int, symbol: String? = nil, kind: String, severity: String) {
        self.file = file
        self.line = line
        self.symbol = symbol
        self.kind = kind
        self.severity = severity
    }
}

/// Deprecated API finding with replacement and notes (for report and scoring).
public struct DeprecatedAPIFinding: Codable, Sendable {
    public let symbol: String
    public let file: String
    public let line: Int
    public let severity: String
    public let replacement: String
    public let notes: String

    public init(symbol: String, file: String, line: Int, severity: String, replacement: String, notes: String) {
        self.symbol = symbol
        self.file = file
        self.line = line
        self.severity = severity
        self.replacement = replacement
        self.notes = notes
    }
}

/// Pod info after lookup in dependency rules.
public struct PodInfo: Codable, Sendable {
    public let name: String
    public let spm_available: Bool
    public let spm_url: String?
    public let migration_difficulty: String
    public let notes: String

    public init(name: String, spm_available: Bool, spm_url: String?, migration_difficulty: String, notes: String) {
        self.name = name
        self.spm_available = spm_available
        self.spm_url = spm_url
        self.migration_difficulty = migration_difficulty
        self.notes = notes
    }
}

/// Objective-C footprint.
public struct ObjCFootprint: Codable, Sendable {
    public let fileCount: Int
    public let mixedModules: Int

    public init(fileCount: Int, mixedModules: Int) {
        self.fileCount = fileCount
        self.mixedModules = mixedModules
    }
}

/// All findings grouped. deprecatedApis and deprecated_apis (full) both present for compatibility.
public struct ArtifactFindings: Codable, Sendable {
    public let deprecatedApis: [Finding]
    /// Full deprecated API findings with replacement and notes (from rules). Optional for backward compatibility.
    public let deprecated_apis: [DeprecatedAPIFinding]

    public let layoutRisks: [Finding]
    public let objcFootprint: ObjCFootprint

    public init(deprecatedApis: [Finding], deprecated_apis: [DeprecatedAPIFinding] = [], layoutRisks: [Finding], objcFootprint: ObjCFootprint) {
        self.deprecatedApis = deprecatedApis
        self.deprecated_apis = deprecated_apis
        self.layoutRisks = layoutRisks
        self.objcFootprint = objcFootprint
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        deprecatedApis = try c.decode([Finding].self, forKey: .deprecatedApis)
        deprecated_apis = try c.decodeIfPresent([DeprecatedAPIFinding].self, forKey: .deprecated_apis) ?? []
        layoutRisks = try c.decode([Finding].self, forKey: .layoutRisks)
        objcFootprint = try c.decode(ObjCFootprint.self, forKey: .objcFootprint)
    }
}

/// Dependency summary; pods populated when manager is CocoaPods and rules are loaded.
public struct ArtifactDependencies: Codable, Sendable {
    public let manager: String
    public let count: Int
    public let outdatedCount: Int
    public let spmMigratable: Bool
    public let pods: [PodInfo]
    public let risk_score: Double?

    public init(manager: String, count: Int, outdatedCount: Int, spmMigratable: Bool, pods: [PodInfo] = [], risk_score: Double? = nil) {
        self.manager = manager
        self.count = count
        self.outdatedCount = outdatedCount
        self.spmMigratable = spmMigratable
        self.pods = pods
        self.risk_score = risk_score
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        manager = try c.decode(String.self, forKey: .manager)
        count = try c.decode(Int.self, forKey: .count)
        outdatedCount = try c.decode(Int.self, forKey: .outdatedCount)
        spmMigratable = try c.decode(Bool.self, forKey: .spmMigratable)
        pods = try c.decodeIfPresent([PodInfo].self, forKey: .pods) ?? []
        risk_score = try c.decodeIfPresent(Double.self, forKey: .risk_score)
    }
}

/// Testing signals.
public struct ArtifactTesting: Codable, Sendable {
    public let hasTests: Bool
    public let testFileCount: Int
    public let coverageSignal: String

    public init(hasTests: Bool, testFileCount: Int, coverageSignal: String) {
        self.hasTests = hasTests
        self.testFileCount = testFileCount
        self.coverageSignal = coverageSignal
    }
}

/// Meta info for the run (repo name, branch, commit, target SDK).
public struct ArtifactMeta: Codable, Sendable {
    public let repo_name: String?
    public let branch: String?
    public let commit: String?
    public let target_sdk: Int
    public let scanDurationMs: Int?
    public let xcodeProjectDetected: Bool?
    public let cliVersion: String?

    public init(repo_name: String? = nil, branch: String? = nil, commit: String? = nil, target_sdk: Int = 26, scanDurationMs: Int? = nil, xcodeProjectDetected: Bool? = nil, cliVersion: String? = nil) {
        self.repo_name = repo_name
        self.branch = branch
        self.commit = commit
        self.target_sdk = target_sdk
        self.scanDurationMs = scanDurationMs
        self.xcodeProjectDetected = xcodeProjectDetected
        self.cliVersion = cliVersion
    }
}

/// Risk signals 0–1 for scoring.
public struct ArtifactSignals: Codable, Sendable {
    public let deprecated_risk: Double
    public let dependency_risk: Double

    public init(deprecated_risk: Double, dependency_risk: Double) {
        self.deprecated_risk = deprecated_risk
        self.dependency_risk = dependency_risk
    }
}

/// Readiness score and band.
public struct ArtifactReadiness: Codable, Sendable {
    public let score: Int
    public let band: String
    public let drivers: [String]

    public init(score: Int, band: String, drivers: [String]) {
        self.score = score
        self.band = band
        self.drivers = drivers
    }
}

/// Root artifact structure emitted by the analyzer.
public struct GlassForgeArtifact: Codable, Sendable {
    public let schemaVersion: String
    public let generatedAt: String
    public let source: ArtifactSource
    public let meta: ArtifactMeta?
    public let summary: ArtifactSummary
    public let ui: ArtifactUI
    public let findings: ArtifactFindings
    public let dependencies: ArtifactDependencies
    public let testing: ArtifactTesting
    public let signals: ArtifactSignals?
    public let readiness: ArtifactReadiness?
    public let liquidGlass: LiquidGlassResult?

    public init(
        schemaVersion: String,
        generatedAt: String,
        source: ArtifactSource,
        meta: ArtifactMeta? = nil,
        summary: ArtifactSummary,
        ui: ArtifactUI,
        findings: ArtifactFindings,
        dependencies: ArtifactDependencies,
        testing: ArtifactTesting,
        signals: ArtifactSignals? = nil,
        readiness: ArtifactReadiness? = nil,
        liquidGlass: LiquidGlassResult? = nil
    ) {
        self.schemaVersion = schemaVersion
        self.generatedAt = generatedAt
        self.source = source
        self.meta = meta
        self.summary = summary
        self.ui = ui
        self.findings = findings
        self.dependencies = dependencies
        self.testing = testing
        self.signals = signals
        self.readiness = readiness
        self.liquidGlass = liquidGlass
    }
}
