//! **A moldura da secção MOTION INPUT** (ciclo 19 do Motion) — o molde do
//! [`super::paint_optional_nav`]: um `Option` decide se a secção existe (ADR-0166).

use crate::plano::{Plano, emoldurada};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::motion_input_edits::InspectorMotionInputInfo;

/// Empurra a secção para o [`Plano`], se o objecto tiver o componente.
pub(crate) fn push_motion_input_section<'a>(
    plano: &mut Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    info: Option<&'a InspectorMotionInputInfo>,
) {
    let Some(info) = info else {
        return;
    };
    emoldurada(
        plano,
        ids::INSP_LIVE_MOTION_INPUT_SECTION,
        store,
        inner_x,
        inner_w,
        header_h,
        move |c, t, y| {
            crate::sections::motion_input::paint_motion_input_section(
                c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, info,
            )
        },
    );
}
