//! **As molduras das secções NAV REGION e NAV AGENT** (plano 30, W4) — e (W7) NAV COST AREA e NAV
//! LINK.
//!
//! ⚠️ **Um ficheiro próprio**, pelo molde do [`super::paint_optional_vida`]: uma moldura, um
//! `Option` que decide se a secção existe, a chamada ao pintor — e um objecto pode ter AS DUAS.

use crate::plano::{Plano, emoldurada};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::nav_edits::InspectorNavInfo;

/// **As secções NAV REGION e NAV AGENT** — empurradas para o [`Plano`], cada uma com a sua
/// moldura (cada uma arrasta-se sozinha, como as da VIDA).
///
/// ⚠️ **Cada secção só existe se o objecto TIVER o componente dela** — ADR-0166.
pub(crate) fn push_nav_sections<'a>(
    plano: &mut Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    info: Option<&'a InspectorNavInfo>,
) {
    let Some(info) = info else {
        return;
    };
    if let Some(r) = &info.region {
        emoldurada(
            plano,
            ids::INSP_LIVE_NAV_REGION_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::nav::paint_nav_region_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, r,
                )
            },
        );
    }
    if let Some(a) = &info.agent {
        let playing = info.clock_playing;
        emoldurada(
            plano,
            ids::INSP_LIVE_NAV_AGENT_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::nav::paint_nav_agent_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, a, playing,
                )
            },
        );
    }
    // ⭐ (W7) A ÁREA DE CUSTO e o ATALHO — o mesmo molde, cada uma com a sua moldura.
    if let Some(a) = &info.cost_area {
        emoldurada(
            plano,
            ids::INSP_LIVE_NAV_COST_AREA_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::nav_custo::paint_nav_cost_area_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, a,
                )
            },
        );
    }
    if let Some(l) = &info.link {
        emoldurada(
            plano,
            ids::INSP_LIVE_NAV_LINK_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::nav_custo::paint_nav_link_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, l,
                )
            },
        );
    }
}
