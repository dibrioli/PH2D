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
    ph2d_node_motion_kaleidoscope::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
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
    for (angle, pivot) in [(90.0, [0.0, 0.0]), (140.0, [3.5, -1.25]), (-70.0, [-2.0, 0.75])] {
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
    for (angle, pivot) in [(90.0, [0.0, 0.0]), (200.0, [2.75, 1.5]), (-120.0, [-1.0, -3.0])] {
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
    // shifted by `offset` so the centroid is off-origin.
    for (amount, radius, offset) in [
        (0.8f32, 6.0f32, [4.0f32, -2.5]),
        (-0.6, 9.0, [-3.0, 1.5]),
        (0.5, 4.0, [0.0, 0.0]),
    ] {
        let (g, out) = spherize_chain(&reg, amount, radius, offset);
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        compare(
            &format!("spherize amount {amount} radius {radius} offset {offset:?}"),
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
        ("projective", 1.0, [2.0, 1.0, -1.0, 2.5, 1.5, -2.0, -2.5, -0.5]),
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
fn spherize_chain(reg: &NodeRegistry, amount: f32, radius: f32, offset: [f32; 2]) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", offset[0]);
    g.set_param(mv, "dy", offset[1]);
    let sph = g.add_node("motion.spherize");
    g.set_param(sph, "radius", radius);
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
