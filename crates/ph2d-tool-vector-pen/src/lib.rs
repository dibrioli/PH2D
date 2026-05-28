#![forbid(unsafe_code)]

//! `ph2d-tool-vector-pen` — Pen tool (Bézier cúbico default visible), the
//! first interactive tool of the Vector Module.
//!
//! Per W1.T1.5 in [`docs/Vector Module/17_plano_de_implementacao.md`](../../../docs/Vector%20Module/17_plano_de_implementacao.md)
//! and ADR-0040 (tool-as-isolated-feature-crate) + ADR-0056 §2.4
//! (RepresentationMode::Cubic = default visible per decisão D Antigravity).
//!
//! ## Smoke W1 Day-7 contract
//!
//! Click 1 adds vertex 0. Click 2 adds vertex 1 plus segment 0→1.
//! Click 3 adds vertex 2 plus segment 1→2. Click 4 NEAR vertex 0
//! (within `close_path_tolerance_px`) triggers close-path: adds the
//! final segment 2→0, wraps the loop in a region, assigns the seeded
//! `default_fill`, and emits a committed [`Ph2dVectorAsset`] via
//! [`VectorPenTool::take_committed_asset`]. The shell bridge (T1.7 —
//! `vector_pen_bridge.rs`) consumes the committed asset, saves it to
//! disk, and resets the tool for the next path.
//!
//! ## Anti-padrões evitados
//!
//! - **Sem `EditorAction` variant novo** (per DIRETRIZ §3.A.2 + ADR-0040
//!   T-close). Pointer events flow via shell downcast of `Tool::as_any_mut`
//!   to [`VectorPenTool::on_canvas_click`] — exception explicitly allowed
//!   by ADR-0040 §3, mirror of Painter T1.5 `PainterTool::queue_pointer`.
//! - **Sem mutate central crate** — ph2d-editor-core/`icons.rs` ganha
//!   apenas o variant `VectorPen` (alphabetical insert; gate
//!   `enum_order_matches_svgs` mantém ordem).
//! - **R4 ergonomic helpers consumidos** (commit `0e246b7`) —
//!   `VectorNetwork::next_*_id`, `StyleTable::insert_fill`,
//!   `EditLog::push_and_apply`, `VectorNetwork::nearest_vertex`,
//!   `Ph2dVectorAsset::from_network`. Zero reimplementação de id
//!   allocation, fill seeding, op apply, hit-test, asset wrapping.
//!
//! ## Shell wiring status (W1)
//!
//! The manifest declares `cluster: "vector_tools"`. T1.7 wires the shell
//! to render that cluster; until then this crate ships as **registered
//! but pill-hidden** — `cargo test -p ph2d-tool-registry-init` still
//! passes (the registry-init codegen picks up `register` + `make`), but
//! the user-visible pill appears only after T1.7 lands the cluster
//! renderer.

pub mod icon;
pub mod tool;

pub use tool::VectorPenTool;

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, Registry, ToolHandler, ToolManifest, Zone};

/// Construct the Vector Pen tool as a boxed trait object for the behavior
/// `ToolRegistry`. Codegen target for `ph2d-tool-sync` (`register_all_tools`).
pub fn make() -> Box<dyn ph2d_editor_core::tool::Tool> {
    Box::new(VectorPenTool::default())
}

/// Placeholder handler para o slot `ToolHandler::Stateful` exigido pelo
/// `ToolManifest`. **Nunca executa** — semântica real vive em
/// `impl Tool for VectorPenTool` (tool.rs) via `ToolRegistry` dispatch.
/// Mirror Painter / BgRemoval — vestígio estrutural pre-ADR-0040.
fn noop_manifest_handler() {}

/// Tool manifest registered with the chrome `ToolRegistry`.
///
/// Cluster `vector_tools`: novo cluster do Vector Module. Shell rendering
/// é T1.7 work — manifest fica registered sem pill visível até lá. Order
/// `10`: primeiro da família (Pencil será `20`, Shape `30`, Select `40`
/// etc. — espaço para inserts entre ferramentas).
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "vector_pen",
    label_key: "tool.vector_pen.label",
    icon_fn: icon::vector_pen_bezpath,
    zone: Zone::TopRight,
    cluster: "vector_tools",
    order: 10,
    a11y_role: Role::Button,
    handler: ToolHandler::Stateful {
        on_activate: noop_manifest_handler as HandlerFn,
        on_deactivate: noop_manifest_handler as HandlerFn,
        on_panel_event: noop_manifest_handler as HandlerFn,
    },
    // In-progress network holds <= 32 vertices (SmallVec inline) +
    // <= 64 segments + <= 8 regions per ADR-0056 §2.2 SmallVec budget.
    // Typical Pen session ~< 1 KB live. Reserve 1 MB safety margin
    // against HR-13 boot check for outlier (large hand-trace before
    // close-path commits).
    memory_budget: MemoryBudget::new(0, 1, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

/// Register the Pen tool with a manifest `Registry`. Codegen target
/// for `ph2d-tool-sync` (`register_all`).
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
        assert_eq!(reg.manifests()[0].id, "vector_pen");
        assert_eq!(reg.manifests()[0].cluster, "vector_tools");
    }

    #[test]
    fn manifest_id_matches_label_key_shape() {
        // Per HR-15 + i18n key gate: label_key = `tool.<id>.label`
        // literal. id uses snake_case to match existing compound tools
        // (color_equalization, equalize_sizes). icon_slug uses hyphen
        // (`vector-pen`) per SVG/Lucide naming convention — same split
        // as bgremoval (id "bgremoval" / icon_slug "bg-removal").
        assert_eq!(MANIFEST.id, "vector_pen");
        assert_eq!(MANIFEST.label_key, "tool.vector_pen.label");
    }

    #[test]
    fn make_constructs_boxed_tool_with_pen_id() {
        let tool = make();
        assert_eq!(tool.id(), ToolId::new("vector_pen"));
    }

    #[test]
    fn pen_tool_is_not_the_editor_default() {
        // Brush proto-tool (editor-core) remains default; Pen is opt-in.
        let tool = make();
        assert!(!tool.is_default());
    }

    #[test]
    fn icon_returns_non_empty_path() {
        let path = icon::vector_pen_bezpath();
        assert!(
            !path.elements().is_empty(),
            "icon BezPath should have at least one element"
        );
    }
}
