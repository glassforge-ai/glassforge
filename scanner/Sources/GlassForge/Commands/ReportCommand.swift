import ArgumentParser
import Foundation

struct ReportCommand: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "report",
        abstract: "Generate glassforge_report.md from an existing artifact (no AI)."
    )

    @Argument(help: "Path to glassforge-artifact.json (or directory containing it)")
    var artifactPath: String = "."

    @Option(name: [.short, .long], help: "Output path for the Markdown report")
    var output: String?

    @Flag(name: [.long], help: "Quiet")
    var quiet: Bool = false

    func run() throws {
        let artifactURL = try resolveArtifactURL(artifactPath)

        let data = try Data(contentsOf: artifactURL)
        let artifact = try JSONDecoder().decode(GlassForgeArtifact.self, from: data)
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
