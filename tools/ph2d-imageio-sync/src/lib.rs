#![forbid(unsafe_code)]
//! Codegen logic for `ph2d-imageio-registry-init`: scan `crates/ph2d-imageio-*`
//! and render the `register_all_importers` body + `register_all_exporters`
//! body + the Cargo `[dependencies]` block.
//!
//! Single source of truth shared by the `ph2d-imageio-sync` **binary**
//! (which rewrites the files) and the registry-init **staleness gate**
//! (which re-renders and asserts the checked-in files match). Adding a
//! format = dropping a crate + running the binary; the gate fails CI if
//! someone forgot.
//!
//! This is the imageio-family twin of `ph2d-tool-sync` (ADR-0040 §2.3)
//! and `ph2d-node-sync` (ADR-0039): the same "drop a crate, zero central
//! edit" mechanism the node + tool fan-outs use. ADR-0054 §2.2.
//!
//! ### Shape difference vs the twins
//!
//! Importer/exporter `register` functions return `()` (like tool, not
//! node), so no `?`. There are **two** kinds of registration (importer
//! and exporter) — a crate may expose one or both via
//! `pub fn register_importer(reg: &mut ImporterRegistry)` and/or
//! `pub fn register_exporter(reg: &mut ExporterRegistry)`. A round-trip
//! crate (PNG, JPEG, ...) typically exposes both.
//!
//! Pure std, line-oriented.

use std::path::Path;

/// Markers delimiting the generated `register_all_importers` region in
/// `crates/ph2d-imageio-registry-init/src/lib.rs`.
pub const IMPORTERS_BEGIN: &str = "// <ph2d-imageio-sync:importers:begin>";
pub const IMPORTERS_END: &str = "// <ph2d-imageio-sync:importers:end>";
/// Markers delimiting the generated `register_all_exporters` region.
pub const EXPORTERS_BEGIN: &str = "// <ph2d-imageio-sync:exporters:begin>";
pub const EXPORTERS_END: &str = "// <ph2d-imageio-sync:exporters:end>";
/// Markers delimiting the generated region in `Cargo.toml` (TOML comments).
pub const TOML_BEGIN: &str = "# <ph2d-imageio-sync:begin>";
pub const TOML_END: &str = "# <ph2d-imageio-sync:end>";

/// Scan `crates_dir` for imageio format crates (`ph2d-imageio-*`),
/// excluding the contract crate (`ph2d-imageio`) and the registry-init
/// aggregator itself (`ph2d-imageio-registry-init`). Returns kebab crate
/// names, sorted alphabetically (deterministic + merge-stable).
pub fn scan_imageio_crates(crates_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(crates_dir)
        .expect("crates dir readable")
        // A real crate: a directory with a Cargo.toml. Rejects stray dirs
        // and scratch folders sharing the prefix.
        .filter_map(Result::ok)
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| {
            n.starts_with("ph2d-imageio-")
                && n != "ph2d-imageio-registry-init"
                // `ph2d-imageio` itself is the contract crate and does not
                // have the `ph2d-imageio-*` prefix shape (single hyphen),
                // but guard against future renames.
                && n != "ph2d-imageio"
        })
        .collect();
    names.sort();
    names
}

fn crate_ident(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}

/// Of `crates`, keep those whose `src/lib.rs` contains `needle`. Used to
/// split format crates by capability: `"pub fn register_importer("` →
/// importer-registry; `"pub fn register_exporter("` → exporter-registry.
/// A crate may match both (typical round-trip), one (decode-only stub,
/// e.g. W0.T5 PNG), or neither (placeholder, excluded from both lists).
/// Text scan — same shape as `ph2d-tool-sync::filter_exposing`.
pub fn filter_exposing(crates_dir: &Path, crates: &[String], needle: &str) -> Vec<String> {
    crates
        .iter()
        .filter(|c| {
            std::fs::read_to_string(crates_dir.join(c).join("src/lib.rs"))
                .map(|s| s.contains(needle))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

/// Generated `register_all_importers` body lines (4-space indented to
/// match the function body; rustfmt leaves these untouched between
/// markers). Returns `()`, so no `?`.
pub fn render_register_importer_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("    {}::register_importer(reg);", crate_ident(c)))
        .collect()
}

/// Generated `register_all_exporters` body lines.
pub fn render_register_exporter_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("    {}::register_exporter(reg);", crate_ident(c)))
        .collect()
}

/// Generated Cargo `[dependencies]` lines for the imageio format crates.
/// The aggregator depends on every format crate; selectively enabling a
/// subset (e.g. mobile builds without PSD) is a future feature-flag
/// concern (ADR-0054 §2.2 leaves the door open).
pub fn render_cargo_dep_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("{c} = {{ path = \"../{c}\" }}"))
        .collect()
}

/// Replace the lines strictly between the `begin` and `end` marker lines
/// with `body_lines`, preserving the marker lines themselves (and
/// everything else). Marker match is on the trimmed line, so indentation
/// is irrelevant. Copy of `ph2d-tool-sync::splice_lines`.
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
    fn idents_replace_hyphens() {
        assert_eq!(crate_ident("ph2d-imageio-png"), "ph2d_imageio_png");
    }

    #[test]
    fn register_importer_lines_have_no_question_mark() {
        let lines = render_register_importer_lines(&["ph2d-imageio-png".to_string()]);
        assert_eq!(lines, vec!["    ph2d_imageio_png::register_importer(reg);"]);
    }

    #[test]
    fn register_exporter_lines_use_exporter_fn() {
        let lines = render_register_exporter_lines(&["ph2d-imageio-png".to_string()]);
        assert_eq!(lines, vec!["    ph2d_imageio_png::register_exporter(reg);"]);
    }

    #[test]
    fn cargo_dep_lines_point_at_path() {
        let lines = render_cargo_dep_lines(&["ph2d-imageio-jpeg".to_string()]);
        assert_eq!(
            lines,
            vec!["ph2d-imageio-jpeg = { path = \"../ph2d-imageio-jpeg\" }"]
        );
    }

    #[test]
    fn splice_replaces_between_markers_only() {
        let content = "head\n    // <ph2d-imageio-sync:importers:begin>\n    OLD;\n    // <ph2d-imageio-sync:importers:end>\ntail\n";
        let out = splice_lines(
            content,
            IMPORTERS_BEGIN,
            IMPORTERS_END,
            &["    NEW;".to_string()],
        );
        assert_eq!(
            out,
            "head\n    // <ph2d-imageio-sync:importers:begin>\n    NEW;\n    // <ph2d-imageio-sync:importers:end>\ntail\n"
        );
    }

    #[test]
    fn splice_is_idempotent() {
        let content = "head\n# <ph2d-imageio-sync:begin>\na = 1\n# <ph2d-imageio-sync:end>\ntail\n";
        let once = splice_lines(content, TOML_BEGIN, TOML_END, &["b = 2".to_string()]);
        let twice = splice_lines(&once, TOML_BEGIN, TOML_END, &["b = 2".to_string()]);
        assert_eq!(once, twice);
    }

    #[test]
    fn empty_body_clears_region() {
        // No imageio crates yet means a freshly seeded registry-init has
        // empty bodies. The splice must accept `body_lines: &[]` and
        // produce a region with just the two markers adjacent.
        let content = "head\n    // <ph2d-imageio-sync:importers:begin>\n    OLD;\n    // <ph2d-imageio-sync:importers:end>\ntail\n";
        let out = splice_lines(content, IMPORTERS_BEGIN, IMPORTERS_END, &[]);
        assert_eq!(
            out,
            "head\n    // <ph2d-imageio-sync:importers:begin>\n    // <ph2d-imageio-sync:importers:end>\ntail\n"
        );
    }
}
