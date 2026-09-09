//! **As TRÊS seções com ESTADO DE PAINEL** — a §11 Animation, a §12 Sockets/Anchors e a TIMERS.
//!
//! ⚠️ **Irmão de [`super::paint_frame_shared`] por CAP de FICHEIRO** (600): a TIMERS levou-o a 658,
//! e o corte por responsabilidade estava à mão porque a família já estava reunida lá dentro. *Um
//! cap de ficheiro e um cap de função medem grandezas diferentes*, e extrair para o mesmo ficheiro
//! curaria um e estouraria o outro — a lição que o par de PRECISAO pagou em 2026-08-20.
//!
//! ⚠️ **Elas andam juntas por uma PROPRIEDADE, não por vizinhança:** são as únicas do Inspector
//! cuja pintura depende de qual LINHA está aberta — um facto que vive no `InspectorState` e que
//! nenhuma outra seção conhece. As restantes leem só o snapshot.
//!
//! ⚠️ **O `paint_anchor_section` FICA no irmão**, e não é inconsistência: ele é chamado daqui, e
//! trazê-lo devolveria o ficheiro-pai a 458 e este a 274 — dois ficheiros a meio do caminho em vez
//! de um corte por responsabilidade. A §12 é a mais antiga das três e vive onde nasceu.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, NoteData, WidgetStore};
use ph2d_editor_core::widget::section_cards::close_section;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

use super::paint_frame::{begin_section, finish_section};
use super::paint_frame_shared::paint_anchor_section;

/// **§11 Animation** — moldura e tudo. Irmã da `paint_anchor_section`, e igual a ela na única
/// coisa que as distingue das outras: ela também precisa do **estado do painel** (qual animação
/// está aberta no editor).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_anim_section(
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
    anim: Option<&ph2d_editor_core::screens::hero::InspectorAnimInfo>,
    selected: &mut usize,
) -> f32 {
    let Some(an) = anim else {
        return y;
    };
    *selected = (*selected).min(an.rows.len().saturating_sub(1));
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_ANIM_SECTION,
        header_h,
    );
    let new_y = crate::sections::anim::paint_anim_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        an,
        *selected,
    );
    // ⚠️ **Sem slot de NOTA**, e é deliberado: os slots são uma lista posicional que as doze
    // seções partilham, e acrescentar um a meio renumeraria as notas que os artistas já colaram.
    // A §11 nasce sem nota; dar-lhe uma é um passo à parte, com a renumeração feita de uma vez.
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_ANIM_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção TIMERS** — moldura e tudo. Irmã da `paint_anim_section` na única coisa que as
/// distingue das outras: ela também precisa do **estado do painel** (qual timer está aberto).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_timer_section(
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
    timer: Option<&ph2d_editor_core::screens::hero::InspectorTimerInfo>,
    selected: &mut usize,
) -> f32 {
    let Some(tm) = timer else {
        return y;
    };
    *selected = (*selected).min(tm.rows.len().saturating_sub(1));
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_TIMER_SECTION,
        header_h,
    );
    let new_y = crate::sections::timers::paint_timer_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        tm,
        *selected,
    );
    // ⚠️ **Sem slot de NOTA**, e é a mesma decisão da §11: os slots são uma lista posicional que as
    // seções partilham, e acrescentar um a meio renumeraria as notas que os artistas já colaram.
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_TIMER_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **A secção SIGNAL ACTIONS** — moldura e tudo. Irmã da `paint_timer_section`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_action_section(
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
    action: Option<&ph2d_editor_core::screens::hero::InspectorActionInfo>,
    selected: &mut usize,
) -> f32 {
    let Some(ac) = action else {
        return y;
    };
    *selected = (*selected).min(ac.rows.len().saturating_sub(1));
    y = close_section(scene, theme, inner_x, inner_w, y);
    let y_before = y;
    begin_section(
        section_tops_y,
        hit_index,
        inner_x,
        inner_w,
        body_top_y,
        y_before,
        ids::INSP_LIVE_ACTION_SECTION,
        header_h,
    );
    let new_y = crate::sections::actions::paint_action_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        inner_x,
        inner_w,
        y,
        ac,
        *selected,
    );
    finish_section(
        scene,
        text_system,
        hit_index,
        store,
        inner_x,
        inner_w,
        ids::INSP_LIVE_ACTION_SECTION,
        y_before,
        new_y,
        &[],
    )
}

/// **As QUATRO seções que precisam do ESTADO do painel** — a §11 Animation, a §12 Sockets/Anchors e
/// a TIMERS, na ordem em que se pintam.
///
/// ⚠️ **Elas andam juntas por uma PROPRIEDADE, não por vizinhança:** são as únicas do Inspector
/// cuja pintura depende de qual LINHA está aberta — um facto que vive no `InspectorState` e que
/// nenhuma outra seção conhece. As restantes leem só o snapshot.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_stateful_sections(
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
    anim: Option<&ph2d_editor_core::screens::hero::InspectorAnimInfo>,
    anim_selected: &mut usize,
    anchor: Option<&ph2d_editor_core::screens::hero::InspectorAnchorInfo>,
    anchor_selected: &mut usize,
    timer: Option<&ph2d_editor_core::screens::hero::InspectorTimerInfo>,
    timer_selected: &mut usize,
    action: Option<&ph2d_editor_core::screens::hero::InspectorActionInfo>,
    action_selected: &mut usize,
    notes: &[Vec<(usize, NoteData)>],
) -> f32 {
    y = paint_anim_section(
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
        anim,
        anim_selected,
    );
    y = paint_anchor_section(
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
        anchor,
        anchor_selected,
        notes,
    );
    y = paint_timer_section(
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
        timer,
        timer_selected,
    );
    paint_action_section(
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
        action,
        action_selected,
    )
}
