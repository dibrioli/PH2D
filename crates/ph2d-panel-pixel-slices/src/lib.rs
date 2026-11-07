//! **`ph2d-panel-pixel-slices`**: a janela SLICES do Pixel Art (`docs/PixelArt/P11_plano.md` §5).
//!
//! O painel não vê a família: o que ela sabe chega pelo [`canal`], e cada gesto volta por ele como um
//! [`canal::Pedido`]. As leis são de `ph2d-pixel-slices`; quem grava no documento é a família.

#![forbid(unsafe_code)]

pub mod canal;
mod cards;
mod event;
pub mod familia;
pub mod ids;
mod paint;
mod state;

pub use state::PixelSlicesPanelState;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelSlicesPanel;

impl Panel for PixelSlicesPanel {
    type State = PixelSlicesPanelState;

    const ID: &'static str = ph2d_editor_core::screens::hero::PANEL_PIXEL_SLICES;
    const NODE_ID: NodeId = ph2d_editor_core::ids::PIXEL_SLICES_PANEL;
    /// Nasce FECHADO: abre e fecha com a Slice Tool na mão.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_slices.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::PixelJumble;
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    /// ⭐ FLUTUA ao lado do painel PIXEL, como a janela Tilemap.
    const CAN_FLOAT: bool = true;

    fn paint(state: &mut PixelSlicesPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PixelSlicesPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        event::populate(store);
    }
}
