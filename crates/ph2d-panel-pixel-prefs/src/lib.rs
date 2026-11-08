//! **`ph2d-panel-pixel-prefs`**: a janela GRID & BACKGROUND do Pixel Art (`docs/PixelArt/P12_plano.md`
//! §9.6, Aseprite O18/O19): a grade, a grade de píxel e o xadrez do documento activo ou dos novos,
//! `Set as Defaults` e `Reset`. O painel lê uma [`canal::PrefsView`] e devolve
//! [`canal::PrefsRequest`]; quem grava é a família ([`canal::troca`]).

#![forbid(unsafe_code)]

pub mod canal;
mod event;
pub use event::pedido;
pub mod ids;
mod paint;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

pub use ph2d_editor_core::screens::hero::PANEL_PIXEL_PREFS;

/// O estado da janela: o âmbito e a amostra cujo seletor está aberto.
#[derive(Default)]
pub struct PixelPrefsPanelState {
    pub scope: canal::Scope,
    pub(crate) a_escolher: Option<NodeId>,
}

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelPrefsPanel;

impl Panel for PixelPrefsPanel {
    type State = PixelPrefsPanelState;

    const ID: &'static str = PANEL_PIXEL_PREFS;
    const NODE_ID: NodeId = ids::PANEL;
    /// Nasce FECHADO: abre por Edit ▸ Grid & Background… ou pelo Run Command.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_prefs.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::PixelJumble;
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    const CAN_FLOAT: bool = true;

    fn paint(st: &mut PixelPrefsPanelState, ctx: &mut PaintCtx) {
        paint::paint(st, ctx);
    }

    fn apply_event(
        st: &mut PixelPrefsPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(st, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        event::populate(store);
    }
}
