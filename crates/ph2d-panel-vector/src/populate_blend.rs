//! O registro dos widgets do **BLEND / MORPH** — irmão do [`super`] pelo teto de 600
//! LOC do painel, e par natural do `paint_blend` (que os PINTA).
//!
//! Registrar é o que os torna clicáveis: pintar + hit-rect não basta — é a classe de bug que já
//! matou botões do vetor uma vez. O gate `architecture_panel_wiring_parity` cobra a
//! correspondência entre os dois arquivos.

use super::{button, slider_chip, slider_chip_int};
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_tool_vector::params::{
    BLEND_STEPS_DEFAULT, MAX_BLEND_STEPS, MORPH_T_DEFAULT, MORPH_T_STEP, blend_steps_to_track,
};

/// Os widgets do Blend Object vivo e do Morph.
///
/// # Os pares slider+chip passam pelo `slider_chip`, e isso NÃO é preferência de estilo
///
/// Estes dois foram registrados **à mão** primeiro (`store.register` × 2), e o que faltou foi a
/// LIGAÇÃO — o slider funcionava e a **caixa ficava muda**: digitar nela não emite o
/// `ValueChanged` do slider (é o `if let Some(slider_id) = store.linked_slider(id)` do
/// `commit_number_buffer`), então nada chegava ao barramento e o blend nunca era re-afinado. O
/// Enio pegou no smoke; o gate `every_chip_is_linked_to_its_slider` pega agora.
///
/// O helper registra **e** liga, numa chamada — é a porta que torna a omissão impossível.
pub(super) fn populate_blend(store: &mut WidgetStore) {
    // BLEND: o slider de passos (com a caixa LIGADA) + os botões, sendo 2 deles as saídas do
    // objeto vivo (Expand / Release).
    //
    // O mapa é afim, como o link pede: `display = track × (MAX−1) + 1` (o inverso exato do
    // `blend_steps_from_track`). E é a variante **INTEIRA** porque a unidade pintada é uma
    // CONTAGEM de passos — meio passo não existe.
    #[allow(clippy::cast_precision_loss)]
    let steps_scale = (MAX_BLEND_STEPS - 1) as f32;
    slider_chip_int(
        store,
        ids::VECTOR_BLEND_STEPS,
        ids::VECTOR_BLEND_STEPS_NUM,
        blend_steps_to_track(BLEND_STEPS_DEFAULT),
        f64::from(BLEND_STEPS_DEFAULT),
        steps_scale,
        1.0,
    );
    store.set_number_range(
        ids::VECTOR_BLEND_STEPS_NUM,
        1.0,
        f64::from(MAX_BLEND_STEPS),
        1.0,
    );
    button(store, ids::VECTOR_BLEND_RUN);
    // **Reset Spine** — volta o spine editado (modo Node) ao automático. Registrar aqui é o que o
    // torna clicável (pintar + hit-rect não basta — a classe de bug dos botões do vetor).
    button(store, ids::VECTOR_BLEND_RESET_SPINE);
    // **Expand** / **Release** (ADR-0128 Fase D) — o mesmo motivo: sem registro, sao pintura.
    button(store, ids::VECTOR_BLEND_EXPAND);
    button(store, ids::VECTOR_BLEND_RELEASE);
    // **MORPH** — o irmão animável do Blend: uma forma só, e o `t` dela é keyável. O slider é a
    // autoria ao vivo (o artista estaciona a forma onde ela fica bem e aperta K).
    button(store, ids::VECTOR_MORPH_RUN);
    // O `t` já É a fração do caminho, então o mapa track→display é a IDENTIDADE (escala 1, offset
    // 0) — e contínuo, não inteiro: é o que faz o morph deslizar entre as formas.
    slider_chip(
        store,
        ids::VECTOR_MORPH_T,
        ids::VECTOR_MORPH_T_NUM,
        MORPH_T_DEFAULT,
        f64::from(MORPH_T_DEFAULT),
        1.0,
        0.0,
    );
    // O range é o domínio inteiro do `t`; o motor clampa lá também (`Plan::at`).
    store.set_number_range(ids::VECTOR_MORPH_T_NUM, 0.0, 1.0, MORPH_T_STEP);
}
