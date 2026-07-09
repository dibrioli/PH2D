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
    /// Build the boot state: register every node op + the **Cavalry M1 demo** (the
    /// gate graph: a 20×10 grid → clone-tiled 20×20 → gradient tint → orbit →
    /// circle falloff → stagger → oscillator → wiggle → Output), terminated by the
    /// Output render node. Transport paused at tick 0 (the bridge auto-plays on
    /// tool entry).
    pub(crate) fn new() -> Self {
        let mut registry = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut registry)
            .expect("motion node registry builds");
        let mut doc = MotionDoc::new();
        let sink = build_cavalry_demo(&mut doc.graph, &registry);
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
            // Below the demo grid's 0.5 gap → distinct dots with clear gaps in the
            // 20×20 lattice (a stagger/scale that writes a `size` column overrides).
            default_size: [0.4, 0.4],
        }
    }

    /// Current playhead in seconds for the given fixed timestep.
    pub(crate) fn playhead(&self, fixed_dt: f64) -> f64 {
        self.transport.playhead(fixed_dt)
    }
}

/// Author the **Cavalry M1 gate demo** into `g`; returns the sink (the Output
/// node) if the graph is well-typed. The chain is
/// `grid → clone → tint → orbit → falloff → stagger → oscillator → wiggle → output`:
///
/// - **grid** 20×10 (200 instances), gap 0.5 → a half-height lattice; emits
///   `Index`/`Count`.
/// - **clone** ×2, **centred**, polar step +Y (angle ¼ turn) of `10 rows · 0.5 =
///   5.0` → the two centred copies tile top/bottom into the exact 20×20 lattice
///   (400 instances, spanning ±4.75, framed by the default `height_world = 10`
///   camera). Its continuous `Index`/`Count` renumbering keeps the colour ramp a
///   single seamless gradient across the whole set — so this reproduces the
///   20×20 beauty shot pixel-for-pixel while demonstrating the cloner live.
/// - **tint** Gradient (red → blue by index) — the whole grid is a colour ramp
///   (upstream of the falloff, so it colours every dot, not just the focus).
/// - **orbit** a gentle whole-grid spin about the origin (upstream of the
///   falloff, so it isn't focus-masked — the lattice slowly rotates as a whole).
/// - **falloff** Circle (radius 4) — a central focus field the behaviours read.
/// - **stagger** a Y tilt across the grid, masked by the falloff.
/// - **oscillator** a travelling Sine Y-wave, masked by the falloff (the centre
///   bounces, the edges hold) — the classic Cavalry focal-motion look.
/// - **wiggle** an organic X jitter on the focus region (value noise) on top.
///
/// The Output node is the render target; the bridge keeps `sink` pointed at it.
fn build_cavalry_demo(g: &mut Graph, reg: &NodeRegistry) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let clone = g.add_node("motion.clone");
    let tint = g.add_node("motion.tint");
    let orbit = g.add_node("motion.orbit");
    let falloff = g.add_node("motion.falloff");
    let stagger = g.add_node("motion.stagger");
    let osc = g.add_node("motion.oscillator");
    let wiggle = g.add_node("motion.wiggle");
    let output = g.add_node("motion.output");
    for (i, (from, to)) in [
        (grid, clone),
        (clone, tint),
        (tint, orbit),
        (orbit, falloff),
        (falloff, stagger),
        (stagger, osc),
        (osc, wiggle),
        (wiggle, output),
    ]
    .into_iter()
    .enumerate()
    {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
        // Lay the chain left-to-right in graph space (connected cards).
        g.set_pos(
            from,
            Pos {
                x: i as f32 * 220.0,
                y: 0.0,
            },
        );
    }
    g.set_pos(output, Pos { x: 1760.0, y: 0.0 });

    // 20×10 half-lattice, gap 0.5 (the clone tiles it into the full 20×20).
    g.set_param(grid, "rows", 10.0);
    g.set_param(grid, "cols", 20.0);
    g.set_param(grid, "gap_x", 0.5);
    g.set_param(grid, "gap_y", 0.5);
    // Clone ×2, centred, +Y step of `rows · gap_y = 5.0` → the two copies tile
    // top/bottom into the exact 20×20 lattice; continuous Index keeps one ramp.
    g.set_param(clone, "count", 2.0);
    g.set_param(clone, "distance", 5.0);
    g.set_param(clone, "angle", 0.25); // ¼ turn → +Y
    g.set_param(clone, "center", 1.0);
    // Gradient tint: red (start) → blue (end) by index (linear-straight RGBA).
    g.set_param(tint, "mode", 1.0);
    g.set_param(tint, "r", 1.0);
    g.set_param(tint, "g", 0.1);
    g.set_param(tint, "b", 0.1);
    g.set_param(tint, "r2", 0.1);
    g.set_param(tint, "g2", 0.3);
    g.set_param(tint, "b2", 1.0);
    // Gentle whole-grid spin about the origin (unmasked — upstream of falloff).
    g.set_param(orbit, "speed", 0.08);
    // Central circle focus (radius 4, smoothstep edge).
    g.set_param(falloff, "radius", 4.0);
    // Y tilt across the grid (masked by the falloff).
    g.set_param(stagger, "channel", 1.0); // Y
    g.set_param(stagger, "min", -1.0);
    g.set_param(stagger, "max", 1.0);
    // Travelling Sine Y-wave (masked by the falloff).
    g.set_param(osc, "channel", 1.0); // Y
    g.set_param(osc, "amplitude", 1.0);
    g.set_param(osc, "frequency", 0.6);
    g.set_param(osc, "phase_stagger", 0.1);
    // Organic X jitter on the focus region (value noise).
    g.set_param(wiggle, "channel", 0.0); // X
    g.set_param(wiggle, "amplitude", 0.5);
    g.set_param(wiggle, "frequency", 1.0);

    // Same "validate on load" the editor runs before cooking — proves the authored
    // graph is well-typed and membrane-clean.
    g.validate(reg).ok()?;
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builds_the_well_typed_cavalry_demo() {
        let state = MotionState::new();
        assert!(state.sink.is_some(), "the demo must have a sink");
        // 9 nodes (grid, clone, tint, orbit, falloff, stagger, oscillator, wiggle,
        // output).
        assert_eq!(state.doc.graph.nodes().len(), 9);
        let sink = state.sink.unwrap();
        assert_eq!(
            state.doc.graph.node(sink).unwrap().type_name,
            "motion.output"
        );
        assert!(state.doc.graph.validate(&state.registry).is_ok());
        assert_eq!(state.playhead(1.0 / 60.0), 0.0); // paused at tick 0
    }

    #[test]
    fn cavalry_demo_gate_cooks_400_animated_coloured_instances() {
        use ph2d_eval_motion::MotionCookPump;
        // The M1 gate, cooked through the real registry: 20×10 grid cloned ×2 =
        // 400 instances, gradient-tinted, and the oscillator animates them (Y
        // moves with the playhead) — proving grid+clone+tint+falloff+stagger+
        // oscillator compose.
        let state = MotionState::new();
        let (uv, size) = (state.default_uv_rect, state.default_size);

        let mut pump = MotionCookPump::new();
        pump.pump(
            &state.doc.graph,
            &state.registry,
            state.sink,
            0,
            0.0,
            uv,
            size,
        );
        assert_eq!(
            pump.instances.len(),
            400,
            "20×10 grid cloned ×2 = 400 instances"
        );
        // A gradient tint means the instances are not all the same colour.
        let c0 = pump.instances[0].tint;
        let clast = pump.instances[399].tint;
        assert_ne!(c0, clast, "gradient tint varies across the grid");

        // A central instance moves in Y as time advances (oscillator, focus-masked).
        let mid = 400 / 2 + 10; // an instance near the centre (inside the falloff)
        let y_at = |ph: f64| {
            let mut p = MotionCookPump::new();
            p.pump(
                &state.doc.graph,
                &state.registry,
                state.sink,
                1,
                ph,
                uv,
                size,
            );
            p.instances[mid].world_pos[1]
        };
        assert!(
            (y_at(0.25) - y_at(0.0)).abs() > 1e-4,
            "the focus region animates over time"
        );
    }
}
