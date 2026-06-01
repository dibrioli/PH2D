#![forbid(unsafe_code)]

//! `ph2d-tool-vector-pencil` — Pencil tool (freehand → Hobby auto-smooth),
//! the second interactive tool of the Vector Module.
//!
//! Per plan **T2.1** (`docs/Vector Module/17_plano_de_implementacao.md` §5)
//! and ADR-0040 (tool-as-isolated-feature-crate). The Pencil records a
//! freehand stroke and, on pointer-up, smooths it into an interpolating
//! cubic Bézier spline via **Hobby's algorithm**
//! ([`ph2d_vector_doc::hobby`]) — explicitly NOT Schneider's least-squares
//! fit (the plan anti-pattern).
//!
//! ## Smoke W2 Day-4 contract
//!
//! Press → drag a freehand curve → release. The recorded samples decimate
//! to knots (≈ 1 per 10 samples), Hobby fits a smooth open spline through
//! them, and the tool emits a committed open **stroked** [`Ph2dVectorAsset`]
//! via [`VectorPencilTool::take_committed_asset`]. The shell bridge
//! (`vector_pencil_bridge.rs`) renders the live preview while dragging and
//! the committed smoothed path after release.
//!
//! ## Anti-padrões evitados
//!
//! - **Sem `EditorAction` variant novo** (DIRETRIZ §3.A.2 + ADR-0040): the
//!   pointer events flow via the shell downcast of [`Tool::as_any_mut`] to
//!   the inherent `begin_stroke` / `extend_stroke` / `finish_stroke`
//!   methods (ADR-0040 §3 exception, mirror of the Pen tool).
//! - **Sem mutate central crate** além do `IconId::VectorPencil` variant
//!   (alphabetical insert; gate `enum_order_matches_svgs` keeps order).
//! - **Renderer:** a Pencil path is an OPEN stroked path. The canonical
//!   `ph2d_vector::draw_vector_network` is fill-only today; committed
//!   stroke rendering is wired in the bridge pending a canonical stroke
//!   pass (Coord item — see the W2 handoff report).
//!
//! [`Ph2dVectorAsset`]: ph2d_vector_doc::Ph2dVectorAsset
//! [`Tool::as_any_mut`]: ph2d_editor_core::tool::Tool::as_any_mut

pub mod icon;
pub mod tool;

pub use tool::{
    DEFAULT_DECIMATION_STRIDE, DEFAULT_MIN_SAMPLE_DISTANCE_PX, DEFAULT_STROKE_WIDTH_PX,
    MAX_STROKE_SAMPLES, PencilStrokeOutcome, StrokeSample, VectorPencilTool,
};

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Construct the Vector Pencil tool as a boxed trait object for the
/// behavior `ToolRegistry`. Codegen target for `ph2d-tool-sync`
/// (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(VectorPencilTool::default())
}

/// Placeholder handler for the `ToolHandler::Stateful` slot the
/// `ToolManifest` requires. **Never executes** — the real semantics live
/// in `impl Tool for VectorPencilTool` (tool.rs) via `ToolRegistry`
/// dispatch. Mirror of the Pen / Painter / BgRemoval vestigial handler.
fn noop_manifest_handler() {}

/// Tool manifest registered with the chrome `ToolRegistry`.
///
/// Cluster `vector_tools` (shared with the Pen). Order `20`: second of the
/// family (Pen `10`, Shape `30`, Select `40` per the plan §5 fan-out).
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "vector_pencil",
    label_key: "tool.vector_pencil.label",
    icon_fn: icon::vector_pencil_bezpath,
    zone: Zone::TopRight,
    cluster: "vector_tools",
    order: 20,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: noop_manifest_handler as HandlerFn,
        on_deactivate: noop_manifest_handler as HandlerFn,
        on_panel_event: noop_manifest_handler as HandlerFn,
    },
    // A live stroke holds raw samples (capped at MAX_STROKE_SAMPLES =
    // 16 k × ~16 B ≈ 256 KB worst case). Reserve 1 MB against the HR-13
    // boot check for an outlier long trace before commit.
    memory_budget: MemoryBudget::new(0, 1, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Register the Pencil tool with a manifest `Registry`. Codegen target for
/// `ph2d-tool-sync` (`register_all`).
pub fn register(reg: &mut Registry) {
    reg.register(&MANIFEST);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::floating_panel::ToolId;

    #[test]
    fn register_attaches_manifest_to_registry() {
        let mut reg = Registry::default();
        register(&mut reg);
        assert_eq!(reg.manifests().len(), 1);
        assert_eq!(reg.manifests()[0].id, "vector_pencil");
        assert_eq!(reg.manifests()[0].cluster, "vector_tools");
    }

    #[test]
    fn manifest_id_matches_label_key_shape() {
        // HR-15 + i18n key gate: label_key = `tool.<id>.label`.
        assert_eq!(MANIFEST.id, "vector_pencil");
        assert_eq!(MANIFEST.label_key, "tool.vector_pencil.label");
    }

    #[test]
    fn pencil_order_follows_pen() {
        // Pen is 10; Pencil is 20 — leaves room for inserts between the
        // family members.
        assert_eq!(MANIFEST.order, 20);
    }

    #[test]
    fn make_constructs_boxed_tool_with_pencil_id() {
        let tool = make();
        assert_eq!(tool.id(), ToolId::new("vector_pencil"));
    }

    #[test]
    fn pencil_tool_is_not_the_editor_default() {
        let tool = make();
        assert!(!tool.is_default());
    }

    #[test]
    fn icon_returns_non_empty_path() {
        let path = icon::vector_pencil_bezpath();
        assert!(!path.elements().is_empty());
    }
}
