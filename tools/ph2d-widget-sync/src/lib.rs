#![forbid(unsafe_code)]
//! Codegen logic for `crates/ph2d-editor-core/src/widget/mod.rs`:
//! scan widget files and render the `mod foo;` declaration block.
//!
//! Wave 10 / Etapa 4.3 — third in the panel/chrome/widget codegen
//! trio. Mirror of `ph2d-chrome-sync` (same scope: mods only;
//! re-exports stay hand-written because per-widget export shapes vary).
//!
//! ## Visibility convention
//!
//! Most widgets are private (`mod foo;`) — only the re-exported
//! items below are public. Two exceptions are public modules
//! (`pub mod panel_chrome;` + `pub mod showcase;`) — they expose
//! sub-modules / sub-paths the showcase callers reach directly.
//!
//! The render below ASKS each widget file: does it contain
//! `pub mod` self-declaration or a `// WIDGET_VISIBILITY: pub` marker?
//! If yes, render `pub mod`. Else render `mod`. (Today the convention
//! is: the 2 public ones are listed by name as PUB_MODULE_OVERRIDE.
//! When more public widgets land, switch to a per-file marker.)

use std::path::Path;

/// Markers delimiting the generated `mod foo;` declaration block.
pub const MOD_BEGIN: &str = "// <ph2d-widget-sync:begin>";
pub const MOD_END: &str = "// <ph2d-widget-sync:end>";

/// Widgets that should be declared `pub mod` instead of `mod`. The
/// canonical list (today: panel_chrome + showcase). Adding a new public
/// widget = append here (Coord-A decision — most widgets are private).
const PUB_MODULE_OVERRIDE: &[&str] = &["panel_chrome", "showcase"];

/// Scan widget dir for `*.rs` files (excluding `mod.rs`). Returns
/// stems (snake_case), sorted alphabetically.
pub fn scan_widget_files(widget_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(widget_dir)
        .expect("widget dir readable")
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .filter(|n| n != "mod")
        .collect();
    names.sort();
    names
}

/// Scan widget dir for sub-directories (each represents a multi-file
/// widget — currently `blender_color_picker/` + `showcase/`).
/// Returns names sorted.
pub fn scan_widget_dirs(widget_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(widget_dir)
        .expect("widget dir readable")
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            e.path()
                .file_name()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .collect();
    names.sort();
    names
}

/// Render the merged `mod` declarations (files + dirs, sorted together).
/// `pub mod` vs `mod` is decided by [`PUB_MODULE_OVERRIDE`].
pub fn render_mod_lines(files: &[String], dirs: &[String]) -> Vec<String> {
    let mut all: Vec<String> = files.iter().chain(dirs.iter()).cloned().collect();
    all.sort();
    all.dedup();
    all.into_iter()
        .map(|n| {
            if PUB_MODULE_OVERRIDE.contains(&n.as_str()) {
                format!("pub mod {n};")
            } else {
                format!("mod {n};")
            }
        })
        .collect()
}

/// Splice helper (same as the other syncs).
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
    fn pub_modules_get_pub_prefix() {
        let lines = render_mod_lines(
            &["button".to_string(), "showcase".to_string()],
            &["panel_chrome".to_string()],
        );
        assert!(lines.contains(&"mod button;".to_string()));
        assert!(lines.contains(&"pub mod showcase;".to_string()));
        assert!(lines.contains(&"pub mod panel_chrome;".to_string()));
    }

    #[test]
    fn render_dedups_when_dir_and_file_overlap() {
        // Defensive: if a widget exists as both file and dir (shouldn't
        // happen, but be safe), dedup keeps just one declaration.
        let lines = render_mod_lines(&["button".to_string()], &["button".to_string()]);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn splice_replaces_between_markers() {
        let content =
            "head\n// <ph2d-widget-sync:begin>\nmod old;\n// <ph2d-widget-sync:end>\ntail\n";
        let out = splice_lines(content, MOD_BEGIN, MOD_END, &["mod new;".to_string()]);
        assert_eq!(
            out,
            "head\n// <ph2d-widget-sync:begin>\nmod new;\n// <ph2d-widget-sync:end>\ntail\n"
        );
    }
}
