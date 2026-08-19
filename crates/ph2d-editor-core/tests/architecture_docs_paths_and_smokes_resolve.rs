//! Architecture gate — the instructional canon must not cite a **path** or a
//! **smoke** that does not exist.
//!
//! ## Why this exists
//!
//! The sibling gate [`architecture_docs_reference_live_gates`] proved the idea:
//! a doc that routes an agent to a dead *gate* sells a safety net that isn't
//! there. It only checks `architecture_*` tokens — and the audit of 2026-08-18
//! measured what escaped through that hole:
//!
//! - **56 caminhos quebrados** citados pelos dois docs que o roteador manda ler
//!   (55 no `SKILL_Stack`, 1 no `CLAUDE.md`). Quatro famílias mecânicas: `../`
//!   num arquivo que já está na raiz · `crates/ph2d-editor/src/**` depois de a
//!   crate virar casca (o código foi para `ph2d-editor-core`) · `tests/<domínio>/`
//!   numa raiz `tests/` que só tem `fixtures/` e `spike/` · árvores apagadas.
//! - **1 smoke morto em 55**: o `CLAUDE.md §5` listava `PH2D_MOTION_PATH_SMOKE`,
//!   que não existe em `.rs` nenhum — o nome real é `PH2D_PATH_SMOKE`. ⚠️ Este é
//!   o defeito mais caro por byte do repo: quem copia o comando abre o app na
//!   cena padrão, não vê nada, e conclui que **a feature quebrou**. O smoke é
//!   onde o Enio APRENDE a ferramenta (`CLAUDE.md §0.8`) — um smoke morto ensina
//!   que o produto está partido.
//!
//! ⚠️ **Escopo idêntico ao do irmão, de propósito:** só o cânone de INSTRUÇÃO.
//! ADR, plano, handoff e `docs/archive/` registram o que FOI ou o que se PLANEIA
//! e citam livremente coisas que morreram — cobrá-los aqui transformaria o gate
//! num gerador de ruído, e um gate ruidoso é desligado.
//!
//! Dep-free (std only).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Paths cited by the canon that legitimately do not resolve on disk.
/// Keep SMALL and give each one a reason — the point is that the canon does not
/// promise files that aren't there.
const ALLOW_DEAD_PATHS: &[&str] = &[
    // Placeholders in prose that show the SHAPE of a path, not a real file.
    "docs/architecture/decisions/NNNN-titulo.md",
];

/// Smokes cited by the canon that no `.rs` reads. Each entry needs a reason.
const ALLOW_DEAD_SMOKES: &[&str] = &[
    // `CLAUDE.md §5` (Timeline) states in the SAME line that this smoke died
    // with the expression-authoring card. An accurate "this is gone" mention is
    // honest documentation, not a broken routing instruction.
    "PH2D_EXPR_SMOKE",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// The instructional canon — same set the sibling gate guards.
fn canon_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("CLAUDE.md"),
        root.join("SKILL_Stack_PH2D_Definitiva.md"),
    ];
    let dir = root.join("docs/IntegracaoMultiAgente");
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(p);
            }
        }
    }
    files.retain(|p| p.exists());
    files.sort();
    files
}

/// Every markdown link destination `](...)` in `src`, already stripped of the
/// anchor/line suffix and percent-decoded for spaces.
///
/// ⚠️ Skips URLs and pure anchors. Keeps `docs/Motion Nodes/` — a destination
/// with a LITERAL SPACE is normal in this repo, and a naive whitespace split is
/// exactly what corrupted an earlier survey of these links.
fn link_targets(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == ']' && bytes[i + 1] == '(' {
            let start = i + 2;
            let mut j = start;
            let mut depth = 1;
            while j < bytes.len() {
                match bytes[j] {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    '\n' => break,
                    _ => {}
                }
                j += 1;
            }
            if j < bytes.len() && depth == 0 {
                let dest: String = bytes[start..j].iter().collect();
                out.insert(dest);
            }
            i = j.max(start);
        }
        i += 1;
    }
    out.into_iter().filter_map(normalize_target).collect()
}

/// `None` for things this gate does not own (URLs, anchors, mail).
fn normalize_target(raw: String) -> Option<String> {
    let d = raw.trim();
    // A title after the path (`path "Title"`) — keep only the path part.
    let d = d.split(" \"").next().unwrap_or(d).trim();
    if d.is_empty() || d.starts_with('#') {
        return None;
    }
    for p in ["http://", "https://", "mailto:", "file:", "<http"] {
        if d.starts_with(p) {
            return None;
        }
    }
    // Anchor / line suffix is not part of the path on disk.
    let d = d.split('#').next().unwrap_or(d);
    let d = d.trim_end_matches('/');
    if d.is_empty() {
        return None;
    }
    Some(d.replace("%20", " "))
}

/// Recursively collect every `PH2D_[A-Z0-9_]+` token that appears in Rust
/// sources. A doc-cited smoke absent from this set is read by nobody.
fn env_tokens_in_code(root: &Path) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for top in ["crates", "shells", "tools"] {
        walk_rs(&root.join(top), &mut |body| {
            let needle = "PH2D_";
            let mut rest = body;
            while let Some(pos) = rest.find(needle) {
                let tail = &rest[pos..];
                let end = tail
                    .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
                    .unwrap_or(tail.len());
                let tok = &tail[..end];
                if tok.len() > needle.len() {
                    found.insert(tok.to_string());
                }
                rest = &tail[end..];
            }
        });
    }
    found
}

fn walk_rs(dir: &Path, f: &mut impl FnMut(&str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            // `target/` under a worktree would dwarf the scan and holds no source.
            if p.file_name().and_then(|s| s.to_str()) == Some("target") {
                continue;
            }
            walk_rs(&p, f);
            continue;
        }
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if let Ok(body) = std::fs::read_to_string(&p) {
            f(&body);
        }
    }
}

/// Smoke names cited in a doc: `PH2D_*_SMOKE` written inline or in backticks.
fn smoke_tokens(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = src;
    while let Some(pos) = rest.find("PH2D_") {
        let tail = &rest[pos..];
        let end = tail
            .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
            .unwrap_or(tail.len());
        let tok = &tail[..end];
        // Only SMOKE vars — diagnostics (`PH2D_*_LOG`, `_DIAG`, `_PERF`) are not
        // routing instructions and churn faster than the docs.
        if tok.ends_with("_SMOKE") {
            out.insert(tok.to_string());
        }
        rest = &tail[end..];
    }
    out
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .map(|x| x.display().to_string())
        .unwrap_or_else(|_| p.display().to_string())
}

#[test]
fn instructional_docs_only_cite_paths_that_exist() {
    let root = workspace_root();
    let allow: BTreeSet<&str> = ALLOW_DEAD_PATHS.iter().copied().collect();
    let mut dead: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for file in canon_files(&root) {
        let Ok(body) = std::fs::read_to_string(&file) else {
            continue;
        };
        let doc = rel(&root, &file);
        let base = file.parent().unwrap_or(&root).to_path_buf();
        for t in link_targets(&body) {
            if allow.contains(t.as_str()) {
                continue;
            }
            // Resolve relative to the doc, as a markdown reader would.
            if base.join(&t).exists() || root.join(&t).exists() {
                continue;
            }
            dead.entry(t).or_default().push(doc.clone());
        }
    }

    assert!(
        dead.is_empty(),
        "the instructional canon links to {} path(s) that DO NOT EXIST — an agent \
         following the router lands on nothing:\n  {}\n\n\
         fix: point at the real path, delete the link, or add it to ALLOW_DEAD_PATHS \
         with a reason. Known mechanical families: a leading `../` in a doc that \
         already lives at the repo root · `crates/ph2d-editor/src/**` (the code moved \
         to `ph2d-editor-core`) · `tests/<domain>/` (the root `tests/` only has \
         `fixtures/` and `spike/`).",
        dead.len(),
        dead.iter()
            .map(|(t, docs)| format!("{t}  ← {}", docs.join(", ")))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn instructional_docs_only_cite_smokes_the_code_reads() {
    let root = workspace_root();
    let live = env_tokens_in_code(&root);

    // Positive control: the scan must find a smoke we KNOW is wired. Without
    // this, a broken walk returns an empty set and every doc-cited smoke reads
    // as "dead" — or, if the assert were inverted, everything passes silently.
    assert!(
        live.contains("PH2D_ADAPTER_SMOKE"),
        "positive control failed: the code scan found no `PH2D_ADAPTER_SMOKE`, so \
         the walk is broken and this gate's verdict is worthless ({} tokens found)",
        live.len()
    );

    let allow: BTreeSet<&str> = ALLOW_DEAD_SMOKES.iter().copied().collect();
    let mut dead: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in canon_files(&root) {
        let Ok(body) = std::fs::read_to_string(&file) else {
            continue;
        };
        let doc = rel(&root, &file);
        for tok in smoke_tokens(&body) {
            if live.contains(&tok) || allow.contains(tok.as_str()) {
                continue;
            }
            dead.entry(tok).or_default().push(doc.clone());
        }
    }

    assert!(
        dead.is_empty(),
        "the canon lists smoke env var(s) that NO `.rs` reads — the Enio copies the \
         command, sees the default scene, and concludes the feature is broken:\n  {}\n\n\
         fix: use the real var name (`git grep -n <NAME>`), or add it to \
         ALLOW_DEAD_SMOKES if the doc is accurately recording that it died.",
        dead.iter()
            .map(|(t, docs)| format!("{t}  ← {}", docs.join(", ")))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
