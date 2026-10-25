//! **§14 — o bloco *Animations*** (GameEngineFuncional N1, `docs/GameEngineFuncional/05_N1_plano.md` §3):
//! que animação toca em cada estado do personagem.
//!
//! Desligado é UMA linha — o interruptor que anexa o `CharacterAnimations`. Ligado, um botão por estado
//! (cada clique passa à animação seguinte da lista do objecto), o limiar de correr e o espelho. Sem
//! animações na lista os botões não se oferecem: o bloco di-lo numa frase.

use super::super::rows::{aviso, num_row_unit, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorCharacterAnimInfo;
use ph2d_i18n::{TextKey, tr, tr_with};

/// O rótulo e o botão de um estado — o nome pela porta das linhas, o botão na coluna do valor.
#[allow(clippy::too_many_arguments)]
fn state_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: TextKey,
    id: ph2d_a11y::NodeId,
    name: &str,
    known: bool,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> (f32, Rect) {
    let row = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        label.tr(),
        seccao,
    );
    // ⭐ O losango do campo: o nome É a chave do catálogo, e o canal sai dela (a lei do `seg_row`).
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_campo(
        scene,
        theme,
        row.dot,
        &[id],
        label.into(),
    ) {
        hit_index.register(alvo, row.dot);
    }
    let text = if known {
        name.to_owned()
    } else {
        tr_with("panel.inspector.player.anim_missing", &[("name", &name)])
    };
    let btn = Button::new(id, &text)
        .kind(ButtonKind::Default)
        .visual(store.button_visual(id));
    paint_button(&btn, row.control, scene, text_system, theme);
    (y + ROW_H_PX + ph2d_tokens::control_gap_px(), row.control)
}

/// Pinta o bloco e devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_anim_block(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    anim: Option<&InspectorCharacterAnimInfo>,
) -> f32 {
    let mut yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("component.character_animations.name"),
        ids::INSP_PLAYER_ANIM,
        &ids::INSP_PLAYER_ANIM_IDS,
        &[
            tr("panel.inspector.player.off"),
            tr("panel.inspector.player.on"),
        ],
        u8::from(anim.is_some()),
    );
    let Some(a) = anim else {
        return yy;
    };
    if a.has_tags {
        let labels = [
            TextKey::new("component.field.character_animations.0"),
            TextKey::new("component.field.character_animations.1"),
            TextKey::new("component.field.character_animations.2"),
            TextKey::new("component.field.character_animations.3"),
        ];
        let sec = ph2d_editor_core::property_row::Seccao::medida(
            text_system,
            1,
            &labels.map(TextKey::tr),
        );
        let (y1, r) = state_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            labels[0],
            ids::INSP_PLAYER_ANIM_IDLE,
            &a.names[0],
            a.known[0],
            sec,
        );
        hit_index.register(ids::INSP_PLAYER_ANIM_IDLE, r);
        let (y2, r) = state_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y1,
            labels[1],
            ids::INSP_PLAYER_ANIM_RUN,
            &a.names[1],
            a.known[1],
            sec,
        );
        hit_index.register(ids::INSP_PLAYER_ANIM_RUN, r);
        let (y3, r) = state_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y2,
            labels[2],
            ids::INSP_PLAYER_ANIM_JUMP,
            &a.names[2],
            a.known[2],
            sec,
        );
        hit_index.register(ids::INSP_PLAYER_ANIM_JUMP, r);
        let (y4, r) = state_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y3,
            labels[3],
            ids::INSP_PLAYER_ANIM_FALL,
            &a.names[3],
            a.known[3],
            sec,
        );
        hit_index.register(ids::INSP_PLAYER_ANIM_FALL, r);
        yy = y4;
    } else {
        yy = aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            yy,
            tr("panel.inspector.player.anim_no_tags"),
            ColorToken::Warn,
        );
    }
    yy = num_row_unit(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        TextKey::new("component.field.character_animations.4"),
        ids::INSP_PLAYER_ANIM_RUN_FROM,
        Some(ph2d_editor_core::widget::Unit::MetersPerSecond),
        None,
    );
    seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        TextKey::new("component.field.character_animations.5"),
        ids::INSP_PLAYER_ANIM_FLIP,
        &ids::INSP_PLAYER_ANIM_FLIP_IDS,
        &[
            tr("panel.inspector.player.off"),
            tr("panel.inspector.player.on"),
        ],
        u8::from(a.flip),
    )
}
