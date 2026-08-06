//! **A chave de ablação do laço de altura nunca é armada em produto.**
//!
//! [`ph2d_painter_brush::ablate`] é `pub` de verdade — tinha de ser, porque `#[cfg(test)]` nesta crate
//! não vale quando quem roda o teste é a `ph2d-tool-painter` (o `cfg(test)` só liga na crate sob
//! teste). O preço dessa visibilidade é este gate: **um flag de ablação que alguém arma e esquece é
//! uma engine com duas leis**, e o modo de falha é silencioso — a silhueta vira um degrau, o AA some,
//! e nada reclama.
//!
//! A varredura cobre as DUAS crates que podem alcançar a chave, e o único sítio legítimo é uma sonda
//! (`measure_*.rs`) ou uma suíte (`*_tests.rs` / `tests.rs`).
//!
//! ⚠️ **Com CONTROLE POSITIVO nas duas pontas** — uma varredura que não acha arquivo nenhum, ou que
//! não acha nenhum sítio legítimo, passaria por vácuo e seria um gate que não pode falhar.

use std::path::{Path, PathBuf};

/// Todo `.rs` sob `dir`.
fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Um arquivo em que armar a ablação é legítimo: sonda ou suíte.
fn is_measurement(p: &Path) -> bool {
    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
    name.starts_with("measure_") || name.ends_with("_tests.rs") || name == "tests.rs"
}

#[test]
fn the_ablation_switch_is_only_armed_by_measurements() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/");
    let mut files = Vec::new();
    walk(&root.join("ph2d-painter-brush/src"), &mut files);
    walk(&root.join("ph2d-tool-painter/src"), &mut files);

    // CONTROLE 1: a varredura de fato leu as duas crates.
    assert!(
        files.len() > 100,
        "a varredura achou só {} arquivos — o caminho está errado e o gate seria vácuo",
        files.len()
    );

    let mut armed_in_measurement = 0usize;
    let mut offenders = Vec::new();
    for p in &files {
        // O próprio módulo DEFINE `set`/`with`; ele não é um chamador.
        if p.file_name().is_some_and(|n| n == "ablate.rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(p) else {
            continue;
        };
        if !(src.contains("ablate::set(") || src.contains("ablate::with(")) {
            continue;
        }
        if is_measurement(p) {
            armed_in_measurement += 1;
        } else {
            offenders.push(p.display().to_string());
        }
    }

    // CONTROLE 2: existe pelo menos um sítio legítimo — senão o `assert` abaixo passa por vácuo e
    // continuaria passando no dia em que alguém renomeasse a API.
    assert!(
        armed_in_measurement > 0,
        "nenhum sítio arma a ablação — a API foi renomeada e este gate parou de olhar para o produto"
    );

    assert!(
        offenders.is_empty(),
        "a chave de ablação do laço de altura é armada FORA de uma sonda: {offenders:?}"
    );
}
