//! **É este ficheiro um módulo de TESTE inteiro?** — a pergunta feita ao PAI, que é quem o compila.
//!
//! ⚠️ **Ela vive AQUI porque tem dois donos, e a segunda cópia é a que diverge.** A `hr15` pagou
//! esta lei em 2026-08-15 (onze `<slug>/tests.rs` lidos como produção, dez a passar por acidente)
//! e a `hr12` tinha o **mesmo** ponto cego, com a mesma consequência e uma linha de allowlist
//! escrita à mão como prova (`paint_wire_tests.rs`, *"a test module, it paints nothing"*). Duas
//! varreduras a fazer a mesma pergunta são duas respostas à espera de discordar.
//!
//! ⚠️ **E a pergunta é feita ao PAI, nunca ao NOME.** Uma lista de nomes (`tests.rs`,
//! `*_tests.rs`, …) é a enumeração que apodrece no dia em que alguém chamar o irmão de outra
//! coisa — e, pior, isentaria um ficheiro de PRODUÇÃO com nome parecido.
//!
//! Não é um alvo de teste: vive em `tests/common/`, que o cargo não compila como binário.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// **Este ficheiro é um módulo de TESTE inteiro?** — perguntado ao PAI, que é quem o gateia.
///
/// ⚠️ **É a mesma lei que o `strip_test_modules` da `hr15` já aplica, uma grafia depois.** Um
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
pub fn is_declared_under_cfg_test(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    let Some(dir) = path.parent() else {
        return false;
    };
    // ⚠️ **O pai pode estar em TRÊS sítios, e o terceiro foi o que a `hr12` não via.** O
    // `<dir>/mod.rs` e o `<dir>.rs` um nível acima são os dois óbvios; o terceiro é um **IRMÃO
    // PLANO** — `src/paint.rs` a declarar `#[path = "paint_tests.rs"] mod paint_tests;`, com os
    // dois no mesmo directório. Era exactamente esse o caso que obrigava uma linha de allowlist
    // escrita à mão por ficheiro (`paint_wire_tests.rs`), e ele é o padrão mais comum nos painéis:
    // um `_tests.rs` cortado do pai pelo tecto de LOC.
    //
    // A varredura do directório é `O(irmãos)` por ficheiro e isto é um gate — o custo medido da
    // suíte inteira ficou abaixo de um segundo.
    let mut parents = vec![dir.join("mod.rs")];
    if let (Some(gp), Some(dir_name)) = (dir.parent(), dir.file_name().and_then(|s| s.to_str())) {
        parents.push(gp.join(format!("{dir_name}.rs")));
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let sib = e.path();
            if sib != path && sib.extension().and_then(|s| s.to_str()) == Some("rs") {
                parents.push(sib);
            }
        }
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
