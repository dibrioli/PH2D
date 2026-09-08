//! **GPU-vs-CPU parity for the DEFORMER family** — `reduce → broadcast → map`,
//! the shape no per-element kernel could express (see
//! `ph2d_nodegraph::reduce_meta`).
//!
//! `motion.bend` and `motion.twist` each need one number about the WHOLE layout
//! before any element can be placed: the X extent about the pivot, and the rim
//! radius. This cooks the same graph through the canonical CPU path
//! (`evaluate_motion_into`, the production lowering) and through the GPU
//! sequencer, and asserts the pictures agree.
//!
//! ## The two nodes get DIFFERENT bounds, and that is the point
//!
//! `motion.bend` folds `abs(v.x − pivot_x)` under `Max`. `Max` is associative
//! *and exact* over floats, and the folded expression is a subtraction and an
//! `abs` — **nothing a device can contract into an FMA**. So the reduction is
//! bit-exact, and the only ε left is the per-element map.
//!
//! `motion.twist` folds `√(dx² + dy²)`, which has a product: a device may
//! contract `dx*dx + dy*dy` where the host does not, so its *rim radius* can
//! differ in the last ulps before the fold ever runs. Both then divide by it,
//! which is the amplifying step.
//!
//! ⚠️ **A deformer amplifies its reduction, so the ε cannot be borrowed from the
//! per-element gates.** `bend` computes `r = x_extent / θ` and multiplies a
//! position by `sin(k·dx)`; an error in the extent moves every element. The
//! bound is derived from what the arithmetic actually delivers — see [`EPS_POS`],
//! which carries the measurement and the head-room, and which the per-element
//! chain's `2e-3` would have over-stated by 50×.
//!
//! `#[ignore]`: needs a real adapter. Run on the GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_deform --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::RenderInstance;
use ph2d_render::SinkStyle;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_motion_falloff::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    ph2d_node_motion_bend::register(&mut reg).unwrap();
    ph2d_node_motion_twist::register(&mut reg).unwrap();
    ph2d_node_motion_spherize::register(&mut reg).unwrap();
    ph2d_node_motion_four_point_warp::register(&mut reg).unwrap();
    ph2d_node_motion_bezier_warp::register(&mut reg).unwrap();
    ph2d_node_motion_kaleidoscope::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_mirror::register(&mut reg).unwrap();
    reg
}

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;
/// 128×128 = 16.384 instances — well past the CPU's parallel threshold, and past
/// the reduction's first block seam (256) and its second level, so the recursion
/// is exercised by the product rather than only by the primitive's own gate.
const SIDE: f32 = 128.0;

/// The kaleidoscope's source side length: `KAL_SIDE² · segments` is the output,
/// kept light (64² × 8 = 32.768).
const KAL_SIDE: usize = 64;

/// Position bound. **Measured, not borrowed** — the gate prints its worst at
/// every size, and this is that number with head-room, not the `2e-3` the
/// per-element chain uses (which would be 50× slack here and would stop being a
/// bound at all).
///
/// Worst observed on this line, printed below: **3,8e-5** at 70.225 elements
/// (twist, the ε-carrying one), 1,1e-5 for bend, 7,2e-7 at a single element. It
/// grows with element count because the layout grows with it — the error is a
/// few ulps at a magnitude of ~45, and an f32 ulp there is 3,8e-6.
///
/// `2e-4` is ~5× the worst measurement (room for another vendor contracting
/// differently) and still **five orders of magnitude** below what the failure
/// this exists to catch produces: a dead reduction leaves the layout flat, which
/// the excursion gate measures at 44 units.
const EPS_POS: f32 = 2e-4;

/// A grid, deformed, then output. `deformer` is the node type; `pivot` moves the
/// reduction off the origin, which is what makes the reduction's VALUE matter
/// (a pivot at the centroid makes several wrong answers look plausible).
fn chain(reg: &NodeRegistry, deformer: &str, angle: f32, pivot: [f32; 2]) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let def = g.add_node(deformer);
    g.set_param(def, "angle", angle);
    g.set_param(def, "pivot_x", pivot[0]);
    g.set_param(def, "pivot_y", pivot[1]);
    let out = g.add_node("motion.output");
    for (a, b) in [(grid, def), (def, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(reg).expect("well-typed");
    (g, out)
}

fn cook_cpu(reg: &NodeRegistry, g: &Graph, out: NodeId) -> Vec<RenderInstance> {
    let mut cook = Cook::new();
    let mut cpu = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook,
        g,
        reg,
        out,
        PLAYHEAD,
        DEFAULT_UV,
        DEFAULT_SIZE,
        &mut cpu,
    )
    .expect("cpu cook");
    cpu
}

/// Cook on the device and read the instances back. Panics if the plan did not
/// claim the whole chain — a deformer that quietly fell back to the CPU would
/// make this gate compare the CPU against itself, which is the shape of a gate
/// that is green over a feature that never ran.
fn cook_gpu(gpu: &GpuContext, reg: &NodeRegistry, g: &Graph, out: NodeId) -> Vec<RenderInstance> {
    let plan = ph2d_gpu_cook::plan(g, reg, reg, out);
    assert!(
        plan.is_fully_gpu(),
        "the deformer chain must be claimed WHOLE — a CPU boundary would make \
         this gate compare the CPU with itself"
    );
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.cook(
        gpu,
        g,
        reg,
        reg,
        &plan,
        &[],
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
        SinkStyle::PLAIN,
    )
    .expect("gpu cook");
    ph2d_gpu_cook::read_instances(gpu, gc.instances().expect("cooked"))
}

fn compare(label: &str, cpu: &[RenderInstance], gpu: &[RenderInstance]) -> f32 {
    assert_eq!(cpu.len(), gpu.len(), "{label}: instance count");
    let mut worst = 0.0f32;
    for (i, (c, g)) in cpu.iter().zip(gpu).enumerate() {
        for k in 0..2 {
            let d = (c.world_pos[k] - g.world_pos[k]).abs();
            worst = worst.max(d);
            assert!(
                d <= EPS_POS,
                "{label}: instance {i} world_pos[{k}]: cpu {} vs gpu {} (|diff| {d:e} > {EPS_POS:e})",
                c.world_pos[k],
                g.world_pos[k]
            );
        }
    }
    eprintln!("{label}: {} instances, worst |Δpos| = {worst:e}", cpu.len());
    worst
}

/// The bend, on the device, agrees with the canonical arc.
///
/// The pivot sits off-centre so the reduction's answer is a number the layout
/// does not otherwise contain: with `pivot_x = 0` the extent is symmetric and a
/// reduction that folded the wrong half would still be right.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_bend_deformer_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for (angle, pivot) in [
        (90.0, [0.0, 0.0]),
        (140.0, [3.5, -1.25]),
        (-70.0, [-2.0, 0.75]),
    ] {
        let (g, out) = chain(&reg, "motion.bend", angle, pivot);
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        compare(&format!("bend angle {angle} pivot {pivot:?}"), &cpu, &dev);
    }
}

/// The twist, on the device, agrees with the canonical spiral.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_twist_deformer_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for (angle, pivot) in [
        (90.0, [0.0, 0.0]),
        (200.0, [2.75, 1.5]),
        (-120.0, [-1.0, -3.0]),
    ] {
        let (g, out) = chain(&reg, "motion.twist", angle, pivot);
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        compare(&format!("twist angle {angle} pivot {pivot:?}"), &cpu, &dev);
    }
}

/// `motion.spherize`, on the device, agrees with the canonical bulge/pinch — the
/// **`Sum` reduction, and TWO of them** (the centroid).
///
/// ⚠️ **The grid is TRANSLATED before the spherize**, so its centroid is a number
/// the reduction must actually compute — not the origin it would be by symmetry.
/// A `Sum` that folded only the first block, or dropped `cy`, would put the lens
/// centre somewhere else and the whole warp would land wrong; with the centroid
/// pinned at the origin (a bare grid) several such bugs would look plausible.
/// This is the `Sum` analogue of moving the pivot off-centre for `bend`.
///
/// The bound is [`EPS_POS`], the same as the `Max` deformers **despite** `Sum`
/// being an ε where `Max` is exact — because the centroid divides the layout-scale
/// sum back down by `n`, so its own error is tiny; measured worst is printed below.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_spherize_deformer_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // amount ∈ {bulge, pinch}, radius spanning the lens sizes; the grid is
    // shifted by `translate` so the centroid is off-origin, and the lens is
    // shifted off that centroid by `lens` — the two are DISTINCT so a kernel
    // that swapped or dropped one is caught (see `spherize_chain`). The last
    // row is the zero-offset control: the lens back on the centroid.
    for (amount, radius, translate, lens) in [
        (0.8f32, 6.0f32, [4.0f32, -2.5], [1.75f32, 0.9]),
        (-0.6, 9.0, [-3.0, 1.5], [-2.25, -1.1]),
        (0.5, 4.0, [0.0, 0.0], [0.0, 0.0]),
    ] {
        let (g, out) = spherize_chain(&reg, amount, radius, translate, lens);
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        compare(
            &format!(
                "spherize amount {amount} radius {radius} translate {translate:?} lens {lens:?}"
            ),
            &cpu,
            &dev,
        );
    }
}

/// `motion.four_point_warp`, on the device, agrees with the canonical corner-pin
/// — the **widest reduction consumer (FOUR: the bounding box), and the first use
/// of `Min`**.
///
/// ⚠️ **Two quads are tested on purpose.** A near-axis-aligned quad exercises the
/// `homography`'s AFFINE branch (`sx`/`sy` ≈ 0), a skewed one exercises the
/// PROJECTIVE branch with a real perspective divide — the two branches are
/// separate code on both sides, and the affine one is the common case (a
/// keystone) that a device-only projective path would silently get wrong.
///
/// The bounding box is `Min`/`Max`, which are bit-exact, so the reduction carries
/// **no** ε — the only ε is the homography arithmetic and the divide, and the
/// bound is [`EPS_POS`] (measured worst printed below).
#[test]
#[ignore = "requires a GPU adapter"]
fn the_four_point_warp_deformer_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // (label, warp, [tl_dx,tl_dy, tr_dx,tr_dy, br_dx,br_dy, bl_dx,bl_dy]).
    let cases: [(&str, f32, [f32; 8]); 3] = [
        // A gentle keystone: top edge pinched in — near-affine, the affine branch.
        ("keystone", 1.0, [1.5, 0.0, -1.5, 0.0, 0.0, 0.0, 0.0, 0.0]),
        // A skewed quad — the projective branch, with a real perspective divide.
        (
            "projective",
            1.0,
            [2.0, 1.0, -1.0, 2.5, 1.5, -2.0, -2.5, -0.5],
        ),
        // Half-applied, to check `warp` scales the corners (a mid-billow pose).
        ("half", 0.5, [3.0, 0.0, -3.0, 0.0, -1.0, -1.0, 1.0, 1.0]),
    ];
    for (label, warp, off) in cases {
        let (g, out) = four_point_chain(&reg, warp, off);
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        compare(&format!("four_point_warp {label}"), &cpu, &dev);
    }
}

/// A grid, corner-pinned by the eight offset params, then output. `warp` is a
/// constant `value.lfo` (a fixed billow — the reproducible authoring case).
fn four_point_chain(reg: &NodeRegistry, warp: f32, off: [f32; 8]) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let fpw = g.add_node("motion.four_point_warp");
    for (name, v) in [
        ("tl_dx", off[0]),
        ("tl_dy", off[1]),
        ("tr_dx", off[2]),
        ("tr_dy", off[3]),
        ("br_dx", off[4]),
        ("br_dy", off[5]),
        ("bl_dx", off[6]),
        ("bl_dy", off[7]),
    ] {
        g.set_param(fpw, name, v);
    }
    let amt = g.add_node("value.lfo");
    g.set_param(amt, "amplitude", 0.0);
    g.set_param(amt, "offset", warp);
    let out = g.add_node("motion.output");
    for (from, to, port) in [(grid, fpw, 0u16), (amt, fpw, 1), (fpw, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(reg).expect("well-typed");
    (g, out)
}

/// `motion.kaleidoscope`, on the device, agrees with the canonical mandala — the
/// **count-changing** `StreamOp::SourceRows` deformer, and the first kernel to
/// READ its template (`ColumnAccess::SourceRead`).
///
/// ⚠️ **The instance COUNT must match first** — the output is `n · segments`, and
/// a wrong count law or a broken fan-out is a different number of things on
/// screen, which `compare`'s length assert catches before any position. The
/// element ORDER also has to agree: the CPU is slice-major (`out[s·n + i]`) and
/// the device dispatches `i = s·src_n + row`, which coincide because `src_n = n`.
///
/// Both the **rotational** (`reflect` off, cyclic Cₖ) and **mirrored** (`reflect`
/// on, the true kaleidoscope Dₖ) symmetries are tested — the odd-slice mirror is a
/// separate branch on both sides. `spin` off-zero rotates the whole pattern, so
/// the reduction-free transform is exercised at a non-trivial angle.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_kaleidoscope_deformer_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // (segments, reflect, spin_deg, pivot).
    let cases: [(f32, bool, f32, [f32; 2]); 4] = [
        (6.0, false, 0.0, [0.0, 0.0]),
        (6.0, true, 0.0, [0.0, 0.0]),
        (8.0, true, 37.0, [2.5, -1.5]),
        (3.0, false, -110.0, [-3.0, 2.0]),
    ];
    for (segments, reflect, spin, pivot) in cases {
        let (g, out) = kaleidoscope_chain(&reg, segments, reflect, spin, pivot);
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        assert_eq!(
            cpu.len(),
            KAL_SIDE * KAL_SIDE * segments as usize,
            "kaleidoscope must fan out to n·segments"
        );
        compare(
            &format!("kaleidoscope seg {segments} reflect {reflect} spin {spin}"),
            &cpu,
            &dev,
        );
    }
}

/// A grid, kaleidoscoped, then output. `spin` is a constant `value.lfo` (a fixed
/// global rotation — the reproducible authoring case).
fn kaleidoscope_chain(
    reg: &NodeRegistry,
    segments: f32,
    reflect: bool,
    spin_deg: f32,
    pivot: [f32; 2],
) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    // A smaller source so `n · segments` stays a light dispatch (64² × 8 = 32.768).
    g.set_param(grid, "rows", KAL_SIDE as f32);
    g.set_param(grid, "cols", KAL_SIDE as f32);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let kal = g.add_node("motion.kaleidoscope");
    g.set_param(kal, "segments", segments);
    g.set_param(kal, "reflect", f32::from(reflect));
    g.set_param(kal, "pivot_x", pivot[0]);
    g.set_param(kal, "pivot_y", pivot[1]);
    let spin = g.add_node("value.lfo");
    g.set_param(spin, "amplitude", 0.0);
    g.set_param(spin, "offset", spin_deg);
    let out = g.add_node("motion.output");
    for (from, to, port) in [(grid, kal, 0u16), (spin, kal, 1), (kal, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(reg).expect("well-typed");
    (g, out)
}

/// A grid, TRANSLATED (so the spherize's centroid is off-origin), spherized, then
/// output. `amount` is a constant `value.lfo` at phase 0 (a fixed bulge/pinch —
/// the LFO is how `amount` is authored, and a constant is the reproducible case).
///
/// ⚠️ **TWO displacements, and they must not be the same number.** `translate` moves
/// the GRID (so the centroid is a number the `Sum` reduction has to compute) and
/// `lens` moves the LENS off that centroid (`offset_x`/`offset_y`, doc 88 §9). A
/// device kernel that dropped the lens offset — or that confused it with the grid
/// translation — would agree with the CPU on every case where the two coincide, so
/// the caller passes them distinct.
fn spherize_chain(
    reg: &NodeRegistry,
    amount: f32,
    radius: f32,
    translate: [f32; 2],
    lens: [f32; 2],
) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", translate[0]);
    g.set_param(mv, "dy", translate[1]);
    let sph = g.add_node("motion.spherize");
    g.set_param(sph, "radius", radius);
    g.set_param(sph, "offset_x", lens[0]);
    g.set_param(sph, "offset_y", lens[1]);
    // A constant amount: a `value.lfo` with amplitude carrying the level and a
    // period so long it does not move at the fixed playhead. Simpler: offset.
    let amt = g.add_node("value.lfo");
    g.set_param(amt, "amplitude", 0.0);
    g.set_param(amt, "offset", amount);
    let out = g.add_node("motion.output");
    for (from, to, port) in [(grid, mv, 0u16), (mv, sph, 0), (amt, sph, 1), (sph, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(reg).expect("well-typed");
    (g, out)
}

/// **O ORGANISMO** — the `PH2D_GPU_COOK_DEMO=16` scene as a chain: the whole
/// reduction channel end to end, `grid(n×n) → move → kaleidoscope → spherize →
/// bend → twist → four_point_warp → output`. The kaleidoscope FANS the source
/// (`n² · segments`), then four count-preserving deformers each fold their own
/// reduction over the stream the previous one produced — `Sum` (centroid), `Max`
/// (x-extent), `Max` (rim radius), `Min`/`Max` (bbox). Every amount is a constant
/// `value.lfo` on port 1 (`ReadBroadcast`, frozen so the parity is reproducible),
/// and every level is non-trivial so no stage passes the layout through flat.
///
/// `side` is a free variable so the parity gate can run it TINY (the scene itself
/// is 200² · 12 = 480.000; here 12² · 12 = 1.728 is enough to cross the fan and
/// every reduction seam while keeping the CPU reference instant).
fn organism_chain(reg: &NodeRegistry, side: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", side);
    g.set_param(grid, "cols", side);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);

    // Park the source off the pivot so each slice is a wedge (the scene's `move`).
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 26.0);
    g.set_param(mv, "dy", 0.0);

    // 1. THE FAN — count-changing (×12), the true Dₙ mirror.
    let kal = g.add_node("motion.kaleidoscope");
    g.set_param(kal, "segments", 12.0);
    g.set_param(kal, "reflect", 1.0);
    // 2. THE DOME — the fanned mandala's centroid (two Sum reductions).
    let sph = g.add_node("motion.spherize");
    g.set_param(sph, "radius", 40.0);
    // 3. THE ARC — the domed form's X extent (Max).
    let bend = g.add_node("motion.bend");
    g.set_param(bend, "angle", 90.0);
    g.set_param(bend, "pivot_x", 0.0);
    g.set_param(bend, "pivot_y", 0.0);
    // 4. THE WIND — the arced form's rim radius (Max).
    let twist = g.add_node("motion.twist");
    g.set_param(twist, "angle", 80.0);
    g.set_param(twist, "pivot_x", 0.0);
    g.set_param(twist, "pivot_y", 0.0);
    // 5. THE PIN — the wound form's bounding box (Min/Max ×4).
    let fpw = g.add_node("motion.four_point_warp");
    for (name, v) in [
        ("tl_dx", 8.0f32),
        ("tl_dy", 3.0),
        ("tr_dx", -6.0),
        ("tr_dy", 9.0),
        ("br_dx", -2.0),
        ("br_dy", 2.0),
        ("bl_dx", 2.0),
        ("bl_dy", 0.0),
    ] {
        g.set_param(fpw, name, v);
    }

    // Constant broadcast amounts (amplitude 0, offset = level), each non-trivial.
    let amt = |g: &mut Graph, level: f32| {
        let n = g.add_node("value.lfo");
        g.set_param(n, "amplitude", 0.0);
        g.set_param(n, "offset", level);
        n
    };
    let spin = amt(&mut g, 30.0); // degrees
    let dome = amt(&mut g, 0.6);
    let arc = amt(&mut g, 0.7);
    let wind = amt(&mut g, 0.7);
    let keystone = amt(&mut g, 0.8);

    let out = g.add_node("motion.output");

    for (from, to, port) in [
        (grid, mv, 0u16),
        (mv, kal, 0),
        (spin, kal, 1),
        (kal, sph, 0),
        (dome, sph, 1),
        (sph, bend, 0),
        (arc, bend, 1),
        (bend, twist, 0),
        (wind, twist, 1),
        (twist, fpw, 0),
        (keystone, fpw, 1),
        (fpw, out, 0),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(reg).expect("well-typed");
    (g, out)
}

/// **The whole organism is claimed WHOLE by the planner** — device-free, so it
/// runs on every lane. This is the property that makes `PH2D_GPU_COOK_DEMO=16` a
/// valid GPU smoke: six kernels deep (a count-changing fan and four reducing
/// deformers) with ZERO CPU boundary. A single node that fell back to the CPU
/// would split the plan, and the scene would silently cook half on each side.
///
/// Mutation note: this catches the failure that no numeric gate can — a deformer
/// whose kernel registration is dropped still cooks correctly on the CPU, so the
/// picture looks right; only `is_fully_gpu()` goes false.
#[test]
fn the_organism_is_claimed_whole_on_the_device() {
    let reg = registry();
    let (g, out) = organism_chain(&reg, 12.0);
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(
        plan.is_fully_gpu(),
        "the organism chain (kaleidoscope → spherize → bend → twist → \
         four_point_warp) must be claimed WHOLE — a CPU boundary anywhere means \
         PH2D_GPU_COOK_DEMO=16 is not the all-on-device smoke it claims to be"
    );
}

/// **The six-deep chain agrees with the CPU** — the reductions do not just each
/// work in isolation (the gates above), they COMPOSE: each stage folds over what
/// the last one produced, on both paths, and the pictures still match.
///
/// ⚠️ **And it fits the SAME [`EPS_POS`] the single stages use** — that is the
/// finding, not a footnote. Four amplifying deformers in series each divide by a
/// reduction, so a last-ulp difference in the fanned centroid *could* ride through
/// the arc, the wind, and the homography and compound. Measured at a tiny size
/// (12² · 12 = 1.728), the worst is **3,5e-5** — essentially the single-stage
/// number; the compounding did NOT blow the bound. So this reuses `compare`,
/// which enforces `EPS_POS` per instance, rather than inventing a looser bound
/// the arithmetic does not need. A dead reduction anywhere breaks the count or
/// moves the picture by tens of units — five orders of magnitude away.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_organism_matches_the_cpu_through_every_folded_stage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (g, out) = organism_chain(&reg, 12.0);
    let cpu = cook_cpu(&reg, &g, out);
    let gpu_out = cook_gpu(&gpu, &reg, &g, out);
    // `compare` enforces EPS_POS per instance and returns the worst; the folded
    // six-kernel chain fits the same bound as one deformer.
    compare("organism (6 kernels deep)", &cpu, &gpu_out);
}

/// **The deformer actually DEFORMS**, and the reduction is what makes it do so.
///
/// ⚠️ Without this the parity gates above are satisfiable by two paths that both
/// do nothing: a reduction stuck at the operator's identity sends `bend` down its
/// degenerate branch on BOTH sides (the CPU's `x_extent < MIN_ANGLE_RAD` and the
/// device's `bd_ext >= 1e-4`), so the grid stays flat, agrees to the bit, and the
/// suite is green over a feature that never ran. This measures the excursion from
/// the undeformed grid and demands it be large.
///
/// ⚠️ **This gate deliberately does NOT check that the deformation is RIGHT** —
/// that is the parity gates' job, and the split is load-bearing. Mutation-tested:
/// killing the fold entirely bleeds all four gates, while swapping the operator
/// (`Max` → `Min`) bleeds the three parity gates and leaves this one **green** —
/// a wrongly-reduced extent still moves the layout by tens of units. Asking one
/// gate both questions would mean neither answer is legible when it fails.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_device_deformers_move_the_layout_they_do_not_pass_it_through() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();

    // The undeformed layout: `angle = 0` is the documented identity of both.
    let (flat_g, flat_out) = chain(&reg, "motion.bend", 0.0, [0.0, 0.0]);
    let flat = cook_gpu(&gpu, &reg, &flat_g, flat_out);

    for (ty, angle) in [("motion.bend", 140.0f32), ("motion.twist", 200.0)] {
        let (g, out) = chain(&reg, ty, angle, [3.5, -1.25]);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        let worst = flat
            .iter()
            .zip(&dev)
            .map(|(a, b)| {
                (a.world_pos[0] - b.world_pos[0])
                    .abs()
                    .max((a.world_pos[1] - b.world_pos[1]).abs())
            })
            .fold(0.0f32, f32::max);
        // The grid spans ~45 units; a real deformation moves the rim by units,
        // and a dead one moves it by zero. The bar sits between, far from both.
        assert!(
            worst > 1.0,
            "{ty} at {angle}° must visibly deform the layout, moved only {worst:e} — \
             a reduction stuck at its identity looks exactly like this"
        );
        eprintln!("{ty}: worst excursion from the flat layout = {worst:.3}");
    }
}

/// **The reduction is a fact about the WHOLE stream, so it must survive the
/// block seam and the recursion** — the property the primitive's own gate proves
/// in isolation, asserted here through the PRODUCT.
///
/// A deformer is scale-independent by construction: `motion.twist` normalises by
/// the rim radius, so the outermost element always turns the full angle no matter
/// how many elements there are. That makes element count a free variable, and it
/// is the one that walks the reduction through one block, several, and a second
/// level — where an off-by-one in the seam is fatal and invisible on one block.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_deformers_agree_at_every_size_that_crosses_a_reduction_seam() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // Sides whose squares straddle the structural boundaries: 1 (a single
    // element), 15 (225 — a partial block), 16 (256 — exactly one block, no
    // recursion), 17 (289 — the first time a second level runs), 256 (65.536 —
    // the two-level boundary), 265 (70.225 — past it).
    for side in [1.0f32, 15.0, 16.0, 17.0, 256.0, 265.0] {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", side);
        g.set_param(grid, "cols", side);
        g.set_param(grid, "gap_x", 0.35);
        g.set_param(grid, "gap_y", 0.25);
        let def = g.add_node("motion.twist");
        g.set_param(def, "angle", 200.0);
        g.set_param(def, "pivot_x", 2.75);
        g.set_param(def, "pivot_y", 1.5);
        let out = g.add_node("motion.output");
        for (a, b) in [(grid, def), (def, out)] {
            g.connect(Edge {
                from: (a, 0),
                to: (b, 0),
                delayed: false,
            })
            .unwrap();
        }
        g.validate(&reg).expect("well-typed");

        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        compare(&format!("twist {side}x{side}"), &cpu, &dev);
    }
}

/// ⭐⭐⭐ **A MESMA SOMA, O MESMO DESVIO — o `motion.spherize` estava a UM TAMANHO
/// de reprovar** (ciclo 3, W1 — doc 106 §4).
///
/// O gate irmão corre a `SIDE = 128` (16 384 elementos) e mede `1,4e-6` contra a
/// barra de `2e-4`: **0,7 %**, que se lê como folga confortável. Ele mede um
/// TAMANHO, não a lei — e a lei é que o centroide da CPU é uma soma **sequencial
/// em `f32`** sobre parciais que chegam à magnitude do layout inteiro, enquanto o
/// dispositivo faz uma soma em **árvore**, que é a mais certa das duas. O erro
/// cresce com `n` e com a distância à origem.
///
/// ⚠️ *Uma folga medida num tamanho é uma afirmação sobre esse tamanho.* Esta
/// sonda varre os dois eixos e imprime a tabela; a cura (acumulador `f64` na
/// CPU, a mesma do `motion.transform`) põe todas as células abaixo de 1 %.
///
/// ```text
/// cargo test -p ph2d-gpu-cook --release --test gpu_cpu_parity_deform -- --ignored --nocapture measure_the_spherize_centroid_epsilon
/// ```
#[test]
#[ignore = "sonda de medição — corra à mão, com GPU"]
fn measure_the_spherize_centroid_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    eprintln!("\n  elementos | deslocamento | max |Δpos|  | % da barra (2e-4)");
    eprintln!("  ----------|--------------|-------------|------------------");
    for side in [128.0f32, 256.0, 512.0, 640.0] {
        for translate in [4.0f32, 40.0, 400.0] {
            let mut g = Graph::new();
            let grid = g.add_node("motion.grid");
            g.set_param(grid, "rows", side);
            g.set_param(grid, "cols", side);
            g.set_param(grid, "gap_x", 0.35);
            g.set_param(grid, "gap_y", 0.25);
            let mv = g.add_node("motion.move");
            g.set_param(mv, "dx", translate);
            g.set_param(mv, "dy", -translate * 0.6);
            let sph = g.add_node("motion.spherize");
            // Uma lente GRANDE — com o raio pequeno só um punhado de elementos
            // vê o centroide, e a sonda mediria a lente em vez da soma.
            g.set_param(sph, "radius", 60.0);
            let amt = g.add_node("value.lfo");
            g.set_param(amt, "amplitude", 0.0);
            g.set_param(amt, "offset", 0.8);
            let out = g.add_node("motion.output");
            for (from, to, port) in [(grid, mv, 0u16), (mv, sph, 0), (amt, sph, 1), (sph, out, 0)] {
                g.connect(Edge {
                    from: (from, 0),
                    to: (to, port),
                    delayed: false,
                })
                .unwrap();
            }
            g.validate(&reg).expect("well-typed");
            let cpu = cook_cpu(&reg, &g, out);
            let dev = cook_gpu(&gpu, &reg, &g, out);
            let mut worst = 0.0f32;
            for (c, d) in cpu.iter().zip(&dev) {
                for k in 0..2 {
                    worst = worst.max((c.world_pos[k] - d.world_pos[k]).abs());
                }
            }
            eprintln!(
                "  {:>9} | {translate:>12.1} | {worst:>11.3e} | {:>15.1}%",
                cpu.len(),
                worst / EPS_POS * 100.0
            );
        }
    }
    eprintln!();
}

/// ⭐⭐ **O PIVÔ-CENTROIDE DO CALEIDOSCÓPIO CHEGA AO DISPOSITIVO** (ciclo 3, W1 — doc 106).
///
/// ⚠️ **O layout é DESLOCADO a montante**, porque a grelha do arnês é centrada na origem e ali
/// o centroide é `(0, 0)`: o teste ficaria verde com o braço do centroide inteiro ausente do
/// kernel. É o mesmo buraco que a 1.ª redacção do gate irmão do `motion.transform` teve.
///
/// ⚠️ E o ponto digitado fica **deliberadamente longe** do centroide: o modo tem de vencer, e
/// um kernel que continuasse a ler `params.pivot_x` desenharia a estrela noutro sítio.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_kaleidoscope_centroid_pivot_rides_the_layout_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", KAL_SIDE as f32);
    g.set_param(grid, "cols", KAL_SIDE as f32);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 5.5);
    g.set_param(mv, "dy", -3.25);
    let kal = g.add_node("motion.kaleidoscope");
    g.set_param(kal, "segments", 5.0);
    g.set_param(kal, "reflect", 1.0);
    g.set_param(kal, "pivot_mode", 2.0);
    g.set_param(kal, "pivot_x", -9.0);
    g.set_param(kal, "pivot_y", 7.0);
    let spin = g.add_node("value.lfo");
    g.set_param(spin, "amplitude", 0.0);
    g.set_param(spin, "offset", 23.0);
    let out = g.add_node("motion.output");
    for (from, to, port) in [
        (grid, mv, 0u16),
        (mv, kal, 0),
        (spin, kal, 1),
        (kal, out, 0),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(&reg).expect("well-typed");
    let cpu = cook_cpu(&reg, &g, out);
    let dev = cook_gpu(&gpu, &reg, &g, out);
    assert_eq!(cpu.len(), KAL_SIDE * KAL_SIDE * 5, "n·segments");
    compare(
        "kaleidoscope pivot = Centroid, layout deslocado",
        &cpu,
        &dev,
    );
}

/// ⭐⭐ **O PIVÔ-CENTROIDE DO `motion.bend` CHEGA AO DISPOSITIVO** (ciclo 3, W1 — doc 106).
///
/// ⚠️ **A extensão em X passou a ser DERIVADA de duas reduções independentes do pivô**
/// (`max(xmax − p, p − xmin)` em vez de `Max(|x − p|)`), porque um `Max` que lê `params.pivot_x`
/// e um pivô que É uma redução seriam **uma redução a depender de outra**, e o sequenciador
/// corre-as todas no mesmo passo. A igualdade é ao bit — o gate irmão acima, que corre com o
/// ponto digitado, é o controlo dela.
///
/// ⚠️ E o layout é deslocado a montante: com a grelha centrada na origem o centroide é `(0,0)` e
/// o teste ficaria verde sobre um kernel sem o braço do centroide.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_bend_centroid_pivot_rides_the_layout_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 6.5);
    g.set_param(mv, "dy", -4.25);
    let bd = g.add_node("motion.bend");
    g.set_param(bd, "angle", 140.0);
    g.set_param(bd, "pivot_mode", 2.0);
    // Um ponto digitado bem longe do centroide: o MODO tem de vencer.
    g.set_param(bd, "pivot_x", -12.0);
    g.set_param(bd, "pivot_y", 8.0);
    let amt = g.add_node("value.lfo");
    g.set_param(amt, "amplitude", 0.0);
    g.set_param(amt, "offset", 1.0);
    let out = g.add_node("motion.output");
    for (from, to, port) in [(grid, mv, 0u16), (mv, bd, 0), (amt, bd, 1), (bd, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(&reg).expect("well-typed");
    let cpu = cook_cpu(&reg, &g, out);
    let dev = cook_gpu(&gpu, &reg, &g, out);
    compare("bend pivot = Centroid, layout deslocado", &cpu, &dev);
}

/// ⭐⭐⭐ **O PIVÔ-CENTROIDE DO `motion.twist` — a redução que LÊ outras duas** (ciclo 3, W1).
///
/// Este é o caso que obrigou o substrato a crescer. O `r_max` do twist mede um RAIO a partir do
/// pivô, e um raio euclidiano **não é separável** como a extensão em X do `motion.bend`
/// (`max|x−p| = max(xmax−p, p−xmin)`, que é exacta): com o pivô no modo `Centroid` a redução
/// depende de outra redução, e até esta wave o sequenciador não sabia encadeá-las. A saída de
/// recusar o dispositivo era exactamente o que a wave existe para desfazer.
///
/// ⚠️ O layout é deslocado a montante — sem isso o centroide é `(0,0)` e o gate ficaria verde
/// sobre um kernel que ignorasse o modo.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_twist_centroid_pivot_rides_the_layout_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 7.25);
    g.set_param(mv, "dy", -5.5);
    let tw = g.add_node("motion.twist");
    g.set_param(tw, "angle", 200.0);
    g.set_param(tw, "pivot_mode", 2.0);
    // Um ponto digitado longe do centroide: o MODO tem de vencer, no kernel E na redução.
    g.set_param(tw, "pivot_x", -14.0);
    g.set_param(tw, "pivot_y", 9.0);
    let amt = g.add_node("value.lfo");
    g.set_param(amt, "amplitude", 0.0);
    g.set_param(amt, "offset", 1.0);
    let out = g.add_node("motion.output");
    for (from, to, port) in [(grid, mv, 0u16), (mv, tw, 0), (amt, tw, 1), (tw, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(&reg).expect("well-typed");
    let cpu = cook_cpu(&reg, &g, out);
    let dev = cook_gpu(&gpu, &reg, &g, out);
    compare("twist pivot = Centroid, layout deslocado", &cpu, &dev);
}

/// ⭐⭐⭐ **O ESPELHO CHEGA AO DISPOSITIVO** (ciclo 3, W2 — doc 106 §2.2) — o último nó da
/// família TRANSFORM sem rota de GPU, e a lacuna era de **cobertura**, não de param: o irmão
/// `motion.kaleidoscope` percorre a mesma forma (`count → k·n`) desde que existe.
///
/// ⚠️ **As quatro células que a lei da contagem tem de acertar antes de qualquer posição:**
/// `Both` dá `2n` e `Reflection Only` dá `n`, em cada eixo. Um `count_law` errado é um número
/// diferente de coisas no ecrã, e o `compare` mede o comprimento primeiro.
///
/// ⚠️ E o layout é deslocado a montante: a linha de espelho É o centroide, então com a grelha
/// na origem o `offset` seria a única coisa a mexer e a redução ficaria por medir.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_mirror_reaches_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for (axis, keep, offset) in [
        (0.0f32, 0.0f32, 0.0f32),
        (1.0, 0.0, 0.0),
        (0.0, 0.0, 2.75),
        (0.0, 1.0, 0.0),
        (1.0, 1.0, -1.5),
    ] {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", SIDE);
        g.set_param(grid, "cols", SIDE);
        g.set_param(grid, "gap_x", 0.35);
        g.set_param(grid, "gap_y", 0.25);
        let mv = g.add_node("motion.move");
        g.set_param(mv, "dx", 5.0);
        g.set_param(mv, "dy", -3.5);
        let mr = g.add_node("motion.mirror");
        g.set_param(mr, "axis", axis);
        g.set_param(mr, "keep", keep);
        g.set_param(mr, "offset", offset);
        let out = g.add_node("motion.output");
        for (from, to) in [(grid, mv), (mv, mr), (mr, out)] {
            g.connect(Edge {
                from: (from, 0),
                to: (to, 0),
                delayed: false,
            })
            .unwrap();
        }
        g.validate(&reg).expect("well-typed");
        let n = (SIDE * SIDE) as usize;
        let cpu = cook_cpu(&reg, &g, out);
        assert_eq!(
            cpu.len(),
            if keep >= 0.5 { n } else { 2 * n },
            "a lei da contagem: keep {keep}"
        );
        let dev = cook_gpu(&gpu, &reg, &g, out);
        compare(
            &format!("mirror axis {axis} keep {keep} offset {offset}"),
            &cpu,
            &dev,
        );
    }
}

/// **E os dois knobs que RECUAM continuam a recuar** — a recusa é o contrato, com o mecanismo
/// escrito no `applicable`: as colunas que não são `P` chegam por um GATHER do template, e
/// tanto o `reindex` (que escreve `Index`/`Count` novos) como o `flip_rot` (que reflecte `rot`
/// e `vel` do gémeo) são outra operação.
#[test]
fn the_mirror_recuses_the_two_knobs_the_gather_cannot_serve() {
    let reg = registry();
    let k =
        ph2d_nodegraph::gpu::KernelResolver::gpu_kernel(&reg, ph2d_node_motion_mirror::MANIFEST.id)
            .expect("o espelho tem kernel");
    let applicable = k.applicable.expect("ele declara o predicado");
    let p = |reindex: f32, flip: f32| {
        move |name: &str| match name {
            "reindex" => reindex,
            "flip_rot" => flip,
            _ => 0.0,
        }
    };
    assert!(
        applicable(&p(0.0, 0.0)),
        "o caminho de omissao vai ao device"
    );
    assert!(!applicable(&p(1.0, 0.0)), "o reindex recua");
    assert!(!applicable(&p(0.0, 1.0)), "o flip_rot recua");
}

// ---------------------------------------------------------------------------
// `motion.bezier_warp` — ciclo 3, W5a (doc 106 §3)
// ---------------------------------------------------------------------------

/// Uma grelha, deformada pela fronteira curva, e a saída. `warp` é um `value.lfo`
/// constante (o caso autorado reproduzível), como no irmão.
///
/// `off` é `[tl, tr, br, bl]` seguido das oito tangentes na ordem
/// `top_a, top_b, right_a, right_b, bottom_a, bottom_b, left_a, left_b` — 24 números,
/// a superfície inteira do nó.
fn bezier_chain(reg: &NodeRegistry, warp: f32, off: [f32; 24]) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let bw = g.add_node("motion.bezier_warp");
    for (k, name) in BEZIER_PARAMS.iter().enumerate() {
        g.set_param(bw, *name, off[k]);
    }
    let amt = g.add_node("value.lfo");
    g.set_param(amt, "amplitude", 0.0);
    g.set_param(amt, "offset", warp);
    let out = g.add_node("motion.output");
    for (from, to, port) in [(grid, bw, 0u16), (amt, bw, 1), (bw, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(reg).expect("well-typed");
    (g, out)
}

/// Os 24 nomes, na ordem em que [`bezier_chain`] os consome.
///
/// ⚠️ **Derivada do MANIFESTO, nunca escrita à mão** — os params deste nó são
/// exactamente os 24 offsets, e uma lista literal aqui envelheceria em silêncio no dia
/// em que um deles fosse renomeado (o `set_param` de um nome desconhecido não é um
/// erro: é um param que ninguém lê).
static BEZIER_PARAMS: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
    ph2d_node_motion_bezier_warp::MANIFEST
        .params
        .iter()
        .map(|p| p.name)
        .collect()
});

/// Uma fronteira com as quatro bordas realmente curvas — o caso que o patch de Coons
/// existe para servir, e o único em que ele difere de toda homografia.
const BEZIER_BILLOW: [f32; 24] = [
    // cantos: TL, TR, BR, BL
    -0.4, 0.3, 0.5, 0.2, 0.3, -0.4, -0.2, -0.5, // tangentes TOP (a, b)
    0.1, 1.2, -0.1, 1.4, // RIGHT
    1.1, 0.2, 1.3, -0.2, // BOTTOM
    -0.1, -1.2, 0.1, -1.4, // LEFT
    -1.1, -0.2, -1.3, 0.2,
];

/// **A cadeia com a fronteira curva é reivindicada INTEIRA pelo planeador** —
/// device-free, logo corre em toda lane.
///
/// ⚠️ **Esta é a metade que nenhum gate numérico apanha.** Sem o kernel registado o nó
/// continua a cozinhar **certo** na CPU — a imagem fica correcta e a paridade abaixo
/// compara a CPU consigo mesma —, e a única coisa que muda é o `is_fully_gpu()`. Foi
/// exactamente essa a razão de o `motion.bezier_warp` ter shipado `CPU-only` durante um
/// ciclo inteiro sem nenhum vermelho: *um nó que cai para a CPU no meio de uma cadeia
/// custa o DISPOSITIVO todo (`50,9×`, doc 98) e não custa um pixel.*
#[test]
fn the_bezier_warp_reaches_the_device() {
    let reg = registry();
    let (g, out) = bezier_chain(&reg, 1.0, BEZIER_BILLOW);
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(
        plan.is_fully_gpu(),
        "a cadeia `grid → bezier_warp → output` tem de ser reivindicada inteira — \
         a fronteira curva é quatro reduções de caixa envolvente mais um patch \
         POLINOMIAL, a mesma forma que o `motion.four_point_warp` já corre"
    );
}

/// **O `motion.bezier_warp` no dispositivo concorda com o patch de Coons da CPU.**
///
/// ⚠️ **A caixa envolvente é `Min`/`Max`, exacta sobre floats**, então a redução não
/// carrega ε nenhum: tudo o que este gate mede é a aritmética do patch — quatro cúbicas
/// de Bernstein e a mistura bilinear —, que um dispositivo pode contrair em `fma` onde
/// o hospedeiro não contrai. A barra é a [`EPS_POS`] da família.
///
/// ⚠️ **O caso NEUTRO está aqui de propósito e não é redundante:** ele é o nó
/// recém-largado (os 24 offsets a zero), e é o ÚNICO caso em que os dois lados correm
/// ramos diferentes do corpo — o atalho da identidade. Sem ele, uma divergência de
/// `-0.0` no default do nó ficaria por medir para sempre.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_bezier_warp_deformer_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // Um lado só arqueado: as tangentes de TOP saem dos terços, o resto fica recto.
    let mut one_edge = [0.0f32; 24];
    one_edge[9] = 1.5; // top_a_dy
    one_edge[11] = 1.5; // top_b_dy
    let cases: [(&str, f32, [f32; 24]); 4] = [
        ("neutro (o nó recém-largado)", 1.0, [0.0; 24]),
        ("uma borda arqueada", 1.0, one_edge),
        ("as quatro bordas", 1.0, BEZIER_BILLOW),
        ("meio warp", 0.5, BEZIER_BILLOW),
    ];
    for (label, warp, off) in cases {
        let (g, out) = bezier_chain(&reg, warp, off);
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        compare(&format!("bezier_warp {label}"), &cpu, &dev);
    }
}

/// **E o deformador DEFORMA** — o controlo que impede o gate acima de ser satisfeito
/// por dois caminhos que ambos não fazem nada.
///
/// ⚠️ A armadilha aqui é concreta e já mordeu esta família: se as quatro reduções
/// ficassem presas na identidade do operador (`0.0`), a caixa seria um ponto, `w` e `h`
/// cairiam abaixo de `EPS` e **os dois lados** devolveriam o layout intacto — em
/// perfeito acordo, sobre um nó que nunca correu. Isto mede a excursão contra o mesmo
/// grafo no neutro.
#[test]
fn the_curved_boundary_actually_moves_the_layout() {
    let reg = registry();
    let (flat, out_flat) = bezier_chain(&reg, 1.0, [0.0; 24]);
    let (bent, out_bent) = bezier_chain(&reg, 1.0, BEZIER_BILLOW);
    let a = cook_cpu(&reg, &flat, out_flat);
    let b = cook_cpu(&reg, &bent, out_bent);
    assert_eq!(a.len(), b.len(), "a contagem não muda");
    let worst = a
        .iter()
        .zip(&b)
        .map(|(p, q)| {
            (p.world_pos[0] - q.world_pos[0])
                .abs()
                .max((p.world_pos[1] - q.world_pos[1]).abs())
        })
        .fold(0.0f32, f32::max);
    assert!(
        worst > 0.2,
        "a fronteira curva tem de mover o layout — pior excursão {worst}, e um valor \
         perto de zero significa que o patch não correu (caixa degenerada, redução \
         morta) e que a paridade acima compara duas identidades"
    );
}
