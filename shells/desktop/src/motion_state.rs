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

#[path = "motion_demo_strobe.rs"]
mod strobe;

use ph2d_eval_motion::MotionCookPump;
use ph2d_motion_doc::{MotionDoc, MotionHistory, MotionTransport};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};

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
    /// Terminal nodes whose output streams are lowered to instances — every
    /// `motion.output` node in the document, in node-id order. Several sinks
    /// compose into one draw, so a document *can* hold independent scenes without
    /// a stream-merging node; the current boot document uses a single scene.
    /// Empty until a well-typed Output node exists.
    pub(crate) sinks: Vec<NodeId>,
    /// `atlas_uv` fallback for instances whose stream carries no `uv_rect`
    /// column (no framing node yet). Set from the composed atlas at init to a
    /// single opaque tile, so instances render as clean solid quads instead of a
    /// whole-atlas thumbnail. `[0,0,1,1]` (whole atlas) until the shell overrides.
    pub(crate) default_uv_rect: [f32; 4],
    /// `size` fallback for instances whose stream carries no `size` column. Kept
    /// **below** the default grid spacing (`1.0`) so bare instances render as
    /// distinct dots rather than a merged solid band.
    pub(crate) default_size: [f32; 2],
}

impl MotionState {
    /// Build the boot state: register every node op + the **default document** —
    /// two small M3 deformer scenes side by side: a grid billowing into
    /// **perspective** as its corners are pinned (`motion.four_point_warp`) on the
    /// left, and a grid **bulging and pinching** like a lens (`motion.spherize`) on
    /// the right. Each is animated by a `value.lfo`. Kept deliberately small (two
    /// linear chains) so each new node reads on its own (docs/Motion Nodes/12, 24).
    /// The earlier scenes (the Cavalry grid rig, the sim scenes, the distributions)
    /// and the earlier value/pulse + M3/M4 chains were removed to keep the boot
    /// document focused; they live in git history and every node keeps its own unit
    /// tests + stays registered (drop them in the editor). Transport paused at tick 0
    /// (bridge auto-plays).
    pub(crate) fn new() -> Self {
        let mut registry = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut registry)
            .expect("motion node registry builds");
        let mut doc = MotionDoc::new();
        let sinks = build_default_document(&mut doc.graph, &registry).unwrap_or_default();
        Self {
            doc,
            history: MotionHistory::new(),
            transport: MotionTransport::new(),
            pump: MotionCookPump::new(),
            registry,
            sinks,
            // Whole-atlas until the shell wires a real tile (init.rs). Headless
            // callers / tests keep this default.
            default_uv_rect: [0.0, 0.0, 1.0, 1.0],
            // Distinct dots with clear gaps for a bare generator (a scale/strobe
            // that writes a `size` column overrides this fallback).
            default_size: [0.4, 0.4],
        }
    }
}

/// Author the **default document** into `g`: two small M4 simulation scenes (the
/// module's most recent work). Returns their sinks (the Output nodes) if the graph
/// is well-typed.
///
/// The scenes — built in the `strobe` sibling module — are two side-by-side deformers
/// animated by the value domain: on the LEFT a grid whose corners are pinned into a
/// keystone, its `warp` a `value.lfo` billowing it into **perspective** and flat again
/// (`motion.four_point_warp`, a projective corner-pin); on the RIGHT a grid whose
/// `amount` is a `value.lfo` swinging from pinch to **bulge** like a lens
/// (`motion.spherize`, a radial distortion). Two distinct deformer families. See
/// docs/Motion Nodes/12 (value), 24 (four-point-warp + spherize).
///
/// The earlier scenes were removed to keep the boot document small and legible: the
/// **Cavalry grid rig**, the sim scenes, the distribution scenes, and the earlier
/// value, pulse and M3/M4 chains. They remain in git history and every node keeps its
/// own unit tests; the nodes stay registered, so any of them can be dropped.
fn build_default_document(g: &mut Graph, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let sinks = strobe::build(g)?;
    // Same "validate on load" the editor runs before cooking — proves the authored
    // graph is well-typed and membrane-clean.
    g.validate(reg).ok()?;
    Some(sinks)
}
#[cfg(test)]
#[path = "motion_state_tests.rs"]
mod tests;
