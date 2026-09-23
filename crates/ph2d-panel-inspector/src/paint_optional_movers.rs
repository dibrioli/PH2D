//! **As MOLDURAS das três secções da família MOVIMENTO** — irmão do
//! [`super::paint_optional_factory`] por CAP de ficheiro de painel.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho, e a família já estava DECLARADA:** a
//! tabela [`ph2d_editor_core::ids::LIVE_SECTIONS`] escreve-o por extenso — *«a 24.ª — TOP-DOWN
//! PLAYER, a primeira da família MOVIMENTO»*, *«a 25.ª — PROJECTILE MOTION, a segunda»*. O seguidor
//! de caminho (suplente #23) é a terceira, e foi ele que levou o irmão a `611` contra o tecto de
//! `600`.
//!
//! ⛔ **Curado por CORTE, nunca por uma entrada no `FILE_OVERAGE_OK`** — aquela lista está VAZIA, e
//! é isso que a torna a catraca mais apertada que existe.
//!
//! ⭐ E o corte paga-se duas vezes: o orquestrador ([`super::paint_optional`]) passa a chamar UMA
//! função em vez de três, que é o que o tira do tecto de 200 LOC por função.

use super::paint_frame::{begin_section, finish_section};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// **A secção TOP-DOWN PLAYER** — moldura e tudo (TOP-20 #13, W3).
///
/// ⚠️ Ela mora neste ficheiro e não no [`super::paint_optional`] pela mesma razão das duas acima:
/// o orquestrador está no tecto de LOC, e as molduras de secção opcional cabem melhor juntas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_topdown_section(
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
    info: Option<&ph2d_editor_core::topdown_edits::InspectorTopDownInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER o mover** — ADR-0166.
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
        ids::INSP_LIVE_TOPDOWN_SECTION,
        header_h,
    );
    let new_y = crate::sections::topdown::paint_topdown_section(
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
        ids::INSP_LIVE_TOPDOWN_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção PROJECTILE MOTION** — moldura e tudo (TOP-20 #14, W3).
///
/// ⚠️ Ela mora neste ficheiro e não no [`super::paint_optional`] pela mesma razão das duas acima:
/// o orquestrador está no tecto de LOC, e as molduras de secção opcional cabem melhor juntas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_projectile_section(
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
    info: Option<&ph2d_editor_core::projectile_edits::InspectorProjectileInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER o projéctil** — ADR-0166.
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
        ids::INSP_LIVE_PROJECTILE_SECTION,
        header_h,
    );
    let new_y = crate::sections::projectile::paint_projectile_section(
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
        ids::INSP_LIVE_PROJECTILE_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção PATH FOLLOW** — moldura e tudo (suplente #23).
///
/// ⚠️ Ela mora neste ficheiro e não no [`super::paint_optional`] pela razão das irmãs: o
/// orquestrador está no tecto de LOC, e as molduras de secção opcional cabem melhor juntas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_path_follow_section(
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
    info: Option<&ph2d_editor_core::path_follow_edits::InspectorPathFollowInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER o seguidor** — ADR-0166.
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
        ids::INSP_LIVE_PATHFOLLOW_SECTION,
        header_h,
    );
    let new_y = crate::sections::path_follow::paint_path_follow_section(
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
        ids::INSP_LIVE_PATHFOLLOW_SECTION,
        y_before,
        new_y,
        &[],
    )
}
