import Foundation

/// Extract pod names from a Podfile (pod 'Name' or pod "Name" or pod 'Name/Subspec').
public func extractPodNames(from podfileURL: URL) -> [String] {
    guard let content = try? String(contentsOf: podfileURL, encoding: .utf8) else { return [] }
    var names: [String] = []
    let pattern = #"pod\s+['\"]([^'"]+)['\"]"#
    let regex = try? NSRegularExpression(pattern: pattern)
    let range = NSRange(content.startIndex..., in: content)
    regex?.enumerateMatches(in: content, options: [], range: range) { match, _, _ in
        guard let match = match, match.numberOfRanges > 1,
              let r = Range(match.range(at: 1), in: content) else { return }
        let name = String(content[r])
        if !name.isEmpty && !name.hasPrefix("#") {
            names.append(name)
        }
    }
    return names
}

/// Look up each pod name in rules; return PodInfo list. Uses base name (e.g. "Alamofire" from "Alamofire/Network").
public func lookupPods(podNames: [String], rules: DependenciesRulesFile) -> [PodInfo] {
    var result: [PodInfo] = []
    let ruleByName: [String: DependencyPodRule] = Dictionary(uniqueKeysWithValues: rules.pods.map { ($0.name, $0) })

    for name in podNames {
        let baseName = name.split(separator: "/").map(String.init).first ?? name
        if let rule = ruleByName[baseName] ?? ruleByName[name] {
            result.append(PodInfo(
                name: name,
                spm_available: rule.spm_url != nil,
                spm_url: rule.spm_url,
                migration_difficulty: rule.migration_difficulty,
                notes: rule.notes
            ))
        } else {
            result.append(PodInfo(
                name: name,
                spm_available: false,
                spm_url: nil,
                migration_difficulty: "unknown",
                notes: "Not in rules; check for SPM support manually."
            ))
        }
    }
    return result
}
