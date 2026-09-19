//! **As molduras das duas secções dos SUPLENTES** — o RAY SENSOR (#21) e a ARMA.
//!
//! ⚠️ **O corte foi imposto pelo tecto de LOC do painel** (o [`super::paint_optional_top20`] chegou
//! a `616` contra `600` ao ganhar a arma) **e é o certo por RESPONSABILIDADE**: aquele ficheiro
//! declara-se, no próprio cabeçalho, como *«a CAUDA da fila do TOP-20»*, e estas duas não são dela.
//! ⛔ *A cura de um tecto é o CORTE, nunca uma entrada nova no `FILE_OVERAGE_OK`.*
//!
//! ⚠️ **As duas juntas e não uma por ficheiro:** elas têm a mesma forma exacta (uma moldura, um
//! `Option` que decide se a secção existe, e a chamada ao pintor), e separá-las faria a terceira
//! ter de escolher entre dois sítios idênticos.

use super::paint_frame::{begin_section, finish_section};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// **A secção RAY SENSOR** — moldura e tudo (suplente #21).
///
/// ⚠️ Ela mora neste ficheiro e não no [`super::paint_optional`] pela razão das vizinhas: o
/// orquestrador está no tecto de LOC, e as molduras de secção opcional cabem melhor juntas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_ray_section(
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
    info: Option<&ph2d_editor_core::ray_edits::InspectorRayInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER o raio** — ADR-0166.
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
        ids::INSP_LIVE_RAY_SECTION,
        header_h,
    );
    let new_y = crate::sections::ray::paint_ray_section(
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
        ids::INSP_LIVE_RAY_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção WEAPON** — moldura e tudo. ⚠️ Sem estado de painel: um objecto tem UMA arma, então não
/// há linha aberta a lembrar.
///
/// ⚠️ Ela mora neste ficheiro e não no [`super::paint_optional`] pela razão das vizinhas: o
/// orquestrador está no tecto de LOC, e as molduras de secção opcional cabem melhor juntas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_weapon_section(
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
    info: Option<&ph2d_editor_core::weapon_edits::InspectorWeaponInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER a arma** — ADR-0166.
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
        ids::INSP_LIVE_WEAPON_SECTION,
        header_h,
    );
    let new_y = crate::sections::weapon::paint_weapon_section(
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
        ids::INSP_LIVE_WEAPON_SECTION,
        y_before,
        new_y,
        &[],
    )
}
