import ArgumentParser
import Foundation

struct ExportCommand: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "export",
        abstract: "Generate a report from an existing artifact."
    )

    @Argument(help: "Path to glassforge-artifact.json (or directory containing it)")
    var artifactPath: String = "."

    @Option(name: [.long], help: "Output format (markdown)")
    var format: ExportFormat = .markdown

    @Option(name: [.short, .long], help: "Output file path (default: glassforge_report.md next to artifact)")
    var output: String?

    @Flag(name: [.long], help: "Quiet (only errors)")
    var quiet: Bool = false

    func run() throws {
        let artifactURL = try resolveArtifactURL(artifactPath)

        let data = try Data(contentsOf: artifactURL)
        let artifact = try JSONDecoder().decode(GlassForgeArtifact.self, from: data)

        switch format {
        case .markdown:
            let markdown = generateMarkdownReport(artifact: artifact)
            let reportURL: URL
            if let output = output {
                reportURL = URL(fileURLWithPath: (output as NSString).expandingTildeInPath)
            } else {
                reportURL = artifactURL.deletingLastPathComponent().appendingPathComponent("glassforge_report.md")
            }
            try markdown.write(to: reportURL, atomically: true, encoding: .utf8)
            if !quiet { print("Report written to \(reportURL.path)") }
        }
    }
}

enum ExportFormat: String, ExpressibleByArgument, CaseIterable {
    case markdown
}
