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
//! TWO blocks are regenerated from a scan of `chrome/*.rs`:
//!   1. the `mod foo;` declaration block, and
//!   2. the `dispatch_all` `||` chain body (ADR-0107, Camada 0).
//!
//! The chain used to be hand-written because **z-order is load-bearing**
//! (per `chrome/mod.rs`: "earlier modules win when ids overlap") — and a
//! naive codegen would force alphabetical order, silently breaking it. We
//! keep z-order WITHOUT the hand-edit by having each handler declare its
//! own priority in a `// ph2d-chrome-sync:z=NN` marker; the chain is
//! emitted sorted by `(z, name)`. That order is a **pure function of the
//! handler set**, so two parallel lines each adding a handler never
//! collide on the shared chain (the whole point of Camada 0).
//!
//! Adding a chrome handler:
//!   1. Drop `chrome/<slug>.rs` with `pub fn apply(hero, event) -> bool`
//!      and a `// ph2d-chrome-sync:z=NN` marker (omit → sorts last, at
//!      [`DEFAULT_Z`], alphabetically among other unmarked handlers).
//!   2. `cargo run -p ph2d-chrome-sync` regenerates BOTH blocks.
//!
//! That's it — no hand-edit of any shared file.

use std::path::Path;

/// Markers delimiting the generated `mod foo;` declaration block.
pub const MOD_BEGIN: &str = "// <ph2d-chrome-sync:begin>";
pub const MOD_END: &str = "// <ph2d-chrome-sync:end>";

/// Markers delimiting the generated `dispatch_all` `||` chain body (ADR-0107,
/// Camada 0). The body between them is a pure function of the handler set,
/// so parallel lines adding handlers never collide on it.
pub const DISPATCH_BEGIN: &str = "// <ph2d-chrome-sync:dispatch-begin>";
pub const DISPATCH_END: &str = "// <ph2d-chrome-sync:dispatch-end>";

/// A handler with no `// ph2d-chrome-sync:z=NN` marker sorts here — after
/// every explicitly-prioritized handler, alphabetically among the unmarked.
/// Keeps "just drop a file" working: a new handler with no opinion on z-order
/// lands at the end of the chain, which is the safe default when its ids
/// don't overlap anyone else's.
pub const DEFAULT_Z: u32 = 1000;

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

/// Read a handler's dispatch priority from its `// ph2d-chrome-sync:z=NN`
/// marker. Lower z = earlier in the chain = wins when two handlers claim
/// the same widget id (the "z-order is load-bearing" invariant). Absent
/// marker → [`DEFAULT_Z`].
pub fn handler_z(chrome_dir: &Path, name: &str) -> u32 {
    let path = chrome_dir.join(format!("{name}.rs"));
    let content = std::fs::read_to_string(path).unwrap_or_default();
    parse_z_marker(&content).unwrap_or(DEFAULT_Z)
}

/// Extract `NN` from the first `ph2d-chrome-sync:z=NN` occurrence.
fn parse_z_marker(content: &str) -> Option<u32> {
    const KEY: &str = "ph2d-chrome-sync:z=";
    let idx = content.find(KEY)?;
    content[idx + KEY.len()..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

/// Handler names ordered for `dispatch_all`: by `(z-marker, name)`. Stable
/// and deterministic, so the rendered chain is a pure function of the
/// handler set — the property that makes concurrent handler additions
/// collision-free (ADR-0107, Camada 0).
pub fn sorted_dispatch_handlers(chrome_dir: &Path) -> Vec<String> {
    let mut names = scan_chrome_handlers(chrome_dir); // already alphabetical
    names.sort_by_key(|n| (handler_z(chrome_dir, n), n.clone()));
    names
}

/// Render the `dispatch_all` body: `<name>::apply(hero, event)` for the
/// first handler, then `|| <name>::apply(hero, event)` for the rest.
/// Indentation (4 / 8 spaces) matches the hand-written original so the
/// migration diff is order-only, not whitespace churn.
pub fn render_dispatch_lines(sorted: &[String]) -> Vec<String> {
    sorted
        .iter()
        .enumerate()
        .map(|(i, n)| {
            if i == 0 {
                format!("    {n}::apply(hero, event)")
            } else {
                format!("        || {n}::apply(hero, event)")
            }
        })
        .collect()
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

    #[test]
    fn parse_z_marker_reads_number() {
        assert_eq!(parse_z_marker("// ph2d-chrome-sync:z=30\nmod x;"), Some(30));
        assert_eq!(
            parse_z_marker("//! doc\n// ph2d-chrome-sync:z=290"),
            Some(290)
        );
        assert_eq!(parse_z_marker("no marker here"), None);
    }

    #[test]
    fn render_dispatch_first_bare_rest_prefixed() {
        let lines = render_dispatch_lines(&[
            "theme".to_string(),
            "radius".to_string(),
            "io_menu".to_string(),
        ]);
        assert_eq!(
            lines,
            vec![
                "    theme::apply(hero, event)".to_string(),
                "        || radius::apply(hero, event)".to_string(),
                "        || io_menu::apply(hero, event)".to_string(),
            ]
        );
    }

    #[test]
    fn sorted_dispatch_orders_by_z_then_name() {
        // Two handlers share z=1000 (unmarked) → alphabetical tiebreak;
        // an explicit z=10 handler sorts ahead of both. Uses a temp dir so
        // the test is hermetic (no dependency on the real chrome/ tree).
        let dir = std::env::temp_dir().join("ph2d_chrome_sync_sort_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("zebra.rs"), "pub fn apply() {}").unwrap();
        std::fs::write(dir.join("alpha.rs"), "pub fn apply() {}").unwrap();
        std::fs::write(
            dir.join("first.rs"),
            "// ph2d-chrome-sync:z=10\npub fn apply() {}",
        )
        .unwrap();
        let sorted = sorted_dispatch_handlers(&dir);
        assert_eq!(sorted, vec!["first", "alpha", "zebra"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
