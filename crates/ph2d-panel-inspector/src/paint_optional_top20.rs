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

/// Os instantâneos das quatro secções da cauda, mais as duas linhas abertas do cérebro.
pub(crate) struct Top20<'a> {
    /// O CÉREBRO (TOP-20 #15).
    pub statemachine: Option<&'a ph2d_editor_core::statemachine_edits::InspectorStateMachineInfo>,
    /// O SCRIPT (TOP-20 #16).
    pub script: Option<&'a ph2d_editor_core::script_edits::InspectorScriptInfo>,
    /// O EMISSOR DE PARTÍCULAS (TOP-20 #18).
    pub particles: Option<&'a ph2d_editor_core::particles_edits::InspectorParticlesInfo>,
    /// O HUD (TOP-20 #20).
    pub hud: Option<&'a ph2d_editor_core::hud_edits::InspectorHudInfo>,
    /// A CUTSCENE (TOP-20 #19).
    pub sequence: Option<&'a ph2d_editor_core::sequence_edits::InspectorSequenceInfo>,
    /// As TAGS (TOP-20 #9).
    pub tags: Option<&'a ph2d_editor_core::screens::hero::InspectorTagsInfo>,
    /// ⚠️ **Duas selecções e não uma** — as listas de estados e de setas são independentes.
    pub sm_state_selected: usize,
    /// Idem, a das setas.
    pub sm_trans_selected: usize,
}

/// Pinta as quatro, pela ordem. ⚠️ **Cada chamada leva o `y` da anterior**: uma cujo `y` se deita
/// fora empilha a secção seguinte por cima dela.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_top20_sections(
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
    infos: Top20,
) -> f32 {
    y = crate::paint_optional_factory::paint_statemachine_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.statemachine,
        infos.sm_state_selected,
        infos.sm_trans_selected,
    );
    y = crate::paint_optional_factory::paint_script_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.script,
    );
    y = paint_particles_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.particles,
    );
    y = paint_hud_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.hud,
    );
    y = paint_sequence_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.sequence,
    );
    crate::paint_optional_factory::paint_tags_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.tags,
    )
}

/// **A secção HUD** — moldura e tudo (TOP-20 #20). ⚠️ Sem estado de painel: um objecto tem UM de
/// cada componente do HUD, então não há linha aberta a lembrar.
#[allow(clippy::too_many_arguments)]
fn paint_hud_section(
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
#[allow(clippy::too_many_arguments)]
fn paint_sequence_section(
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
fn paint_particles_section(
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
