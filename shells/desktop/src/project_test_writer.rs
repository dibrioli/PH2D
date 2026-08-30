//! **O ESCRITOR de arquivo de projeto das suítes** — filho de `project_tests`
//! (declarado lá via `#[path]`), e re-exportado por ele para que os outros filhos
//! (`sculpt`, `tape`, `field`, `pattern_art`) o alcancem por `super::*`.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, e foi o teto de LOC do HR-18 que o cobrou:** aqui *o que
//! um arquivo de projeto É em disco*, lá *o que um load faz com ele*. As quatro funções são uma
//! escada — cada uma acrescenta um campo — e um segundo escritor divergiria no próximo campo novo,
//! que é a razão pela qual elas já eram uma só.

use super::*;

/// Grava um arquivo de projeto em `path` com o esquema `schema`. Passar
/// `PROJECT_SCHEMA` produz um arquivo que o loader ACEITA; qualquer outro número
/// produz um que ele RECUSA — os dois caminhos que os gates abaixo separam.
///
/// `timeline` são os bytes do `TimelineDoc` (vazio = projeto sem animação).
pub(super) fn write_project_with(path: &std::path::Path, schema: u32, timeline: Vec<u8>) {
    write_project_full(path, schema, timeline, Vec::new());
}

/// O mesmo, com os bytes da ESCULTURA — o 8º campo do arquivo (v52).
pub(super) fn write_project_full(
    path: &std::path::Path,
    schema: u32,
    timeline: Vec<u8>,
    sculpt: Vec<u8>,
) {
    write_project_art(path, schema, timeline, sculpt, Vec::new());
}

/// O mesmo, com os bytes da **ARTE DOS PADRÕES** — o 16.º campo do arquivo (v101).
pub(super) fn write_project_art(
    path: &std::path::Path,
    schema: u32,
    timeline: Vec<u8>,
    sculpt: Vec<u8>,
    pattern_art: Vec<u8>,
) {
    let file = ProjectFile {
        state: empty_state(),
        assets: Vec::new(),
        painted: Vec::new(),
        motion: String::new(),
        timeline,
        physics: Default::default(),
        tokens: Vec::new(),
        settings: crate::project_settings::collect(Default::default()),
        sculpt,
        baked_forms: Vec::new(),
        player_tape: ph2d_physics_ecs::TapeWire::default(),
        sprite_pixels: Vec::new(),
        stable_id_counter: ph2d_ecs::StableId::FIRST,
        input_map: ph2d_input::InputMap::new(),
        pattern_art,
        catalogs: Vec::new(),
    };
    let bytes = postcard::to_allocvec(&(schema, &file)).expect("serializa");
    std::fs::write(path, bytes).expect("grava o arquivo de projeto");
}

pub(super) fn write_project(path: &std::path::Path, schema: u32) {
    write_project_with(path, schema, Vec::new());
}
