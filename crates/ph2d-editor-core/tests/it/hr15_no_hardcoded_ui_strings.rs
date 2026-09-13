//! HR-15 enforcement: no hardcoded UI strings in widget production code.
//!
//! Catches the two patterns that feed user-facing text to AccessKit /
//! visible UI text and bypass localization:
//!
//! - `.label("...")` — a11y label setter.
//! - `.placeholder("...")` — text input placeholder.
//!
//! Doc comments, `#[cfg(test)]` mod blocks (and their `#[test]`
//! functions), and strings passed to non-UI APIs (token names, log
//! messages, panics) are NOT targets — they aren't user-visible UI
//! strings, and HR-15 explicitly exempts them.
//!
//! ⛔⛔ **ESTA PROSA PROMETIA A LEI INTEIRA E ENTREGA DUAS METADES — corrigido em 2026-09-10.**
//!
//! Ela dizia: *«Until the Fluent runtime (`ph2d-i18n`) is wired and `t!(...)` exists, this test
//! enforces a frozen baseline… When `t!(...)` ships, reduce `BASELINE` entries to 0»*. ⚠️ **A porta
//! CHEGOU e a frase ficou:** o `ph2d_i18n::tr` existe, é usado por seis tabelas
//! (`vector` · `sculpt3d` · `model3d` · `chrome` · e as chaves do `lib.rs`), e a `ph2d-editor-core`
//! migrou os rótulos dela em 2026-09-10. *Um gate à espera de uma coisa que já chegou é uma frase
//! que faz a próxima pessoa acreditar que o HR-15 está fechado.*
//!
//! # ⚠️ O que este gate cobre, e o que ele NÃO cobre
//!
//! | metade do HR-15 | quem a mede | alcance |
//! |---|---|---|
//! | o rótulo de **acessibilidade** e o *placeholder* (`.label(…)` · `.placeholder(…)`) | **este ficheiro** | só `src/widget/`, com baseline congelada |
//! | o texto **PINTADO** (o que o artista lê no ecrã) | [`no_label_of_this_crate_is_written_in_the_painter`](no_label_of_this_crate_is_written_in_the_painter.rs) | a crate inteira, por ponto fixo sobre as portas de texto |
//!
//! ⛔ **Nenhum dos dois sozinho é «o HR-15»**, e foi exactamente por isso que `418` literais
//! pintados viveram anos sem uma régua: este gate lê dois padrões, e o texto que o artista **vê**
//! não passa por nenhum deles. Censo e mecanismo:
//! `docs/UI_New_and_Simple/medicoes/07_o_buraco_do_hr15_o_texto_PINTADO.md`.
//!
//! A `BASELINE` abaixo continua a ser o que é: **dívida congelada que só encolhe**. Um ficheiro
//! fora dela tem de ter zero, e um que desça abaixo do número obriga a baixar a entrada — é a
//! metade de obsolescência, e é ela que impede a lista de virar licença (`CLAUDE.md` §5.0).
//!
//! Failure modes this catches:
//! - New widget introduces a hardcoded `.label(...)` or
//!   `.placeholder(...)` outside test code.
//! - Existing widget adds violations beyond its baseline count.
//! - Existing widget reduces violations below baseline (forces the
//!   maintainer to drop the baseline entry, keeping the list honest).

use crate::cfg_test_modules;
use cfg_test_modules::is_declared_under_cfg_test;

use std::fs;
use std::path::{Path, PathBuf};

/// Frozen baseline. Each entry: (relative path under `src/widget/`,
/// allowed violation count in production code). Adding entries
/// requires review — every entry is technical debt against HR-15.
const BASELINE: &[(&str, usize)] = &[
    ("card.rs", 1),    // .label("card") — generic a11y fallback
    ("divider.rs", 1), // .label("separator") — generic a11y fallback
    ("popover.rs", 1), // .label("popover") — generic a11y fallback
    // Wave 8 Phase 2.A — showcase is a hardcoded English reference
    // gallery (peripheral agents read it to see canonical widget
    // appearance). i18n migration tracked separately.
    ("showcase/inputs.rs", 2),
    // Wave 10 / Etapa 5.1: panel-* extension. Hardcoded placeholders
    // for chrome fields (name input, search) — replaced when Fluent
    // runtime ships. Path key is `<crate>/src/<rel>`.
    // ✅ **A entrada da HIERARQUIA saiu (2026-09-13, `line/UIUX`) — e desta vez a DÍVIDA também.**
    // O `"Search…"` do `paint_head.rs` passou a `tr("panel.hierarchy.search")`: o literal saiu do
    // binário, não só do alcance deste scanner (o contrário da nota do `physics_rows.rs` abaixo). O
    // painel inteiro fala pela tabela, com gate próprio na crate dele
    // (`every_word_this_panel_shows_comes_from_the_string_table`, régua da `ph2d-label-census`).
    // O `placeholder` da busca do NAVEGADOR DE ASSETS (plano `docs/Components/07`, wave A5).
    // ⚠️ Mesma forma e mesma dívida do irmão da Hierarquia, uma linha acima — é literalmente o
    // mesmo widget a fazer o mesmo trabalho no painel ao lado —, e cai com ele quando o `t!(…)`
    // shipar. ⛔ E ele fica VISÍVEL de propósito: passar a string por um argumento faria a
    // contagem cair para 0 sem tirar o literal do binário, que é a isenção silenciosa que as
    // duas notas abaixo já nomeiam.
    ("ph2d-panel-asset-browser/src/paint.rs", 1),
    // ✅ **As SETE entradas do INSPECTOR saíram (2026-09-13, `line/UIUX`) — e a DÍVIDA com elas.**
    // Os `placeholder` do nome (`identity.rs`), do nome da âncora (`anchors.rs`), da §11
    // (`anim_rows.rs`, 3), dos TIMERS (2), das SIGNAL ACTIONS (3), do AUDIO e da CAMERA passaram a
    // `tr("panel.inspector.…")`, e com eles os dois do `physics_rows.rs` que a nota W-SignalLeave
    // dizia escondidos numa tabela, fora do alcance deste scanner. O painel inteiro fala pela tabela,
    // com gate próprio na crate dele (`every_word_this_panel_shows_comes_from_the_string_table`,
    // régua da `ph2d-label-census`).
    // The Audio Editor's empty-state TextInput placeholder ("No clip loaded").
    // Generic English fallback — replaced when Fluent runtime ships.
    // The clip-name placeholder moved to `paint_sections.rs` when the panel was split
    // into collapsible sections (2026-07-12); same one string, new file.
    ("ph2d-panel-audio-editor/src/paint_sections.rs", 1),
];

#[test]
fn no_new_hardcoded_ui_strings_in_widgets() {
    let widget_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/widget");
    let crates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut counts: Vec<(String, usize)> = Vec::new();
    walk(&widget_root, &widget_root, &mut |relpath, abspath| {
        if abspath.extension().and_then(|s| s.to_str()) != Some("rs") {
            return;
        }
        let content = fs::read_to_string(abspath).expect("read widget file");
        let production = strip_test_modules(&content);
        let n = count_violations(&production);
        if n > 0 {
            let rel = relpath.to_string_lossy().replace('\\', "/");
            counts.push((rel, n));
        }
    });
    // Wave 10 / Etapa 5.1: extend scope to panel-* crates. Their hardcoded
    // strings get frozen via PANEL_BASELINE — gate prevents NET ADDITIONS
    // while Fluent runtime is pending. Path key is `<crate>/src/<rel>`.
    if let Ok(entries) = fs::read_dir(&crates_root) {
        let mut panel_dirs: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?.to_string();
                if path.is_dir()
                    && name.starts_with("ph2d-panel-")
                    && name != "ph2d-panel-registry-init"
                {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        panel_dirs.sort();
        for panel_dir in &panel_dirs {
            let src = panel_dir.join("src");
            if !src.is_dir() {
                continue;
            }
            let crate_name = panel_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            walk(&src, &src, &mut |relpath, abspath| {
                if abspath.extension().and_then(|s| s.to_str()) != Some("rs") {
                    return;
                }
                let content = fs::read_to_string(abspath).expect("read panel file");
                let production = strip_test_modules(&content);
                let n = count_violations(&production);
                if n > 0 {
                    let rel = relpath.to_string_lossy().replace('\\', "/");
                    counts.push((format!("{crate_name}/src/{rel}"), n));
                }
            });
        }
    }

    let mut errors = Vec::new();

    for (file, found) in &counts {
        let expected = BASELINE
            .iter()
            .find(|(b, _)| *b == file.as_str())
            .map(|(_, n)| *n)
            .unwrap_or(0);
        if *found > expected {
            errors.push(format!(
                "  {file} — found {found} violation(s), baseline allows {expected}"
            ));
        }
    }

    for (b_file, b_count) in BASELINE {
        let current = counts
            .iter()
            .find(|(f, _)| f.as_str() == *b_file)
            .map(|(_, n)| *n)
            .unwrap_or(0);
        if current < *b_count {
            errors.push(format!(
                "  baseline obsolete: {b_file} expected {b_count}, now {current} — please reduce BASELINE entry"
            ));
        }
    }

    assert!(
        errors.is_empty(),
        "HR-15 violation — hardcoded UI strings in widget code:\n{}\n\n\
         Either replace with `t!(...)` (when Fluent runtime ships) or — \
         if the string is a stable generic fallback — adjust BASELINE in \
         this test.",
        errors.join("\n"),
    );
}

/// Strip `#[cfg(test)]` mod blocks (and their `#[test]` functions)
/// from the source so production-only code remains. Brace-counting
/// based: when `#[cfg(test)]` is seen, skip until the matching `}`.
/// Naive about strings containing `{`/`}` — adequate for widget code,
/// where brace literals in user-facing strings are extremely rare.
fn strip_test_modules(src: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[cfg(test)]") {
            i += 1;
            // Find next non-attribute line.
            while i < lines.len() && lines[i].trim_start().starts_with("#[") {
                i += 1;
            }
            if i >= lines.len() {
                break;
            }
            // If the item is a single-line statement (use/const), just
            // skip that one line and continue.
            let next = lines[i];
            if next.trim_end().ends_with(';') && !next.contains('{') {
                i += 1;
                continue;
            }
            // Walk braces until depth returns to 0.
            let mut depth: i32 = 0;
            let mut started = false;
            loop {
                if i >= lines.len() {
                    break;
                }
                for c in lines[i].chars() {
                    match c {
                        '{' => {
                            depth += 1;
                            started = true;
                        }
                        '}' => depth -= 1,
                        _ => {}
                    }
                }
                i += 1;
                if started && depth <= 0 {
                    break;
                }
            }
        } else {
            out.push_str(line);
            out.push('\n');
            i += 1;
        }
    }
    out
}

/// Count `.label("...")` and `.placeholder("...")` occurrences in the
/// production slice. Skips lines that look like comments (`//`).
fn count_violations(production: &str) -> usize {
    let mut n = 0;
    for line in production.lines() {
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        n += count_pattern(line, ".label(\"");
        n += count_pattern(line, ".placeholder(\"");
    }
    n
}

fn count_pattern(haystack: &str, needle: &str) -> usize {
    let mut n = 0;
    let mut idx = 0;
    while idx < haystack.len() {
        match haystack[idx..].find(needle) {
            Some(p) => {
                n += 1;
                idx += p + needle.len();
            }
            None => break,
        }
    }
    n
}

fn walk(root: &Path, dir: &Path, cb: &mut dyn FnMut(&Path, &Path)) {
    for entry in fs::read_dir(dir).expect("read widget dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, cb);
        } else if let Ok(rel) = path.strip_prefix(root) {
            if is_declared_under_cfg_test(&path) {
                continue;
            }
            cb(rel, &path);
        }
    }
}
