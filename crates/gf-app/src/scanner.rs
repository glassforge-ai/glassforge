//! Scanner subprocess management.
//!
//! Locates the GlassForge Swift scanner binary and invokes it as a child process,
//! streaming its output to the terminal.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result, bail};

/// Search order for the scanner binary.
const SCANNER_RELATIVE: &str = "scanner/.build/release/glassforge";
const SCANNER_IN_PATH: &str = "glassforge-scanner";

/// Locate the scanner binary, returning the first path that exists.
///
/// Search order:
/// 1. `<workspace_root>/scanner/.build/release/glassforge`
/// 2. `glassforge-scanner` on `$PATH`
pub fn find_scanner() -> Result<PathBuf> {
    // Try relative path from the workspace root (one level up from crates/).
    // We use the CARGO_MANIFEST_DIR at compile time, but at runtime we look
    // relative to the current working directory first.
    let relative = PathBuf::from(SCANNER_RELATIVE);
    if relative.exists() {
        return Ok(relative.canonicalize()?);
    }

    // Try to find it on PATH.
    if let Ok(output) = Command::new("which").arg(SCANNER_IN_PATH).output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Ok(PathBuf::from(path_str));
            }
        }
    }

    bail!(
        "Scanner binary not found.\n\
         Looked for:\n  \
         1. ./{SCANNER_RELATIVE}\n  \
         2. `{SCANNER_IN_PATH}` on $PATH\n\n\
         Build it with: cd scanner && swift build -c release"
    )
}

/// Run the scanner's `analyze` subcommand on the given path.
///
/// Returns the child process exit status so the caller can propagate the exit code.
pub fn run_scan(
    scanner_bin: &Path,
    repo_path: &str,
    output_dir: Option<&str>,
    report: bool,
    fail_under: Option<u32>,
) -> Result<ExitStatus> {
    let mut cmd = Command::new(scanner_bin);
    cmd.arg("analyze").arg(repo_path);

    if let Some(dir) = output_dir {
        cmd.arg("--output-dir").arg(dir);
    }
    if report {
        cmd.arg("--report");
    }
    if let Some(threshold) = fail_under {
        cmd.arg("--fail-under").arg(threshold.to_string());
    }

    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    tracing::info!(scanner = %scanner_bin.display(), repo = repo_path, "launching scanner");

    let status = cmd
        .status()
        .with_context(|| format!("failed to execute scanner at {}", scanner_bin.display()))?;

    Ok(status)
}
