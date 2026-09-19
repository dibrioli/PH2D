//! **As molduras das DUAS secções do ABANÃO** (suplente #25) — irmão do
//! [`super::paint_optional_top20_tail`].
//!
//! ⚠️ **Ficheiro próprio por CAP de LOC**, e o corte é por RESPONSABILIDADE: o
//! [`super::paint_optional_top20`] estava a `591` de `600` e cada moldura custa ~`45` linhas ⇒ duas
//! lá dentro estouravam-no. ⛔ **Curado por CORTE, nunca por uma entrada no `FILE_OVERAGE_OK`** —
//! aquela lista está VAZIA.
//!
//! ⚠️ **As duas moram aqui e NUNCA aparecem juntas na tela**: a da câmera pede um `CameraShake` e a
//! do emissor um `ShakeEmitter`, que vivem em objectos diferentes. *Elas são um assunto, não uma
//! sequência.*

use super::paint_frame::{begin_section, finish_section};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// **A secção CAMERA SHAKE** — moldura e tudo.
///
/// ⚠️ **Sem `selected`, ao contrário da irmã:** ela não é uma lista — uma câmera treme de **uma**
/// maneira.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_shake_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    info: Option<&ph2d_editor_core::shake_edits::InspectorShakeInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER o componente** — ADR-0166.
    let Some(info) = info else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_SHAKE_SECTION,
        header_h,
    );
    let new_y = crate::sections::shake::paint_shake_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        info,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_SHAKE_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção SHAKE EMITTER** — moldura e tudo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_shake_emitter_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    info: Option<&ph2d_editor_core::shake_edits::InspectorEmitterInfo>,
    selected: usize,
) -> f32 {
    let Some(info) = info else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_EMITTER_SECTION,
        header_h,
    );
    let new_y = crate::sections::shake_emitter::paint_shake_emitter_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        info,
        selected.min(info.rows.len().saturating_sub(1)),
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_EMITTER_SECTION,
        y_before,
        new_y,
        &[],
    )
}
