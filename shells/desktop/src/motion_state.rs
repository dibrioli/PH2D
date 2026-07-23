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

/// The DEFORMER scene, its own file for the same reason: it answers "does a node
/// whose kernel needs one number about the WHOLE stream run on the device?",
/// which none of the per-element or neighbourhood scenes can.
#[path = "motion_state_gpu_demos_deform.rs"]
mod gpu_deform_demo;
#[path = "motion_state_gpu_demos.rs"]
mod gpu_demos;
/// The NEIGHBOURHOOD scenes (ADR-0140), split out for the same reason: they
/// answer the interacting-sim question none of the throughput scenes can.
#[path = "motion_state_gpu_neighbour_demos.rs"]
mod gpu_neighbour_demos;
/// The panel scene, split out at the HR-18 cap — see the module's own note on
/// why the seam is the QUESTION each scene answers, not the line count.
#[path = "motion_state_gpu_panel_demo.rs"]
mod gpu_panel_demo;
/// The Lloyd/JFA scene (ADR-0139), its own file for the same reason: it answers
/// "does a node whose cook is a multi-pass ALGORITHM run on the device?".
#[path = "motion_state_gpu_voronoi_demo.rs"]
mod gpu_voronoi_demo;
/// The sim-zone scene (ADR-0135), its own file for the same reason: it answers
/// "does the state-loop CONTAINER run on the device?", which none of the others do.
#[path = "motion_state_gpu_zone_demo.rs"]
mod gpu_zone_demo;

use gpu_deform_demo::{
    build_gpu_deform_demo_document, build_gpu_deform_organism_demo_document,
    build_gpu_four_point_warp_demo_document, build_gpu_kaleidoscope_demo_document,
    build_gpu_spherize_demo_document,
};
use gpu_demos::{
    build_gpu_demo_document, build_gpu_emitter_demo_document, build_gpu_hybrid_demo_document,
    build_gpu_sea_demo_document, build_gpu_sim_demo_document,
};
use gpu_neighbour_demos::{
    build_gpu_boids_demo_document, build_gpu_collide_demo_document, build_gpu_sweep_demo_document,
};
use gpu_panel_demo::build_gpu_panel_demo_document;
use gpu_voronoi_demo::build_gpu_voronoi_demo_document;
use gpu_zone_demo::build_gpu_zone_demo_document;

use ph2d_eval_motion::MotionCookPump;
use ph2d_motion_doc::{MotionDoc, MotionHistory};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::format::ParseError;
use ph2d_nodegraph::graph::NodeId;

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
    /// **The subgraph the editor is standing IN** (doc 57): `None` = the root canvas.
    /// Navigation, so it is not in the document (not serialized, not undoable) — but
    /// it is not in the PANEL either: an undo that unmakes the group you are inside
    /// has to be able to put you back on solid ground, and only the shell sees the
    /// undo (`subgraph::clamp_level`, every frame).
    pub(crate) level: Option<u32>,
    /// GPU/M5 Fase 1 (ADR-0126): the GPU-resident sequencer. Only driven when
    /// [`Self::gpu_enabled`]; persistent so its pipeline caches + buffer pool
    /// survive across frames (the GPU twin of the pump's reused buffer).
    pub(crate) gpu_cook: ph2d_gpu_cook::GpuCook,
    /// Did THIS frame's instances come from the GPU path? When `true`, present
    /// binds `gpu_cook.instances()` directly (zero readback) and
    /// `pump.instances` is stale — set/cleared by the bridge every frame.
    pub(crate) gpu_live: bool,
    /// The GPU cook path — **ON by default** since the editor learned to read a
    /// GPU-resident frame (the tap feeds the readouts/probe/digest, Fase 4);
    /// `PH2D_GPU_COOK=0` opts back out. The CPU pump remains the CANONICAL
    /// path either way (replay-hash, parity oracles — ADR-0126).
    pub(crate) gpu_enabled: bool,
}

/// **Is the GPU cook path on?** — the policy, as a pure function of the env var,
/// so it can be gated without a test mutating the process environment.
///
/// ON unless explicitly switched off. See the call site for why the default
/// flipped and why flipping it is safe.
pub(crate) fn gpu_enabled_from_env(var: Option<&str>) -> bool {
    !matches!(var, Some("0"))
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
        // GPU/M5 ready-to-smoke documents (opt-in; the regular boot document is
        // untouched): `PH2D_GPU_COOK_DEMO=1` = the F1.1 1250×1600 (2.000.000)
        // chain that is 100% GPU under `PH2D_GPU_COOK=1`; `=2` = the F1.2 HYBRID
        // chain whose first node (an oscillator on the uncovered Rotation channel)
        // has no kernel, so the CPU cooks the prefix and the GPU runs the suffix;
        // `=3` = the Fase 3 SIMULATION (490.000 particles in a force loop whose
        // state ping-pongs on the device, ADR-0127); `=4` = the SEA (490.000
        // raining onto a travelling wave); `=5` = the emitter FOUNTAIN (ADR-0130,
        // the id-gather: particles born/killed across a sliding window, paired by
        // arithmetic — the scene the fixed-grid demos could not be).
        let sinks = match std::env::var("PH2D_GPU_COOK_DEMO").as_deref() {
            Ok("1") => build_gpu_demo_document(&mut doc, &registry).unwrap_or_default(),
            Ok("2") => build_gpu_hybrid_demo_document(&mut doc, &registry).unwrap_or_default(),
            Ok("3") => build_gpu_sim_demo_document(&mut doc, &registry).unwrap_or_default(),
            Ok("4") => build_gpu_sea_demo_document(&mut doc, &registry).unwrap_or_default(),
            // ADR-0130: the emitter FOUNTAIN — the id-gather (particles born/killed
            // across a sliding window, paired by arithmetic; fatia 5 tunes it live).
            Ok("5") => build_gpu_emitter_demo_document(&mut doc, &registry).unwrap_or_default(),
            // The PANEL scene: 262.144 instances through both domains, so every
            // kind of card reading is on screen at once — the smoke for the GPU
            // path becoming the default.
            Ok("6") => build_gpu_panel_demo_document(&mut doc, &registry).unwrap_or_default(),
            // ADR-0140: the MURMURATION — the interacting sim (each agent reads its
            // neighbours) that the spatial grid lifts from a few-hundred-agent toy
            // to a swarm on the device (524.288, sized so it never stutters when the
            // flock gathers; the ceiling is millions).
            Ok("7") => build_gpu_boids_demo_document(&mut doc, &registry).unwrap_or_default(),
            // ADR-0140 Fase 5: the breathing PACKING — the second grid client,
            // and the first ITERATED kernel (the grid is rebuilt per sweep).
            Ok("8") => build_gpu_collide_demo_document(&mut doc, &registry).unwrap_or_default(),
            // ADR-0140 Fase 5: the spread SWEEP — the diagnostic scene. A slow,
            // linear triangle sweep of `spread` so the GPU meter shows a smooth
            // mountain (the cost is a function of the packing, no reach-boundary
            // step) rather than a staircase.
            Ok("9") => build_gpu_sweep_demo_document(&mut doc, &registry).unwrap_or_default(),
            // ADR-0135: the SIM-ZONE family — a fixed-population snow globe, the
            // state-loop container (`sim.zone` + `sim.step` + `sim.collide`) 100% on
            // the device. The boot snow's physics minus birth/death (which are
            // count-changing and still cook on the pump).
            Ok("10") => build_gpu_zone_demo_document(&mut doc, &registry).unwrap_or_default(),
            // ADR-0139: the breathing HONEYCOMB — the first engine ALGORITHM
            // (Lloyd relaxation via jump flooding), and the cap that fell with
            // it: 20.000 points where the CPU-era node capped at 600.
            Ok("11") => build_gpu_voronoi_demo_document(&mut doc, &registry).unwrap_or_default(),
            // The DEFORMER family: the whole-stream reduction channel. Two
            // deformers CHAINED, so the second one's fold must measure what the
            // first one produced (see the scene's own note).
            Ok("12") => build_gpu_deform_demo_document(&mut doc, &registry).unwrap_or_default(),
            // The `Sum` half of the deformer channel: the centroid lens (two
            // reductions on one node).
            Ok("13") => build_gpu_spherize_demo_document(&mut doc, &registry).unwrap_or_default(),
            // The widest reduction consumer: the bounding-box corner-pin (four
            // reductions, the first use of Min).
            Ok("14") => {
                build_gpu_four_point_warp_demo_document(&mut doc, &registry).unwrap_or_default()
            }
            // The count-changing deformer: the mandala fan-out (StreamOp
            // SourceRows, the first kernel to READ its template).
            Ok("15") => {
                build_gpu_kaleidoscope_demo_document(&mut doc, &registry).unwrap_or_default()
            }
            // O ORGANISMO: the whole reduction channel end to end — count-changing
            // fan (SourceRead) then the four count-preserving deformers, each
            // folding its reduction over the live stream the previous one produced.
            Ok("16") => {
                build_gpu_deform_organism_demo_document(&mut doc, &registry).unwrap_or_default()
            }
            _ => build_default_document(&mut doc, &registry).unwrap_or_default(),
        };
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
            level: None,
            gpu_cook: ph2d_gpu_cook::GpuCook::new(),
            gpu_live: false,
            // **ON by default** (GPU/M5, 2026-07-18). It was opt-in for one
            // reason and the reason is gone: a GPU-resident cook does not feed
            // the CPU memo, so the graph panel's readouts, postage stamps, wire
            // march and probe all went blank exactly on the documents worth
            // watching. `GpuCook::tap` now answers them for a measured +0,075 ms
            // (`bounded_readback_cost_probe`), and the panel reads the device.
            //
            // Turning it on is not a claim that every document runs there:
            // `gpu_route` still recuses a multi-sink or time-scoped document
            // whole, and `plan` recuses any chain with an uncovered node in it.
            // Those all fall through to the CPU pump exactly as before — which is
            // why the flip is safe and why it is a DEFAULT rather than a
            // requirement.
            //
            // `PH2D_GPU_COOK=0` forces the CPU pump: an escape for bisecting a
            // suspected device-path bug against the canonical path, which stays
            // the CPU's (ADR-0126 — the replay-hash never runs on a GPU).
            gpu_enabled: gpu_enabled_from_env(std::env::var("PH2D_GPU_COOK").ok().as_deref()),
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
    ///   field we own. `App::project_load_from` rewinds it the moment a file is accepted, and
    ///   both halves of that are gated: the rewind itself in `project::tests`, and what it buys
    ///   us in `a_clock_that_was_not_rewound_opens_the_document_mid_scene`. The reason stands:
    ///   a playhead at t=40s into a graph that has never been cooked is not a resumption, it is
    ///   a lie about a simulation that never ran.
    /// - **undo** belongs to the document that was edited, not to the file that replaced it.
    /// - the **probe**, the **flow digests**, the panel's **selection** and the **level the
    ///   editor is standing in** (doc 57) all name things by id. A stale selection is the
    ///   sharpest of them: the params panel would happily edit whichever node inherited the
    ///   number. A stale LEVEL is the strangest — the new document's group `2` is not the room
    ///   you were in, and you would be looking at a canvas you never opened.
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
        self.level = None;
        // The GPU path re-plans against the new graph next frame; until then
        // its instance buffer describes the OLD document, so it must not draw.
        self.gpu_live = false;
        // …and its SIMULATION state (last tick's `pre` columns, keyed by node id)
        // is the old document's — a new graph that reuses those ids for a `pre`
        // source would read a stranger's flakes-in-the-air. Forget it, exactly as
        // the pump above was replaced (ADR-0130 D7: a document change invalidates
        // the sim); the sim re-bakes from the seed under the new graph next frame.
        self.gpu_cook.forget_state();
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
fn build_default_document(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    let demo = strobe::build(&mut doc.graph)?;
    // Same "validate on load" the editor runs before cooking — proves the authored
    // graph is well-typed and membrane-clean.
    doc.graph.validate(reg).ok()?;
    // **The boot document ships a SUBGRAPH** (doc 57): the six nodes that age, colour,
    // shrink and fade a flake are folded into ONE card, sitting inline in the chain
    // with one socket on each side. So the feature is on the canvas the moment the tool
    // opens — double-click the card and you are inside it — and nobody has to build a
    // graph to find out that groups exist.
    //
    // The snow is **byte-identical** with the group as without it (gate:
    // `grouping_never_changes_the_cook`). That is the whole claim of the design, and
    // the boot document is where it is easiest to see: the flakes still fall.
    let sid = 0;
    // The centroid of what it folds — the SAME place the Ctrl+G gesture would put it
    // (`subgraph::group`), so the boot document is a document the artist could have
    // authored, not a special case the code knows about.
    let mut sum = (0.0f32, 0.0f32);
    for n in &demo.aging {
        let p = doc.graph.pos(*n)?;
        sum = (sum.0 + p.x, sum.1 + p.y);
    }
    let n = demo.aging.len() as f32;
    doc.subgraphs.push(ph2d_motion_doc::Subgraph {
        id: sid,
        parent: None,
        x: sum.0 / n,
        y: sum.1 / n,
        title: "Age & Fade".to_string(),
    });
    for id in &demo.aging {
        doc.members.insert(*id, sid);
    }
    Some(demo.sinks)
}
#[cfg(test)]
#[path = "motion_state_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_state_gpu_tests.rs"]
mod gpu_tests;

#[cfg(test)]
#[path = "motion_state_gpu_neighbour_tests.rs"]
mod gpu_neighbour_tests;

#[cfg(test)]
#[path = "motion_state_gpu_voronoi_tests.rs"]
mod gpu_voronoi_tests;

#[cfg(test)]
#[path = "motion_gpu_coverage.rs"]
mod gpu_coverage;
