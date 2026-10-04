//! ⭐⭐⭐ **AS FORMAS NA PLACA chegam ao quadro, e ENTRE o mundo e o chrome** (doc 121 do Motion,
//! W2) — o elo da shell. ⚠️ `include_str!` porque o presente pede uma janela e uma placa: a LEI
//! (`motion_shape_placa`, com os gates de decisão e de paridade de pixel) e *este elo* são as duas
//! metades, e cada uma reprova por um motivo diferente.
//!
//! *Mutações: apagar o `|=` (a camada é colada num acumulador que o compositor não lê) · não
//! forçar as faixas do documento (o documento fica na cena Vello, POR CIMA das formas) · apagar o
//! `!…ativa()` (as formas desenhadas DUAS vezes) · mover a colagem para depois do chrome (as formas
//! por cima dos gizmos) · desenhar a camada DEPOIS do halo (doc 121 §9.14: o halo redesenha as
//! formas DESTE quadro, e leria as do anterior).*
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
        em("self.motion_shell.placa.camada()"),
        em("vello_pass.render_to_intermediate("),
    );
    // ⭐ doc 121 §9.14 — a camada DESENHA-SE antes dos passes de luz (o halo redesenha-a).
    let em_present = |agulha: &str| present.find(agulha).expect(agulha);
    assert!(
        em_present("self.motion_shell.placa.desenha(") < em_present("super::present_fx::run("),
        "as formas desenham-se ANTES do halo que as redesenha"
    );
    assert!(
        cima < placa && placa < vello,
        "as formas vão DEPOIS do mundo (as faixas de cima) e ANTES do chrome (a cena Vello)"
    );
    assert!(chrome.contains("ph2d_render::BandSource::Formas"));
}

/// ⭐⭐⭐ **A rota do DISPOSITIVO chega ao quadro** (doc 121 W3): a decisão lê as formas do
/// cozimento e o presente passa o buffer dele à placa — os dois SÓ com o cozimento vivo, que é a
/// pergunta que o passe de sprites já faz (`motion.gpu_live`). *Mutações: apagar o `filter` de uma
/// das pontas (um quadro que caiu para a CPU desenharia as formas do ANTERIOR) · decidir sempre
/// pela CPU (as formas do dispositivo, caladas nas sprites, não se desenhariam em sítio nenhum).*
#[test]
fn a_rota_do_dispositivo_chega_ao_quadro() {
    let bandas = include_str!("fase_vector_bands.rs");
    assert!(bandas.contains(".formas().filter(|_| motion.gpu_live)"));
    assert!(bandas.contains("self.motion_shell.placa.decide_do_dispositivo("));
    let present = include_str!("present.rs");
    assert!(present.contains(".formas().filter(|_| motion.gpu_live)"));
    assert!(present.contains("let buffer = do_dispositivo.map(|f| f.buffer());"));
}
