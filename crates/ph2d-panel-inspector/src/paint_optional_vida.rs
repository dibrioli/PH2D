//! **As molduras das secções HEALTH e DAMAGE** (plano 28, W3).
//!
//! ⚠️ **Um ficheiro próprio e não o dos suplentes**: aquele declara-se «as duas secções dos
//! SUPLENTES», e estas duas são de outra família. ⭐ Elas têm a forma exacta das vizinhas (uma
//! moldura, um `Option` que decide se a secção existe, a chamada ao pintor) — com uma diferença: um
//! objecto pode ter AS DUAS (um inimigo que também magoa), e aí elas pintam-se em sequência.

use crate::plano::{Plano, emoldurada};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::vida_edits::InspectorVidaInfo;

/// **As secções HEALTH, DAMAGE e HEALTH BAR** — empurradas para o [`Plano`], cada uma com a sua
/// moldura (2026-09-29: cada uma arrasta-se sozinha).
///
/// ⚠️ **Cada secção só existe se o objecto TIVER o componente dela** — ADR-0166.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_vida_sections<'a>(
    plano: &mut Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    info: Option<&'a InspectorVidaInfo>,
) {
    let Some(info) = info else {
        return;
    };
    if let Some(h) = &info.health {
        emoldurada(
            plano,
            ids::INSP_LIVE_HEALTH_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::vida::paint_health_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, info, h,
                )
            },
        );
    }
    if let Some(d) = &info.damage {
        emoldurada(
            plano,
            ids::INSP_LIVE_DAMAGE_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::vida_dano::paint_damage_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, info, d,
                )
            },
        );
    }
    // ⭐ A terceira — HEALTH BAR (plano 28, W4): a barra do inimigo mora com a vida dele, e a do
    // placar mora num objecto que só tem a barra.
    if let Some(b) = &info.bar {
        emoldurada(
            plano,
            ids::INSP_LIVE_HEALTH_BAR_SECTION,
            store,
            inner_x,
            inner_w,
            header_h,
            move |c, t, y| {
                crate::sections::vida_barra::paint_health_bar_section(
                    c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, info, b,
                )
            },
        );
    }
}
