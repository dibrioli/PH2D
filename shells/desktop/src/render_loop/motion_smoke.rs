//! W2.T3 visual smoke — the Motion vertical on screen, behind a flag.
//!
//! This is **debug scaffolding**, not a production code path. It exists so the
//! human (Enio) can confirm the one thing the headless tests cannot: that the
//! `ph2d-eval-motion` lowering convention (a cooked instance stream →
//! `RenderInstance`s) maps correctly onto the real sprite renderer — positions,
//! coordinate system, units. The headless integration test
//! (`ph2d-eval-motion/tests/motion_vertical.rs`) already proves the data
//! (grid→transform→clone → 27 instances at the expected positions); this draws
//! those same 27 instances so they can be eyeballed.
//!
//! Gated behind the `PH2D_MOTION_SMOKE` env var (`=1`) so the normal app is
//! untouched. When on, it **replaces** the M5 demo's sim-tick + extract for the
//! frame (it owns the canvas) — see `render_loop::mod`.
//!
//! The image is **static by design**: every node in the vertical is `Pure`
//! (combinational), so re-cooking each frame reproduces the same stream — there
//! is no `Temporal` node to animate it (that is a fan-out item). The point here
//! is the spatial mapping, not motion.

use std::sync::OnceLock;

use ph2d_ecs::PresentWorld;
use ph2d_eval_motion::evaluate_motion;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_render::SpriteRenderer;

/// Is the Motion smoke scene enabled? Read once from `PH2D_MOTION_SMOKE=1`.
pub(super) fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_MOTION_SMOKE").is_some_and(|v| v == "1"))
}

/// Presentation-only framing (does NOT touch node semantics):
/// the vertical emits positions in `x∈[0,6], y∈[0,2]`; this offset re-centers
/// that span on the origin so it sits inside the camera's `~±WORLD_HALF` view.
const VIEW_CENTER_OFFSET: [f32; 2] = [-3.0, -1.0];
/// The grid is spaced 1 world unit apart and the lowering defaults `size` to
/// `[1,1]` (the generator emits no `size` column), so unit sprites would tile
/// edge-to-edge into a solid band. Shrink them below the spacing — presentation
/// only — so the grid reads as discrete sprites.
const DISPLAY_SIZE: [f32; 2] = [0.6, 0.6];
/// Number of cells in the demo atlas (matches `sim_populate`'s `i % 16`).
const ATLAS_CELLS: usize = 16;

/// Cook the Motion vertical and publish its instances into `present` for this
/// frame's sprite pass. Builds the graph + registry inline each call: trivial
/// for 27 instances, and it keeps the debug glue self-contained (no new state
/// threaded through the shell's `init`). Re-cooking every frame is exactly what
/// the W2.T3 "live-preview" would do — here the output is constant because the
/// vertical is combinational.
pub(super) fn run(present: &mut PresentWorld, renderer: &SpriteRenderer) {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).expect("motion.grid registers");
    ph2d_node_motion_transform::register(&mut reg).expect("motion.transform registers");
    ph2d_node_motion_clone::register(&mut reg).expect("motion.clone registers");

    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let xf = g.add_node("motion.transform");
    let clone = g.add_node("motion.clone");
    g.connect(Edge {
        from: (grid, 0),
        to: (xf, 0),
        delayed: false,
    })
    .expect("grid → transform");
    g.connect(Edge {
        from: (xf, 0),
        to: (clone, 0),
        delayed: false,
    })
    .expect("transform → clone");
    // The same "validate on load" the editor view will run before cooking —
    // proves the authored graph is well-typed and membrane-clean.
    g.validate(&reg).expect("motion vertical is well-typed");

    let mut cook = Cook::new();
    // Pure vertical → playhead is irrelevant; cook at 0.0.
    let instances =
        evaluate_motion(&mut cook, &g, &reg, clone, 0.0).expect("motion vertical cooks");

    // The lowering leaves `atlas_uv` at the whole-sheet default (`[0,0,1,1]`)
    // because a Motion node carries no texture — correct for the data path, but
    // it would sample all 16 atlas cells into each quad (sparse → thin specks).
    // For the visual smoke, point each instance at one real atlas cell, exactly
    // like `sim_populate`, so it draws as a solid sprite. Presentation only.
    let atlas = renderer.atlas();
    let world = present.world_mut();
    world.clear_entities();
    for (idx, mut ri) in instances.into_iter().enumerate() {
        ri.world_pos[0] += VIEW_CENTER_OFFSET[0];
        ri.world_pos[1] += VIEW_CENTER_OFFSET[1];
        ri.size = DISPLAY_SIZE;
        ri.atlas_uv = atlas.region_uv((idx % ATLAS_CELLS) as u32);
        world.spawn(ri);
    }
}
