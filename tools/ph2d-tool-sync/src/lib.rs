#![forbid(unsafe_code)]
//! Codegen logic for `ph2d-tool-registry-init`: scan `crates/ph2d-tool-*` and
//! render the `register_all` body + the Cargo `[dependencies]` block.
//!
//! Single source of truth shared by the `ph2d-tool-sync` **binary** (which
//! rewrites the files) and the registry-init **staleness gate** (which
//! re-renders and asserts the checked-in files match). Adding a tool is then
//! dropping a crate + running the binary; the gate fails CI if someone forgot.
//!
//! This is the tool-family twin of `ph2d-node-sync` (ADR-0040 §2.3): the same
//! "drop a crate, zero central edit" mechanism the node fan-out uses. The one
//! shape difference is the registration call — a tool manifest `register`
//! returns `()`, a node `register` returns `Result`, so the rendered line has
//! no `?`.
//!
//! Pure std, line-oriented.

use std::path::Path;

/// Markers delimiting the generated `register_all` (manifests) region.
pub const RS_BEGIN: &str = "// <ph2d-tool-sync:begin>";
pub const RS_END: &str = "// <ph2d-tool-sync:end>";
/// Markers delimiting the generated `register_all_tools` (behavior /
/// `Box<dyn Tool>`) region in `src/lib.rs`.
pub const TOOLS_BEGIN: &str = "// <ph2d-tool-sync:tools:begin>";
pub const TOOLS_END: &str = "// <ph2d-tool-sync:tools:end>";
/// Markers delimiting the generated region in `Cargo.toml` (TOML comments).
pub const TOML_BEGIN: &str = "# <ph2d-tool-sync:begin>";
pub const TOML_END: &str = "# <ph2d-tool-sync:end>";

/// Scan `crates_dir` for tool crates (`ph2d-tool-*`), excluding the registry
/// contract crate and the registry-init aggregator themselves. Returns kebab
/// crate names, sorted (the sort is the deterministic, merge-stable order the
/// `architecture_register_all_alphabetical` gate also enforces).
pub fn scan_tool_crates(crates_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(crates_dir)
        .expect("crates dir readable")
        // A real crate: a directory with a Cargo.toml. Rejects stray dirs,
        // scratch folders, and bare symlinks that share the `ph2d-tool-` prefix
        // but would generate a `register` call to a crate that does not exist.
        .filter_map(Result::ok)
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| {
            n.starts_with("ph2d-tool-")
                && n != "ph2d-tool-registry"
                && n != "ph2d-tool-registry-init"
                // Wave 10 / Etapa 1.B audit fix [3.1]: ph2d-tool-runtime
                // is shell-facing infra, not a tool. It bates the prefix
                // `ph2d-tool-*` (kept for namespace consistency with
                // ph2d-tool-registry / -registry-init) but exposes no
                // `pub fn register` nor `pub fn make` — without this
                // exclusion, `cargo_deps_in_sync_with_folder` would add
                // an inert `ph2d-tool-runtime` dep to
                // `ph2d-tool-registry-init/Cargo.toml` that nothing
                // actually uses.
                && n != "ph2d-tool-runtime"
        })
        .collect();
    names.sort();
    names
}

fn crate_ident(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}

/// Of `crates`, keep those whose `src/lib.rs` contains `needle`. Used to
/// split tool crates by capability: `"pub fn register"` → data-registry
/// (manifest/chrome) tools; `"pub fn make"` → behavior-registry
/// (modal/palette) tools. A crate may match both (e.g. bgremoval), one
/// (make_square = register only; brush = make only), driving which
/// generated list it lands in. Text scan, like the alphabetical gate.
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

/// The generated `register_all` body lines (4-space indented to match the
/// function body; rustfmt leaves these untouched). Tool `register` returns
/// `()`, so no `?` — unlike the node twin.
pub fn render_register_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("    {}::register(reg);", crate_ident(c)))
        .collect()
}

/// The generated `register_all_tools` body lines — one boxed `make()`
/// per behavior (modal/palette) tool crate, registered into the
/// `ToolRegistry`. Tool register returns `()`.
pub fn render_register_tools_lines(crates: &[String]) -> Vec<String> {
    crates
        .iter()
        .map(|c| format!("    reg.register({}::make());", crate_ident(c)))
        .collect()
}

/// The generated Cargo `[dependencies]` lines for the tool crates.
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
        assert_eq!(crate_ident("ph2d-tool-bgremoval"), "ph2d_tool_bgremoval");
    }

    #[test]
    fn register_lines_have_no_question_mark() {
        // Tool register returns (), unlike node register's Result.
        let lines = render_register_lines(&["ph2d-tool-padding".to_string()]);
        assert_eq!(lines, vec!["    ph2d_tool_padding::register(reg);"]);
    }

    #[test]
    fn splice_replaces_between_markers_only() {
        let content =
            "head\n    // <ph2d-tool-sync:begin>\n    OLD;\n    // <ph2d-tool-sync:end>\ntail\n";
        let out = splice_lines(content, RS_BEGIN, RS_END, &["    NEW;".to_string()]);
        assert_eq!(
            out,
            "head\n    // <ph2d-tool-sync:begin>\n    NEW;\n    // <ph2d-tool-sync:end>\ntail\n"
        );
    }

    #[test]
    fn splice_is_idempotent() {
        let content = "head\n# <ph2d-tool-sync:begin>\na = 1\n# <ph2d-tool-sync:end>\ntail\n";
        let once = splice_lines(content, TOML_BEGIN, TOML_END, &["b = 2".to_string()]);
        let twice = splice_lines(&once, TOML_BEGIN, TOML_END, &["b = 2".to_string()]);
        assert_eq!(once, twice);
    }
}
