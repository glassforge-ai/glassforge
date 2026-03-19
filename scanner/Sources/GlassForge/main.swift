import ArgumentParser
import Foundation

@main
struct GlassForgeCLI: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "glassforge",
        abstract: "iOS modernization intelligence engine — deterministic analyzer.",
        version: "0.2.0",
        subcommands: [
            AnalyzeCommand.self,
            ValidateCommand.self,
            ExportCommand.self,
            ReportCommand.self,
            UploadCommand.self,
            VersionCommand.self,
        ],
        defaultSubcommand: nil
    )
}
