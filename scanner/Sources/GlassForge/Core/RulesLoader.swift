import Foundation

/// A single deprecated API rule from rules/deprecated_apis_ios26.json
public struct DeprecatedAPIRule: Codable, Sendable {
    public let symbol: String
    public let since_sdk: Int
    public let replacement: String
    public let severity: String
    public let category: String
    public let notes: String
}

/// A single pod rule from rules/dependencies_ios26.json
public struct DependencyPodRule: Codable, Sendable {
    public let name: String
    public let spm_url: String?
    public let migration_difficulty: String
    public let notes: String
}

public struct DependenciesRulesFile: Codable, Sendable {
    public let pods: [DependencyPodRule]
}

/// Loads rule files from the Rules directory (bundle or fallback path).
public enum RulesLoader: Sendable {
    /// Bundle that contains Rules (SPM generates .module when target has resources).
    public static var rulesBundle: Bundle { Bundle.module }

    /// Load deprecated API rules. Returns empty array if file missing or invalid.
    public static func loadDeprecatedAPIRules(bundle: Bundle? = nil) -> [DeprecatedAPIRule] {
        let b = bundle ?? rulesBundle
        guard let url = b.url(forResource: "deprecated_apis_ios26", withExtension: "json", subdirectory: "Rules")
            ?? b.url(forResource: "deprecated_apis_ios26", withExtension: "json") else {
            return []
        }
        guard let data = try? Data(contentsOf: url),
              let rules = try? JSONDecoder().decode([DeprecatedAPIRule].self, from: data) else {
            return []
        }
        return rules
    }

    /// Load dependency rules. Returns empty pods array if file missing or invalid.
    public static func loadDependencyRules(bundle: Bundle? = nil) -> DependenciesRulesFile {
        let b = bundle ?? rulesBundle
        guard let url = b.url(forResource: "dependencies_ios26", withExtension: "json", subdirectory: "Rules")
            ?? b.url(forResource: "dependencies_ios26", withExtension: "json") else {
            return DependenciesRulesFile(pods: [])
        }
        guard let data = try? Data(contentsOf: url),
              let file = try? JSONDecoder().decode(DependenciesRulesFile.self, from: data) else {
            return DependenciesRulesFile(pods: [])
        }
        return file
    }

    /// Load Liquid Glass rules. Returns nil if file missing or invalid.
    public static func loadLiquidGlassRules(bundle: Bundle? = nil) -> LiquidGlassRules? {
        let b = bundle ?? rulesBundle
        guard let url = b.url(forResource: "liquid_glass_ios26", withExtension: "json", subdirectory: "Rules")
            ?? b.url(forResource: "liquid_glass_ios26", withExtension: "json") else {
            return nil
        }
        guard let data = try? Data(contentsOf: url),
              let rules = try? JSONDecoder().decode(LiquidGlassRules.self, from: data) else {
            return nil
        }
        return rules
    }
}
