//! Git worktree management for agent session isolation.
//!
//! Each agent session gets its own worktree so concurrent agents
//! can work on the same repo without conflicts.

use std::path::{Path, PathBuf};
use std::process::Command;

use gf_core::ForgeError;

/// Information about an active worktree.
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub session_id: String,
}

/// Create a git worktree for a session.
pub fn create_worktree(repo_dir: &Path, session_id: &str) -> Result<PathBuf, ForgeError> {
    let worktree_dir = repo_dir.join(".worktrees").join(session_id);
    let branch = format!("forge/{}", session_id);

    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            &worktree_dir.to_string_lossy(),
            "-b",
            &branch,
        ])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| ForgeError::Internal(format!("failed to run git worktree add: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ForgeError::Internal(format!(
            "git worktree add failed: {}",
            stderr.trim()
        )));
    }

    Ok(worktree_dir)
}

/// Remove a worktree and delete its branch.
pub fn remove_worktree(repo_dir: &Path, session_id: &str) -> Result<(), ForgeError> {
    let worktree_dir = repo_dir.join(".worktrees").join(session_id);
    let branch = format!("forge/{}", session_id);

    let _ = Command::new("git")
        .args([
            "worktree",
            "remove",
            &worktree_dir.to_string_lossy(),
            "--force",
        ])
        .current_dir(repo_dir)
        .output();

    let _ = Command::new("git")
        .args(["branch", "-D", &branch])
        .current_dir(repo_dir)
        .output();

    Ok(())
}

/// List all forge-managed worktrees (branches matching `forge/*`).
pub fn list_worktrees(repo_dir: &Path) -> Result<Vec<WorktreeInfo>, ForgeError> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| ForgeError::Internal(format!("failed to run git worktree list: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = Some(branch.to_string());
        } else if line.is_empty() {
            if let (Some(path), Some(ref branch)) = (&current_path, &current_branch) {
                if let Some(sid) = branch.strip_prefix("forge/") {
                    worktrees.push(WorktreeInfo {
                        path: path.clone(),
                        branch: branch.clone(),
                        session_id: sid.to_string(),
                    });
                }
            }
            current_path = None;
            current_branch = None;
        }
    }

    // Handle last entry if stdout doesn't end with empty line
    if let (Some(ref path), Some(ref branch)) = (&current_path, &current_branch) {
        if let Some(sid) = branch.strip_prefix("forge/") {
            worktrees.push(WorktreeInfo {
                path: path.clone(),
                branch: branch.clone(),
                session_id: sid.to_string(),
            });
        }
    }

    Ok(worktrees)
}

/// Check if a directory is inside a git repository.
pub fn is_git_repo(dir: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
