//! `ph2d-panel-painter-layers` — typed `Panel<State>` para o painel de
//! camadas do Painter (ADR-0029 + W3.T3.4 plan §6, design 02_layers.md).
//!
//! Right-docked (Inspector geometry slot, mirror do `painter_sidebar`);
//! visível só quando o `painter` tool é ativo (shell drives
//! `panel_visible("painter_layers")` from the active-tool id, em
//! `render_loop/painter_bridge.rs`). Lista as layers do `LayerStack`:
//! thumb + name + visibility + opacity slider + blend dropdown.
//!
//! ## SCAFFOLD (Coordenador, caminho B)
//!
//! Este crate entrega: chrome canon (surface, corner dot, title "Layers",
//! close, drag/resize handles), o snapshot publish (`set_current_layers`),
//! a docagem (layout slot + visibility), o registro no panel-registry-init,
//! e um body **placeholder** ("No layers"). O Implementador preenche as
//! layer rows reais — ver `// TODO(impl W3.T3.4)` em `paint.rs`.
//!
//! ## Param canon location
//!
//! O `LayerStack` canônico vive no `PainterTool` (shell-side ToolRegistry).
//! Cada frame o shell publica um snapshot via [`set_current_layers`] →
//! `paint` lê. Edits sairão via `EditorAction::ToolPanelEvent` (ADR-0040
//! TG-B canal genérico) → shell chama `PainterTool::handle_panel_event`.

#![forbid(unsafe_code)]

mod blend;
mod event;
pub mod ids;
mod paint;
mod populate;
pub mod state;

pub use state::{PainterLayersPanelState, last_content_h, last_visible_h, set_current_layers};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker que implementa o contrato typed Painter Layers panel.
pub struct PainterLayersPanel;

impl Panel for PainterLayersPanel {
    type State = PainterLayersPanelState;

    const ID: &'static str = "painter_layers";
    const NODE_ID: NodeId = ph2d_editor_core::ids::PAINTER_LAYERS_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut PainterLayersPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PainterLayersPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
