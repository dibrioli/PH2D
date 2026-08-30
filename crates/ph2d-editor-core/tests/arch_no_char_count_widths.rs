//! Wave 10 / Etapa 5.3 — ban `.chars().count() *` as a string-width
//! proxy in any UI-painting crate.
//!
//! Recurring bug class (UI_Bugs §3.3, §9.16, §10.1): code measured a
//! string's display width via `s.chars().count() * GLYPH_W`. This
//! breaks for:
//!   - non-monospaced glyphs (Inter is proportional, NOT monospace)
//!   - combining marks (`é` = 1 char, 1 grapheme, but multiple cluster)
//!   - emojis / wide CJK (1 char ≠ 1 advance)
//!   - kerning pairs
//!
//! Canonical replacement: `text_system.measure_text(label, font_size)`
//! which uses the live `cosmic-text` shaping pipeline.
//!
//! Scope: `crates/ph2d-editor-core/src/{widget,screens}/` +
//! `crates/ph2d-panel-*/src/**`. Gate skips comments and the
//! `text_system` crate itself (where char-counting is part of glyph
//! iteration, not width measurement).
//!
//! ⛔ **Ele afirmava `violations.is_empty()` sem afirmar que tinha VISTO alguma coisa.** O
//! `collect_rs` saía em silêncio no `Err` do `read_dir` e a descoberta dos painéis era um
//! `if let Ok(…)` sem `else`: renomear uma raiz — um split de módulo, um `crates/` reorganizado —
//! esvaziava o corpus e o gate ficava **VERDE com todas as violações na árvore**. *Um balde que
//! ninguém enche lê-se como perfeito.*
//!
//! A metade que faltava tem duas partes, e nenhuma é um número mágico: o `collect_rs` **falha
//! alto** (uma raiz desta lista existe, ou a lista está errada) e cada raiz varrida tem de
//! contribuir **pelo menos um `.rs`** — o piso é o comprimento da própria lista de raízes, que é
//! derivada do disco.

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn no_chars_count_as_width_proxy() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_root = crate_root.join("..");
    let mut scan_roots: Vec<PathBuf> = vec![
        crate_root.join("src/widget"),
        crate_root.join("src/screens"),
    ];
    // ⚠️ **`expect`, não `if let Ok`:** um `crates/` ilegível deixava a varredura com duas raízes e
    // ZERO painéis, em silêncio — e os painéis são metade do alcance deste gate.
    let entries = fs::read_dir(&crates_root)
        .unwrap_or_else(|e| panic!("não consegui ler {}: {e}", crates_root.display()));
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
        "nenhuma crate `ph2d-panel-*` em {} — a descoberta partiu, e sem ela este gate varre \
         metade do que promete",
        crates_root.display()
    );
    panel_srcs.sort();
    scan_roots.extend(panel_srcs);

    let mut files = Vec::new();
    for r in &scan_roots {
        let before = files.len();
        collect_rs(r, &mut files);
        // ⚠️ **O controle positivo, raiz a raiz.** Uma raiz que se mudou de sítio some do corpus
        // sem um aviso; aqui ela some com o nome dela na mensagem.
        assert!(
            files.len() > before,
            "a raiz {} não rendeu um `.rs` sequer — ou ela mudou de sítio, ou este gate está a \
             varrer o vazio",
            r.display()
        );
    }

    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        let Ok(src) = fs::read_to_string(path) else {
            continue;
        };
        for (lineno, line) in src.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if line.contains("CHARS-COUNT-OK") {
                continue;
            }
            if line.contains(".chars().count()") && line_uses_as_width(line) {
                violations.push(format!(
                    "{}:{} — `{}`",
                    path.display(),
                    lineno + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "`.chars().count() * <width>` used as a string-width proxy:\n  \
         {}\n\n\
         Use `text_system.measure_text(label, font_size)` for true width \
         (Inter is proportional — `chars().count()` ≠ pixel advance). \
         If the count is genuinely needed as a count (not width), append \
         `// CHARS-COUNT-OK: <reason>`.",
        violations.join("\n  ")
    );
}

/// True when the line uses `.chars().count()` in a context that looks
/// like width arithmetic — multiplied by something, or compared to a
/// pixel/width-named variable.
fn line_uses_as_width(line: &str) -> bool {
    // Pattern A: `.chars().count() *` (arithmetic)
    if line.contains(".chars().count() *") || line.contains(".chars().count()*") {
        return true;
    }
    // Pattern B: `width = .chars().count()` or similar assignment
    let Some(cc_pos) = line.find(".chars().count()") else {
        return false;
    };
    let pre = &line[..cc_pos];
    let pre_low = pre.to_ascii_lowercase();
    // The assignment target / surrounding token mentions width/w/advance
    // — heuristic but catches the real bug patterns.
    pre_low.contains("width =")
        || pre_low.contains("_w =")
        || pre_low.contains("advance")
        || pre_low.contains("text_w")
        || pre_low.contains("label_w")
}

/// Todo `.rs` sob `dir`, recursivo — e **um `dir` ilegível é um erro, não um vazio**.
///
/// ⛔ Ele fazia `let Ok(..) else { return }`: apontá-lo a um caminho que não existe devolvia uma
/// lista vazia, o chamador afirmava `violations.is_empty()` e o gate passava com a árvore inteira
/// por varrer. *A saída silenciosa transformava «não medi» em «está limpo», que são o mesmo byte.*
fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("não consegui varrer {}: {e}", dir.display()));
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn detector_flags_width_misuse_only() {
    assert!(line_uses_as_width("let w = label.chars().count() * 7.0;"));
    assert!(line_uses_as_width(
        "let text_w = s.chars().count() as f32 * GLYPH_W;"
    ));
    // Negative: counting for iteration limit, not width.
    assert!(!line_uses_as_width("if s.chars().count() > 64 { … }"));
    assert!(!line_uses_as_width("let n = s.chars().count();"));
}

/// **Uma raiz que não existe REPROVA** — o controle da metade que faltava.
///
/// ⚠️ É a mutação escrita como teste: antes, apontar o [`collect_rs`] a um caminho inexistente
/// devolvia `[]`, o gate acima lia zero violações e ficava verde com a árvore por varrer. Sem este
/// controle a cura seria uma afirmação sobre si mesma.
#[test]
#[should_panic(expected = "não consegui varrer")]
fn a_missing_scan_root_fails_loud_instead_of_reading_as_clean() {
    let ghost = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/esta-raiz-nao-existe");
    let mut out = Vec::new();
    collect_rs(&ghost, &mut out);
}
