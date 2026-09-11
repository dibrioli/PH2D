//! **Todo cabeçalho que o painel PINTA está na lista das colapsáveis** — o arch-gate da classe.
//!
//! # Por que este arquivo existe
//!
//! O irmão `seam.rs::every_section_header_is_registered_as_collapsible` percorre
//! `ids::VECTOR_SECTIONS` e prova que **tudo o que está na lista** está marcado. É a metade
//! errada: uma seção que PINTA um header e **não** está na lista é invisível para ele — e foi
//! exatamente o que aconteceu com o Text on Path e o Pattern on Path, que chegaram à `main` em
//! 2026-07-23 fora da lista, com o gate verde. O chevron pintava, o hit-rect registava, e o
//! clique não dobrava nada: `dispatch` consulta `is_collapsible_section` antes de disparar o
//! toggle, então esquecer a entrada **não dá erro em lado nenhum**.
//!
//! Este gate faz a pergunta ao contrário: varre as chamadas de `section_header` no FONTE e exige
//! que cada id nomeado ali apareça na lista. Uma tabela escrita à mão dentro do gate driftaria da
//! tela pela mesma razão que a lista driftou.
//!
//! # Por NOME, e não por valor
//!
//! Um `NodeId` é o hash de uma string: dado o fonte, só há o identificador. Os dois lados são
//! lidos como texto — o mesmo método do `architecture_panel_wiring_parity`, e pela mesma razão.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn panel_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Os identificadores `VECTOR_SECTION_*` que aparecem como 1º argumento de `section_header(`.
fn painted_headers() -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let dir = panel_src();
    for entry in std::fs::read_dir(&dir).expect("src/ do painel") {
        let path = entry.expect("entrada").path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let src = std::fs::read_to_string(&path).expect("fonte legível");
        let mut rest = src.as_str();
        while let Some(at) = rest.find("section_header(") {
            rest = &rest[at + "section_header(".len()..];
            // O id é o 1º argumento; a chamada pode quebrar linha entre o `(` e ele.
            let head: String = rest.chars().take(120).collect();
            if let Some(i) = head.find("ids::VECTOR_SECTION_") {
                let name: String = head[i + "ids::".len()..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                found.insert(name);
            }
        }
    }
    found
}

/// Os identificadores dentro do array `VECTOR_SECTIONS` (que mora na `ph2d-editor-core`).
fn listed_sections() -> BTreeSet<String> {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ph2d-editor-core/src/ids/chrome/vector_sections.rs");
    let src = std::fs::read_to_string(&file).expect("a lista das seções");
    let start = src
        .find("pub const VECTOR_SECTIONS")
        .expect("a const existe");
    let body = &src[start..];
    let end = body.find("];").expect("o array fecha");
    body[..end]
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| w.starts_with("VECTOR_SECTION_"))
        .map(str::to_owned)
        .collect()
}

/// **Um cabeçalho pintado que não está na lista é um chevron MORTO.**
#[test]
fn every_painted_section_header_is_collapsible() {
    let painted = painted_headers();
    // Controle positivo: se o scanner deixar de achar as chamadas, ele passa a afirmar nada — e
    // um gate que não pode falhar é pior que gate nenhum.
    assert!(
        painted.len() >= 20,
        "o scanner achou só {} chamadas de `section_header` — ele quebrou, não o produto",
        painted.len()
    );
    let listed = listed_sections();
    let orphans: Vec<&String> = painted.difference(&listed).collect();
    assert!(
        orphans.is_empty(),
        "cabeçalhos PINTADOS e fora de `VECTOR_SECTIONS` (chevron que não dobra, sem erro em \
         lado nenhum): {orphans:?}"
    );
}
