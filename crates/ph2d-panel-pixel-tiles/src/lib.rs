//! **`ph2d-panel-pixel-tiles`**: a tela dos tilemaps do Pixel Art (`docs/PixelArt/P10_plano.md` §5).
//!
//! O painel não vê a família: o que ela sabe chega pelo [`canal`], e cada gesto volta por ele como um
//! [`canal::Pedido`]. Quem escreve no documento é a família, com as leis de `ph2d-pixel-tiles`.

#![forbid(unsafe_code)]

pub mod canal;
mod cards;
mod event;
pub mod familia;
pub mod ids;
mod paint;
mod state;

pub use state::PixelTilesPanelState;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelTilesPanel;

impl Panel for PixelTilesPanel {
    type State = PixelTilesPanelState;

    const ID: &'static str = ph2d_editor_core::screens::hero::PANEL_PIXEL_TILES;
    const NODE_ID: NodeId = ph2d_editor_core::ids::PIXEL_TILES_PANEL;
    /// Nasce FECHADO: abre e fecha com a camada de tilemap activa.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_tiles.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::LayoutGrid;
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    /// ⭐ FLUTUA ao lado do painel PIXEL: como aba ficaria atrás dele; em meia coluna, o Tileset nem cabia.
    const CAN_FLOAT: bool = true;

    fn paint(state: &mut PixelTilesPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PixelTilesPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        event::populate(store);
    }
}
