//! `MotionState` — the shell-owned runtime aggregate for the Motion Nodes module
//! (Motion Nodes M0.T8). Held on `AppGfx.motion`; driven per frame by
//! `render_loop::motion_bridge` while the `motion` tool is active.
//!
//! Bundles the persistable document ([`MotionDoc`], with undo [`MotionHistory`])
//! with the runtime pieces that never persist: the **persistent** [`Cook`] (its
//! memo + `pre` feedback must survive across frames), the node [`NodeRegistry`]
//! (the `OpResolver`), the current sink node, and a reused `Vec<RenderInstance>`
//! lowering buffer (so the steady-state cook path is zero-alloc — gated by M0.T12).
//!
//! **No transport (W4.T7).** Motion used to keep a `MotionTransport` of its own
//! here, advanced by each frame's fixed steps, while the timeline ran
//! `ph2d_core::Playhead` — two clocks that each advanced themselves, and so two
//! clocks that could drift. The editor now has ONE: the bridge DERIVES the tick it
//! cooks from the playhead (`motion_bridge::motion_tick`), and the pump's own
//! `last_cooked_tick` is the only record of where the sim stands. Do not add a
//! tick back here.
//!
//! Document ≠ tool (ADR-0040): the `MotionTool` is a thin activation handle; all
//! the state lives here in the shell, mirroring `AppGfx.vec_scene`.

#[path = "motion_demo_strobe.rs"]
mod strobe;

use ph2d_eval_motion::MotionCookPump;
use ph2d_motion_doc::{MotionDoc, MotionHistory};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::format::ParseError;
use ph2d_nodegraph::graph::{Graph, NodeId};

/// Runtime state for the Motion Nodes editor. One instance on `AppGfx`.
pub(crate) struct MotionState {
    /// The persistable document (the graph is the only part that cooks).
    pub(crate) doc: MotionDoc,
    /// Snapshot undo/redo of `doc`. The graph-edit intents push onto it (M1
    /// Phase 1b: connect / disconnect / add / delete / drag), and Ctrl+Z/Y drive
    /// [`MotionHistory::undo`]/[`redo`] from the shell (Phase 1b-3).
    pub(crate) history: MotionHistory,
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
    /// `size` fallback for instances whose stream carries no `size` column.
    ///
    /// **It is the IDENTITY, and it may not be anything else** (`SIZE_IDENTITY`):
    /// every node that materializes `size` builds it from unit scale, so a
    /// fallback that disagrees makes those nodes resize the scene just by being
    /// dropped (doc 39 — it was `0.4`, so a `motion.scale` at `amount = 1` scaled
    /// every quad by 2.5×). A document that wants small quads says so, with a
    /// `motion.scale` — it does not get it from a number hidden in the shell.
    pub(crate) default_size: [f32; 2],
    /// F2 probe: the node whose output the editor is reading, and the ring of its
    /// most recent readings (oldest first). UI-only — it never touches the cook.
    pub(crate) probe: Option<NodeId>,
    pub(crate) probe_ring: Vec<f32>,
    /// F3 flow: last frame's digest of each cooked node's output, keyed by `NodeId.0`. A node
    /// whose digest MOVED has data running down its wires, and the panel marches them
    /// (TouchDesigner's animated wire). UI-only — the cook never reads it.
    pub(crate) flow_digest: std::collections::BTreeMap<u32, u64>,
}

impl MotionState {
    /// Build the boot state: register every node op + the **default document** — driven by
    /// `motion.expression` formulas: on the left a **spiral** whose x/y are cos/sin
    /// expressions plotted through `motion.make_point`; on the right a grid whose **colour
    /// wave** is an expression fed to a ramp. Both formulas read `t`, so they animate.
    /// Kept deliberately small (docs/Motion Nodes/12, 32). The earlier scenes (the
    /// Cavalry grid rig, the sim scenes, the deformer scenes) and the earlier
    /// value/pulse + M3/M4 chains were removed to keep the boot document focused; they
    /// live in git history and every node keeps its own unit tests + stays registered
    /// (drop them in the editor). Transport paused at tick 0 (bridge auto-plays).
    pub(crate) fn new() -> Self {
        let mut registry = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut registry)
            .expect("motion node registry builds");
        let mut doc = MotionDoc::new();
        let sinks = build_default_document(&mut doc.graph, &registry).unwrap_or_default();
        Self {
            doc,
            history: MotionHistory::new(),
            pump: MotionCookPump::new(),
            registry,
            sinks,
            // Whole-atlas until the shell wires a real tile (init.rs). Headless
            // callers / tests keep this default.
            default_uv_rect: [0.0, 0.0, 1.0, 1.0],
            // The SAME unit scale every node assumes when it materializes `size`.
            default_size: ph2d_nodegraph::attr::SIZE_IDENTITY,
            probe: None,
            probe_ring: Vec::new(),
            flow_digest: std::collections::BTreeMap::new(),
        }
    }

    /// **Install a saved document** (the project's Ctrl+O path) — parse the canonical text
    /// and replace the current one, runtime and all.
    pub(crate) fn load_text(&mut self, text: &str) -> Result<(), ParseError> {
        let doc = MotionDoc::from_text(text)?;
        self.install(doc);
        Ok(())
    }

    /// Adopt `doc`, **discarding every runtime trace of the one before it**.
    ///
    /// The document is the only thing a project stores; everything else here is derived. But
    /// "derived" is not the same as "harmless", because the runtime is keyed by NODE ID — and
    /// node ids are small integers that the next document reuses for entirely different nodes:
    ///
    /// - the **`Cook` is the simulation's living state**, not a cache — it holds the flakes
    ///   that are in the air. The pump is therefore replaced OUTRIGHT, not merely
    ///   `mark_dirty`'d (which invalidates the scrub cache but keeps the memo and the `pre`
    ///   feedback). A fresh pump says what it means.
    /// - the **clock** goes back to 0 — but that is now the CALLER's job, not ours (W4.T7):
    ///   Motion keeps no transport of its own, and the editor's ONE `Playhead` is not a
    ///   field we own. `App::project_load` rewinds it after a successful load. The reason
    ///   stands: a playhead at t=40s into a graph that has never been cooked is not a
    ///   resumption, it is a lie about a simulation that never ran.
    /// - **undo** belongs to the document that was edited, not to the file that replaced it.
    /// - the **probe**, the **flow digests** and the panel's **selection** all name nodes by
    ///   id. A stale selection is the sharpest of the three: the params panel would happily
    ///   edit whichever node inherited the number.
    ///
    /// `sinks` is the exception that proves the rule — the bridge recomputes it from the graph
    /// every frame, so it heals itself; it is cleared anyway so a headless caller between the
    /// load and the first pump never reads the old graph's outputs.
    fn install(&mut self, doc: MotionDoc) {
        self.doc = doc;
        self.pump = MotionCookPump::new();
        self.history = MotionHistory::new();
        self.sinks.clear();
        self.probe = None;
        self.probe_ring.clear();
        self.flow_digest.clear();
        #[cfg(feature = "panel-motion-graph")]
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    }
}

/// Author the **default document** into `g`: two small M4 simulation scenes (the
/// module's most recent work). Returns their sinks (the Output nodes) if the graph
/// is well-typed.
///
/// The scenes — built in the `strobe` sibling module — are formula-driven: on the LEFT a
/// **spiral** (`motion.expression` cos/sin formulas → `motion.make_point`, 144 points that
/// rotate); on the RIGHT a grid coloured by a scrolling **expression wave** (`sin(t·2 +
/// f·a)` fed to a ramp's `t`). The formulas live in the graph's text channel (doc 32).
/// See docs/Motion Nodes/12 (value), 32 (expression).
///
/// The earlier scenes were removed to keep the boot document small and legible: the
/// **Cavalry grid rig**, the sim scenes, the deformer scenes, and the earlier value,
/// pulse and M3/M4 chains. They remain in git history and every node keeps its own unit
/// tests; the nodes stay registered, so any of them can be dropped.
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
