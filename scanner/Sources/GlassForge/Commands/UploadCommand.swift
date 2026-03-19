import ArgumentParser
import Foundation

struct UploadCommand: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "upload",
        abstract: "Upload a GlassForge artifact to the cloud API (artifact only, no source code)."
    )

    @Argument(help: "Path to glassforge-artifact.json (or directory containing it)")
    var artifactPath: String = "."

    @Option(name: [.long], help: "GlassForge cloud API endpoint")
    var endpoint: String = "https://api.glassforge.dev"

    @Option(name: [.long], help: "API key (or set GLASSFORGE_API_KEY env var)")
    var apiKey: String?

    @Flag(name: [.long], help: "Quiet (only errors and run ID)")
    var quiet: Bool = false

    func run() throws {
        let artifactURL = try resolveArtifactURL(artifactPath)

        let key = apiKey ?? ProcessInfo.processInfo.environment["GLASSFORGE_API_KEY"]
        guard let resolvedKey = key, !resolvedKey.isEmpty else {
            throw UploadError.missingAPIKey
        }

        let data = try Data(contentsOf: artifactURL)

        // Validate the artifact decodes before uploading
        let artifact = try JSONDecoder().decode(GlassForgeArtifact.self, from: data)

        if !quiet {
            print("Uploading artifact (\(data.count) bytes) to \(endpoint)...")
            print("  Schema: \(artifact.schemaVersion)")
            if let score = artifact.readiness?.score, let band = artifact.readiness?.band {
                print("  Score: \(score)/100 (\(band))")
            }
        }

        // Cloud backend is not yet implemented. Print a clear message.
        print("")
        print("Upload is not yet available — cloud backend is under development.")
        print("Artifact validated and ready for upload when the API is live.")
        print("")
        print("To use this command once available:")
        print("  export GLASSFORGE_API_KEY=<your-key>")
        print("  glassforge upload ./glassforge-artifact.json")
    }
}

enum UploadError: Error, CustomStringConvertible {
    case missingAPIKey

    var description: String {
        switch self {
        case .missingAPIKey:
            return "No API key provided. Set GLASSFORGE_API_KEY or pass --api-key."
        }
    }
}
