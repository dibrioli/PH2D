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
    /// GPU/M5 Fase 1 (ADR-0122): the GPU-resident sequencer. Only driven when
    /// [`Self::gpu_enabled`]; persistent so its pipeline caches + buffer pool
    /// survive across frames (the GPU twin of the pump's reused buffer).
    pub(crate) gpu_cook: ph2d_gpu_cook::GpuCook,
    /// Did THIS frame's instances come from the GPU path? When `true`, present
    /// binds `gpu_cook.instances()` directly (zero readback) and
    /// `pump.instances` is stale — set/cleared by the bridge every frame.
    pub(crate) gpu_live: bool,
    /// `PH2D_GPU_COOK=1` — the Fase 1 opt-in. The CPU pump stays the default
    /// (it feeds the graph panel's readouts/probe and is the canonical path);
    /// flipping the default is a Fase 4 decision, after the editor reads GPU.
    pub(crate) gpu_enabled: bool,
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
        // state ping-pongs on the device, ADR-0123).
        let sinks = match std::env::var("PH2D_GPU_COOK_DEMO").as_deref() {
            Ok("1") => build_gpu_demo_document(&mut doc, &registry).unwrap_or_default(),
            Ok("2") => build_gpu_hybrid_demo_document(&mut doc, &registry).unwrap_or_default(),
            Ok("3") => build_gpu_sim_demo_document(&mut doc, &registry).unwrap_or_default(),
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
            gpu_enabled: std::env::var("PH2D_GPU_COOK").is_ok_and(|v| v == "1"),
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
/// The GPU/M5 Fase 1 **ready-to-smoke** document (`PH2D_GPU_COOK_DEMO=1`):
/// `grid(1250×1600) → oscillator(Y wave) → move → output` — **2.000.000
/// instances**, every node kernel-covered, so `PH2D_GPU_COOK=1` runs it 100%
/// GPU-resident (`ph2d_gpu_cook::plan` claims the whole chain; the renderer binds
/// the lowering's buffer with zero readback). Measured on the RTX at ~4 ms/frame
/// for the cook (probe `gpu_cook_millions_timing`), i.e. the roadmap's "millions
/// at 60fps" with headroom. The same chain, at 25.6k, is the parity gate's
/// fixture. Auto-plays on tool entry like every boot document.
fn build_gpu_demo_document(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    // 1250 × 1600 = exactly 2.000.000 cells (well under the grid's 16.7M cap).
    g.set_param(grid, "rows", 1250.0);
    g.set_param(grid, "cols", 1600.0);
    // A dense lattice: the quads are unit-sized (the shell's `default_size`
    // identity), so gap 1.0 tiles them edge-to-edge — zoom out and the whole
    // 1600×1250 field reads as a shimmering cloth of two million quads.
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "channel", 1.0); // Y — the kernel-covered channels are X/Y
    g.set_param(osc, "amplitude", 6.0);
    g.set_param(osc, "frequency", 0.5);
    // A travelling wave across the field. The phase advances per row-major index,
    // so the stagger is scaled down for 2M cells (~500 cycles across the cloth,
    // the same band density the 262k version read at 0.002).
    g.set_param(osc, "phase_stagger", 0.00025);
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    for (i, n) in [grid, osc, mv, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    g.connect(Edge {
        from: (grid, 0),
        to: (osc, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (osc, 0),
        to: (mv, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (mv, 0),
        to: (out, 0),
        delayed: false,
    })
    .ok()?;
    g.validate(reg).ok()?;
    Some(vec![out])
}

/// The GPU/M5 **Fase 3 simulation** ready-to-smoke document
/// (`PH2D_GPU_COOK_DEMO=3`, ADR-0123): `grid(700×700) → integrate → output` with
/// the force loop `integrate --pre--> vortex → attractor → drag → curl -->
/// integrate.forces` — **490.000 particles, simulated 100% on the GPU**.
///
/// The forces existed since M2 but had never run a frame on the GPU: they are
/// only ever evaluated INSIDE the `pre` loop, and the loop was what the plan
/// refused. Under `PH2D_GPU_COOK=1` the whole thing is claimed (no boundary),
/// the state ping-pongs as held `Arc`s across ticks, and scrubbing the playhead
/// backwards restores a device-side checkpoint instead of showing the future.
///
/// The mix is the classic stable orbit (vortex + attractor at one centre + drag
/// — without drag a pure vortex spirals outward by centrifugal drift), plus curl
/// noise so the cloud breathes instead of settling into a clean ring.
fn build_gpu_sim_demo_document(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 700.0);
    g.set_param(grid, "cols", 700.0);
    g.set_param(grid, "gap_x", 0.12);
    g.set_param(grid, "gap_y", 0.12);
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");

    // The force chain, source→sink inside the loop.
    let vortex = g.add_node("force.vortex");
    g.set_param(vortex, "strength", 14.0);
    g.set_param(vortex, "radius", 46.0);
    g.set_param(vortex, "clockwise", 1.0);
    let attractor = g.add_node("force.attractor");
    g.set_param(attractor, "strength", 9.0);
    g.set_param(attractor, "radius", 46.0);
    g.set_param(attractor, "curve", 0.0); // Linear: a steady inward pull
    let drag = g.add_node("force.drag");
    g.set_param(drag, "coefficient", 0.35);
    let curl = g.add_node("force.curl");
    g.set_param(curl, "strength", 6.0);
    g.set_param(curl, "scale", 0.06);
    g.set_param(curl, "speed", 0.4);
    g.set_param(curl, "octaves", 2.0);

    for (i, n) in [grid, ig, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 220.0,
                y: 120.0,
            },
        );
    }
    for (i, n) in [vortex, attractor, drag, curl].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 300.0,
            },
        );
    }
    g.connect(Edge {
        from: (grid, 0),
        to: (ig, 0),
        delayed: false,
    })
    .ok()?;
    // The feedback the user never draws: last tick's state into the chain head.
    g.connect(Edge {
        from: (ig, 0),
        to: (vortex, 0),
        delayed: true,
    })
    .ok()?;
    g.connect(Edge {
        from: (vortex, 0),
        to: (attractor, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (attractor, 0),
        to: (drag, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (drag, 0),
        to: (curl, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (curl, 0),
        to: (ig, 1),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (ig, 0),
        to: (out, 0),
        delayed: false,
    })
    .ok()?;
    g.validate(reg).ok()?;
    Some(vec![out])
}

/// The GPU/M5 **F1.2 hybrid** ready-to-smoke document (`PH2D_GPU_COOK_DEMO=2`):
/// `grid(360×360) → oscillator(Rotation) → oscillator(Y) → scale → output`.
///
/// The FIRST oscillator targets the **Rotation** channel, which the oscillator
/// kernel does not cover (`applicable` = X/Y only) — so the plan puts the CPU
/// boundary there: the CPU pump cooks `grid → oscillator(Rotation)` (a travelling
/// spin wave in the `rot` column), its stream crosses to the GPU ONCE, and the
/// GPU runs `oscillator(Y) → scale → output` (a travelling height wave + a size
/// pulse) plus the lowering — zero readback. So the field waves in Y AND the
/// quads spin, the two halves computed on opposite sides of the CPU↔GPU seam.
/// 129.600 instances. Auto-plays on tool entry like every boot document.
fn build_gpu_hybrid_demo_document(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 360.0);
    g.set_param(grid, "cols", 360.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    // CPU boundary: Rotation is outside the kernel's X/Y coverage.
    let spin = g.add_node("motion.oscillator");
    g.set_param(spin, "channel", 2.0); // Rotation — no kernel → the boundary
    g.set_param(spin, "amplitude", 45.0); // degrees
    g.set_param(spin, "frequency", 0.5);
    g.set_param(spin, "phase_stagger", 0.004);
    // GPU suffix: a Y wave, then a size pulse.
    let wave = g.add_node("motion.oscillator");
    g.set_param(wave, "channel", 1.0); // Y — kernel-covered
    g.set_param(wave, "amplitude", 5.0);
    g.set_param(wave, "frequency", 0.5);
    g.set_param(wave, "phase_stagger", 0.003);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 1.6);
    let out = g.add_node("motion.output");
    for (i, n) in [grid, spin, wave, scale, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    g.connect(Edge {
        from: (grid, 0),
        to: (spin, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (spin, 0),
        to: (wave, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (wave, 0),
        to: (scale, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (scale, 0),
        to: (out, 0),
        delayed: false,
    })
    .ok()?;
    g.validate(reg).ok()?;
    Some(vec![out])
}

#[cfg(test)]
#[path = "motion_state_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_state_gpu_tests.rs"]
mod gpu_tests;
