//! **As MOLDURAS das secções FACTORY e LIFECYCLE** (TOP-20 #11 e #12, W3) — irmão do
//! [`super::paint_optional`] por CAP de ficheiro de painel.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho:** estas duas são as únicas do orquestrador
//! que leem o MESMO snapshot e o filtram por metades diferentes (`factory.is_some()` /
//! `lifecycle.is_some()`), e ficam melhor uma ao lado da outra.

use super::paint_frame::{begin_section, finish_section};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// **A secção FACTORY** — moldura e tudo (TOP-20 #11, W3). ⚠️ Sem estado de painel, como a da
/// câmera: um objecto tem UMA fábrica, então não há linha aberta a lembrar.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_factory_section(
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
    info: Option<&ph2d_editor_core::screens::hero::InspectorFactoryInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER a fábrica** — ADR-0166. Um objecto com só uma
    // vida entra pelo irmão de baixo, e nenhum dos dois paga o cabeçalho do outro.
    let Some(info) = info.filter(|i| i.factory.is_some()) else {
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
        ids::INSP_LIVE_FACTORY_SECTION,
        header_h,
    );
    let new_y = crate::sections::factory::paint_factory_section(
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
        ids::INSP_LIVE_FACTORY_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção LIFECYCLE** — moldura e tudo (TOP-20 #12, W3).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_lifecycle_section(
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
    info: Option<&ph2d_editor_core::screens::hero::InspectorFactoryInfo>,
) -> f32 {
    let Some(info) = info.filter(|i| i.lifecycle.is_some()) else {
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
        ids::INSP_LIVE_LIFECYCLE_SECTION,
        header_h,
    );
    let new_y = crate::sections::factory::paint_lifecycle_section(
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
        ids::INSP_LIVE_LIFECYCLE_SECTION,
        y_before,
        new_y,
        &[],
    )
}
