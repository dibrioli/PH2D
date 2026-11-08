//! **`ph2d-panel-pixel-history`**: a janela UNDO HISTORY do Pixel Art (`docs/PixelArt/P12_plano.md`
//! §4.1 H, Aseprite O11–O14). O painel lê uma [`canal::HistoryView`] e devolve
//! [`canal::HistoryRequest`]; quem desfaz é a família (`History::go_to`).

#![forbid(unsafe_code)]

pub mod canal;
mod event;
pub mod ids;
mod paint;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// O `Panel::ID`. ⚠️ Falta a entrada em `ph2d-editor-core::screens::hero::panel_ids` (nasce FECHADO).
pub const PANEL_PIXEL_HISTORY: &str = "pixel_history";

/// O estado da janela: nada além do store.
#[derive(Default)]
pub struct PixelHistoryPanelState;

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelHistoryPanel;

impl Panel for PixelHistoryPanel {
    type State = PixelHistoryPanelState;

    const ID: &'static str = PANEL_PIXEL_HISTORY;
    const NODE_ID: NodeId = ids::PANEL;
    /// Nasce FECHADO: abre por Edit ▸ Undo History.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_history.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::History;
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    const CAN_FLOAT: bool = true;

    fn paint(_: &mut PixelHistoryPanelState, ctx: &mut PaintCtx) {
        paint::paint(ctx);
    }

    fn apply_event(
        _: &mut PixelHistoryPanelState,
        _: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(ev)
    }

    fn populate(store: &mut WidgetStore) {
        event::populate(store);
    }
}
