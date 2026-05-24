#![forbid(unsafe_code)]
//! Codegen logic for `ph2d-panel-registry-init`: scan `crates/ph2d-panel-*`
//! and render the `build_typed_registry` body + the Cargo `[dependencies]`
//! + `[features]` blocks.
//!
//! Wave 10 / Etapa 4.1 (per docs/plans/2026-05-wave-10-perfection.md §IV).
//! Mirror of `ph2d-tool-sync` with two shape differences:
//!
//! 1. Each panel has its own Cargo feature `panel-<slug>` — panels are
//!    opt-out at build time (`--no-default-features`). Tools aren't.
//! 2. Each panel exports a typed `<CamelSlug>Panel` struct that gets
//!    pushed via `ErasedPanel::new::<...>()` — not via a free `register()`
//!    fn. We parse the struct name from the panel crate's `src/lib.rs`
//!    (looking for `pub struct *Panel`).
//!
//! Pure std, line-oriented (no `syn` dep — keeps the tool tiny + matches
//! the same idiom as ph2d-tool-sync / ph2d-node-sync).

use std::path::Path;

/// Markers delimiting the generated `build_typed_registry` body in
/// `crates/ph2d-panel-registry-init/src/lib.rs`.
pub const RS_BEGIN: &str = "// <ph2d-panel-sync:begin>";
pub const RS_END: &str = "// <ph2d-panel-sync:end>";
/// Markers delimiting the generated `[dependencies]` region in Cargo.toml.
pub const TOML_DEPS_BEGIN: &str = "# <ph2d-panel-sync:deps:begin>";
pub const TOML_DEPS_END: &str = "# <ph2d-panel-sync:deps:end>";
/// Markers delimiting the generated `[features]` region in Cargo.toml.
pub const TOML_FEATURES_BEGIN: &str = "# <ph2d-panel-sync:features:begin>";
pub const TOML_FEATURES_END: &str = "# <ph2d-panel-sync:features:end>";

/// Scan `crates_dir` for panel crates (`ph2d-panel-*`), excluding the
/// `ph2d-panel-registry-init` aggregator. Returns kebab crate names,
/// sorted (deterministic merge-stable order).
pub fn scan_panel_crates(crates_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(crates_dir)
        .expect("crates dir readable")
        .filter_map(Result::ok)
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.starts_with("ph2d-panel-") && n != "ph2d-panel-registry-init")
        .collect();
    names.sort();
    names
}

/// Snake-case identifier for the crate (used in `use`/path expressions).
fn crate_ident(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}

/// Public re-export of `crate_ident` for the staleness gate test in
/// `ph2d-panel-registry-init` (the gate uses it to build the expected
/// (ident, struct) pairs for semantic comparison).
pub fn crate_ident_for_test(crate_name: &str) -> String {
    crate_ident(crate_name)
}

/// Cargo feature slug for the panel — strip the `ph2d-panel-` prefix.
/// E.g. `ph2d-panel-bgremoval` → `panel-bgremoval`.
fn feature_slug(crate_name: &str) -> String {
    let stripped = crate_name
        .strip_prefix("ph2d-panel-")
        .expect("scan_panel_crates filters by prefix");
    format!("panel-{stripped}")
}

/// Parse the exported `*Panel` struct name from the crate's `src/lib.rs`.
/// Looks for `pub struct <Name>Panel` (the canonical convention every
/// panel crate follows). Returns `None` if the crate has no exported
/// `*Panel` struct in its lib root (unlikely — the gate below catches it).
pub fn panel_struct_name(crates_dir: &Path, crate_name: &str) -> Option<String> {
    let lib = crates_dir.join(crate_name).join("src/lib.rs");
    let content = std::fs::read_to_string(&lib).ok()?;
    for line in content.lines() {
        let trimmed = line.trim_start();
        // Skip doc-comments + regular comments.
        if trimmed.starts_with("//") {
            continue;
        }
        // Match `pub struct <Name>Panel`. The name ends at the first
        // whitespace, `{`, `<`, `(` or `;`.
        if let Some(rest) = trimmed.strip_prefix("pub struct ") {
            // First token until non-ident chars.
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            let name = &rest[..end];
            if name.ends_with("Panel") {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// The generated `build_typed_registry` body lines — one
/// `#[cfg(feature = ...)] reg.push(ErasedPanel::new::<...>())` block per
/// panel. 4-space indented to match the function body.
///
/// Emits 1-line form for each panel. `cargo fmt` (run by the sync's main
/// after the splice — Wave 10 / Etapa 4 audit [C-1] fix) will break long
/// type paths into the multi-line form rustfmt prefers. The staleness
/// gate compares SEMANTICALLY via [`extract_registered_panels`] (which
/// is whitespace/formatting-tolerant), NOT byte-equal — so rustfmt's
/// reformatting doesn't trip the gate.
pub fn render_register_lines(crates_dir: &Path, crates: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for crate_name in crates {
        let feature = feature_slug(crate_name);
        let struct_name = panel_struct_name(crates_dir, crate_name).unwrap_or_else(|| {
            panic!(
                "panel crate `{crate_name}` has no exported `*Panel` struct \
                     in src/lib.rs — the panel-sync convention requires `pub \
                     struct <CamelSlug>Panel`"
            )
        });
        let ident = crate_ident(crate_name);
        out.push(format!("    #[cfg(feature = \"{feature}\")]"));
        out.push(format!(
            "    reg.push(ErasedPanel::new::<{ident}::{struct_name}>());"
        ));
    }
    out
}

/// Extract the (crate_ident, struct_name) pairs registered between the
/// `:begin` / `:end` markers in a `lib.rs`. Whitespace-tolerant —
/// works whether rustfmt has reformatted the block 1-line or multi-line.
/// Used by the staleness gate to compare semantic content vs raw text.
pub fn extract_registered_panels(content: &str) -> Vec<(String, String)> {
    let start = match content.find(RS_BEGIN) {
        Some(s) => s,
        None => return Vec::new(),
    };
    let end = match content[start..].find(RS_END) {
        Some(o) => start + o,
        None => return Vec::new(),
    };
    let block = &content[start..end];
    // Strip whitespace + collapse so multi-line `ErasedPanel::new::<\n
    //     foo::Bar,\n>()` becomes a single sequence we can regex.
    let collapsed: String = block.split_whitespace().collect::<Vec<_>>().join(" ");
    // Find every `ErasedPanel::new::< <ident>::<Name> ,? >`.
    let mut out: Vec<(String, String)> = Vec::new();
    let needle = "ErasedPanel::new::<";
    let mut search = collapsed.as_str();
    while let Some(pos) = search.find(needle) {
        let after = &search[pos + needle.len()..];
        // Read up to `>`.
        let close = match after.find('>') {
            Some(c) => c,
            None => break,
        };
        let inner = after[..close].trim().trim_end_matches(',').trim();
        // `inner` is `<ident>::<Name>` (e.g. "ph2d_panel_bgremoval::BgRemovalPanel").
        if let Some((ident, name)) = inner.rsplit_once("::") {
            out.push((ident.to_string(), name.to_string()));
        }
        search = &after[close..];
    }
    out
}

/// The generated Cargo `[dependencies]` lines for the panel crates.
/// Each panel is `optional = true` so the corresponding `[features]`
/// entry below toggles it.
pub fn render_cargo_dep_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("{c} = {{ path = \"../{c}\", optional = true }}"))
        .collect()
}

/// The generated Cargo `[features]` lines — one feature per panel.
/// `panel-X = ["dep:ph2d-panel-X"]` toggles the optional dependency.
pub fn render_cargo_feature_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| {
            let feature = feature_slug(c);
            format!("{feature} = [\"dep:{c}\"]")
        })
        .collect()
}

/// Replace lines strictly between the `begin` and `end` markers with
/// `body_lines`, preserving the marker lines themselves. Marker match
/// is on the trimmed line (indentation-tolerant).
pub fn splice_lines(content: &str, begin: &str, end: &str, body_lines: &[String]) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let count = |marker: &str| lines.iter().filter(|l| l.trim() == marker.trim()).count();
    assert_eq!(
        count(begin),
        1,
        "begin marker must appear exactly once: {begin}"
    );
    assert_eq!(count(end), 1, "end marker must appear exactly once: {end}");
    let bi = lines
        .iter()
        .position(|l| l.trim() == begin.trim())
        .expect("begin marker present");
    let ei = lines
        .iter()
        .position(|l| l.trim() == end.trim())
        .expect("end marker present");
    assert!(bi < ei, "begin marker must precede end marker");

    let mut out: Vec<String> = Vec::new();
    out.extend(lines[..=bi].iter().map(|s| s.to_string()));
    out.extend(body_lines.iter().cloned());
    out.extend(lines[ei..].iter().map(|s| s.to_string()));
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_slug_strips_prefix() {
        assert_eq!(feature_slug("ph2d-panel-bgremoval"), "panel-bgremoval");
        assert_eq!(
            feature_slug("ph2d-panel-color-equalization"),
            "panel-color-equalization"
        );
    }

    #[test]
    fn crate_ident_replaces_hyphens() {
        assert_eq!(
            crate_ident("ph2d-panel-color-equalization"),
            "ph2d_panel_color_equalization"
        );
    }

    #[test]
    fn splice_replaces_between_markers_only() {
        let content =
            "head\n    // <ph2d-panel-sync:begin>\n    OLD;\n    // <ph2d-panel-sync:end>\ntail\n";
        let out = splice_lines(content, RS_BEGIN, RS_END, &["    NEW;".to_string()]);
        assert_eq!(
            out,
            "head\n    // <ph2d-panel-sync:begin>\n    NEW;\n    // <ph2d-panel-sync:end>\ntail\n"
        );
    }

    #[test]
    fn render_cargo_dep_lines_marks_optional() {
        let lines = render_cargo_dep_lines(&["ph2d-panel-padding".to_string()]);
        assert_eq!(
            lines,
            vec!["ph2d-panel-padding = { path = \"../ph2d-panel-padding\", optional = true }"]
        );
    }

    #[test]
    fn render_cargo_feature_lines_dep_link() {
        let lines = render_cargo_feature_lines(&["ph2d-panel-padding".to_string()]);
        assert_eq!(lines, vec!["panel-padding = [\"dep:ph2d-panel-padding\"]"]);
    }
}
