//! As linhas do cartão Animation abaixo da lista (`crate::cards_anim`): Playback, ritmo, More e Who Picks.

use super::*;
use crate::plano::Tela;
use ph2d_editor_core::screens::hero::{AnimCycleMode, InspectorAnimInfo};
use ph2d_i18n::tr;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resto(
    t: &mut Tela<'_>,
    theme: Theme,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    selected: usize,
) -> f32 {
    let mut y = super::anim::subtitle(
        t.scene,
        t.text,
        theme,
        x,
        w,
        y,
        tr("panel.inspector.anim_card.playback"),
    );
    y = super::anim_card::transport_row(t.scene, t.text, theme, t.hit, store, x, w, y, info, false);
    if info.current_dangling() {
        y = super::rows::aviso(
            t.scene,
            t.text,
            theme,
            x,
            w,
            y,
            tr("panel.inspector.animation.this_sprite_has_no_animation"),
            ColorToken::Danger,
        );
    }
    y = ritmo_e_ciclo(
        t.scene, t.text, theme, t.hit, store, x, w, y, info, selected,
    );
    y = super::anim::advanced_group(
        t.scene,
        t.text,
        theme,
        t.hit,
        store,
        x,
        w,
        y,
        info,
        selected,
        tr("panel.inspector.anim_card.more"),
        Some(tr("panel.inspector.anim_card.more_summary")),
    );
    so_quem_escolhe(t, theme, store, x, w, y, info)
}

/// Só o Who Picks e as regras: o corpo inteiro do cartão durante o Play com uma regra acesa (Crítica B3).
pub(crate) fn so_quem_escolhe(
    t: &mut Tela<'_>,
    theme: Theme,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
) -> f32 {
    let sec = super::anim_rows::seccao_da_animacao(t.text);
    super::anim_card::paint_chooser(t.scene, t.text, theme, t.hit, store, x, w, y, info, sec)
}

/// **Frames/s e Speed na mesma linha** (irmãos, lei de 04/11: o `fluir` do cartão junta os dois pedidos) e o
/// Cycle por baixo: o que o cartão Animation mostra da animação escolhida. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn ritmo_e_ciclo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    selected: usize,
) -> f32 {
    let seccao = super::anim_rows::seccao_da_animacao(text_system);
    let row = super::anim_rows::selected_row(info, selected);
    let mut cur_y = y;
    if row.is_some() {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.anim_card.fps"),
            &[ids::INSP_ANIM_FPS],
            1.0,
            None,
            seccao,
        );
    }
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ph2d_i18n::TextKey::new("component.field.sprite_animator.3"),
        &[ids::INSP_ANIM_SPEED],
        0.1, // LITERAL-PX-OK: passo de scrub em MÚLTIPLOS de velocidade, não em pixels
        None,
        seccao,
    );
    match row {
        Some(row) => cycle_rows(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            row,
            seccao,
        ),
        None => cur_y,
    }
}

/// O Cycle da animação e o aviso de que algum quadro tem tempo próprio. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn cycle_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &ph2d_editor_core::screens::hero::InspectorAnimRow,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = y;
    let mode = AnimCycleMode::of_row(row.direction_tag, row.repeat);
    let rotulos = [
        tr("panel.inspector.anim_card.cycle_repeat"),
        tr("panel.inspector.anim_card.cycle_once"),
        tr("panel.inspector.anim_card.cycle_ping_pong"),
    ];
    let segmentos: Vec<(&str, bool, NodeId)> = AnimCycleMode::ALL
        .iter()
        .zip(rotulos)
        .zip(ids::INSP_ANIM_CYCLE)
        .map(|((&m, l), id)| (l, m == mode, id))
        .collect();
    cur_y = ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.anim_card.cycle"),
        &segmentos,
        seccao,
    );
    // Uma animação com tempo PRÓPRIO em algum quadro (§8.12, dado no editor) di-lo: sem isto o ritmo
    // acima mentiria sobre ela.
    if row.has_per_frame_timing() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.anim_card.own_times"),
            ColorToken::Warn,
        );
    }
    cur_y
}
