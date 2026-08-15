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
//! Until the Fluent runtime (`ph2d-i18n`) is wired and `t!(...)`
//! exists, this test enforces a **frozen baseline**: each file in
//! `BASELINE` is allowed exactly the listed number of violations.
//! Any file not in `BASELINE` must have zero. When `t!(...)` ships,
//! reduce `BASELINE` entries to 0 (or remove) and the test becomes
//! strict.
//!
//! Failure modes this catches:
//! - New widget introduces a hardcoded `.label(...)` or
//!   `.placeholder(...)` outside test code.
//! - Existing widget adds violations beyond its baseline count.
//! - Existing widget reduces violations below baseline (forces the
//!   maintainer to drop the baseline entry, keeping the list honest).

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
    ("ph2d-panel-hierarchy/src/paint.rs", 1),
    // The entity-name TextInput placeholder ("Name…"). Moved from
    // sections.rs to sections/identity.rs in the §T2.1 per-section split;
    // i18n migration tracked separately (replaced when Fluent ships).
    ("ph2d-panel-inspector/src/sections/identity.rs", 1),
    // ⚠️ **W-SignalLeave: a entrada da §11 saiu daqui, e a DÍVIDA NÃO.** O
    // scanner conta literais dentro de `.placeholder("…")`, e a §11 passou a ter
    // DOIS campos de sinal (chegada e saída) pintados por uma função só, com os
    // dois textos numa TABELA — então nenhum dos dois cruza o padrão que este
    // gate procura, e a contagem caiu de 1 para 0.
    //
    // As duas strings continuam hardcoded, em
    // `ph2d-panel-inspector/src/sections/physics_rows.rs` ("Signal on hit…" e
    // "Signal on leave…"), e migram junto com o irmão da row de Name quando o
    // Fluent chegar — a `ph2d-panel-inspector` não depende de `ph2d-i18n`, e a
    // alternativa hoje seria uma dependência nova para duas strings, com um
    // segundo mecanismo de texto convivendo com o primeiro.
    //
    // A entrada é REMOVIDA em vez de zerada porque o gate exige que a lista seja
    // exata: uma entrada em 0 vale o mesmo que ausência, e ausência é o que o
    // arquivo de fato tem *pelo padrão que este scanner enxerga*. O gate segue
    // afiado para o próximo `.placeholder("literal")` que nascer ali.
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

/// **Este ficheiro é um módulo de TESTE inteiro?** — perguntado ao PAI, que é quem o gateia.
///
/// ⚠️ **É a mesma lei que o [`strip_test_modules`] já aplica, uma grafia depois.** Um
/// `#[cfg(test)] mod tests { … }` inline é removido; um `#[cfg(test)] mod tests;` que resolve para
/// `<slug>/tests.rs` era lido como PRODUÇÃO — e o ficheiro é literalmente o mesmo código, movido
/// para o irmão pelo tecto de LOC. Medido em 2026-08-15: a `ph2d-editor-core` tem **onze** desses
/// ficheiros, e **dez passavam por acidente** (nenhum deles usava `.label("`/`.placeholder("`); o
/// décimo-primeiro — o `text_input/tests.rs` — nasceu de um split e trouxe um `.placeholder`
/// consigo, virando o gate vermelho sobre código que nunca correu em produção.
///
/// ⚠️ **E a pergunta é feita ao PAI de propósito, nunca ao NOME do ficheiro.** Uma lista de nomes
/// (`tests.rs`, `*_tests.rs`, …) é a enumeração que apodrece no dia em que alguém chamar o irmão
/// de outra coisa — e, pior, isentaria um ficheiro de produção com nome parecido. Quem sabe se
/// isto é teste é a declaração que o compila.
fn is_declared_under_cfg_test(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    let Some(dir) = path.parent() else {
        return false;
    };
    // O pai de `<dir>/<stem>.rs` é o `<dir>/mod.rs` — ou, num módulo sem directório, o
    // `<dir>.rs` um nível acima. Os dois podem declarar o filho.
    let mut parents = vec![dir.join("mod.rs")];
    if let (Some(gp), Some(dir_name)) = (dir.parent(), dir.file_name().and_then(|s| s.to_str())) {
        parents.push(gp.join(format!("{dir_name}.rs")));
    }
    parents
        .iter()
        .any(|p| fs::read_to_string(p).is_ok_and(|src| declares_cfg_test_mod(&src, stem, path, p)))
}

/// O pai declara `mod <stem>;` (ou um `#[path = "…"] mod …;` que resolve para `path`) sob um
/// `#[cfg(test)]`?
fn declares_cfg_test_mod(src: &str, stem: &str, path: &Path, parent: &Path) -> bool {
    let lines: Vec<&str> = src.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if !t.starts_with("mod ") && !t.starts_with("pub mod ") && !t.starts_with("pub(") {
            continue;
        }
        // As linhas de atributo IMEDIATAMENTE acima da declaração.
        let mut cfg_test = false;
        let mut declared: Option<PathBuf> = None;
        let mut j = i;
        while j > 0 {
            let a = lines[j - 1].trim();
            if !a.starts_with("#[") {
                break;
            }
            if a.starts_with("#[cfg(test)]") {
                cfg_test = true;
            }
            if let Some(rest) = a.strip_prefix("#[path = \"")
                && let Some(rel) = rest.split('"').next()
            {
                declared = parent.parent().map(|d| d.join(rel));
            }
            j -= 1;
        }
        if !cfg_test {
            continue;
        }
        let matches_by_name = t
            .trim_end_matches(';')
            .rsplit(' ')
            .next()
            .is_some_and(|name| name == stem);
        let matches_by_path = declared.as_deref() == Some(path);
        if matches_by_name || matches_by_path {
            return true;
        }
    }
    false
}
