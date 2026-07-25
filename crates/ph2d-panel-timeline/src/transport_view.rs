//! **Os toggles de VISTA da barra de transporte** — o que a tela MOSTRA, não comando de
//! documento nem de autoria: Speed (velocidade no graph editor, W5), Onion on/off e Onion
//! Keys/Frames (ADR-0142 W3). Extraídos de `transport::paint_item` (cap de LOC de fn e de
//! arquivo); é um módulo IRMÃO do `widgets`, e usa o mesmo `toggle` do transporte.

use super::widgets::toggle;
use super::{BarView, Item};
use crate::ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_timeline::{OnionMode, TimelineViewSnapshot};
use ph2d_tokens::Theme;

/// Pinta o toggle de vista `item` (Speed · Onion · OnionMode). Lê do snapshot como todo
/// toggle da barra, então o switch pintado não pode discordar do que o passe desenha.
pub(super) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    item: Item,
    snap: &TimelineViewSnapshot,
    view: BarView,
) {
    // `toggle` devolve a largura pintada; a barra já reservou a célula, então aqui ela é
    // descartada (`let _`), como os arms de toggle do `paint_item` faziam.
    let _: f32 = match item {
        Item::Speed => toggle(
            ctx,
            theme,
            x,
            y,
            ids::TIMELINE_SPEED,
            ph2d_i18n::tr("panel.timeline.speed"),
            view.speed_view,
        ),
        Item::Onion => toggle(
            ctx,
            theme,
            x,
            y,
            ids::TIMELINE_ONION,
            ph2d_i18n::tr("panel.timeline.onion"),
            snap.onion.enabled,
        ),
        Item::OnionMode => toggle(
            ctx,
            theme,
            x,
            y,
            ids::TIMELINE_ONION_MODE,
            ph2d_i18n::tr("panel.timeline.onion_keys"),
            snap.onion.mode == OnionMode::Keys,
        ),
        _ => 0.0,
    };
}
