//! `MotionState` — the shell-owned runtime aggregate for the Motion Nodes module
//! (Motion Nodes M0.T8). Held on `AppGfx.motion`; driven per frame by
//! `render_loop::motion_bridge` while the `motion` tool is active.
//!
//! Bundles the persistable document ([`MotionDoc`], with undo [`MotionHistory`])
//! with the runtime pieces that never persist: the [`MotionTransport`] (play /
//! pause / tick), the **persistent** [`Cook`] (its memo + `pre` feedback must
//! survive across frames), the node [`NodeRegistry`] (the `OpResolver`), the
//! current sink node, and a reused `Vec<RenderInstance>` lowering buffer (so the
//! steady-state cook path is zero-alloc — gated by M0.T12).
//!
//! Document ≠ tool (ADR-0040): the `MotionTool` is a thin activation handle; all
//! the state lives here in the shell, mirroring `AppGfx.vec_scene`.

use ph2d_eval_motion::MotionCookPump;
use ph2d_motion_doc::{MotionDoc, MotionHistory, MotionTransport};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Runtime state for the Motion Nodes editor. One instance on `AppGfx`.
pub(crate) struct MotionState {
    /// The persistable document (the graph is the only part that cooks).
    pub(crate) doc: MotionDoc,
    /// Snapshot undo/redo of `doc`. The graph-edit intents push onto it (M1
    /// Phase 1b: connect / disconnect / add / delete / drag), and Ctrl+Z/Y drive
    /// [`MotionHistory::undo`]/[`redo`] from the shell (Phase 1b-3).
    pub(crate) history: MotionHistory,
    /// Playback transport (playhead = `tick × fixed_dt`).
    pub(crate) transport: MotionTransport,
    /// Per-frame cook driver (persistent [`Cook`] + reused instance buffer). Its
    /// [`MotionCookPump::pump`] re-cooks only on a dirty frame, so a paused frame
    /// is zero-alloc (M0.T12). The rendered slice is `pump.instances`.
    pub(crate) pump: MotionCookPump,
    /// Registered node ops (the `OpResolver` the cook resolves against).
    pub(crate) registry: NodeRegistry,
    /// Terminal node whose output stream is lowered to instances (`None` until a
    /// well-typed sink exists).
    pub(crate) sink: Option<NodeId>,
    /// `atlas_uv` fallback for instances whose stream carries no `uv_rect`
    /// column (the M0 case — no framing node yet). Set from the composed atlas
    /// at init to a single opaque tile, so the raw default document renders as
    /// clean solid quads instead of a whole-atlas thumbnail. `[0,0,1,1]`
    /// (whole atlas) until the shell overrides it.
    pub(crate) default_uv_rect: [f32; 4],
    /// `size` fallback for instances whose stream carries no `size` column (the
    /// M0 case). Kept **below** the default grid spacing (`1.0`) so the raw
    /// default document renders as distinct dots rather than a merged solid band.
    pub(crate) default_size: [f32; 2],
}

impl MotionState {
    /// Build the boot state: register every node op + the default
    /// grid→transform→clone vertical (the M0 smoke graph promoted to a real
    /// document), transport paused at tick 0.
    pub(crate) fn new() -> Self {
        let mut registry = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut registry)
            .expect("motion node registry builds");
        let mut doc = MotionDoc::new();
        let sink = build_default_vertical(&mut doc.graph, &registry);
        Self {
            doc,
            history: MotionHistory::new(),
            transport: MotionTransport::new(),
            pump: MotionCookPump::new(),
            registry,
            sink,
            // Whole-atlas until the shell wires a real tile (init.rs). Headless
            // callers / tests keep this default.
            default_uv_rect: [0.0, 0.0, 1.0, 1.0],
            // Half the default grid spacing (1.0) → distinct dots with clear
            // gaps in the raw M0 output (framing/size producers land in M1).
            default_size: [0.5, 0.5],
        }
    }

    /// Current playhead in seconds for the given fixed timestep.
    pub(crate) fn playhead(&self, fixed_dt: f64) -> f64 {
        self.transport.playhead(fixed_dt)
    }
}

/// Author the default grid→transform→clone vertical into `g`; returns the sink
/// (the clone node) if the graph is well-typed. Mirror of the retired
/// `motion_smoke` graph, now the document's initial content.
fn build_default_vertical(g: &mut Graph, reg: &NodeRegistry) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let xf = g.add_node("motion.transform");
    let clone = g.add_node("motion.clone");
    g.connect(Edge {
        from: (grid, 0),
        to: (xf, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (xf, 0),
        to: (clone, 0),
        delayed: false,
    })
    .ok()?;
    // M1 — lay the default chain left-to-right in graph space so the editor
    // renders it as connected cards (the panel auto-fits until the user pans).
    g.set_pos(grid, Pos { x: 0.0, y: 0.0 });
    g.set_pos(xf, Pos { x: 260.0, y: 0.0 });
    g.set_pos(clone, Pos { x: 520.0, y: 0.0 });
    // Same "validate on load" the editor runs before cooking — proves the
    // authored graph is well-typed and membrane-clean.
    g.validate(reg).ok()?;
    Some(clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builds_a_well_typed_default_vertical_with_a_sink() {
        let state = MotionState::new();
        assert!(state.sink.is_some(), "default vertical must have a sink");
        // 3 nodes authored (grid, transform, clone).
        assert_eq!(state.doc.graph.nodes().len(), 3);
        // Well-typed against the registry (validate succeeded in `new`).
        assert!(state.doc.graph.validate(&state.registry).is_ok());
        // Paused at tick 0 → playhead 0.
        assert_eq!(state.playhead(1.0 / 60.0), 0.0);
    }
}
