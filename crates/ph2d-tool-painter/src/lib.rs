#![forbid(unsafe_code)]

//! ph2d-tool-painter — Painter (sucessor do Procreate), isolated tool crate.
//!
//! Stateful workhorse tool — active in the TopBar `image_tools` cluster.
//! Takeover model when activated (suprime chrome PH2D normal e instala
//! topbar+sidebar Procreate-style; vide [ADR-0043 §1.1](../../../docs/architecture/decisions/0043-painter-contract.md)).
//!
//! Cascata W0 ratificada em 2026-05-26 (11 ADRs: 0043..0053). Esta crate
//! implementa o contrato ADR-0043 (`PainterTool` + `PainterParams` +
//! `PainterUiEdit` + `PainterUiSnapshot`). Sub-sistemas adjacentes vivem em
//! crates filhos:
//! - `ph2d-painter-brush` — Brush/Stamp/atlas/Mixbox/ProceduralGrain (ADR-0044)
//! - `ph2d-painter-stroke` — StrokeHistory + WAL + recovery (ADR-0046+0052)
//! - `ph2d-painter-input` — PointerSource + curves + palm rejection (ADR-0050)
//! - `ph2d-painter-mcp` — MCP streaming (ADR-0047)
//! - `ph2d-panel-painter-*` — sidebar / layers / brush-studio / inspector / color
//!
//! W1 T1.1 status: **skeleton stub.** `Tool` impl com painc-com-`todo!()`
//! nos métodos que exigem brush engine real (W1 T1.4+). Smoke W1 day 1:
//! abrir app sem panic. Pill ainda não aparece (T1.2 implementa).

pub mod compositor;
pub mod icon;
pub mod layers;
pub mod params;
pub mod tool;
pub mod undo;

pub use compositor::{
    LayerImage, LayerPixelSource, MapPixelSource, Region, composite, composite_region,
};
pub use layers::{
    GroupLayer, HARD_CAP_LAYERS, Layer, LayerId, LayerKind, LayerModifiers, LayerStack,
    MAX_GROUP_DEPTH, MaskLayer, RasterLayer,
};
pub use params::PainterParams;
// Re-export the effects surface so the layers panel can name adjustment params /
// blend modes without a direct `ph2d-painter-effects` import.
pub use ph2d_painter_effects::adjustments::{
    AdjustmentKind, AdjustmentLayer, AdjustmentParams, CurvesParams, HsbParams, SELCOLOR_BUCKETS,
    adjustment_segment_params, adjustment_slider_params, adjustment_toggle_params,
    channel_mixer_slider_params, colorbalance_display_luts, curve_value_at, curves_display_luts,
    gradient_map_lut, gradient_stop_color_params, selective_color_slider_params,
    set_adjustment_segment_param, set_adjustment_slider_param, set_adjustment_toggle_param,
    set_channel_mixer_param, set_selective_color_param,
};
pub use ph2d_painter_effects::{BlendMode, MAX_BLEND_MODES};
// Re-export the brush blend + falloff surface so the layers panel's Brush
// section can name + enumerate them without a direct `ph2d-painter-brush` import.
pub use ph2d_painter_brush::{
    BrushBlend, Falloff, FalloffCurve, FalloffPoint, HandleType, JitterUnit, MAX_BRUSH_BLEND_MODES,
    MAX_FALLOFF, MAX_FALLOFF_POINTS, MAX_HANDLE_TYPES, StrokeMethod, eval_falloff_curve,
};
pub use tool::{
    BRUSH_COUNT_SLIDER_MAX, BRUSH_JITTER_ABS_MAX_PX, BRUSH_SIZE_MAX_PX, BRUSH_SIZE_MIN_PX,
    BRUSH_SMOOTH_FACTOR_MAX, BRUSH_SMOOTH_FACTOR_MIN, BRUSH_SMOOTH_RADIUS_MAX_PX,
    BRUSH_SMOOTH_RADIUS_MIN_PX, BRUSH_SPACING_MAX, BrushSettings, PainterTool,
    brush_falloff_weight_at, set_pending_select_mods,
};
pub use undo::{DEFAULT_MAX_DEPTH, UndoController};

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Construct the Painter tool as a boxed trait object for the behavior
/// `ToolRegistry`. Codegen target for `ph2d-tool-sync` (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(PainterTool::default())
}

/// Placeholder handler para o slot `ToolHandler::Stateful` exigido pelo
/// `ToolManifest`. **Nunca executa** — a semântica real de activate/
/// deactivate/panel_event vive em `impl Tool for PainterTool` (tool.rs)
/// via `ToolRegistry` dispatch. Esses handlers do manifest são vestígio
/// estrutural pre-ADR-0040 (quando o registry de manifests também era
/// behavior registry); hoje são no-op puro. Idem em outros tools
/// (bgremoval/padding/etc.).
fn noop_manifest_handler() {}

pub const MANIFEST: ToolManifest = ToolManifest {
    id: "painter",
    label_key: "tool.painter.label",
    icon_fn: icon::painter_bezpath,
    // `image_tools` cluster — pill na TopBar lado da Image Tools toggle.
    // Order 130: ÚLTIMO do cluster (rightmost real), depois de upscale (120).
    // Painter é workhorse stateful, sucessor do Procreate; ocupa o slot
    // rightmost como signature feature. Audit 2026-05-26 (C-1) corrigiu
    // collision prévia com real_size=70.
    zone: Zone::TopRight,
    cluster: "image_tools",
    order: 130,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: noop_manifest_handler as HandlerFn,
        on_deactivate: noop_manifest_handler as HandlerFn,
        on_panel_event: noop_manifest_handler as HandlerFn,
    },
    // Worst-case working buffer: 4K canvas RGBA8 layer texture
    // (~64 MB). Reserved against ram_mb so HR-13 boot check fires
    // se plataforma com budget apertado tentar Painter ativo.
    memory_budget: MemoryBudget::new(0, 64, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

pub fn register(reg: &mut Registry) {
    reg.register(&MANIFEST);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_attaches_manifest_to_registry() {
        let mut reg = Registry::default();
        register(&mut reg);
        assert_eq!(reg.manifests().len(), 1);
        assert_eq!(reg.manifests()[0].id, "painter");
    }

    #[test]
    fn make_constructs_boxed_tool() {
        let tool = make();
        assert_eq!(
            tool.id(),
            ph2d_editor_core::floating_panel::ToolId::new("painter")
        );
    }

    #[test]
    fn manifest_matches_iconid_slug() {
        // MANIFEST.id deve bater com IconId::Painter.slug() ("painter").
        assert_eq!(MANIFEST.id, "painter");
    }
}
