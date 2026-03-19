import Foundation

/// Liquid Glass rule types loaded from liquid_glass_ios26.json
public struct LiquidGlassPositiveSignal: Codable, Sendable {
    public let pattern: String
    public let kind: String
    public let description: String
    public let notes: String
}

public struct LiquidGlassMigrationTarget: Codable, Sendable {
    public let pattern: String
    public let kind: String
    public let replacement: String
    public let severity: String
    public let description: String
    public let notes: String
}

public struct LiquidGlassAntiPattern: Codable, Sendable {
    public let pattern: String
    public let kind: String
    public let severity: String
    public let description: String
    public let replacement: String
    public let notes: String
}

public struct LiquidGlassRules: Codable, Sendable {
    public let positive_signals: [LiquidGlassPositiveSignal]
    public let migration_targets: [LiquidGlassMigrationTarget]
    public let anti_patterns: [LiquidGlassAntiPattern]
}

/// A single Liquid Glass finding.
public struct LiquidGlassFinding: Codable, Sendable {
    public let file: String
    public let line: Int
    public let pattern: String
    public let kind: String
    public let category: String  // "positive", "migration", "anti_pattern"
    public let severity: String  // "info" for positive, "medium"/"high" for issues
    public let replacement: String?
    public let notes: String

    public init(file: String, line: Int, pattern: String, kind: String, category: String, severity: String, replacement: String?, notes: String) {
        self.file = file
        self.line = line
        self.pattern = pattern
        self.kind = kind
        self.category = category
        self.severity = severity
        self.replacement = replacement
        self.notes = notes
    }
}

/// Aggregate Liquid Glass analysis result.
public struct LiquidGlassResult: Codable, Sendable {
    public let readinessLevel: String  // "adopted", "partial", "not_started"
    public let positiveSignalCount: Int
    public let migrationTargetCount: Int
    public let antiPatternCount: Int
    public let findings: [LiquidGlassFinding]

    public init(readinessLevel: String, positiveSignalCount: Int, migrationTargetCount: Int, antiPatternCount: Int, findings: [LiquidGlassFinding]) {
        self.readinessLevel = readinessLevel
        self.positiveSignalCount = positiveSignalCount
        self.migrationTargetCount = migrationTargetCount
        self.antiPatternCount = antiPatternCount
        self.findings = findings
    }
}

/// Analyzes Swift files for Liquid Glass adoption, migration targets, and anti-patterns.
public struct LiquidGlassAnalyzer: Sendable {
    private let rules: LiquidGlassRules

    public init(rules: LiquidGlassRules) {
        self.rules = rules
    }

    public func analyze(swiftFiles: [URL], repoRoot: URL) -> LiquidGlassResult {
        var findings: [LiquidGlassFinding] = []
        var positiveCount = 0
        var migrationCount = 0
        var antiPatternCount = 0

        for url in swiftFiles {
            guard let content = try? String(contentsOf: url, encoding: .utf8) else { continue }
            let relativePath = url.path.replacingOccurrences(of: repoRoot.path + "/", with: "")
            let lines = content.components(separatedBy: .newlines)

            for (index, line) in lines.enumerated() {
                let lineNumber = index + 1

                // Check positive signals
                for signal in rules.positive_signals where line.contains(signal.pattern) {
                    positiveCount += 1
                    findings.append(LiquidGlassFinding(
                        file: relativePath, line: lineNumber, pattern: signal.pattern,
                        kind: signal.kind, category: "positive", severity: "info",
                        replacement: nil, notes: signal.notes
                    ))
                }

                // Check migration targets
                for target in rules.migration_targets where line.contains(target.pattern) {
                    migrationCount += 1
                    findings.append(LiquidGlassFinding(
                        file: relativePath, line: lineNumber, pattern: target.pattern,
                        kind: target.kind, category: "migration", severity: target.severity,
                        replacement: target.replacement, notes: target.notes
                    ))
                }

                // Check anti-patterns
                for anti in rules.anti_patterns where line.contains(anti.pattern) {
                    antiPatternCount += 1
                    findings.append(LiquidGlassFinding(
                        file: relativePath, line: lineNumber, pattern: anti.pattern,
                        kind: anti.kind, category: "anti_pattern", severity: anti.severity,
                        replacement: anti.replacement, notes: anti.notes
                    ))
                }
            }
        }

        let readinessLevel: String
        if positiveCount > 0 && migrationCount == 0 && antiPatternCount == 0 {
            readinessLevel = "adopted"
        } else if positiveCount > 0 {
            readinessLevel = "partial"
        } else {
            readinessLevel = "not_started"
        }

        return LiquidGlassResult(
            readinessLevel: readinessLevel,
            positiveSignalCount: positiveCount,
            migrationTargetCount: migrationCount,
            antiPatternCount: antiPatternCount,
            findings: findings
        )
    }
}
