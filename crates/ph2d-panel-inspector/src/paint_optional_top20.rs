//! **A CAUDA da cadeia de secções opcionais** — as quatro que a fila do TOP-20 trouxe: o cérebro
//! (#15), o script (#16), o emissor de partículas (#18) e as tags (#9).
//!
//! ⚠️ **O corte foi imposto pelo tecto de FUNÇÃO do painel** (a `paint_optional_sections` chegou a
//! `214` contra `200` ao ganhar o emissor) **e é o certo por RESPONSABILIDADE**: estas são as que
//! chegam por wave, e a próxima entra num sítio só. ⛔ *A cura de um tecto é o CORTE, nunca uma
//! entrada nova no `FN_OVERAGE_OK`.*
//!
//! ⚠️ **As quatro num SACO e não em quatro argumentos** — a função já leva onze, e o tecto de
//! parâmetros do clippy é um aviso a dizer que uma lista destas se lê por posição, que é onde dois
//! instantâneos do mesmo tipo trocam de sítio em silêncio.

use super::paint_frame::{begin_section, finish_section};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

// ⭐ Os instantâneos vivem no irmão, pelo tecto de LOC — ver o cabeçalho dele.
pub(crate) use super::paint_optional_top20_infos::{Selecoes, Top20};

/// **A secção HUD** — moldura e tudo (TOP-20 #20). ⚠️ Sem estado de painel: um objecto tem UM de
/// cada componente do HUD, então não há linha aberta a lembrar.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_hud_section(
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
    info: Option<&ph2d_editor_core::hud_edits::InspectorHudInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER um dos quatro** — ADR-0166.
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
        ids::INSP_LIVE_HUD_SECTION,
        header_h,
    );
    let new_y = crate::sections::hud::paint_hud_section(
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
        ids::INSP_LIVE_HUD_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção SEQUENCE** — moldura e tudo (TOP-20 #19, W3). ⚠️ Sem estado de painel: um objecto
/// toca UMA cutscene, então não há linha aberta a lembrar.
/// A VIGIA DO CONTADOR — moldura e tudo. Irmã da [`paint_sequence_section`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_counter_watch_section(
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
    info: Option<&ph2d_editor_core::counter_watch_edits::InspectorCounterWatchInfo>,
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
        ids::INSP_LIVE_WATCH_SECTION,
        header_h,
    );
    let new_y = crate::sections::counter_watch::paint_counter_watch_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        info,
        selected,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_WATCH_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// O GATILHO — moldura e tudo. Irmão da [`paint_counter_watch_section`], e vizinho dela na ordem
/// de propósito: as duas fazem um SINAL nascer, uma de um número e a outra de uma tecla.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_action_trigger_section(
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
    info: Option<&ph2d_editor_core::action_trigger_edits::InspectorActionTriggerInfo>,
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
        ids::INSP_LIVE_TRIGGER_SECTION,
        header_h,
    );
    let new_y = crate::sections::action_trigger::paint_action_trigger_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        info,
        selected,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_TRIGGER_SECTION,
        y_before,
        new_y,
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sequence_section(
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
    info: Option<&ph2d_editor_core::sequence_edits::InspectorSequenceInfo>,
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
        ids::INSP_LIVE_SEQ_SECTION,
        header_h,
    );
    let new_y = crate::sections::sequence::paint_sequence_section(
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
        ids::INSP_LIVE_SEQ_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção PARTICLES** — moldura e tudo (TOP-20 #18, W3). ⚠️ Sem estado de painel: um objecto
/// tem UM emissor, então não há linha aberta a lembrar (a lei da câmera e da fábrica).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_particles_section(
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
    info: Option<&ph2d_editor_core::particles_edits::InspectorParticlesInfo>,
) -> f32 {
    // ⚠️ **A secção só existe se o objecto TIVER o emissor** — ADR-0166.
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
        ids::INSP_LIVE_PARTICLES_SECTION,
        header_h,
    );
    let new_y = crate::sections::particles::paint_particles_section(
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
        ids::INSP_LIVE_PARTICLES_SECTION,
        y_before,
        new_y,
        &[],
    )
}
