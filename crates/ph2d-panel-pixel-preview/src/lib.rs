//! **`ph2d-panel-pixel-preview`** — a janela PREVIEW do Pixel Art (P5, ADR-0196), `Panel<State>`
//! tipado (ADR-0029), no idioma do Aseprite 1.3.18: um painel FLUTUANTE (arrasta-se pelo título,
//! `×` fecha) com o quadro activo do sprite em Draw, zoom próprio pela roda e um tocar só dela
//! (Shift+Enter, ou o botão do título).
//!
//! O painel não vê a família: a imagem chega pelo [`canal`], publicada por `ph2d-app-pixel`, e o
//! botão do título sai como [`EditorAction::Pixel`](ph2d_editor_core::action_bus::EditorAction::Pixel).

#![forbid(unsafe_code)]

pub mod canal;
pub mod crt;
mod event;
pub mod ids;
mod paint;
mod state;

pub use canal::{PreviewFrame, PreviewPlay, current, publish};
pub use state::PixelPreviewPanelState;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelPreviewPanel;

impl Panel for PixelPreviewPanel {
    type State = PixelPreviewPanelState;

    const ID: &'static str = ph2d_editor_core::screens::hero::PANEL_PIXEL_PREVIEW;
    const NODE_ID: NodeId = ph2d_editor_core::ids::PIXEL_PREVIEW_PANEL;
    /// Nasce FECHADA: Shift+Enter ou F7 abrem-na.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_preview.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Play;
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    /// ⭐ FLUTUA: a Preview do Aseprite é uma janela solta por cima do editor.
    const CAN_FLOAT: bool = true;

    fn paint(state: &mut PixelPreviewPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PixelPreviewPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        event::populate(store);
    }
}
