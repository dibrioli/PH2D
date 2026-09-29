//! ⭐⭐⭐ **AS FORMAS NA PLACA chegam ao quadro, e ENTRE o mundo e o chrome** (doc 121 do Motion,
//! W2) — o elo da shell. ⚠️ `include_str!` porque o presente pede uma janela e uma placa: a LEI
//! (`motion_shape_placa`, com os gates de decisão e de paridade de pixel) e *este elo* são as duas
//! metades, e cada uma reprova por um motivo diferente.
//!
//! *Mutações: apagar o `|=` (a camada é colada num acumulador que o compositor não lê) · não
//! forçar as faixas do documento (o documento fica na cena Vello, POR CIMA das formas) · apagar o
//! `!…ativa()` (as formas desenhadas DUAS vezes) · mover a colagem para depois do chrome (as formas
//! por cima dos gizmos).*
#[test]
fn as_formas_na_placa_entram_entre_o_mundo_e_o_chrome() {
    let present = include_str!("present.rs");
    assert!(present.contains("plan.banded |= self.motion_shell.placa.ativa();"));
    let bandas = include_str!("fase_vector_bands.rs");
    assert!(bandas.contains("doc_bands_of(frame_order, placa)"));
    let overlays = include_str!("fase_vector_overlays.rs");
    assert!(overlays.contains("if motion_tool_active && !self.motion_shell.placa.ativa() {"));
    let chrome = include_str!("present_chrome.rs");
    let em = |agulha: &str| chrome.find(agulha).expect(agulha);
    let (cima, placa, vello) = (
        em("present_bands::draw_upper_bands("),
        em("self.motion_shell.placa.desenha("),
        em("vello_pass.render_to_intermediate("),
    );
    assert!(
        cima < placa && placa < vello,
        "as formas vão DEPOIS do mundo (as faixas de cima) e ANTES do chrome (a cena Vello)"
    );
    assert!(chrome.contains("ph2d_render::BandSource::Formas"));
}
