#![forbid(unsafe_code)]
//! Codegen logic for `ph2d-node-registry-init`: scan `crates/ph2d-node-*` and
//! render the `register_all_nodes` body + the Cargo `[dependencies]` block.
//!
//! Single source of truth shared by the `ph2d-node-sync` **binary** (which
//! rewrites the files) and the registry-init **staleness gate** (which
//! re-renders and asserts the checked-in files match). Adding a node is then
//! dropping a crate + running the binary; the gate fails CI if someone forgot.
//!
//! Pure std, line-oriented (matching the diffable-format philosophy of
//! `ph2d-nodegraph::format`).

use std::path::Path;

/// Markers delimiting the generated region in `src/lib.rs` (Rust comments).
pub const RS_BEGIN: &str = "// <ph2d-node-sync:begin>";
pub const RS_END: &str = "// <ph2d-node-sync:end>";
/// Markers delimiting the generated region in `Cargo.toml` (TOML comments).
pub const TOML_BEGIN: &str = "# <ph2d-node-sync:begin>";
pub const TOML_END: &str = "# <ph2d-node-sync:end>";

/// Scan `crates_dir` for node crates (`ph2d-node-*`), excluding the registry
/// and registry-init themselves. Returns kebab crate names, sorted (the sort
/// is the deterministic, merge-stable order the registry relies on).
pub fn scan_node_crates(crates_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(crates_dir)
        .expect("crates dir readable")
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| {
            n.starts_with("ph2d-node-")
                && n != "ph2d-node-registry"
                && n != "ph2d-node-registry-init"
        })
        .collect();
    names.sort();
    names
}

fn crate_ident(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}

/// The generated `register_all_nodes` body lines (4-space indented to match the
/// function body; rustfmt leaves these untouched).
pub fn render_register_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("    {}::register(reg)?;", crate_ident(c)))
        .collect()
}

/// The generated Cargo `[dependencies]` lines for the node crates.
pub fn render_cargo_dep_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("{c} = {{ path = \"../{c}\" }}"))
        .collect()
}

/// Replace the lines strictly between the `begin` and `end` marker lines with
/// `body_lines`, preserving the marker lines themselves (and everything else).
/// Marker match is on the trimmed line, so indentation is irrelevant.
pub fn splice_lines(content: &str, begin: &str, end: &str, body_lines: &[String]) -> String {
    let lines: Vec<&str> = content.lines().collect();
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
        assert_eq!(
            crate_ident("ph2d-node-motion-clone"),
            "ph2d_node_motion_clone"
        );
    }

    #[test]
    fn splice_replaces_between_markers_only() {
        let content =
            "head\n    // <ph2d-node-sync:begin>\n    OLD;\n    // <ph2d-node-sync:end>\ntail\n";
        let out = splice_lines(content, RS_BEGIN, RS_END, &["    NEW;".to_string()]);
        assert_eq!(
            out,
            "head\n    // <ph2d-node-sync:begin>\n    NEW;\n    // <ph2d-node-sync:end>\ntail\n"
        );
    }

    #[test]
    fn splice_is_idempotent() {
        let content = "head\n# <ph2d-node-sync:begin>\na = 1\n# <ph2d-node-sync:end>\ntail\n";
        let once = splice_lines(content, TOML_BEGIN, TOML_END, &["b = 2".to_string()]);
        let twice = splice_lines(&once, TOML_BEGIN, TOML_END, &["b = 2".to_string()]);
        assert_eq!(once, twice);
    }
}
