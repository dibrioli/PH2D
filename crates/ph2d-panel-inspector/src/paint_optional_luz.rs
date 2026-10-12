//! **As molduras dos cartões da LUZ** (Luz 2D) — a lâmpada (W4) e o CORPO na luz («Light
//! Occluder» · «Unlit»): irmão do [`super::paint_optional_shake`]. Cada cartão só existe se o objeto
//! TIVER o componente dele (ADR-0166); os três passam pela MESMA moldura.

use super::paint_frame::{begin_section, finish_section};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::luz_edits::{InspectorCorpoDaLuzInfo, InspectorLuzInfo};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Onde um cartão se pinta: a coluna, o `y` e a banda do cabeçalho.
#[derive(Clone, Copy)]
pub(crate) struct Lugar {
    pub inner_x: f32,
    pub inner_w: f32,
    pub y: f32,
    pub header_h: f32,
}

/// A moldura partilhada: fecha a secção anterior, regista o cabeçalho e a pega, pinta o `corpo` e
/// fecha o contorno.
#[allow(clippy::too_many_arguments)]
fn moldura(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    theme: ph2d_tokens::Theme,
    l: Lugar,
    section: NodeId,
    corpo: impl FnOnce(&mut VectorScene, &mut TextSystem, &mut HitIndex, f32) -> f32,
) -> f32 {
    let y = close_section(scene, theme, l.inner_x, l.inner_w, l.y);
    begin_section(hit_index, l.inner_x, l.inner_w, y, section, l.header_h);
    let new_y = corpo(scene, text_system, hit_index, y);
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        l.inner_x,
        l.inner_w,
        section,
        y,
        new_y,
    )
}

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
    y: f32,
    header_h: f32,
    info: Option<&InspectorLuzInfo>,
) -> f32 {
    let Some(info) = info else {
        return y;
    };
    let l = Lugar {
        inner_x,
        inner_w,
        y,
        header_h,
    };
    moldura(
        scene,
        text_system,
        hit_index,
        store,
        theme,
        l,
        ids::INSP_LIVE_LUZ_SECTION,
        |scene, text, hit, y| {
            crate::sections::luz::paint_luz_section(
                scene, text, theme, hit, store, inner_x, inner_w, y, info,
            )
        },
    )
}

/// **O cartão LIGHT OCCLUDER** — só existe se o objeto TIVER a `BloqueiaLuz`.
pub(crate) fn paint_oclusor_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    l: Lugar,
    info: Option<&InspectorCorpoDaLuzInfo>,
) -> f32 {
    if !info.is_some_and(|i| i.oclusor.is_some()) {
        return l.y;
    }
    moldura(
        scene,
        text_system,
        hit_index,
        store,
        theme,
        l,
        ids::INSP_LIVE_LUZ_OCLUSOR_SECTION,
        |scene, text, hit, y| {
            crate::sections::luz_corpo::paint_oclusor(
                scene,
                text,
                theme,
                hit,
                store,
                (l.inner_x, l.inner_w, y),
            )
        },
    )
}

/// **O cartão UNLIT** — só existe se o objeto TIVER a `NaoRecebeLuz`.
pub(crate) fn paint_sem_luz_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    l: Lugar,
    info: Option<&InspectorCorpoDaLuzInfo>,
) -> f32 {
    if !info.is_some_and(|i| i.sem_luz) {
        return l.y;
    }
    moldura(
        scene,
        text_system,
        hit_index,
        store,
        theme,
        l,
        ids::INSP_LIVE_LUZ_SEM_LUZ_SECTION,
        |scene, text, hit, y| {
            crate::sections::luz_corpo::paint_sem_luz(
                scene,
                text,
                theme,
                hit,
                store,
                (l.inner_x, l.inner_w, y),
            )
        },
    )
}
