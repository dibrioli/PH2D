//! **A moldura da secção LUZ** (Luz 2D, W4) — irmã do [`super::paint_optional_shake`].

use super::paint_frame::{begin_section, finish_section};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// **A secção LUZ** — moldura e tudo. Só existe se o objeto TIVER a `Luz` (ADR-0166).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_luz_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
    header_h: f32,
    info: Option<&ph2d_editor_core::luz_edits::InspectorLuzInfo>,
) -> f32 {
    let Some(info) = info else {
        return y;
    };
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        hit_index,
        inner_x,
        inner_w,
        y_before,
        ids::INSP_LIVE_LUZ_SECTION,
        header_h,
    );
    let new_y = crate::sections::luz::paint_luz_section(
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
        ids::INSP_LIVE_LUZ_SECTION,
        y_before,
        new_y,
    )
}
