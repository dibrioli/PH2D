//! `ph2d-panel-painter-sidebar` — typed `Panel<State>` para o Painter
//! sidebar (ADR-0029 + W2.T2.1 plan §5).
//!
//! Right-docked (Inspector geometry slot); visível só quando `painter`
//! tool é ativo (shell drives `panel_visible("painter_sidebar")` from
//! active-tool id). Controles:
//! - Size slider (top, normalizado 0..1 → `size_px` 1..2048 via chip)
//! - Opacity slider (bottom, normalizado 0..1 → `%`)
//! - Modifier square (centro, eyedropper-while-held) — **T2.4**, ainda sem paint
//! - Undo/Redo buttons (2/3-finger tap) — **T2.2** (replay engine), ainda sem paint
//!
//! **Live preview renderiza no canvas** (shell swap canvas overlay
//! via `painter_bridge::take_preview_arc`), não dentro deste panel.
//!
//! ## Param canon location
//!
//! `PainterTool` (shell-side ToolRegistry, unreachable from HeroScreen)
//! mantém canon de params. Cada frame o shell publica `PainterUiSnapshot`
//! via [`set_current_painter_snapshot`] → `paint` lê. Edits saem via
//! `EditorAction::ToolPanelEvent` (ADR-0040 TG-B canal genérico) → shell
//! chama `PainterTool::handle_panel_event` que despacha via `PainterUiEdit`.

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod populate;
pub mod state;

pub use state::{
    PainterSidebarPanelState, last_content_h, last_visible_h, set_current_painter_snapshot,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker que implementa o contrato typed Painter Sidebar.
pub struct PainterSidebarPanel;

impl Panel for PainterSidebarPanel {
    type State = PainterSidebarPanelState;

    const ID: &'static str = "painter_sidebar";
    const NODE_ID: NodeId = ph2d_editor_core::ids::PAINTER_SIDEBAR_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut PainterSidebarPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PainterSidebarPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
