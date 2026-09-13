//! Architecture gate (blindagem Fase 0.3) — instructional docs must not
//! route agents to **dead gates**.
//!
//! ## Why this exists
//!
//! `DIRETIVA_IMPLEMENTACAO.md` §2 tells implementers to rely on the gates
//! `architecture_studio_slider_wiring` and `architecture_studio_cycler_wiring`
//! — both of which were DELETED with the brush studio (ADR-0099) and exist in
//! zero `.rs` files. An agent following the doc trusts a safety net that is no
//! longer there. (During the 2026-06-20 diagnosis a careful sub-agent reading
//! the docs reported those two gates as live — proving the doc misleads even
//! attentive readers.)
//!
//! This meta-gate scans the **instructional canon** — the docs the router
//! sends agents to per-task — for `architecture_*` gate references and fails
//! if any names a gate that does not exist as a real test. Historical ADRs and
//! forward-looking plans (`docs/architecture/decisions/`, `docs/plans/`,
//! `docs/archive/`, `HANDOFF_*`) are intentionally OUT of scope: they record
//! what was/what's-planned, not what an implementer must do now.
//!
//! Only `architecture_*` tokens are checked (the `architecture_` prefix is the
//! project's gate-naming convention). Glob/placeholder forms in prose
//! (`architecture_*`, `architecture_<dom>_contract_surface`) don't match the
//! `architecture_[a-z0-9_]+` token shape, so they're naturally ignored.
//!
//! Dep-free (std only).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Doc `architecture_*` references that are intentionally allowed despite no
/// matching test (e.g. a gate documented as deliberately not-yet-built). Keep
/// SMALL — the whole point is that docs don't promise gates that don't exist.
const ALLOW_DOC_GATE_REFS: &[&str] = &[
    // CLAUDE.md §6 documents this gate as REMOVED by ADR-0099 (the
    // `ph2d-painter-contracts` crate was deleted with the painting engine).
    // It is an accurate historical "this was removed" mention, not a live
    // routing instruction — so the reference is honest, not misleading.
    "architecture_painter_contract_surface",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// The instructional canon — files the per-task router (CLAUDE.md §1) points
/// agents at. NOT ADRs/plans/archive/handoffs.
fn canon_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("CLAUDE.md"),
        root.join("SKILL_Stack_PH2D_Definitiva.md"),
    ];
    let dir = root.join("docs/IntegracaoMultiAgente");
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            // ⚠️ Os `HANDOFF_*` moram AQUI desde a W2 (12/09) e são registo do que foi, não canon: o
            // doc deste ficheiro já os declarava fora, e o código não os tirava — um handoff que cite
            // um gate renomeado depois ficaria vermelho para sempre sem mentir a ninguém.
            let handoff = p
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("HANDOFF_"));
            if p.extension().and_then(|s| s.to_str()) == Some("md") && !handoff {
                files.push(p);
            }
        }
    }
    files.retain(|p| p.exists());
    files.sort();
    files
}

/// Pull every `architecture_<ident>` token out of `src`.
fn architecture_tokens(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let needle = "architecture_";
    let mut rest = src;
    while let Some(pos) = rest.find(needle) {
        let tail = &rest[pos..];
        let end = tail
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(tail.len());
        let tok = &tail[..end];
        // Require at least one char after the `architecture_` prefix.
        if tok.len() > needle.len() {
            out.insert(tok.to_string());
        }
        rest = &tail[end..];
    }
    out
}

/// Live gate names: every test-file stem under `{crates,shells,tools}/*/tests/` plus every
/// `fn <name>` / `mod <name>` declared in those files. (A gate is referenced
/// in docs either by its file name or, occasionally, its test-fn name.)
///
/// ⚠️ **Até 2026-09-13 só `crates/` era varrido**, e os gates da SHELL (`shells/desktop/tests/it`,
/// 800 testes — entre eles `architecture_no_downcast_to_concrete_tool_in_shell`) liam-se como
/// MORTOS: um doc que os citasse reprovava sobre uma referência verdadeira. Os três directórios são
/// os membros da workspace que têm `tests/`.
fn live_gate_names(root: &Path) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let member_dirs = ["crates", "shells", "tools"]
        .iter()
        .filter_map(|top| std::fs::read_dir(root.join(top)).ok())
        .flat_map(|rd| rd.flatten());
    for entry in member_dirs {
        let tests = entry.path().join("tests");
        // Desde 2026-09-10 (onda W1) os gates vivem em `tests/it/*.rs`, um módulo por ficheiro
        // de UM binário por crate; os `tests/*.rs` soltos são as crates com um ficheiro só.
        let Ok(test_files) = std::fs::read_dir(&tests) else {
            continue;
        };
        let it_files = std::fs::read_dir(tests.join("it")).into_iter().flatten();
        for tf in test_files.flatten().chain(it_files.flatten()) {
            let path = tf.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.insert(stem.to_string());
            }
            if let Ok(body) = std::fs::read_to_string(&path) {
                for kw in ["fn ", "mod "] {
                    let mut rest = body.as_str();
                    while let Some(pos) = rest.find(kw) {
                        let tail = &rest[pos + kw.len()..];
                        let end = tail
                            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                            .unwrap_or(tail.len());
                        let ident = &tail[..end];
                        if !ident.is_empty() {
                            names.insert(ident.to_string());
                        }
                        rest = &tail[end..];
                    }
                }
            }
        }
    }
    names
}

/// ⭐ **A régua dos gates vivos vê os membros com `tests/` — a SHELL incluída.** Sentinela: um gate
/// da shell e um de uma crate têm de estar na população (até 2026-09-13 a shell não estava, e um doc
/// que a citasse reprovava sobre uma referência verdadeira; nenhum doc do canon a cita hoje, logo sem
/// esta sentinela a cura não tinha quem a medisse).
#[test]
fn the_live_gate_census_sees_the_shell_and_the_crates() {
    let names = live_gate_names(&workspace_root());
    for gate in [
        "architecture_no_downcast_to_concrete_tool_in_shell",
        "architecture_docs_reference_live_gates",
    ] {
        assert!(
            names.contains(gate),
            "a régua dos gates vivos não vê `{gate}`"
        );
    }
}

/// ⭐ **O canon deixa os `HANDOFF_*` de fora e mantém os docs que mandam** — as duas metades, com a
/// população do lado de fora medida (sem handoffs no directório, «ficam de fora» não mediria nada).
#[test]
fn the_canon_leaves_the_handoffs_out_and_keeps_the_directives() {
    let root = workspace_root();
    let nomes: Vec<String> = canon_files(&root)
        .iter()
        .filter_map(|p| p.file_name()?.to_str().map(str::to_owned))
        .collect();
    for doc in ["CLAUDE.md", "DIRETRIZ.md"] {
        assert!(nomes.iter().any(|n| n == doc), "o canon perdeu `{doc}`");
    }
    let dentro: Vec<&String> = nomes.iter().filter(|n| n.starts_with("HANDOFF_")).collect();
    assert!(dentro.is_empty(), "handoffs entraram no canon: {dentro:?}");
    let no_directorio = std::fs::read_dir(root.join("docs/IntegracaoMultiAgente"))
        .expect("o directório do canon existe")
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("HANDOFF_"))
        .count();
    assert!(
        no_directorio > 0,
        "nenhum HANDOFF_* no directório — a metade «fica de fora» mede nada"
    );
}

#[test]
fn instructional_docs_only_reference_existing_gates() {
    let root = workspace_root();
    let live = live_gate_names(&root);
    let allow: BTreeSet<&str> = ALLOW_DOC_GATE_REFS.iter().copied().collect();

    // token -> the canon files that mention it
    let mut dead: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in canon_files(&root) {
        let Ok(body) = std::fs::read_to_string(&file) else {
            continue;
        };
        let rel = file
            .strip_prefix(&root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| file.display().to_string());
        for tok in architecture_tokens(&body) {
            if live.contains(&tok) || allow.contains(tok.as_str()) {
                continue;
            }
            dead.entry(tok).or_default().push(rel.clone());
        }
    }

    assert!(
        dead.is_empty(),
        "instructional docs reference `architecture_*` gates that DO NOT EXIST \
         as tests — agents are routed to a safety net that isn't there:\n  {}\n\n\
         fix: point the doc at the real gate (e.g. `architecture_panel_wiring_parity`), \
         delete the stale reference, or — if it's a deliberately-future gate — add \
         it to ALLOW_DOC_GATE_REFS with a reason.",
        dead.iter()
            .map(|(tok, files)| format!("{tok}  ← {}", files.join(", ")))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
