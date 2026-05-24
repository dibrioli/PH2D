//! `ph2d-memory-gc` — validate that paths in the LLM's persistent
//! memory file still exist in the codebase.
//!
//! Wave 10 / Etapa 6.4. The Anthropic agent memory system stores
//! `[Title](path.md)` index entries in MEMORY.md plus inline
//! `[file](crates/...)` references in memory files. When the repo
//! refactors (file rename, crate split, removed module), those links
//! rot — future agents act on a memory entry that points at thin air.
//!
//! This tool walks MEMORY.md + every `.md` next to it, extracts
//! markdown links + raw paths, and reports the ones that no longer
//! resolve relative to the workspace root. Output goes to stdout in a
//! human-readable + grep-friendly form.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// A single broken reference: where it appeared (`source`), the raw
/// path text (`target`), and the resolved absolute path that failed
/// to exist.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BrokenRef {
    pub source: PathBuf,
    pub line: usize,
    pub target_text: String,
    pub resolved: PathBuf,
}

/// Scan a memory directory and return every broken path reference.
/// `workspace_root` is the base for resolving paths that look like
/// repo-rooted (`crates/...`, `docs/...`, etc.); sibling-file
/// references in `MEMORY.md` resolve relative to the source file's
/// directory.
pub fn scan_memory_dir(memory_dir: &Path, workspace_root: &Path) -> Vec<BrokenRef> {
    let mut out: Vec<BrokenRef> = Vec::new();
    let files = collect_md(memory_dir);
    for path in &files {
        let source_dir = path.parent().unwrap_or(memory_dir).to_path_buf();
        let Ok(src) = fs::read_to_string(path) else {
            continue;
        };
        for (lineno, line) in src.lines().enumerate() {
            for target in extract_path_targets(line) {
                let resolved = resolve(workspace_root, &source_dir, &target);
                if !resolved.exists() {
                    out.push(BrokenRef {
                        source: path.clone(),
                        line: lineno + 1,
                        target_text: target,
                        resolved,
                    });
                }
            }
        }
    }
    out.sort();
    out
}

/// Extract every markdown link target `(...)` AND every raw
/// `crates/...` / `docs/...` / `shells/...` / `tools/...` /
/// `tests/...` / `scripts/...` token from a single line. Skips
/// `http://` / `https://` / `file:///` URLs and `#anchor`-only refs.
pub fn extract_path_targets(line: &str) -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();

    // Markdown links: `[text](target)`.
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b']' && i + 1 < bytes.len() && bytes[i + 1] == b'(' {
            let target_start = i + 2;
            let mut depth = 1i32;
            let mut j = target_start;
            while j < bytes.len() {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            if j < bytes.len() {
                let raw = &line[target_start..j];
                if let Some(target) = normalize_target(raw) {
                    out.insert(target);
                }
            }
            i = j + 1;
            continue;
        }
        i += 1;
    }

    // Raw repo-root paths in prose: any bare `crates/...` /
    // `docs/...` / `shells/...` / `tools/...` token bounded by
    // whitespace, punctuation or end-of-string. Identifies the most
    // common pattern in memory files (where authors mention paths
    // without markdown linking them).
    for token in tokens_of(line) {
        for prefix in [
            "crates/", "docs/", "shells/", "tools/", "tests/", "scripts/", ".github/",
        ] {
            if token.starts_with(prefix)
                && !token.contains("://")
                && let Some(target) = normalize_target(token)
            {
                out.insert(target);
            }
        }
    }

    out.into_iter().collect()
}

fn normalize_target(raw: &str) -> Option<String> {
    let raw = raw.trim();
    // Strip optional title: `path "title"`.
    let raw = raw.split_once(' ').map(|(p, _)| p).unwrap_or(raw);
    // Strip `#anchor` and `?query`.
    let raw = raw.split('#').next().unwrap_or(raw);
    let raw = raw.split('?').next().unwrap_or(raw);
    if raw.is_empty() {
        return None;
    }
    if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("mailto:") {
        return None;
    }
    // Strip wrapping `<>` / `"`.
    let raw = raw.trim_start_matches('<').trim_end_matches('>');
    let raw = raw.trim_matches('"');
    Some(raw.to_string())
}

/// Split `line` on whitespace and markdown-friendly boundaries.
/// Strips trailing punctuation (`.`, `,`, `:`, `;`, `)`) from each
/// token so `shells/desktop/src/integration.rs.` (end-of-sentence
/// period) and `shells/desktop:` (colon-prefix-list intro) parse as
/// `shells/desktop/src/integration.rs` and `shells/desktop`.
fn tokens_of(line: &str) -> Vec<&str> {
    line.split(|c: char| c.is_whitespace() || ",;()[]<>".contains(c))
        .map(|s| s.trim_end_matches(|c: char| ".,;:)".contains(c)))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Resolve `target` against the right base directory.
///
/// - `file://...` → parsed as URL, used as absolute path.
/// - `./` / `../` prefix → relative to `source_dir`.
/// - Starts with one of the canonical repo-root segments
///   (`crates/`, `docs/`, `shells/`, `tools/`, `tests/`, `scripts/`,
///   `.github/`) → resolved under `workspace_root`.
/// - Otherwise → sibling of `source_dir` (e.g. a plain `user_role.md`
///   in MEMORY.md is a sibling, not a repo-root file).
fn resolve(workspace_root: &Path, source_dir: &Path, target: &str) -> PathBuf {
    if let Some(stripped) = target.strip_prefix("file://") {
        return PathBuf::from(stripped);
    }
    if target.starts_with('.') {
        return source_dir.join(target);
    }
    const REPO_PREFIXES: &[&str] = &[
        "crates/", "docs/", "shells/", "tools/", "tests/", "scripts/", ".github/",
    ];
    if REPO_PREFIXES.iter().any(|p| target.starts_with(p)) {
        return workspace_root.join(target);
    }
    source_dir.join(target)
}

fn collect_md(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_markdown_link() {
        let line = "- [User: Enio](user_role.md) — owner";
        assert_eq!(extract_path_targets(line), vec!["user_role.md".to_string()]);
    }

    #[test]
    fn extracts_repo_root_prose_path() {
        let line = "see crates/ph2d-color/src/lib.rs for the canonical types";
        let extracted = extract_path_targets(line);
        assert!(extracted.contains(&"crates/ph2d-color/src/lib.rs".to_string()));
    }

    #[test]
    fn skips_http_urls() {
        let line = "[link](https://example.com/foo.md)";
        assert!(extract_path_targets(line).is_empty());
    }

    #[test]
    fn strips_anchor() {
        let line = "see [§3](docs/DIRETRIZ.md#section-3)";
        assert_eq!(
            extract_path_targets(line),
            vec!["docs/DIRETRIZ.md".to_string()]
        );
    }

    #[test]
    fn strips_file_url_prefix() {
        let workspace = PathBuf::from("/tmp/wks");
        let memory = workspace.join("memory");
        let resolved = resolve(&workspace, &memory, "file:///tmp/wks/docs/X.md");
        assert_eq!(resolved, PathBuf::from("/tmp/wks/docs/X.md"));
    }

    #[test]
    fn repo_root_prefix_resolves_under_workspace() {
        let workspace = PathBuf::from("/tmp/wks");
        let memory = workspace.join(".claude/memory");
        let resolved = resolve(&workspace, &memory, "crates/foo/src/lib.rs");
        assert_eq!(resolved, PathBuf::from("/tmp/wks/crates/foo/src/lib.rs"));
    }

    #[test]
    fn bare_filename_resolves_as_sibling() {
        let workspace = PathBuf::from("/tmp/wks");
        let memory = workspace.join(".claude/memory");
        let resolved = resolve(&workspace, &memory, "user_role.md");
        assert_eq!(
            resolved,
            PathBuf::from("/tmp/wks/.claude/memory/user_role.md")
        );
    }
}
