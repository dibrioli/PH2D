//! ⛔⛔ **O *Edit Prefab* da Hierarquia e o do painel vetorial são O MESMO VERBO** (2026-09-07).
//!
//! # Porque um censo, e porquê este
//!
//! O par de gates da `ph2d-panel-hierarchy` prova que o clique levanta `HierRequest::EditPrefab`
//! com a linha certa, e o `vec_component_general_tests` prova o que o verbo FAZ. O que fica no meio
//! é o braço da shell que liga os dois — e ele nasceu **copiado do braço do `Instantiate`**.
//!
//! ⚠️ *Um `Verb::Place` esquecido ali daria um item de menu chamado «Edit Prefab» que **põe outra
//! cópia**: perfeitamente ligado, perfeitamente errado, e com os dois gates das pontas verdes.*
//!
//! ⛔ Ele é textual porque o braço vive dentro do laço de quadro da `render_loop`, cuja função tem
//! ~35 argumentos e um `AppGfx` com uma surface de janela real.

use std::path::Path;

/// O corpo do ficheiro sem comentários — senão o censo lê o que o código DIZ sobre si.
fn code_of(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O item do menu pede o verbo de ABRIR, e não o de instanciar.**
///
/// **Mutação que deve sangrar:** o braço do `EditPrefab` a pedir `Verb::Place`.
#[test]
fn the_hierarchy_edit_prefab_row_asks_for_the_open_verb() {
    let body = code_of("render_loop/mod.rs");
    // ⚠️ **A âncora é só o nome da variante** — a 1.ª redacção procurava `EditPrefab { row }` e o
    // `cargo fmt` partiu o braço em quatro linhas no minuto seguinte. *Um censo ancorado na
    // formatação mede o `rustfmt`, não o código.*
    let at = body
        .find("HierRequest::EditPrefab")
        .expect("o braço do `Edit Prefab` da Hierarquia desapareceu do dreno");
    let arm = &body[at..(at + 240).min(body.len())];
    assert!(
        arm.contains("Verb::Edit"),
        "o item `Edit Prefab` da Hierarquia pede outro verbo — ele esta' ligado e faz outra coisa \
         que o rotulo nao diz:\n{arm}"
    );
}
