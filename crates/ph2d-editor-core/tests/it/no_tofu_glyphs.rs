//! Arch gate: no out-of-font ("tofu") glyphs in UI string literals.
//!
//! The bundled font (Inter) does NOT cover the Unicode arrow block
//! (U+2190..U+21FF — `→ ← ↵ …`) nor the technical-symbols block
//! (U+2300..U+23FF — `⌘ ⎕ …`). Any such char in a string the editor
//! renders shows up as a tofu box. This bug recurred ≥3×: the
//! Cmd/Return glyphs in topbar tooltips + ListItem/ContextMenu samples,
//! and the `→` in the "Tool → X" / "Theme → X" / "View → X" toasts.
//! See `docs/UI_Bugs/README.md` §9.19 + `docs/Image Tools Bugs/README.md`
//! §3 and DIRETRIZ §4.1 rule 1.
//!
//! This gate scans the editor-core + desktop-shell source for those
//! glyphs INSIDE string literals (comments are skipped — math/doc
//! comments legitimately use `→ ⊙ ⟹`). Fix a hit by switching to ASCII
//! (`Cmd+S`, `->`) or the one safe non-ASCII separator `·` (U+00B7,
//! Latin-1, in-font — the topbar already uses it).
//!
//! ⛔ **Ele afirmava `violations.is_empty()` sem afirmar que tinha VISTO alguma coisa** — a mesma
//! metade que faltava ao irmão `arch_no_char_count_widths.rs`, e pela mesma linha de código: o
//! `collect_rs` saía em silêncio no `Err` do `read_dir`, e a descoberta dos painéis era um
//! `if let Ok(…)` sem `else`. Renomear uma raiz esvaziava o corpus e o gate ficava **VERDE com
//! todos os tofus da árvore**. *Um zero de «não medi» e um de «está limpo» são o mesmo byte.*
//!
//! A cura não é um número mágico: o `collect_rs` **falha alto**, e cada raiz da lista tem de
//! contribuir pelo menos um `.rs` — o piso é a própria lista de raízes, derivada do disco.

use std::fs;
use std::path::{Path, PathBuf};

/// `true` for a char in a block the bundled Inter font doesn't cover,
/// so it renders as tofu in any UI string.
fn is_tofu(c: char) -> bool {
    let cp = c as u32;
    (0x2190..=0x21FF).contains(&cp) // arrows: → ← ↵ ⇒ …
        || (0x2300..=0x23FF).contains(&cp) // technical symbols: ⌘ ⎕ ⌥ …
}

/// Collect every `.rs` file under `dir` (recursive) — and **an unreadable
/// `dir` is an error, never an empty list**.
///
/// ⛔ It used `let Ok(..) else { return }`: pointing it at a path that no
/// longer exists returned `[]`, the caller asserted `violations.is_empty()`,
/// and the gate went green with the whole tree unscanned.
fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("could not scan {}: {e}", dir.display()));
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Walk `src` as Rust source and return `(line, char)` for every tofu
/// glyph that appears INSIDE a string literal (normal or raw). Line
/// comments (`//`), block comments (`/* */`) and char literals are
/// skipped — only string-literal content is inspected.
fn tofu_in_strings(src: &str) -> Vec<(usize, char)> {
    #[derive(PartialEq)]
    enum St {
        Normal,
        Line,
        Block,
        Str,
        RawStr,
        RawHash,
    }
    let b: Vec<char> = src.chars().collect();
    let n = b.len();
    let mut i = 0;
    let mut line = 1usize;
    let mut st = St::Normal;
    let mut hits = Vec::new();
    while i < n {
        let c = b[i];
        if c == '\n' {
            line += 1;
        }
        match st {
            St::Normal => {
                if c == '/' && b.get(i + 1) == Some(&'/') {
                    st = St::Line;
                    i += 2;
                } else if c == '/' && b.get(i + 1) == Some(&'*') {
                    st = St::Block;
                    i += 2;
                } else if c == 'r' && b.get(i + 1) == Some(&'#') && b.get(i + 2) == Some(&'"') {
                    st = St::RawHash;
                    i += 3;
                } else if c == 'r' && b.get(i + 1) == Some(&'"') {
                    st = St::RawStr;
                    i += 2;
                } else if c == '"' {
                    st = St::Str;
                    i += 1;
                } else if c == '\'' {
                    // char literal or lifetime — skip the next token
                    // crudely (enough to avoid wrongly entering a string).
                    i += 1;
                } else {
                    i += 1;
                }
            }
            St::Line => {
                if c == '\n' {
                    st = St::Normal;
                }
                i += 1;
            }
            St::Block => {
                if c == '*' && b.get(i + 1) == Some(&'/') {
                    st = St::Normal;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            St::Str => {
                if c == '\\' {
                    i += 2; // skip escaped char
                } else if c == '"' {
                    st = St::Normal;
                    i += 1;
                } else {
                    if is_tofu(c) {
                        hits.push((line, c));
                    }
                    i += 1;
                }
            }
            St::RawStr => {
                if c == '"' {
                    st = St::Normal;
                    i += 1;
                } else {
                    if is_tofu(c) {
                        hits.push((line, c));
                    }
                    i += 1;
                }
            }
            St::RawHash => {
                if c == '"' && b.get(i + 1) == Some(&'#') {
                    st = St::Normal;
                    i += 2;
                } else {
                    if is_tofu(c) {
                        hits.push((line, c));
                    }
                    i += 1;
                }
            }
        }
    }
    hits
}

#[test]
fn no_tofu_glyphs_in_ui_strings() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_root = crate_root.join("..");
    let mut scan_roots: Vec<PathBuf> = vec![
        crate_root.join("src"),
        crate_root.join("../../shells/desktop/src"),
    ];
    // Wave 10 / Etapa 5.1: panel-* crates paint UI strings (panel titles,
    // toggle labels, section headers). They were a blind spot — any
    // arrow/cmd glyph there rendered as a tofu box in production UI.
    // ⚠️ **`expect`, not `if let Ok`:** an unreadable `crates/` silently left
    // the sweep with two roots and ZERO panels — and the panels are the blind
    // spot this whole block was added to close.
    let entries = fs::read_dir(&crates_root)
        .unwrap_or_else(|e| panic!("could not read {}: {e}", crates_root.display()));
    let mut panel_srcs: Vec<PathBuf> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let name = path.file_name()?.to_str()?.to_string();
            if path.is_dir()
                && name.starts_with("ph2d-panel-")
                && name != "ph2d-panel-registry-init"
            {
                let src = path.join("src");
                if src.is_dir() { Some(src) } else { None }
            } else {
                None
            }
        })
        .collect();
    assert!(
        !panel_srcs.is_empty(),
        "no `ph2d-panel-*` crate under {} — the discovery broke, and without it this gate scans \
         half of what it promises",
        crates_root.display()
    );
    panel_srcs.sort();
    scan_roots.extend(panel_srcs);

    let mut files = Vec::new();
    for r in &scan_roots {
        // ⚠️ **The positive control, root by root.** A root that moved drops out
        // of the corpus without a word; here it drops out with its name in the
        // message.
        let before = files.len();
        collect_rs(r, &mut files);
        assert!(
            files.len() > before,
            "scan root {} yielded not one `.rs` — either it moved, or this gate is scanning the \
             void",
            r.display()
        );
    }
    // This gate's OWN source names the glyphs in comments/strings as
    // examples — skip it so it doesn't flag itself.
    files.retain(|p| !p.ends_with("no_tofu_glyphs.rs"));

    let mut violations = Vec::new();
    for path in &files {
        let Ok(raw) = fs::read_to_string(path) else {
            continue;
        };
        let text = raw.replace("\r\n", "\n");
        for (line, c) in tofu_in_strings(&text) {
            violations.push(format!(
                "{}:{line}  U+{:04X} '{c}'",
                path.display(),
                c as u32
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "Out-of-font (tofu) glyph(s) in UI string literal(s):\n  {}\n\n\
         Inter (bundled) doesn't cover the arrow / technical-symbol \
         blocks — these render as a tofu box on screen. Use ASCII \
         (`Cmd+S`, `->`) or `·` (U+00B7). See docs/UI_Bugs §9.19.",
        violations.join("\n  ")
    );
}

#[test]
fn detector_flags_arrow_and_cmd_in_strings_only() {
    // Positive: arrow + Cmd glyph inside a string are caught.
    assert_eq!(tofu_in_strings("let x = \"Tool \u{2192} Pad\";").len(), 1);
    assert_eq!(tofu_in_strings("v(\"\u{2318}B\")").len(), 1);
    // Negative: the same glyph in a line/block comment is ignored.
    assert!(tofu_in_strings("// a \u{2192} b").is_empty());
    assert!(tofu_in_strings("/* a \u{2318} b */").is_empty());
    // Negative: the safe middot in a string is fine.
    assert!(tofu_in_strings("\"Tool \u{00b7} Pad\"").is_empty());
}

/// **A missing scan root FAILS instead of reading as clean** — the control for
/// the half that was missing.
///
/// ⚠️ It is the mutation written as a test: before, pointing [`collect_rs`] at a
/// path that does not exist returned `[]`, the gate above saw zero violations
/// and went green with the tree unscanned. Without this control the cure would
/// be a claim about itself.
#[test]
#[should_panic(expected = "could not scan")]
fn a_missing_scan_root_fails_loud_instead_of_reading_as_clean() {
    let ghost = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/this-root-does-not-exist");
    let mut out = Vec::new();
    collect_rs(&ghost, &mut out);
}
