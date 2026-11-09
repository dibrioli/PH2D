//! **Os controlos do «Outline»** no cartão Shape (W4, plano `f2_contorno_e_formas_plano.md`): Detail,
//! Alpha cutoff, Ignore specks under, e o aviso de quando o tecto de peças subiu o Detail.

use super::physics_cartao::Tinta;
use super::rows::num_row_unit;
use super::*;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_editor_core::widget::Unit;
use ph2d_i18n::{tr, tr_with};

/// As três linhas do traço, cada uma com o ponto de chave das outras linhas do cartão.
#[allow(clippy::too_many_arguments)]
pub(super) fn controlos(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let mut y = y;
    for (rotulo, id, unidade) in [
        (
            "card.insp.phys.outline_detail",
            ids::INSP_PHYS_OUTLINE_DETAIL,
            Some(Unit::Px),
        ),
        (
            "card.insp.phys.outline_alpha",
            ids::INSP_PHYS_OUTLINE_ALPHA,
            None,
        ),
        (
            "card.insp.phys.outline_specks",
            ids::INSP_PHYS_OUTLINE_SPECKS,
            Some(Unit::Px),
        ),
    ] {
        y = num_row_unit(
            scene,
            text,
            theme,
            hit,
            store,
            x,
            w,
            y,
            tr(rotulo),
            id,
            unidade,
            None,
        );
    }
    y
}

/// «Too detailed — Detail raised to {x} px to stay under {n} pieces», só quando o tecto subiu.
pub(super) fn aviso(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let Some((usado, n)) = info.outline.and_then(|o| o.raised) else {
        return y;
    };
    let x = ph2d_editor_core::interaction::format_number(f64::from(usado));
    let frase = tr_with("card.insp.phys.outline_raised", &[("x", &x), ("n", &n)]);
    t.frase(y, &frase, ColorToken::Warn)
}
