import Foundation

/// Generate deterministic Markdown report from artifact (no AI).
public func generateMarkdownReport(artifact: GlassForgeArtifact) -> String {
    let repoName = artifact.meta?.repo_name ?? artifact.source.path ?? "—"
    let branch = artifact.meta?.branch ?? "—"
    let commit = artifact.meta?.commit ?? "—"
    let targetSDK = artifact.meta?.target_sdk ?? 26
    let score = artifact.readiness?.score ?? 0
    let band = artifact.readiness?.band ?? "unknown"
    let drivers = artifact.readiness?.drivers ?? []
    let totalFiles = artifact.summary.swiftFiles + artifact.summary.objcFiles
    let swiftPercent = totalFiles > 0 ? Int(100.0 * Double(artifact.summary.swiftFiles) / Double(totalFiles)) : 100
    let objcPercent = totalFiles > 0 ? Int(100.0 * Double(artifact.summary.objcFiles) / Double(totalFiles)) : 0
    let uiStack = artifact.ui.usesSwiftUI && artifact.ui.usesUIKit ? "SwiftUI + UIKit" : (artifact.ui.usesSwiftUI ? "SwiftUI" : (artifact.ui.usesUIKit ? "UIKit" : "—"))
    let depRiskNorm = artifact.dependencies.risk_score ?? 0
    let hasBlocking = artifact.findings.deprecated_apis.contains { $0.severity == "blocking" }
    let hasHighDepRisk = (artifact.dependencies.risk_score ?? 0) > 40
    let objcPercentFloat = totalFiles > 0 ? Double(artifact.summary.objcFiles) / Double(totalFiles) : 0

    var md = """
    # GlassForge iOS Modernization Report

    **Repository:** \(repoName)  
    **Branch:** \(branch)  
    **Commit:** \(commit)  
    **Analyzed at:** \(artifact.generatedAt)  
    **Target SDK:** iOS \(targetSDK)

    ---

    ## 1. iOS Readiness Score

    **Score:** **\(score) / 100** (\(band))

    Key drivers:

    """
    for d in drivers {
        md += "- \(d)\n"
    }
    md += """
    Interpretation:

    - 80–100: Ready / Low modernization effort  
    - 60–79: Moderate effort required  
    - 40–59: High effort; plan a dedicated modernization cycle  
    - 0–39: Critical; significant modernization required

    ---

    ## 2. Language & UI Profile

    **Language mix:**

    - Swift: **\(swiftPercent)%**  
    - Objective‑C: **\(objcPercent)%**

    **UI stack:**

    - Detected UI stack: **\(uiStack)**  
    - SwiftUI in use: **\(artifact.ui.usesSwiftUI ? "Yes" : "No")**  
    - UIKit in use: **\(artifact.ui.usesUIKit ? "Yes" : "No")**  
    - Storyboard files: **\(artifact.ui.storyboardCount)**  
    - XIB files: **\(artifact.ui.xibCount)**  
    - Hard‑coded frame / layout risks: **\(artifact.findings.layoutRisks.count)**

    Observation:

    """
    if artifact.ui.storyboardCount > 0 {
        md += "- Storyboard presence suggests higher UI migration effort, especially for complex flows.\n"
    }
    if artifact.ui.usesSwiftUI {
        md += "- SwiftUI is already in use; consider expanding SwiftUI footprint for new screens.\n"
    }
    if artifact.summary.objcFiles > 0 {
        md += "- Objective‑C footprint may increase the cost of certain SDK and UI migrations.\n"
    }
    if artifact.ui.storyboardCount == 0 && !artifact.ui.usesSwiftUI && artifact.summary.objcFiles == 0 {
        md += "- No major UI or language risks identified in this profile.\n"
    }

    md += """
    ---

    ## 3. Dependencies & SPM Migration

    **Dependency manager:** \(artifact.dependencies.manager)

    """
    if artifact.dependencies.manager == "CocoaPods" {
        md += "The project currently uses CocoaPods. For iOS \(targetSDK) and future SDKs, migrating to Swift Package Manager where possible reduces tooling risk and simplifies maintenance.\n\n"
    }
    md += "**Pods analyzed:**\n\n"
    if artifact.dependencies.pods.isEmpty {
        md += "No pods listed (or project does not use CocoaPods).\n\n"
    } else {
        md += "| Pod name | SPM available | Difficulty | Notes |\n"
        md += "|----------|---------------|------------|-------|\n"
        for p in artifact.dependencies.pods {
            let spm = p.spm_available ? "Yes" : "No"
            md += "| \(p.name) | \(spm) | \(p.migration_difficulty) | \(p.notes) |\n"
        }
        md += "\n"
    }
    md += "Overall dependency risk score: **\(Int(depRiskNorm)) / 100**\n\n---\n\n## 4. Deprecated APIs\n\nThe following deprecated or removed APIs were detected and may affect iOS \(targetSDK) upgrades.\n\n"
    if artifact.findings.deprecated_apis.isEmpty {
        md += "No deprecated APIs from the current ruleset were detected.\n\n"
    } else {
        md += "| Symbol | File | Line | Severity | Replacement | Notes |\n"
        md += "|--------|------|------|----------|-------------|-------|\n"
        for f in artifact.findings.deprecated_apis {
            md += "| `\(f.symbol)` | `\(f.file)` | \(f.line) | \(f.severity) | `\(f.replacement)` | \(f.notes) |\n"
        }
        md += "\nBlocking items (severity = blocking) should be addressed before attempting a full iOS \(targetSDK) rollout.\n\n"
    }

    // Liquid Glass section
    md += "---\n\n## 5. Liquid Glass Readiness\n\n"
    if let lg = artifact.liquidGlass {
        let levelLabel: String
        switch lg.readinessLevel {
        case "adopted": levelLabel = "Adopted"
        case "partial": levelLabel = "Partial"
        default: levelLabel = "Not Started"
        }
        md += "**Readiness level:** **\(levelLabel)**\n\n"
        md += "- Positive signals (native glass API usage): **\(lg.positiveSignalCount)**\n"
        md += "- Legacy materials needing migration: **\(lg.migrationTargetCount)**\n"
        md += "- Anti-patterns (manual blurs, hard-coded overlays): **\(lg.antiPatternCount)**\n\n"

        let migrationFindings = lg.findings.filter { $0.category == "migration" }
        let antiPatternFindings = lg.findings.filter { $0.category == "anti_pattern" }

        if !migrationFindings.isEmpty {
            md += "**Migration targets:**\n\n"
            md += "| Pattern | File | Line | Replacement | Notes |\n"
            md += "|---------|------|------|-------------|-------|\n"
            for f in migrationFindings {
                md += "| `\(f.pattern)` | `\(f.file)` | \(f.line) | `\(f.replacement ?? "—")` | \(f.notes) |\n"
            }
            md += "\n"
        }

        if !antiPatternFindings.isEmpty {
            md += "**Anti-patterns to fix:**\n\n"
            md += "| Pattern | File | Line | Severity | Replacement | Notes |\n"
            md += "|---------|------|------|----------|-------------|-------|\n"
            for f in antiPatternFindings {
                md += "| `\(f.pattern)` | `\(f.file)` | \(f.line) | \(f.severity) | `\(f.replacement ?? "—")` | \(f.notes) |\n"
            }
            md += "\n"
        }

        if lg.positiveSignalCount > 0 && lg.migrationTargetCount == 0 && lg.antiPatternCount == 0 {
            md += "The codebase has adopted Liquid Glass APIs with no legacy materials or anti-patterns detected.\n\n"
        }
    } else {
        md += "Liquid Glass analysis was not performed (rules not loaded).\n\n"
    }

    md += "---\n\n## 6. Key Risks\n\nBased on the deterministic analysis, the main risks are:\n\n"
    if hasBlocking {
        md += "- **Blocking deprecated APIs present** – must be removed or replaced to ensure compatibility.\n"
    }
    if hasHighDepRisk {
        md += "- **Dependency risk** – one or more CocoaPods lack SPM equivalents or show high migration difficulty.\n"
    }
    if artifact.ui.storyboardCount > 0 {
        md += "- **Storyboard‑heavy UIKit UI** – increases the effort required for UI modernization and Glass‑style adaptation.\n"
    }
    if objcPercentFloat > 0.3 {
        md += "- **Significant Objective‑C footprint** – increases complexity of adopting modern Swift patterns and APIs.\n"
    }
    if let lg = artifact.liquidGlass, lg.antiPatternCount > 0 {
        md += "- **Liquid Glass anti-patterns** – manual blurs or hard-coded overlays that bypass system accessibility settings.\n"
    }
    if let lg = artifact.liquidGlass, lg.migrationTargetCount > 0 {
        md += "- **Legacy material usage** – old material APIs that should migrate to native `.glassEffect()` for iOS \(targetSDK).\n"
    }
    let hasLGIssues = (artifact.liquidGlass?.antiPatternCount ?? 0) + (artifact.liquidGlass?.migrationTargetCount ?? 0) > 0
    if !hasBlocking && !hasHighDepRisk && artifact.ui.storyboardCount == 0 && objcPercentFloat <= 0.3 && !hasLGIssues {
        md += "- No critical risks identified.\n"
    }

    md += """
    ---

    ## 7. Recommended Next Steps

    **Phase 1 – Critical fixes (1–2 weeks)**

    1. Remove or replace all **blocking** deprecated APIs listed above.
    2. Identify high‑risk dependencies and evaluate SPM or alternative libraries.
    3. Stabilize UI flows that rely on fragile storyboard patterns.

    **Phase 2 – UI & Dependency Modernization (2–4 weeks)**

    1. Plan CocoaPods → SPM migration for pods with low/medium difficulty.
    2. Gradually move new or refactored screens toward SwiftUI where appropriate.
    3. Reduce hard‑coded frame usage in critical views to improve layout resilience.
    4. Replace legacy materials (`.ultraThinMaterial`, etc.) with `.glassEffect()`.
    5. Replace manual `.blur(radius:)` with native Liquid Glass APIs.

    **Phase 3 – Readiness Re‑scan**

    1. Re‑run `glassforge analyze` after Phase 1–2.
    2. Verify that the iOS Readiness Score has improved and blocking risks are removed.
    3. Use the new score and report as input for your iOS \(targetSDK) rollout plan.

    ---

    _This report was generated locally by GlassForge v0.2.0 using deterministic analysis only. No source code was uploaded._
    """
    return md
}
