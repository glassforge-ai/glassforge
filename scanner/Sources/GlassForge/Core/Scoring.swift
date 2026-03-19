import Foundation

/// Weights for severity (for deprecated_risk).
private let severityWeight: [String: Double] = [
    "blocking": 3,
    "high": 2,
    "medium": 1,
    "low": 0.5,
]

/// Compute deprecated_risk 0–1 from deprecated API findings (weighted count, normalized).
public func computeDeprecatedRisk(deprecatedApis: [DeprecatedAPIFinding], maxWeightedCount: Double = 50) -> Double {
    let weighted = deprecatedApis.reduce(0.0) { sum, f in
        sum + (severityWeight[f.severity] ?? 1)
    }
    return min(1.0, weighted / maxWeightedCount)
}

/// Compute dependency_risk 0–1 from manager and pods (CocoaPods with many no-SPM = higher risk).
public func computeDependencyRisk(manager: String, pods: [PodInfo], hasSwiftPackage: Bool) -> Double {
    if manager == "swift" && hasSwiftPackage {
        return 0.1
    }
    if manager == "none" {
        return 0.0
    }
    if manager == "cocoapods" || manager == "carthage" {
        let total = pods.count
        guard total > 0 else { return 0.4 }
        let noSPM = pods.filter { !$0.spm_available }.count
        let highDiff = pods.filter { $0.migration_difficulty == "high" }.count
        let risk = 0.3 + 0.4 * (Double(noSPM) / Double(total)) + 0.3 * (Double(highDiff) / Double(total))
        return min(1.0, risk)
    }
    return 0.5
}

/// Category weights (must sum to 100).
private let categoryWeights: [String: Double] = [
    "deprecated": 22,
    "dependency": 18,
    "ui": 15,
    "liquidGlass": 10,
    "objc": 15,
    "layout": 10,
    "testing": 10,
]

/// Compute readiness score 0–100 and band. Drivers are short explanation strings.
public func computeReadiness(
    summary: ArtifactSummary,
    ui: ArtifactUI,
    findings: ArtifactFindings,
    dependencies: ArtifactDependencies,
    testing: ArtifactTesting,
    signals: ArtifactSignals,
    liquidGlass: LiquidGlassResult? = nil
) -> ArtifactReadiness {
    let deprecatedRisk = signals.deprecated_risk
    let depScore = max(0, 100 - Int(deprecatedRisk * 100))

    let depRisk = signals.dependency_risk
    let depDepScore = max(0, 100 - Int(depRisk * 100))

    let storyboardPenalty = min(30, ui.storyboardCount * 3 + ui.xibCount)
    let swiftUIBonus = ui.usesSwiftUI ? 15 : 0
    let uiScore = max(0, 100 - storyboardPenalty + swiftUIBonus)

    // Liquid Glass scoring: positive signals boost, anti-patterns and migration targets penalize
    let lgScore: Int
    if let lg = liquidGlass {
        let positiveBonus = min(30, lg.positiveSignalCount * 10)
        let migrationPenalty = min(40, lg.migrationTargetCount * 5)
        let antiPatternPenalty = min(50, lg.antiPatternCount * 10)
        lgScore = max(0, min(100, 50 + positiveBonus - migrationPenalty - antiPatternPenalty))
    } else {
        lgScore = 50 // neutral when not analyzed
    }

    let totalFiles = summary.swiftFiles + summary.objcFiles
    let objcRatio = totalFiles > 0 ? Double(summary.objcFiles) / Double(totalFiles) : 0
    let objcScore = max(0, 100 - Int(objcRatio * 80))

    let layoutCount = findings.layoutRisks.count
    let layoutScore = max(0, 100 - layoutCount * 2)

    let testScore = testing.hasTests ? (testing.testFileCount > 10 ? 100 : 50 + testing.testFileCount * 5) : 20
    let testScoreCapped = min(100, testScore)

    let raw = Int(
        (Double(depScore) * categoryWeights["deprecated"]! / 100) +
        (Double(depDepScore) * categoryWeights["dependency"]! / 100) +
        (Double(uiScore) * categoryWeights["ui"]! / 100) +
        (Double(lgScore) * categoryWeights["liquidGlass"]! / 100) +
        (Double(objcScore) * categoryWeights["objc"]! / 100) +
        (Double(layoutScore) * categoryWeights["layout"]! / 100) +
        (Double(testScoreCapped) * categoryWeights["testing"]! / 100)
    )
    let score = max(0, min(100, raw))

    let band: String
    if score >= 75 { band = "green" }
    else if score >= 50 { band = "yellow" }
    else { band = "red" }

    var drivers: [String] = []
    if deprecatedRisk > 0.2 { drivers.append("Deprecated API usage (\(findings.deprecated_apis.count) findings)") }
    if depRisk > 0.3 { drivers.append("Dependency risk (CocoaPods/legacy)") }
    if ui.storyboardCount + ui.xibCount > 5 { drivers.append("Storyboard/XIB footprint") }
    if let lg = liquidGlass {
        if lg.antiPatternCount > 0 { drivers.append("Liquid Glass anti-patterns (\(lg.antiPatternCount) issues)") }
        if lg.migrationTargetCount > 0 { drivers.append("Legacy materials need Liquid Glass migration (\(lg.migrationTargetCount) targets)") }
        if lg.readinessLevel == "adopted" { drivers.append("Liquid Glass already adopted") }
    }
    if objcRatio > 0.2 { drivers.append("Objective-C footprint (\(summary.objcFiles) files)") }
    if layoutCount > 10 { drivers.append("Layout risk (\(layoutCount) fixed-frame usages)") }
    if !testing.hasTests { drivers.append("No test signal") }
    if drivers.isEmpty { drivers.append("No major risks identified") }

    return ArtifactReadiness(score: score, band: band, drivers: drivers)
}
