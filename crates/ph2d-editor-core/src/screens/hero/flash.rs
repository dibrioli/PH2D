//! **O PISCAR DA PALETA** (F4, D10) — o `Ctrl+K` que ensina: escolher um comando faz piscar o botão
//! onde ele MORA, para quem conhecia a UI antiga a achar na nova (o *Feature Finder* do Affinity).
//!
//! ⚠️ **O estado guarda o COMANDO, não o botão** ([`WidgetStore::flash`]): cada pintor pergunta se o
//! seu alvo é uma PORTA para ele — o próprio botão, o `⋯` da fila que o guarda, ou o pulldown da
//! área que o contém. Um botão escondido atrás do `⋯` não pisca no vazio.

use crate::ids;
use crate::interaction::WidgetStore;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme};

/// A intensidade do anel em `slot` agora, se `slot` é uma porta para o comando a piscar.
#[must_use]
pub fn level_at(store: &WidgetStore, steady: bool, slot: NodeId) -> Option<f32> {
    let (id, level) = store.flashing(steady)?;
    let has = |rows: &[crate::widget::ToolRailEntry]| rows.iter().any(|e| e.node_id() == Some(id));
    // O pulldown que contém o comando — e é ELE que vai para o `⋯` quando a fila não cabe.
    let pill = (0..ids::MAX_AREA_MENUS)
        .find(|&s| u8::try_from(s).is_ok_and(|s| has(store.area_menu_rows(s))))
        .map(ids::area_menu_button);
    let behind_more = store
        .tool_overflow()
        .iter()
        .any(|e| e.node_id().is_some_and(|n| n == id || Some(n) == pill));
    let home = slot == id
        || Some(slot) == pill
        || (slot == ids::TOOL_BAR_OVERFLOW && behind_more);
    home.then_some(level)
}

/// O anel sobre `rect`, pela porta da moldura do tema.
pub fn paint_ring(scene: &mut ph2d_vector::VectorScene, theme: Theme, rect: Rect, level: f32) {
    let colour = crate::paint::resolve(ColorToken::FocusRing, theme).with_alpha(level);
    crate::paint::stroke_frame(
        scene,
        rect,
        crate::paint::frame_radius(theme, Radius::Sm.px()),
        theme,
        ph2d_tokens::visuals::Feel::Active,
        StrokeToken::Heavy.px(),
        colour,
    );
}

/// O anel de `slot`, se ele é uma porta para o comando a piscar.
pub fn ring_if_home(
    scene: &mut ph2d_vector::VectorScene,
    store: &WidgetStore,
    steady: bool,
    theme: Theme,
    slot: NodeId,
    rect: Rect,
) {
    if let Some(level) = level_at(store, steady, slot) {
        paint_ring(scene, theme, rect, level);
    }
}
