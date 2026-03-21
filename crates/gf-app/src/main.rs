//! GlassForge CLI: iOS modernization platform.
//!
//! Subcommands:
//!   scan     — invoke the Swift scanner on an iOS codebase
//!   migrate  — run AI-powered migration on scan findings
//!   verify   — compare before/after scan artifacts

use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "glassforge", version, about = "iOS modernization platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan an iOS codebase for modernization readiness
    Scan {
        /// Path to the iOS repository
        path: String,
        /// Output directory for artifacts (default: repo dir)
        #[arg(long)]
        output_dir: Option<String>,
        /// Also generate markdown report
        #[arg(long)]
        report: bool,
        /// Fail if score is below this threshold
        #[arg(long)]
        fail_under: Option<u32>,
    },
    /// Run AI-powered migration on scan findings
    Migrate {
        /// Path to GlassForge scan artifact JSON
        #[arg(long, short)]
        artifact: String,
        /// Path to the iOS repository to modernize
        #[arg(long, short)]
        repo: String,
        /// Dry run: plan only, don't execute
        #[arg(long)]
        dry_run: bool,
        /// Skip git worktree isolation
        #[arg(long)]
        no_worktree: bool,
    },
    /// Compare before/after scan artifacts
    Verify {
        /// Path to the before-migration artifact
        #[arg(long)]
        before: String,
        /// Path to the after-migration artifact
        #[arg(long)]
        after: String,
    },
    /// Start the web server with embedded frontend
    Serve {
        /// Bind address
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port number
        #[arg(long, default_value_t = 4173)]
        port: u16,
        /// Path to personas directory
        #[arg(long)]
        personas_dir: Option<String>,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    // Initialize tracing.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            path,
            output_dir,
            report,
            fail_under,
        } => cmd_scan(&path, output_dir.as_deref(), report, fail_under),

        Commands::Migrate {
            artifact,
            repo,
            dry_run,
            no_worktree,
        } => cmd_migrate(&artifact, &repo, dry_run, no_worktree).await,

        Commands::Verify { before, after } => cmd_verify(&before, &after),

        Commands::Serve {
            host,
            port,
            personas_dir,
        } => gf_app::serve::cmd_serve(&host, port, personas_dir.as_deref()).await,
    }
}

// ---------------------------------------------------------------------------
// scan
// ---------------------------------------------------------------------------

fn cmd_scan(
    path: &str,
    output_dir: Option<&str>,
    report: bool,
    fail_under: Option<u32>,
) -> ExitCode {
    let scanner_bin = match gf_app::scanner::find_scanner() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::from(1);
        }
    };

    tracing::info!(scanner = %scanner_bin.display(), "found scanner binary");

    match gf_app::scanner::run_scan(&scanner_bin, path, output_dir, report, fail_under) {
        Ok(status) => {
            if status.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(status.code().unwrap_or(1) as u8)
            }
        }
        Err(e) => {
            eprintln!("Error running scanner: {e}");
            ExitCode::from(1)
        }
    }
}

// ---------------------------------------------------------------------------
// migrate
// ---------------------------------------------------------------------------

async fn cmd_migrate(artifact: &str, repo: &str, dry_run: bool, no_worktree: bool) -> ExitCode {
    let artifact_path = Path::new(artifact);
    let repo_path = Path::new(repo);

    if !artifact_path.exists() {
        eprintln!("Error: artifact file does not exist: {artifact}");
        return ExitCode::from(1);
    }
    if !repo_path.is_dir() {
        eprintln!("Error: repo path is not a directory: {repo}");
        return ExitCode::from(1);
    }

    match gf_app::migrate::run_migrate(artifact_path, repo_path, dry_run, no_worktree).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}

// ---------------------------------------------------------------------------
// verify
// ---------------------------------------------------------------------------

fn cmd_verify(before: &str, after: &str) -> ExitCode {
    let before_path = Path::new(before);
    let after_path = Path::new(after);

    match gf_app::verify::run_verify(before_path, after_path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}
