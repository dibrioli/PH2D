//! **As molduras das secções HEALTH e DAMAGE** (plano 28, W3).
//!
//! ⚠️ **Um ficheiro próprio e não o dos suplentes**: aquele declara-se «as duas secções dos
//! SUPLENTES», e estas duas são de outra família. ⭐ Elas têm a forma exacta das vizinhas (uma
//! moldura, um `Option` que decide se a secção existe, a chamada ao pintor) — com uma diferença: um
//! objecto pode ter AS DUAS (um inimigo que também magoa), e aí elas pintam-se em sequência.

use super::paint_frame::{begin_section, finish_section};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::vida_edits::InspectorVidaInfo;
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// **As secções HEALTH e DAMAGE** — moldura e tudo. Devolve o `y` seguinte.
///
/// ⚠️ **Cada secção só existe se o objecto TIVER o componente dela** — ADR-0166.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_vida_sections(
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
    info: Option<&InspectorVidaInfo>,
) -> f32 {
    let Some(info) = info else {
        return y;
    };
    if let Some(h) = &info.health {
        y = close_section(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_HEALTH_SECTION,
            header_h,
        );
        let new_y = crate::sections::vida::paint_health_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            info,
            h,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_HEALTH_SECTION,
            y_before,
            new_y,
            &[],
        );
    }
    if let Some(d) = &info.damage {
        y = close_section(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_DAMAGE_SECTION,
            header_h,
        );
        let new_y = crate::sections::vida_dano::paint_damage_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            info,
            d,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_DAMAGE_SECTION,
            y_before,
            new_y,
            &[],
        );
    }
    y
}
