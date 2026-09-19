//! **As molduras das secções que chegam por WAVE** — a cauda do [`super::paint_optional_top20`].
//!
//! ⚠️ **Irmão por CAP de LOC**, e o corte é por RESPONSABILIDADE: o ficheiro-mãe bateu `615` contra
//! `600` ao ganhar a moldura do tween, e ele já era a cauda do `paint_optional`. ⇒ *a cauda ganhou
//! uma cauda*, que é exactamente o que o comentário de lá previa por escrito (*«a próxima entra num
//! sítio só»*).
//!
//! ⛔ **Curado por CORTE, nunca por uma entrada no `FILE_OVERAGE_OK`** — aquela lista está VAZIA.

use super::paint_frame::{begin_section, finish_section};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// **A secção TWEEN** — moldura e tudo (suplente #22).
///
/// ⚠️ **Com `selected`, ao contrário da vizinha do raio:** um `Tweens` é uma LISTA, e qual linha
/// está aberta é um facto da UI — o molde dos timers, da vigia e do gatilho.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_tween_section(
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
    info: Option<&ph2d_editor_core::tween_edits::InspectorTweenInfo>,
    selected: usize,
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
        ids::INSP_LIVE_TWEEN_SECTION,
        header_h,
    );
    let new_y = crate::sections::tween::paint_tween_section(
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
        ids::INSP_LIVE_TWEEN_SECTION,
        y_before,
        new_y,
        &[],
    )
}
