//! **`ph2d-panel-pixel-timeline`** — a LINHA DO TEMPO do Pixel Art (P4, ADR-0192), `Panel<State>`
//! tipado (ADR-0029), no idioma do Aseprite 1.3: camadas em linhas × frames em colunas, na banda de
//! baixo.
//!
//! A barra de cima (navegar, tocar, papel vegetal, o menu Frame), a coluna das camadas (a mesma
//! linha que o painel PIXEL pinta, [`ph2d_editor_core::pixel_layer_row`]), as tags por cima dos
//! números dos frames (com a duração de cada um), a grelha dos cels (ligados = uma pílula) e as
//! propriedades do frame, do cel e da tag activos.
//!
//! O painel não vê a família: o instantâneo chega pelo vocabulário de
//! [`ph2d_editor_core::pixel_panel`], publicado pela shell ([`set_current_pixel`]), e cada gesto sai
//! como [`EditorAction::Pixel`](ph2d_editor_core::action_bus::EditorAction::Pixel).

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod paint_grid;
mod paint_props;
mod state;

pub use state::{PixelTimelinePanelState, current_pixel, set_current_pixel};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelTimelinePanel;

impl Panel for PixelTimelinePanel {
    type State = PixelTimelinePanelState;

    const ID: &'static str = ph2d_editor_core::screens::hero::PANEL_PIXEL_TIMELINE;
    const NODE_ID: NodeId = ph2d_editor_core::ids::PIXEL_TIMELINE_PANEL;
    /// Nasce FECHADO: a família abre-o ao entrar em Pixel Sprite ▸ Draw.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_timeline.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Film;
    /// Uma grelha de frames é larga e baixa: só a banda de baixo a exprime.
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::BOTTOM;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::Bottom;

    fn paint(state: &mut PixelTimelinePanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PixelTimelinePanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        event::populate(store);
    }
}
