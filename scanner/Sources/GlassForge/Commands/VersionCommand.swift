import ArgumentParser
import Foundation

struct VersionCommand: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "version",
        abstract: "Print GlassForge version and artifact schema version."
    )

    func run() throws {
        print("GlassForge 0.2.0")
        print("Artifact schema: \(ArtifactSchema.version)")
    }
}
