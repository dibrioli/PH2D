#![forbid(unsafe_code)]
//! `motion.verlet-rope` — a **sequential** rope/chain simulation: `count` points
//! linked by fixed-length segments, integrated with position-based Verlet and
//! relaxed against distance constraints, hanging from a pinned anchor that can be
//! ANIMATED (Motion Nodes M4, simulation — doc 01 §3 / doc 21). Slide the anchor
//! and the rope whips and swings behind it with real follow-through; let it hang
//! and gravity pulls it into a catenary. The canonical "cloth strand / whip /
//! pendulum chain" of every motion pack.
//!
//! **Algorithm — Jakobsen's *Advanced Character Physics* (2001), the gold
//! standard.** Verlet stores each point's *current* and *previous* position; the
//! step is `x' = x + (x − x_prev)·(1−damp) + a·dt²` (velocity is implicit in the
//! position pair, so there is no separate velocity state to diverge), then a few
//! **relaxation passes** pull each segment back to its rest length. It is
//! unconditionally stable (unlike explicit spring meshes, which explode when
//! stiff) and dead simple. Position-based constraints are exactly why cloth/rope
//! sims use Verlet rather than force springs.
//!
//! ## Topology (the `pre` self-loop)
//!
//! A sequential node like `motion.spring`/`motion.integrate`: its state (the point
//! positions + their previous positions) rides its own output through the `state`
//! feedback port, which the editor auto-plumbs as the `pre` self-loop on add
//! (`out --pre--> state`). At tick 0 the `pre` reads Empty → the node **seeds** a
//! straight strand from the anchor; from tick 1 on it steps. `dt` derives from the
//! state's own `sim_t` column (playhead at last eval), clamped to `[0, MAX_DT]` —
//! no cross-crate timestep, and a backwards playhead jump (loop wrap) freezes for
//! one tick instead of exploding, exactly like `motion.integrate`.
//!
//! ## The anchor is animatable (value inputs)
//!
//! `anchor_x`/`anchor_y` are **value** inputs (doc 12): wire a `value.lfo` and the
//! pinned head slides, whipping the whole chain. Unconnected, the anchor reads as
//! the origin `(0, 0)`. `pin_tail` optionally also pins the far end to a fixed
//! point (a washing-line / suspension bridge) instead of leaving it free (a whip).
//!
//! Transcendental-free (HR-5): Verlet integration + constraint relaxation are
//! arithmetic and one IEEE `sqrt` per segment (the segment length), like
//! `motion.spring`. Deterministic → `Effect::Temporal` (reads the playhead), replays
//! bit-for-bit.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `anchor_*` inputs — the per-instance scalar field on the
/// `v` column (mirror of `motion.look_at::VALUE`; kept local, leaf crate).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Ceiling on a single integration step (see `motion.integrate`): in steady state
/// `dt` is the fixed timestep; this only guards a pathological playhead jump.
const MAX_DT: f32 = 0.1;
/// Below this a segment length is treated as zero (skip the normalise).
const EPS: f32 = 1e-6;
/// With `pin_tail`, the far end is fixed at this fraction of the rope length from
/// the head — leaving 25% slack so the span sags into a catenary instead of
/// pulling taut into a straight line.
const PINNED_SPAN: f32 = 0.75;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.verlet_rope"),
    name: "motion.verlet_rope",
    inputs: &[
        // The pinned head, as two value fields (so it can be animated). Optional:
        // unconnected reads as 0 → the origin.
        PortSpec {
            name: "anchor_x",
            ty: VALUE,
        },
        PortSpec {
            name: "anchor_y",
            ty: VALUE,
        },
        // The feedback port — auto-wired `out --pre--> state` on add.
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Temporal: `eval` reads `ctx.playhead()` (stamps `sim_t`, derives `dt`).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Number of points in the chain. Clamped ≥2 and capped like the grid.
        ParamSpec {
            name: "count",
            default: 24.0,
        },
        // Total rest length (world units); segment rest = length / (count−1).
        ParamSpec {
            name: "length",
            default: 6.0,
        },
        // Downward gravity magnitude (accel, world units/s²).
        ParamSpec {
            name: "gravity",
            default: 9.0,
        },
        // Constraint relaxation passes — higher = stiffer/less stretchy.
        ParamSpec {
            name: "iterations",
            default: 24.0,
        },
        // Verlet velocity damping per step ∈ [0,1) — settles the swing.
        ParamSpec {
            name: "damping",
            default: 0.02,
        },
        // 0 = free tail (a whip); 1 = the far end is pinned too (a bridge/line).
        ParamSpec {
            name: "pin_tail",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The value coordinate for element 0 (the anchor): **unconnected (empty) → 0.0**;
/// otherwise the first element (broadcast).
fn value_head(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.0)
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn vec2_col(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The simulation parameters resolved at eval (all arithmetic-ready).
struct Params {
    count: usize,
    seg_rest: f32,
    gravity: f32,
    iterations: usize,
    damping: f32,
    pin_tail: bool,
}

/// Seed a straight, horizontal strand of `count` points from `anchor`, pinned at
/// index 0 (previous == current → at rest). The first gravity step then swings it.
fn seed(anchor: [f32; 2], p: &Params) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let pos: Vec<[f32; 2]> = (0..p.count)
        .map(|i| [anchor[0] + i as f32 * p.seg_rest, anchor[1]])
        .collect();
    let prev = pos.clone();
    (pos, prev)
}

/// One Verlet step + constraint relaxation as a pure function.
///
/// `pos`/`prev` are this tick's entry state (last tick's positions). Returns the
/// new `(pos, prev)`: `prev_out` = the entry `pos` (Verlet's memory), `pos_out` =
/// the integrated + relaxed positions. Pinned points (head, and tail if
/// `pin_tail`) are clamped to their fixed targets every pass.
fn step(
    mut pos: Vec<[f32; 2]>,
    prev: &[[f32; 2]],
    anchor: [f32; 2],
    tail_pin: [f32; 2],
    dt: f32,
    p: &Params,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let n = pos.len();
    // Verlet's memory for the NEXT tick is this tick's entry positions.
    let prev_out = pos.clone();
    let keep = 1.0 - p.damping;
    let ga = p.gravity * dt * dt; // a·dt² (downward, so subtract from y)

    // Integrate every point; the pins are overwritten right after.
    for i in 0..n {
        let (c, pv) = (pos[i], prev[i]);
        let mut np = [
            c[0] + (c[0] - pv[0]) * keep,
            c[1] + (c[1] - pv[1]) * keep - ga,
        ];
        // NaN/∞ guard (reference parity): a diverged point recovers at the anchor.
        if !(np[0].is_finite() && np[1].is_finite()) {
            np = anchor;
        }
        pos[i] = np;
    }
    pin(&mut pos, anchor, tail_pin, p);

    // Relaxation: pull each segment back to its rest length. A pinned endpoint
    // holds; a free one takes the full correction, else each takes half.
    for _ in 0..p.iterations {
        for i in 0..n.saturating_sub(1) {
            let (a, b) = (pos[i], pos[i + 1]);
            let d = [b[0] - a[0], b[1] - a[1]];
            let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if dist < EPS {
                continue;
            }
            let diff = (dist - p.seg_rest) / dist;
            let (pa, pb) = (is_pinned(i, n, p), is_pinned(i + 1, n, p));
            let (wa, wb) = match (pa, pb) {
                (true, true) => (0.0, 0.0),
                (true, false) => (0.0, 1.0),
                (false, true) => (1.0, 0.0),
                (false, false) => (0.5, 0.5),
            };
            pos[i] = [a[0] + d[0] * diff * wa, a[1] + d[1] * diff * wa];
            pos[i + 1] = [b[0] - d[0] * diff * wb, b[1] - d[1] * diff * wb];
        }
        pin(&mut pos, anchor, tail_pin, p);
    }
    (pos, prev_out)
}

/// Whether point `i` of `n` is pinned (head always; tail when `pin_tail`).
fn is_pinned(i: usize, n: usize, p: &Params) -> bool {
    i == 0 || (p.pin_tail && i + 1 == n)
}

/// Clamp the pinned points to their fixed targets.
fn pin(pos: &mut [[f32; 2]], anchor: [f32; 2], tail_pin: [f32; 2], p: &Params) {
    if let Some(head) = pos.first_mut() {
        *head = anchor;
    }
    if p.pin_tail
        && let Some(tail) = pos.last_mut()
    {
        *tail = tail_pin;
    }
}

/// The whole node as a pure function: seed on the first tick / a count change,
/// else step. Returns the emitted stream (`P` + the `rope_prev`/`sim_t` state).
fn simulate(anchor: [f32; 2], state: &Stream, playhead: f32, p: &Params) -> Stream {
    let s_pos = vec2_col(state, "P");
    let s_prev = vec2_col(state, "rope_prev");
    // The tail's fixed point sits at PINNED_SPAN of the rope length along +x from
    // the head, so the extra length sags. Derived (not stored) so it never drifts.
    let tail_pin = [
        anchor[0] + PINNED_SPAN * (p.count as f32 - 1.0) * p.seg_rest,
        anchor[1],
    ];

    let (pos, prev) = if s_pos.len() == p.count && s_prev.len() == p.count {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        step(s_pos, &s_prev, anchor, tail_pin, dt, p)
    } else {
        // Tick 0 (Empty state) or a count change → re-seed the strand.
        seed(anchor, p)
    };

    Stream::new(pos.len())
        .with("P", Column::Vec2(pos))
        .with("rope_prev", Column::Vec2(prev))
        .with("sim_t", Column::Scalar(vec![playhead; p.count]))
}

struct MotionVerletRope;

impl NodeOp for MotionVerletRope {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS).max(2);
        let length = ctx.param("length").max(0.0);
        let p = Params {
            count,
            seg_rest: length / (count as f32 - 1.0),
            gravity: ctx.param("gravity"),
            iterations: (ctx.param("iterations").round() as i64).clamp(1, 128) as usize,
            damping: ctx.param("damping").clamp(0.0, 0.99),
            pin_tail: ctx.param("pin_tail") >= 0.5,
        };
        let playhead = ctx.playhead() as f32;
        let anchor = [
            value_head(&scalar_col(ctx.input(0), VALUE_COL)),
            value_head(&scalar_col(ctx.input(1), VALUE_COL)),
        ];
        let state = ctx.input(2);
        let out = simulate(anchor, state, playhead, &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionVerletRope))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Verlet Rope",
            // Source: it mints its own point stream (like Grid / Scatter), then
            // simulates it.
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Points",
        min: 2.0,
        max: 200.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "length",
        label: "Length",
        min: 0.5,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "gravity",
        label: "Gravity",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "iterations",
        label: "Stiffness",
        min: 1.0,
        max: 128.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "damping",
        label: "Damping",
        min: 0.0,
        max: 0.5,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pin_tail",
        label: "Pin Tail",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Free", "Pinned"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn params(count: usize, length: f32, gravity: f32, pin_tail: bool) -> Params {
        Params {
            count,
            seg_rest: length / (count as f32 - 1.0),
            gravity,
            iterations: 32,
            damping: 0.0,
            pin_tail,
        }
    }

    /// The longest segment length in the chain (the stretch we constrain).
    fn max_seg(pos: &[[f32; 2]]) -> f32 {
        pos.windows(2)
            .map(|w| {
                let d = [w[1][0] - w[0][0], w[1][1] - w[0][1]];
                (d[0] * d[0] + d[1] * d[1]).sqrt()
            })
            .fold(0.0, f32::max)
    }

    /// Drive `ticks` fixed steps through the pure `simulate`, feeding each tick's
    /// output back as the next tick's state (what the `pre` loop does live).
    fn run(anchor: impl Fn(f32) -> [f32; 2], p: &Params, ticks: usize, dt: f32) -> Vec<Stream> {
        let mut state = Stream::new(0); // Empty → tick 0 seeds
        let mut frames = Vec::new();
        for k in 0..ticks {
            let t = k as f32 * dt;
            let out = simulate(anchor(t), &state, t, p);
            state = out.clone();
            frames.push(out);
        }
        frames
    }

    fn pos_of(s: &Stream) -> Vec<[f32; 2]> {
        vec2_col(s, "P")
    }

    /// Tick 0 seeds a straight horizontal strand from the anchor — the pin at the
    /// anchor, each point one rest-length along +x, all collinear.
    #[test]
    fn seeds_a_straight_strand_at_tick_zero() {
        let p = params(5, 4.0, 9.0, false);
        let out = simulate([1.0, 2.0], &Stream::new(0), 0.0, &p);
        let pos = pos_of(&out);
        assert_eq!(pos.len(), 5);
        assert_eq!(pos[0], [1.0, 2.0], "head pinned at the anchor");
        for (i, q) in pos.iter().enumerate() {
            assert!((q[0] - (1.0 + i as f32)).abs() < 1e-5, "point {i} along +x");
            assert!((q[1] - 2.0).abs() < 1e-5, "flat (no gravity yet)");
        }
    }

    /// The defining property: the segments stay at their rest length even after
    /// the rope has swung for a while under gravity. FALSIFIED against no
    /// relaxation — an unconstrained Verlet chain stretches without bound as
    /// gravity accelerates it, so the longest segment blows past the rest length.
    #[test]
    fn the_constraint_holds_the_segment_lengths() {
        let p = params(16, 6.0, 9.0, false);
        let frames = run(|_| [0.0, 0.0], &p, 200, 1.0 / 60.0);
        let last = pos_of(frames.last().unwrap());
        let longest = max_seg(&last);
        assert!(
            longest < p.seg_rest * 1.05,
            "segments held near rest {} (longest {longest}); an unconstrained chain stretches",
            p.seg_rest
        );
        // And it really did move (gravity did work) — the tail fell well below.
        assert!(
            last[15][1] < -1.0,
            "the free tail hangs down: {}",
            last[15][1]
        );
    }

    /// Gravity pulls the free tail DOWN below the pinned head over time. A dead
    /// (zero-gravity) rope stays flat — the falsification.
    #[test]
    fn gravity_hangs_the_free_tail_below_the_anchor() {
        let live = params(12, 5.0, 12.0, false);
        let dead = params(12, 5.0, 0.0, false);
        let tail = |p: &Params| pos_of(run(|_| [0.0, 0.0], p, 180, 1.0 / 60.0).last().unwrap())[11];
        assert!(tail(&live)[1] < -1.0, "live gravity: tail sinks");
        assert!(
            (tail(&dead)[1]).abs() < 1e-3,
            "zero gravity: the tail stays flat"
        );
    }

    /// The pinned head tracks the (animated) anchor EXACTLY, every tick — the
    /// whip's handle. A moving anchor also drags the rest of the chain along
    /// (the tail is pulled well away from where a still anchor would leave it).
    #[test]
    fn a_moving_anchor_pins_the_head_and_whips_the_chain() {
        let p = params(14, 5.0, 6.0, false);
        // Anchor slides in +x with time.
        let frames = run(|t| [3.0 * t, 0.0], &p, 120, 1.0 / 60.0);
        for (k, f) in frames.iter().enumerate() {
            let head = pos_of(f)[0];
            let t = k as f32 / 60.0;
            assert!(
                (head[0] - 3.0 * t).abs() < 1e-4 && head[1].abs() < 1e-4,
                "head glued to the anchor at tick {k}: {head:?}"
            );
        }
        // The dragged rope ends up displaced in +x vs a still-anchored one.
        let still = run(|_| [0.0, 0.0], &p, 120, 1.0 / 60.0);
        let moved_tail = pos_of(frames.last().unwrap())[13][0];
        let still_tail = pos_of(still.last().unwrap())[13][0];
        assert!(
            moved_tail > still_tail + 0.5,
            "the moving anchor dragged the chain (+x {moved_tail} vs {still_tail})"
        );
    }

    /// `pin_tail` fixes BOTH ends: the tail holds its far point instead of falling,
    /// and the middle sags between them (a suspension line).
    #[test]
    fn pin_tail_fixes_both_ends_into_a_hanging_line() {
        let p = params(11, 6.0, 12.0, true);
        let last = pos_of(run(|_| [0.0, 0.0], &p, 200, 1.0 / 60.0).last().unwrap());
        let far = PINNED_SPAN * (p.count as f32 - 1.0) * p.seg_rest;
        assert_eq!(last[0], [0.0, 0.0], "head pinned");
        assert!(
            (last[10][0] - far).abs() < 1e-3,
            "tail pinned at its far point"
        );
        assert!((last[10][1]).abs() < 1e-3, "tail holds its height");
        // The 25% slack sags the middle well below the endpoints.
        assert!(
            last[5][1] < -0.4,
            "the span sags in the middle: {}",
            last[5][1]
        );
    }

    /// Deterministic replay (HR-5: arithmetic + IEEE sqrt only): two runs of the
    /// same rope produce bit-identical trajectories.
    #[test]
    fn replay_is_deterministic() {
        let p = params(20, 6.0, 9.0, false);
        let a = run(|t| [2.0 * t, 0.0], &p, 90, 1.0 / 60.0);
        let b = run(|t| [2.0 * t, 0.0], &p, 90, 1.0 / 60.0);
        for (fa, fb) in a.iter().zip(&b) {
            assert_eq!(pos_of(fa), pos_of(fb));
        }
    }

    /// Without the `pre` state loop the node re-seeds every tick → a straight,
    /// still strand (the reference's "only simulates with feedback" footnote).
    #[test]
    fn without_the_state_loop_it_stays_a_straight_strand() {
        let p = params(8, 5.0, 12.0, false);
        for k in 0..30 {
            // Empty state every tick.
            let out = simulate([0.0, 0.0], &Stream::new(0), k as f32 / 60.0, &p);
            let pos = pos_of(&out);
            assert!(pos.iter().all(|q| q[1].abs() < 1e-6), "flat, no sim at {k}");
        }
    }

    /// A poisoned (non-finite) state point recovers at the anchor instead of
    /// freezing the whole chain (reference NaN guard).
    #[test]
    fn non_finite_state_recovers() {
        let p = params(4, 3.0, 9.0, false);
        let state = Stream::new(4)
            .with(
                "P",
                Column::Vec2(vec![[0.0, 0.0], [f32::NAN, 1.0], [2.0, 0.0], [3.0, 0.0]]),
            )
            .with("rope_prev", Column::Vec2(vec![[0.0, 0.0]; 4]))
            .with("sim_t", Column::Scalar(vec![0.0; 4]));
        let out = simulate([0.0, 0.0], &state, 1.0 / 60.0, &p);
        assert!(
            pos_of(&out)
                .iter()
                .all(|q| q[0].is_finite() && q[1].is_finite()),
            "the diverged point was reset, not propagated"
        );
    }

    /// Cooks through the registry with the `pre` self-loop, exactly as the editor
    /// wires it — proving the node is registered and steps live.
    #[test]
    fn registers_and_steps_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionVerletRope as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let rope = g.add_node("motion.verlet_rope");
        g.set_param(rope, "count", 10.0);
        g.set_param(rope, "gravity", 12.0);
        // The pre self-loop (out --pre--> state), as the plumbing auto-wires it.
        g.connect(Edge {
            from: (rope, 0),
            to: (rope, 2),
            delayed: true,
        })
        .unwrap();

        let mut cook = Cook::new();
        // Tick 0: seeded flat.
        let out0 = cook.cook(&g, &Ops, rope, 0.0).unwrap();
        assert!(matches!(out0[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 10));
        // Advance a second of frames; the tail must have fallen.
        for k in 0..60 {
            let t = k as f64 / 60.0;
            cook.cook(&g, &Ops, rope, t).unwrap();
            cook.advance_tick(&g, &Ops, t).unwrap();
        }
        let out = cook.cook(&g, &Ops, rope, 1.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => {
                assert!(v[9][1] < -0.5, "the tail hangs after a second: {}", v[9][1])
            }
            _ => panic!("P"),
        }
    }
}
