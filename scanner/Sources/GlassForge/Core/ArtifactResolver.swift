import Foundation

/// Resolve an artifact path argument to a concrete file URL.
/// Accepts a path to the JSON file directly, or a directory containing `glassforge-artifact.json`.
func resolveArtifactURL(_ path: String) throws -> URL {
    let url: URL
    if path == "." {
        url = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
    } else {
        url = URL(fileURLWithPath: (path as NSString).expandingTildeInPath)
    }

    var artifactURL = url
    var isDir: ObjCBool = false
    if FileManager.default.fileExists(atPath: url.path, isDirectory: &isDir), isDir.boolValue {
        artifactURL = url.appendingPathComponent("glassforge-artifact.json")
    }

    guard FileManager.default.fileExists(atPath: artifactURL.path) else {
        throw ArtifactResolverError.notFound(artifactURL.path)
    }
    return artifactURL
}

enum ArtifactResolverError: Error, CustomStringConvertible {
    case notFound(String)

    var description: String {
        switch self {
        case .notFound(let path):
            return "Artifact not found at \(path)"
        }
    }
}
