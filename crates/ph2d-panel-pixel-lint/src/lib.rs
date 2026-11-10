//! **`ph2d-panel-pixel-lint`**: a janela LINT do Pixel Art (`docs/PixelArt/P13_plano.md` §6, U-Lint).
//! Lista os avisos do `ph2d-pixel-lint` por regra (nome, contagem, severidade); cada regra liga e
//! desliga; Color limit. Clicar num aviso marca as células e pede à família que centre o canvas.

#![forbid(unsafe_code)]

pub mod canal;
mod event;
pub mod ids;
mod paint;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

pub use paint::chave_da_regra;
/// O `Panel::ID` (`ph2d-editor-core::screens::hero::panel_ids`; nasce FECHADO).
pub use ph2d_editor_core::screens::hero::PANEL_PIXEL_LINT;

/// O estado da janela: nada além do store e do canal.
#[derive(Default)]
pub struct PixelLintPanelState;

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelLintPanel;

impl Panel for PixelLintPanel {
    type State = PixelLintPanelState;

    const ID: &'static str = PANEL_PIXEL_LINT;
    const NODE_ID: NodeId = ids::PANEL;
    /// Nasce FECHADO: abre por Window ▸ Lint.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_lint.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Bug;
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    const CAN_FLOAT: bool = true;

    fn paint(_: &mut PixelLintPanelState, ctx: &mut PaintCtx) {
        paint::paint(ctx);
    }

    fn apply_event(
        _: &mut PixelLintPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        if ph2d_editor_core::panel::cards::borda::fechou(host, &ev, Self::NODE_ID, Self::ID) {
            return EventOutcome::Consumed;
        }
        event::apply_event(host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        event::populate(store);
    }
}
