//! Persona loader and matcher for GlassForge.
//!
//! Reads `.md` persona files with YAML-style frontmatter, parses them into
//! [`Persona`] structs, and provides lookup by name or finding source.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum PersonaError {
    #[error("failed to read directory {path}: {source}")]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to read file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
}

// ---------------------------------------------------------------------------
// Persona
// ---------------------------------------------------------------------------

/// A loaded persona with parsed frontmatter and full markdown body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Human-readable name from frontmatter `name:` field.
    pub name: String,
    /// One-line description from frontmatter `description:` field.
    pub description: String,
    /// The full markdown body after the frontmatter (used as the system prompt).
    pub system_prompt: String,
    /// Optional color hint for UI display.
    pub color: Option<String>,
    /// Optional emoji for UI display.
    pub emoji: Option<String>,
    /// Optional vibe / tagline.
    pub vibe: Option<String>,
    /// Path of the source file relative to the load directory.
    pub source_file: String,
}

// ---------------------------------------------------------------------------
// Frontmatter parser
// ---------------------------------------------------------------------------

/// Parse YAML-style frontmatter delimited by `---` lines.
///
/// Returns `(frontmatter_map, body)` where body is everything after the
/// closing `---`. If no frontmatter is found, the map is empty and the
/// entire content is returned as the body.
fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
    let mut lines = content.lines();

    // First line must be exactly "---"
    match lines.next() {
        Some(line) if line.trim() == "---" => {}
        _ => {
            return (HashMap::new(), content.to_string());
        }
    }

    let mut fm_lines = Vec::new();
    let mut found_end = false;
    for line in lines.by_ref() {
        if line.trim() == "---" {
            found_end = true;
            break;
        }
        fm_lines.push(line);
    }

    if !found_end {
        // Malformed — treat entire content as body.
        return (HashMap::new(), content.to_string());
    }

    // Parse key: value pairs from frontmatter
    let mut map = HashMap::new();
    for line in &fm_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim().to_string();
            if !key.is_empty() && !value.is_empty() {
                map.insert(key, value);
            }
        }
    }

    // Remaining lines are the body
    let body: String = lines.collect::<Vec<_>>().join("\n");
    // Trim leading blank lines from body
    let body = body.trim_start_matches('\n').to_string();

    (map, body)
}

// ---------------------------------------------------------------------------
// PersonaLoader
// ---------------------------------------------------------------------------

/// Loads persona `.md` files from a directory and provides lookup.
pub struct PersonaLoader {
    personas: Vec<Persona>,
}

impl PersonaLoader {
    /// Load all `.md` persona files from `dir` (recursively).
    pub fn load_from_dir(dir: &Path) -> Result<Self, PersonaError> {
        let mut personas = Vec::new();
        walk_md_files(dir, dir, &mut personas)?;
        tracing::info!(count = personas.len(), dir = %dir.display(), "loaded personas");
        Ok(Self { personas })
    }

    /// Find a persona by name using case-insensitive partial match.
    ///
    /// An exact (case-insensitive) match is preferred. If none is found,
    /// the first persona whose name contains `name` (case-insensitive) is
    /// returned.
    pub fn find_by_name(&self, name: &str) -> Option<&Persona> {
        let lower = name.to_lowercase();
        // Prefer exact match
        if let Some(p) = self.personas.iter().find(|p| p.name.to_lowercase() == lower) {
            return Some(p);
        }
        // Fall back to substring match
        self.personas
            .iter()
            .find(|p| p.name.to_lowercase().contains(&lower))
    }

    /// Return all loaded personas.
    pub fn all(&self) -> &[Persona] {
        &self.personas
    }
}

/// Recursively walk `dir` and parse every `.md` file into a `Persona`.
fn walk_md_files(
    dir: &Path,
    root: &Path,
    out: &mut Vec<Persona>,
) -> Result<(), PersonaError> {
    let entries = fs::read_dir(dir).map_err(|source| PersonaError::ReadDir {
        path: dir.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| PersonaError::ReadDir {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            walk_md_files(&path, root, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            match parse_persona_file(&path, root) {
                Ok(persona) => out.push(persona),
                Err(e) => {
                    tracing::warn!(error = %e, "skipping persona file");
                }
            }
        }
    }
    Ok(())
}

/// Parse a single `.md` file into a [`Persona`].
fn parse_persona_file(path: &Path, root: &Path) -> Result<Persona, PersonaError> {
    let content = fs::read_to_string(path).map_err(|source| PersonaError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;

    let (fm, body) = parse_frontmatter(&content);

    let rel = path.strip_prefix(root).unwrap_or(path);
    let source_file = rel.to_string_lossy().to_string();

    // Derive name: prefer frontmatter, fall back to first `# ` heading, then filename.
    let name = fm
        .get("name")
        .cloned()
        .or_else(|| {
            body.lines()
                .find(|l| l.trim().starts_with("# "))
                .map(|l| l.trim().trim_start_matches("# ").trim().to_string())
        })
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("persona")
                .replace('-', " ")
        });

    let description = fm
        .get("description")
        .cloned()
        .unwrap_or_default();

    let color = fm.get("color").cloned();
    let emoji = fm.get("emoji").cloned();
    let vibe = fm.get("vibe").cloned();

    Ok(Persona {
        name,
        description,
        system_prompt: body,
        color,
        emoji,
        vibe,
        source_file,
    })
}

// ---------------------------------------------------------------------------
// Finding-source mapping
// ---------------------------------------------------------------------------

/// Given a finding source identifier, return the persona name best suited
/// to handle it.
pub fn persona_for_finding_source(source: &str) -> &'static str {
    match source {
        "DeprecatedApi" => "iOS Deprecated API Fixer",
        "LiquidGlass" => "iOS Liquid Glass Migrator",
        _ => "iOS Deprecated API Fixer",
    }
}

/// The persona name of the migration verifier.
pub fn verifier_persona_name() -> &'static str {
    "iOS Migration Verifier"
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_frontmatter_basic() {
        let md = "\
---
name: iOS Liquid Glass Migrator
description: Migrates legacy effects
color: blue
emoji: x
vibe: Transforms blurs
---

# iOS Liquid Glass Migrator

Body content here.
";
        let (fm, body) = parse_frontmatter(md);
        assert_eq!(fm.get("name").unwrap(), "iOS Liquid Glass Migrator");
        assert_eq!(fm.get("color").unwrap(), "blue");
        assert_eq!(fm.get("emoji").unwrap(), "x");
        assert_eq!(fm.get("vibe").unwrap(), "Transforms blurs");
        assert!(body.contains("# iOS Liquid Glass Migrator"));
        assert!(body.contains("Body content here."));
    }

    #[test]
    fn parse_frontmatter_missing() {
        let md = "# No Frontmatter\n\nJust body.\n";
        let (fm, body) = parse_frontmatter(md);
        assert!(fm.is_empty());
        assert_eq!(body, md);
    }

    #[test]
    fn parse_frontmatter_unclosed() {
        let md = "---\nname: Broken\nno closing delimiter\n";
        let (fm, _body) = parse_frontmatter(md);
        assert!(fm.is_empty());
    }

    #[test]
    fn persona_file_parsing() {
        let dir = std::env::temp_dir().join("gf_persona_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let md = "\
---
name: Test Agent
description: A test persona
color: red
emoji: T
vibe: Testing things
---

# Test Agent

You are Test Agent.
";
        fs::write(dir.join("test-agent.md"), md).unwrap();

        let loader = PersonaLoader::load_from_dir(&dir).unwrap();
        assert_eq!(loader.all().len(), 1);

        let p = &loader.all()[0];
        assert_eq!(p.name, "Test Agent");
        assert_eq!(p.description, "A test persona");
        assert_eq!(p.color.as_deref(), Some("red"));
        assert_eq!(p.emoji.as_deref(), Some("T"));
        assert_eq!(p.vibe.as_deref(), Some("Testing things"));
        assert!(p.system_prompt.contains("You are Test Agent."));
        assert_eq!(p.source_file, "test-agent.md");

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_by_name_exact_and_partial() {
        let dir = std::env::temp_dir().join("gf_persona_find_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("alpha.md"),
            "---\nname: iOS Deprecated API Fixer\ndescription: fixes\n---\n\nBody A\n",
        )
        .unwrap();
        fs::write(
            dir.join("beta.md"),
            "---\nname: iOS Liquid Glass Migrator\ndescription: migrates\n---\n\nBody B\n",
        )
        .unwrap();
        fs::write(
            dir.join("gamma.md"),
            "---\nname: iOS Migration Verifier\ndescription: verifies\n---\n\nBody C\n",
        )
        .unwrap();

        let loader = PersonaLoader::load_from_dir(&dir).unwrap();
        assert_eq!(loader.all().len(), 3);

        // Exact match (case-insensitive)
        let p = loader.find_by_name("ios deprecated api fixer").unwrap();
        assert_eq!(p.name, "iOS Deprecated API Fixer");

        // Partial match
        let p = loader.find_by_name("Liquid Glass").unwrap();
        assert_eq!(p.name, "iOS Liquid Glass Migrator");

        // No match
        assert!(loader.find_by_name("nonexistent").is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn finding_source_mapping() {
        assert_eq!(
            persona_for_finding_source("DeprecatedApi"),
            "iOS Deprecated API Fixer"
        );
        assert_eq!(
            persona_for_finding_source("LiquidGlass"),
            "iOS Liquid Glass Migrator"
        );
        assert_eq!(
            persona_for_finding_source("Unknown"),
            "iOS Deprecated API Fixer"
        );
    }

    #[test]
    fn verifier_persona() {
        assert_eq!(verifier_persona_name(), "iOS Migration Verifier");
    }

    #[test]
    fn recursive_loading() {
        let dir = std::env::temp_dir().join("gf_persona_recursive_test");
        let _ = fs::remove_dir_all(&dir);
        let sub = dir.join("subdir");
        fs::create_dir_all(&sub).unwrap();

        fs::write(
            dir.join("top.md"),
            "---\nname: Top Persona\ndescription: top\n---\n\nTop body\n",
        )
        .unwrap();
        fs::write(
            sub.join("nested.md"),
            "---\nname: Nested Persona\ndescription: nested\n---\n\nNested body\n",
        )
        .unwrap();

        let loader = PersonaLoader::load_from_dir(&dir).unwrap();
        assert_eq!(loader.all().len(), 2);

        let nested = loader.find_by_name("Nested Persona").unwrap();
        assert_eq!(nested.source_file, "subdir/nested.md");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn fallback_name_from_heading() {
        let dir = std::env::temp_dir().join("gf_persona_heading_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // No name in frontmatter — should fall back to heading
        fs::write(
            dir.join("agent.md"),
            "---\ndescription: some agent\n---\n\n# Heading Agent\n\nBody\n",
        )
        .unwrap();

        let loader = PersonaLoader::load_from_dir(&dir).unwrap();
        assert_eq!(loader.all()[0].name, "Heading Agent");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn fallback_name_from_filename() {
        let dir = std::env::temp_dir().join("gf_persona_filename_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // No frontmatter name, no heading
        fs::write(dir.join("cool-agent.md"), "Just plain text.\n").unwrap();

        let loader = PersonaLoader::load_from_dir(&dir).unwrap();
        assert_eq!(loader.all()[0].name, "cool agent");

        let _ = fs::remove_dir_all(&dir);
    }
}
