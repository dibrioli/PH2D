//! **`ph2d-panel-pixel-export`** — as janelas de Export e Import do Pixel Art (P9b, ADR-0196), em
//! cartões: Export As, Export Sprite Sheet com a prévia ao vivo e Import Sprite Sheet com a grelha
//! sobre a imagem. Abrem pelo menu File.
//!
//! O painel não vê a família: o que ela sabe chega pelo [`canal`], e o verbo de cada janela volta por
//! ele como um [`canal::Pedido`]. Quem escolhe o caminho e grava é a família.

#![forbid(unsafe_code)]

pub mod canal;
mod cards_export;
mod cards_folha;
mod cards_import;
mod cards_import_como;
mod cards_tileset;
mod comum;
mod event;
pub mod ids;
mod paint;
mod state;
pub mod textos;

pub use canal::{Contexto, Imagem, Pedido, Tela};
pub use state::PixelExportPanelState;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelExportPanel;

impl Panel for PixelExportPanel {
    type State = PixelExportPanelState;

    const ID: &'static str = ph2d_editor_core::screens::hero::PANEL_PIXEL_EXPORT;
    const NODE_ID: NodeId = ph2d_editor_core::ids::PIXEL_EXPORT_PANEL;
    /// Nasce FECHADA: o menu File abre-a.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_export.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Save;
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    const CAN_FLOAT: bool = true;

    fn paint(state: &mut PixelExportPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PixelExportPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        event::populate(store);
    }
}
