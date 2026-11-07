//! As linhas do cartão Animation abaixo da lista (`crate::cards_anim`): Playback, ritmo, More e Who Picks.

use super::*;
use crate::plano::Tela;
use ph2d_editor_core::screens::hero::InspectorAnimInfo;
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
    y = super::anim_rows::ritmo_e_ciclo(
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
    let sec = super::anim_rows::seccao_da_animacao(t.text);
    super::anim_card::paint_chooser(t.scene, t.text, theme, t.hit, store, x, w, y, info, sec)
}
