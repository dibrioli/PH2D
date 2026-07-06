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

mod adjust_menu;
mod blend;
mod brush_fallback;
mod dropdown_popover;
mod event;
mod event_brush_forward;
pub mod ids;
mod number_field;
mod paint;
mod paint_adjust;
mod paint_brush;
mod paint_brush_top;
mod paint_clone;
mod paint_composite;
mod paint_deform;
mod paint_falloff;
mod paint_inpaint;
mod paint_mask;
mod paint_mask_row;
mod paint_ramp_widget;
mod paint_rows;
mod paint_selection;
mod paint_shape;
mod paint_shape_dab;
mod paint_shape_layers;
mod paint_shape_ramp;
mod paint_stencil;
mod paint_stroke;
mod paint_symmetry;
mod paint_texture;
mod paint_texture_ramp;
mod paint_watercolor;
mod populate;
mod populate_brush_chips;
mod populate_deform;
pub mod state;
mod state_dropdowns;
mod state_ramp;

pub use state::{
    FalloffHit, PainterLayersPanelState, falloff_canvas_norm, falloff_hit_test, last_content_h,
    last_visible_h, selected_falloff_point, set_current_brush,
    set_current_brush_shape_color_preview, set_current_brush_shape_image,
    set_current_brush_texture_image, set_current_dock_shows_layers, set_current_layers,
    set_current_mask_grayscale_view, set_current_selection, set_selected_falloff_point,
};

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
