//! **A célula de VISTA do onion/speed na barra de transporte** — o que a tela MOSTRA, não
//! comando de documento nem de autoria: Speed (velocidade no graph editor, W5), Onion on/off,
//! Onion Keys/Frames (ADR-0142 W3) e a **engrenagem** que abre o card de settings (W3b — um
//! botão, cujo Click a shell resolve). Extraídos de `transport::paint_item` (cap de LOC de fn
//! e de arquivo); é um módulo IRMÃO do `widgets`, e usa o mesmo `toggle`/`icon_button`.

use super::widgets::{icon_button, toggle};
use super::{BarView, Item};
use crate::ids;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_timeline::{OnionMode, TimelineViewSnapshot};
use ph2d_tokens::Theme;

/// Pinta a célula de vista `item` (Speed · Onion · OnionMode · OnionSettings). Os toggles leem do
/// snapshot como todo toggle da barra (o switch pintado não pode discordar do que o passe desenha);
/// a engrenagem é um `icon_button` cujo Click abre o card no `hero.store` da shell.
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
        // The gear opens the settings card. A plain button (not a view toggle): its Click reaches
        // the shell, which owns the `hero.store` the card lives in.
        Item::OnionSettings => icon_button(
            ctx,
            theme,
            x,
            y,
            ids::TIMELINE_ONION_SETTINGS,
            IconId::Settings,
        ),
        _ => 0.0,
    };
}
