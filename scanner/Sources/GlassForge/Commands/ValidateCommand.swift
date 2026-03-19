import ArgumentParser
import Foundation

struct ValidateCommand: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "validate",
        abstract: "Validate a GlassForge artifact against the schema. Exits 0 if valid, 1 if invalid."
    )

    @Argument(help: "Path to glassforge-artifact.json (or directory containing it)")
    var artifactPath: String = "."

    @Flag(name: [.long], help: "Quiet (only exit code, no messages)")
    var quiet: Bool = false

    func run() throws {
        let artifactURL = try resolveArtifactURL(artifactPath)

        let data: Data
        do {
            data = try Data(contentsOf: artifactURL)
        } catch {
            throw ValidateError.cannotRead(artifactURL.path, error.localizedDescription)
        }

        let artifact: GlassForgeArtifact
        do {
            artifact = try JSONDecoder().decode(GlassForgeArtifact.self, from: data)
        } catch let error as DecodingError {
            if !quiet {
                print("INVALID: \(artifactURL.path)")
                switch error {
                case .keyNotFound(let key, let context):
                    print("  Missing key: \(key.stringValue) at \(context.codingPath.map(\.stringValue).joined(separator: "."))")
                case .typeMismatch(let type, let context):
                    print("  Type mismatch: expected \(type) at \(context.codingPath.map(\.stringValue).joined(separator: "."))")
                case .valueNotFound(let type, let context):
                    print("  Value not found: \(type) at \(context.codingPath.map(\.stringValue).joined(separator: "."))")
                case .dataCorrupted(let context):
                    print("  Data corrupted at \(context.codingPath.map(\.stringValue).joined(separator: "."))")
                @unknown default:
                    print("  Decode error: \(error)")
                }
            }
            throw ExitCode(1)
        } catch {
            if !quiet {
                print("INVALID: \(artifactURL.path)")
                print("  Error: \(error)")
            }
            throw ExitCode(1)
        }

        var warnings: [String] = []

        if artifact.schemaVersion != ArtifactSchema.version {
            warnings.append("Schema version mismatch: artifact has \(artifact.schemaVersion), CLI expects \(ArtifactSchema.version)")
        }
        if artifact.summary.swiftFiles == 0 && artifact.summary.objcFiles == 0 {
            warnings.append("No Swift or Obj-C files in summary")
        }
        if artifact.readiness == nil {
            warnings.append("Missing readiness score")
        }
        if artifact.signals == nil {
            warnings.append("Missing risk signals")
        }

        if !quiet {
            print("VALID: \(artifactURL.path)")
            print("  Schema: \(artifact.schemaVersion)")
            print("  Generated: \(artifact.generatedAt)")
            if let score = artifact.readiness?.score, let band = artifact.readiness?.band {
                print("  Score: \(score)/100 (\(band))")
            }
            for w in warnings {
                print("  WARNING: \(w)")
            }
        }
    }
}

enum ValidateError: Error, CustomStringConvertible {
    case cannotRead(String, String)

    var description: String {
        switch self {
        case .cannotRead(let path, let reason):
            return "Cannot read artifact at \(path): \(reason)"
        }
    }
}
