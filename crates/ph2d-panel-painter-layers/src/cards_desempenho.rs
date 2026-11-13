//! O cartão **Performance** (n.º 8, dono 13/11): uma caixa de marcar debaixo do Wet Paint, desligada de fábrica — o
//! palpite do motor Wet. Só existe com o Wet Paint (nos outros meios a caixa não mudaria nada).

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, ferramenta};
use ph2d_i18n::tr;
use ph2d_tool_painter::BrushSettings;
use ph2d_tool_painter::ids as pids;

pub(crate) fn presente(brush: &BrushSettings) -> bool {
    crate::cards_wet::presente(brush)
}

pub(crate) fn titulo() -> &'static str {
    tr("card.painter.performance.title")
}

/// Com o store MUTÁVEL, antes de pintar: a dica diz o que a caixa troca.
pub(crate) fn prepare(store: &mut WidgetStore, b: &BrushSettings) {
    if presente(b) {
        store.set_tooltip(
            pids::PAINTER_PERFORMANCE_PALPITE,
            tr("card.painter.performance.guess_tip"),
        );
    }
}

pub(crate) fn corpo(c: &mut CardCtx<'_>, y: f32) -> f32 {
    ferramenta::caixa(
        c,
        pids::PAINTER_PERFORMANCE_PALPITE,
        "card.painter.performance.guess",
        ph2d_tool_painter::palpite_ligado(),
        y,
    )
}
