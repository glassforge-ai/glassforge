import Foundation

/// Counts mixed modules: directories that contain both Swift and Obj-C files.
public func countMixedModules(swiftFiles: [URL], objcFiles: [URL]) -> Int {
    func directory(for url: URL) -> String {
        url.deletingLastPathComponent().path
    }
    let swiftDirs = Set(swiftFiles.map { directory(for: $0) })
    let objcDirs = Set(objcFiles.map { directory(for: $0) })
    return swiftDirs.intersection(objcDirs).count
}
