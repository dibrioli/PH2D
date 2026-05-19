//! ToolManifest — declarative description of an editor tool.
//!
//! Per `docs/Migracao/2026-05-convention-by-discovery.md` §4.4. Each
//! tool-crate (`crates/ph2d-tool-<slug>/`) exports one `MANIFEST`
//! const of this type plus a `pub fn register(reg: &mut Registry)`
//! that hands it to the registry. The registry then derives chrome
//! (TopBar/LeftRail), icon table, NodeId table, and dispatch from
//! the collected manifests.
//!
//! PR 1 — Foundation. Manifest shape only; no consumers yet. The
//! shadow-mode wiring lands in PR 4 (make-square pilot).

use ph2d_core::MemoryBudget;
use ph2d_vector::BezPath as VectorBezPath;

use crate::Zone;

/// Stable string id for a tool. Hashed to `NodeId` via
/// [`crate::node_id`] (PR 2). Example: `"make_square"`,
/// `"grid_snap"`, `"bgremoval"`.
pub type ToolId = &'static str;

/// Cluster id within a [`Zone`]. Example: `"image_tools"` (TopBar),
/// `"selection_tools"` (LeftRail). Validated at registry build time
/// against the canonical cluster table per [ADR-0023].
///
/// [ADR-0023]: docs/architecture/decisions/0023-ui-ux-baseline.md
pub type ClusterId = &'static str;

/// Translation key per HR-15. Example: `"tool.make_square.label"`.
/// CI lint (PR 3) validates presence in `pt-BR.ftl` + `en-US.ftl`
/// once Fluent bundles land.
pub type LabelKey = &'static str;

/// Accessibility role per HR-12. Re-export from `accesskit::Role` via
/// `ph2d-a11y::Role`.
pub type Role = ph2d_a11y::Role;

/// Vello `BezPath` — the icon glyph in a 24×24 design space.
/// Re-exported via `ph2d-vector` per SKILL §5 (avoid version skew with
/// vello's bundled kurbo).
pub type BezPath = VectorBezPath;

/// MCP exposure flags per HR-8 / HR-11. **Reserved in PR 1**; real
/// wiring (auto-expose to `ph2d-mcp::tools`, audit log emission,
/// confirmation-token gating for destructive ops) is a separate
/// sub-project — see `docs/Migracao/2026-05-convention-by-discovery.md`
/// §3.1 row HR-10 / HR-11.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct McpExposure {
    /// When `true`, this tool is automatically exposed as an MCP tool.
    /// Default: `false` until the MCP sub-project lands.
    pub exposed: bool,
    /// When `true`, mutations require confirmation-token per HR-11.
    pub destructive: bool,
    /// When `true`, all parameters are restricted to opaque handles
    /// (`Entity`, `Handle<T>`) per HR-8. Default: `true`.
    pub handle_only: bool,
}

impl McpExposure {
    pub const fn reserved() -> Self {
        Self {
            exposed: false,
            destructive: false,
            handle_only: true,
        }
    }
}

/// Discriminator: Action one-shot vs Tool stateful.
/// "Action one-shot": click → fn runs.
/// "Tool stateful": persistent state + Procreate-style panel.
///
/// PR 1 declares the variants and stores function pointers; concrete
/// `ToolCtx` / `PanelEvent` types arrive with the dispatcher in PR 9.
#[derive(Copy, Clone)]
pub enum ToolHandler {
    /// One-shot click handler. Used for actions like Trim Transparency,
    /// Make Square, Re-import.
    OneShot {
        /// Click handler. Receives a context giving access to the
        /// active selection, sim/present worlds, history, asset DB.
        on_click: HandlerFn,
    },
    /// Stateful tool with a Procreate-style floating panel.
    Stateful {
        on_activate: HandlerFn,
        on_deactivate: HandlerFn,
        on_panel_event: HandlerFn,
    },
}

/// Type-erased function pointer for handlers. Concrete signature lands
/// with the dispatcher in PR 9; PR 1 keeps it opaque so the manifest
/// shape can be stabilized without committing to `ToolCtx` details.
///
/// **Invariant:** never points to a closure capturing heap data; must
/// be a `fn` item or `'static` fn ptr to keep handlers trivially
/// `Copy` and HR-3 compliant (no allocation per dispatch).
pub type HandlerFn = fn();

/// The canonical declarative description of a tool.
#[derive(Copy, Clone)]
pub struct ToolManifest {
    /// Stable string id. Hashed to `NodeId`. Must be unique across all
    /// registered tools — registry init panics on collision (HR-5
    /// determinism + clear error message per PR 2).
    pub id: ToolId,

    /// Translation key resolving to the human label. HR-15.
    pub label_key: LabelKey,

    /// Icon glyph. Function pointer returns a fresh `BezPath` per
    /// call (cheap — paths are small kurbo curves). Lives in the tool
    /// crate; no central icon registry per `convention-by-discovery`
    /// §4.4 (Agente 1's refinement).
    pub icon_fn: fn() -> BezPath,

    /// Which canvas zone hosts the tool's chrome (TopBar, LeftRail,
    /// Sidebar, Center overlay) per ADR-0023.
    pub zone: Zone,

    /// Cluster within the zone (e.g. `"image_tools"`). Validated against
    /// canonical cluster table at registry build time.
    pub cluster: ClusterId,

    /// Sort order within `(zone, cluster)`. Determinism: registry sorts
    /// by `(cluster, order, id)` so cross-platform ordering is stable
    /// (HR-5).
    pub order: u32,

    /// Accessibility role. HR-12. Most tools are `Role::Button`;
    /// stateful tools with toggle semantics may use `Role::ToggleButton`.
    pub a11y_role: Role,

    /// Click / panel-event dispatch.
    pub handler: ToolHandler,

    /// Per-subsystem budget the tool needs while active. Registry
    /// aggregates and refuses boot if sum > platform max (HR-13).
    pub memory_budget: MemoryBudget,

    /// `true` if the tool mutates `SimWorld`. Used by CI lint to enforce
    /// HR-5 ordering rules + ADR-0021 boundary. Most tools are
    /// `false` (PresentWorld-only chrome).
    pub touches_sim: bool,

    /// MCP exposure flags. Reserved in PR 1; wired in MCP sub-project.
    pub mcp: McpExposure,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::Role;

    fn dummy_icon() -> BezPath {
        VectorBezPath::new()
    }
    fn dummy_handler() {}

    /// Smoke: a manifest can be constructed in const context. This is
    /// the contract every tool-crate relies on (its `MANIFEST` is a
    /// `pub const`). We assert on a field to keep the const live so
    /// the fn-pointers above aren't flagged dead_code.
    #[test]
    fn manifest_is_const_constructible() {
        const M: ToolManifest = ToolManifest {
            id: "_test",
            label_key: "tool._test.label",
            icon_fn: dummy_icon,
            zone: Zone::TopRight,
            cluster: "_test_cluster",
            order: 0,
            a11y_role: Role::Button,
            handler: ToolHandler::OneShot {
                on_click: dummy_handler,
            },
            memory_budget: MemoryBudget::new(0, 0, 0),
            touches_sim: false,
            mcp: McpExposure::reserved(),
        };
        assert_eq!(M.id, "_test");
        assert_eq!(M.cluster, "_test_cluster");
    }

    #[test]
    fn mcp_exposure_default_is_handle_only_and_unexposed() {
        let r = McpExposure::reserved();
        assert!(!r.exposed, "reserved manifests must not auto-expose");
        assert!(!r.destructive);
        assert!(r.handle_only, "handle-only is the safe default (HR-8)");
    }
}
