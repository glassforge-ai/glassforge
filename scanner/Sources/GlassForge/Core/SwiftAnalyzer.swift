import Foundation

/// Known deprecated symbols (iOS 26 / common). Key: symbol name substring; Value: severity.
public enum DeprecatedSymbols {
    public static let list: [(pattern: String, severity: String)] = [
        ("keyWindow", "high"),
        ("UIApplication.shared.keyWindow", "high"),
        ("UIScreen.main.bounds", "medium"),
        ("UIApplication.shared.windows", "high"),
        ("statusBarFrame", "medium"),
        ("UIApplication.shared.statusBarFrame", "medium"),
    ]
}

/// Layout risk patterns (fixed frame, constant).
public enum LayoutRiskPatterns {
    public static let list: [(pattern: String, kind: String)] = [
        (".constant(", "constraintConstant"),
        ("frame =", "fixedFrame"),
        ("CGRect(", "fixedFrame"),
        (".frame(width:", "fixedFrame"),
        (".frame(height:", "fixedFrame"),
    ]
}

/// Result of analyzing Swift/Obj-C sources: deprecated API findings (from rules), layout risks, UI usage.
public struct SwiftAnalysisResult: Sendable {
    public var deprecatedApis: [Finding]
    public var deprecated_apis: [DeprecatedAPIFinding]
    public var layoutRisks: [Finding]
    public var usesSwiftUI: Bool
    public var usesUIKit: Bool
    public var swiftUIFileCount: Int
    public var uiKitFileCount: Int

    public init(deprecatedApis: [Finding] = [], deprecated_apis: [DeprecatedAPIFinding] = [], layoutRisks: [Finding] = [], usesSwiftUI: Bool = false, usesUIKit: Bool = false, swiftUIFileCount: Int = 0, uiKitFileCount: Int = 0) {
        self.deprecatedApis = deprecatedApis
        self.deprecated_apis = deprecated_apis
        self.layoutRisks = layoutRisks
        self.usesSwiftUI = usesSwiftUI
        self.usesUIKit = usesUIKit
        self.swiftUIFileCount = swiftUIFileCount
        self.uiKitFileCount = uiKitFileCount
    }
}

public struct SwiftAnalyzer: Sendable {
    private let deprecatedPatterns: [(pattern: String, severity: String)]
    private let layoutPatterns: [(pattern: String, kind: String)]

    public init(
        deprecatedPatterns: [(pattern: String, severity: String)] = DeprecatedSymbols.list,
        layoutPatterns: [(pattern: String, kind: String)] = LayoutRiskPatterns.list
    ) {
        self.deprecatedPatterns = deprecatedPatterns
        self.layoutPatterns = layoutPatterns
    }

    /// Analyze Swift and optionally Obj-C files; use rules for deprecated API (produces deprecated_apis).
    public func analyze(swiftFiles: [URL], objcFiles: [URL] = [], repoRoot: URL, deprecatedRules: [DeprecatedAPIRule]) -> SwiftAnalysisResult {
        var result = SwiftAnalysisResult()
        let allCodeFiles = swiftFiles + objcFiles
        for url in swiftFiles {
            guard let content = try? String(contentsOf: url, encoding: .utf8) else { continue }
            if content.contains("import SwiftUI") {
                result.usesSwiftUI = true
                result.swiftUIFileCount += 1
            }
            if content.contains("import UIKit") {
                result.usesUIKit = true
                result.uiKitFileCount += 1
            }
        }
        for url in allCodeFiles {
            guard let content = try? String(contentsOf: url, encoding: .utf8) else { continue }
            let relativePath = url.path.replacingOccurrences(of: repoRoot.path + "/", with: "")
            let lines = content.components(separatedBy: .newlines)

            for (index, line) in lines.enumerated() {
                let lineNumber = index + 1
                for rule in deprecatedRules where line.contains(rule.symbol) {
                    result.deprecated_apis.append(DeprecatedAPIFinding(
                        symbol: rule.symbol,
                        file: relativePath,
                        line: lineNumber,
                        severity: rule.severity,
                        replacement: rule.replacement,
                        notes: rule.notes
                    ))
                }
                for (pattern, severity) in deprecatedPatterns {
                    if line.contains(pattern) {
                        result.deprecatedApis.append(Finding(
                            file: relativePath,
                            line: lineNumber,
                            symbol: pattern,
                            kind: "deprecated",
                            severity: severity
                        ))
                    }
                }
                for (pattern, kind) in layoutPatterns {
                    if line.contains(pattern) {
                        result.layoutRisks.append(Finding(
                            file: relativePath,
                            line: lineNumber,
                            symbol: nil,
                            kind: kind,
                            severity: "medium"
                        ))
                    }
                }
            }
        }
        return result
    }
}
