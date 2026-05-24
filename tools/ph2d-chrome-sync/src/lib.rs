#![forbid(unsafe_code)]
//! Codegen logic for `crates/ph2d-editor-core/src/screens/hero/chrome/mod.rs`:
//! scan sibling `*.rs` handler files and render the `mod foo;`
//! declaration block.
//!
//! Wave 10 / Etapa 4.2 — third in the panel/chrome/widget codegen
//! trio (mirrors `ph2d-panel-sync` and `ph2d-widget-sync`).
//!
//! ## Scope
//!
//! Only the `mod foo;` declaration block is regenerated. The
//! `dispatch_all` chain stays hand-written because **z-order is
//! load-bearing** (per `chrome/mod.rs` doc-comment: "earlier modules
//! in the `||` chain win when ids overlap"). Codegening it would
//! force alphabetical order — silently breaking dispatch.
//!
//! Adding a chrome handler:
//!   1. Drop `chrome/<slug>.rs` with `pub fn apply(hero, event) -> bool`.
//!   2. `cargo run -p ph2d-chrome-sync` regenerates the `mod` block.
//!   3. Edit `dispatch_all` BY HAND to insert the new handler at the
//!      right z-order position (the function the gate enforces was
//!      `apply_chain_includes_every_mod`, vide staleness gate).

use std::path::Path;

/// Markers delimiting the generated `mod foo;` declaration block.
pub const MOD_BEGIN: &str = "// <ph2d-chrome-sync:begin>";
pub const MOD_END: &str = "// <ph2d-chrome-sync:end>";

/// Scan the chrome dir for `*.rs` handler files (excluding `mod.rs`).
/// Returns kebab-snake names (just the file stem; chrome handlers use
/// snake_case file names by convention). Sorted alphabetically for
/// stable diffs.
pub fn scan_chrome_handlers(chrome_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(chrome_dir)
        .expect("chrome dir readable")
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

/// Render `mod <name>;` declarations — one per handler file. No
/// indentation (these go at module-level).
pub fn render_mod_lines(handlers: &[String]) -> Vec<String> {
    handlers.iter().map(|n| format!("mod {n};")).collect()
}

/// Splice helper (clone of the one in ph2d-tool-sync / ph2d-panel-sync).
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

/// Audit helper for the staleness gate: parse the `dispatch_all`
/// function body and return the set of handler names it references
/// (e.g. `["theme", "radius", "view_toggles", ...]`). The gate uses
/// this to verify every `mod`'d handler is also IN the dispatch_all
/// chain — if you add a handler file but forget to wire it into
/// dispatch_all, the gate catches it.
pub fn dispatch_all_referenced_handlers(content: &str) -> Vec<String> {
    let start = match content.find("pub fn dispatch_all") {
        Some(s) => s,
        None => return Vec::new(),
    };
    let body_start = match content[start..].find('{') {
        Some(o) => start + o,
        None => return Vec::new(),
    };
    let body_end = match content[body_start..].find("\n}") {
        Some(o) => body_start + o,
        None => return Vec::new(),
    };
    let body = &content[body_start..body_end];
    // Each handler appears as `<name>::apply(...)`. Extract every
    // identifier immediately followed by `::apply`.
    let mut out: Vec<String> = Vec::new();
    for token in body.split_whitespace() {
        if let Some(ident) = token.strip_suffix("::apply(hero,") {
            out.push(ident.to_string());
        } else if let Some(rest) = token.strip_suffix("::apply") {
            // Defensive: any token ending in ::apply (in case the
            // formatting changes).
            out.push(rest.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_mod_lines_emit_decls() {
        let lines = render_mod_lines(&["theme".to_string(), "radius".to_string()]);
        assert_eq!(lines, vec!["mod theme;", "mod radius;"]);
    }

    #[test]
    fn splice_replaces_between_markers() {
        let content =
            "head\n// <ph2d-chrome-sync:begin>\nmod old;\n// <ph2d-chrome-sync:end>\ntail\n";
        let out = splice_lines(content, MOD_BEGIN, MOD_END, &["mod new;".to_string()]);
        assert_eq!(
            out,
            "head\n// <ph2d-chrome-sync:begin>\nmod new;\n// <ph2d-chrome-sync:end>\ntail\n"
        );
    }

    #[test]
    fn dispatch_all_parser_finds_handlers() {
        let content = r#"
pub fn dispatch_all(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    theme::apply(hero, event)
        || radius::apply(hero, event)
        || view_toggles::apply(hero, event)
}
"#;
        let found = dispatch_all_referenced_handlers(content);
        assert_eq!(
            found,
            vec![
                "radius".to_string(),
                "theme".to_string(),
                "view_toggles".to_string()
            ]
        );
    }
}
