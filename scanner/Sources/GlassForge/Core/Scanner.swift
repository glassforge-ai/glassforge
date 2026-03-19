import Foundation

/// Result of scanning a repository root: file paths by type.
public struct ScanResult: Sendable {
    public var swiftFiles: [URL]
    public var objcFiles: [URL]
    public var storyboards: [URL]
    public var xibs: [URL]
    public var totalLines: Int
    public var xcodeProjectDetected: Bool

    public init(swiftFiles: [URL] = [], objcFiles: [URL] = [], storyboards: [URL] = [], xibs: [URL] = [], totalLines: Int = 0, xcodeProjectDetected: Bool = false) {
        self.swiftFiles = swiftFiles
        self.objcFiles = objcFiles
        self.storyboards = storyboards
        self.xibs = xibs
        self.totalLines = totalLines
        self.xcodeProjectDetected = xcodeProjectDetected
    }
}

/// Scans a directory for Swift/Obj-C/UI files and counts lines.
public struct DirectoryScanner: Sendable {
    public static let defaultSwiftLimit = 10_000
    public static let defaultObjCLimit = 5_000

    private let swiftLimit: Int
    private let objcLimit: Int
    private let excludePatterns: [String]

    public init(swiftLimit: Int = defaultSwiftLimit, objcLimit: Int = defaultObjCLimit, excludePatterns: [String] = []) {
        self.swiftLimit = swiftLimit
        self.objcLimit = objcLimit
        self.excludePatterns = excludePatterns
    }

    public func scan(at url: URL) throws -> ScanResult {
        var result = ScanResult()
        let fm = FileManager.default

        // Detect Xcode project
        let contents = (try? fm.contentsOfDirectory(atPath: url.path)) ?? []
        result.xcodeProjectDetected = contents.contains { $0.hasSuffix(".xcodeproj") || $0.hasSuffix(".xcworkspace") }

        guard let enumerator = fm.enumerator(
            at: url,
            includingPropertiesForKeys: [.isRegularFileKey],
            options: [.skipsHiddenFiles]
        ) else {
            throw ScannerError.cannotEnumerate(url)
        }

        while let item = enumerator.nextObject() as? URL {
            let path = item.path
            if path.contains("/.git/") || path.contains("DerivedData") || path.contains(".build/") {
                enumerator.skipDescendants()
                continue
            }

            // Apply user exclude patterns
            let relativePath = path.replacingOccurrences(of: url.path + "/", with: "")
            if excludePatterns.contains(where: { relativePath.hasPrefix($0) || relativePath.contains("/\($0)") }) {
                enumerator.skipDescendants()
                continue
            }

            guard let resource = try? item.resourceValues(forKeys: [.isRegularFileKey]),
                  resource.isRegularFile == true else { continue }

            switch item.pathExtension.lowercased() {
            case "swift":
                if result.swiftFiles.count < swiftLimit {
                    result.swiftFiles.append(item)
                }
            case "m", "mm":
                if result.objcFiles.count < objcLimit {
                    result.objcFiles.append(item)
                }
            case "storyboard":
                result.storyboards.append(item)
            case "xib":
                result.xibs.append(item)
            default:
                break
            }
        }

        var totalLines = 0
        for u in result.swiftFiles + result.objcFiles {
            if let count = lineCount(url: u) {
                totalLines += count
            }
        }
        result.totalLines = totalLines

        return result
    }

    private func lineCount(url: URL) -> Int? {
        guard let data = try? Data(contentsOf: url),
              let content = String(data: data, encoding: .utf8) else { return nil }
        return content.components(separatedBy: .newlines).count
    }
}

public enum ScannerError: Error, Sendable {
    case cannotEnumerate(URL)
}
