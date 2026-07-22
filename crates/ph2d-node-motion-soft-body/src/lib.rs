#![forbid(unsafe_code)]
//! `motion.soft_body` — a **shape-matching** deformable body: a `rows×cols` mesh of
//! particles that squashes, stretches and JIGGLES back to its rest shape, like jelly
//! (Motion Nodes M4, simulation — doc 01 §3 / doc 22). The continuum-media
//! counterpart to the discrete `motion.verlet_rope`/`motion.boids`: a 2D body that
//! deforms as a whole and always recovers its form.
//!
//! **Algorithm.** The *shape-matching constraint* is Müller et al., *Meshless
//! Deformations Based on Shape Matching* (SIGGRAPH 2005): find the single best-fit
//! frame `(M, c)` of the rest shape to the deformed cloud, whose *goal* for each
//! particle is `gᵢ = M·qᵢ + c` (`qᵢ = xᵢ⁰ − c₀`). `M` is the **rigid** rotation `R`
//! (the polar factor of `A_pq = Σ pᵢ qᵢᵀ`), optionally blended with the paper's
//! area-preserved **linear** map `A = A_pq A_qq⁻¹` by `stretch` (`M = β·A + (1−β)·R`)
//! for squash & stretch. The frame math lives in the `shape` sibling module.
//!
//! The *integration* is the **Position-Based Dynamics** reformulation of shape
//! matching (Müller et al. 2007; Matthias Müller's "Ten Minute Physics"), NOT the
//! 2005 paper's velocity-blend `v += α(g−x)/h`: predict under gravity + inertia,
//! project each particle a fraction `stiffness` toward its goal (computed from the
//! *predicted* cloud), then read velocity back as `v = (x_new − x_old)/dt`. Same
//! author lineage, and the modern-standard, unconditionally-stable scheme (the goal
//! is always a valid pose — nothing to explode; no per-edge constraint list).
//!
//! Masses are uniform here (`wᵢ = mᵢ = 1` — exact for this even grid, so the paper's
//! mass-weighted centroid/`A_pq` reduce to the plain sums). The 2D polar factor has
//! a closed form with NO trig, so `R` costs one `sqrt` (HR-5).
//!
//! ## Topology (the `pre` self-loop)
//!
//! Sequential like the other sims: each particle's `P` and `sb_vel` ride the `state`
//! feedback port, auto-plumbed as the `pre` self-loop on add. Tick 0 (`pre` = Empty)
//! **seeds** the rest mesh at the anchor; from tick 1 it steps. `dt` derives from
//! the state's own `sim_t`, clamped to `[0, MAX_DT]` (a loop-wrap freezes one tick).
//!
//! ## The anchor pins the top row (value inputs)
//!
//! `anchor_x`/`anchor_y` are **value** inputs: with `pin` on, the top row is held
//! rigid at the anchor (a hanging jelly flag), and sliding the anchor with a
//! `value.lfo` wobbles the whole body. Unconnected → the origin. `pin` off lets the
//! body fall freely.
//!
//! Transcendental-free (HR-5): prediction, the 2×2 polar decomposition (`sqrt` only)
//! and the goal pull are arithmetic. Deterministic → `Effect::Temporal`, replays
//! bit-for-bit.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod shape;
use shape::{rest_shape, shape_goals};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `anchor_*` inputs (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Ceiling on a single step (see `motion.integrate`): guards a playhead jump.
const MAX_DT: f32 = 0.1;
/// Below this a magnitude is treated as zero (skip the normalise / division).
const EPS: f32 = 1e-6;
/// Grid dimensions are clamped to this many cells per side (mesh cost is O(rows·cols)).
const MAX_SIDE: i64 = 40;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.soft_body"),
    name: "motion.soft_body",
    inputs: &[
        // The pin anchor, as two value fields (animatable). Optional: unconnected → 0.
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
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "rows",
            default: 4.0,
        },
        ParamSpec {
            name: "cols",
            default: 4.0,
        },
        ParamSpec {
            name: "spacing",
            default: 0.7,
        },
        ParamSpec {
            name: "gravity",
            default: 9.0,
        },
        // Shape recovery ∈ [0,1]: 1 = rigid snap-back, low = slow gooey wobble.
        ParamSpec {
            name: "stiffness",
            default: 0.4,
        },
        // Linear-deformation blend β ∈ [0,1] (Müller 2005): 0 = rigid, higher =
        // more squash & stretch (area-preserved). Default 0 → pure rigid.
        ParamSpec {
            name: "stretch",
            default: 0.0,
        },
        ParamSpec {
            name: "damping",
            default: 0.03,
        },
        // 1 = pin the top row to the anchor (a hanging jelly); 0 = free fall.
        ParamSpec {
            name: "pin",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Resolved parameters (arithmetic-ready).
struct Params {
    rows: usize,
    cols: usize,
    spacing: f32,
    gravity: f32,
    stiffness: f32,
    /// The Müller 2005 linear-deformation blend `β` ∈ [0,1]: 0 = pure rigid (holds
    /// shape), higher = more squash & stretch (area-preserved).
    beta: f32,
    damping: f32,
    pin: bool,
}

impl Params {
    /// Whether particle `i` is pinned (the top row, indices `0..cols`, when `pin`).
    fn is_pinned(&self, i: usize) -> bool {
        self.pin && i < self.cols
    }
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

fn value_head(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.0)
}

/// The pinned target for the top-row particle at grid column `c`: the anchor plus
/// that particle's rest offset, so the top edge stays a rigid bar sliding with the
/// anchor.
fn pin_target(anchor: [f32; 2], rest: &[[f32; 2]], i: usize) -> [f32; 2] {
    [anchor[0] + rest[i][0], anchor[1] + rest[i][1]]
}

/// One shape-matching step as a pure function. `pos`/`vel` are this tick's entry
/// state; returns the new `(pos, vel)`.
fn step(
    pos: &[[f32; 2]],
    vel: &[[f32; 2]],
    anchor: [f32; 2],
    rest: &[[f32; 2]],
    dt: f32,
    p: &Params,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let n = pos.len();
    // Predict under inertia + gravity; pinned particles jump to their pin target.
    let mut pred = vec![[0.0f32; 2]; n];
    for i in 0..n {
        if p.is_pinned(i) {
            pred[i] = pin_target(anchor, rest, i);
        } else {
            pred[i] = [
                pos[i][0] + vel[i][0] * dt,
                pos[i][1] + vel[i][1] * dt - p.gravity * dt * dt,
            ];
        }
    }
    // Match the rest shape and pull each particle a fraction toward its goal.
    let goals = shape_goals(&pred, rest, p.beta);
    let mut out_pos = vec![[0.0f32; 2]; n];
    let mut out_vel = vec![[0.0f32; 2]; n];
    let keep = 1.0 - p.damping;
    for i in 0..n {
        let mut np = if p.is_pinned(i) {
            pred[i] // pinned particles stay exactly on the pin
        } else {
            [
                pred[i][0] + (goals[i][0] - pred[i][0]) * p.stiffness,
                pred[i][1] + (goals[i][1] - pred[i][1]) * p.stiffness,
            ]
        };
        // Velocity from the position change (this is what jiggles).
        let mut nv = [
            (np[0] - pos[i][0]) / dt * keep,
            (np[1] - pos[i][1]) / dt * keep,
        ];
        if !(np[0].is_finite() && np[1].is_finite() && nv[0].is_finite() && nv[1].is_finite()) {
            np = pin_target(anchor, rest, i); // NaN guard: recover on the rest frame
            nv = [0.0, 0.0];
        }
        out_pos[i] = np;
        out_vel[i] = nv;
    }
    (out_pos, out_vel)
}

/// The whole node as a pure function: seed on the first tick / a shape change,
/// else step. Emits `P` + the `sb_vel`/`sim_t` state.
fn simulate(anchor: [f32; 2], state: &Stream, playhead: f32, p: &Params) -> Stream {
    let rest = rest_shape(p.rows, p.cols, p.spacing);
    let n = rest.len();
    let s_pos = vec2_col(state, "P");
    let s_vel = vec2_col(state, "sb_vel");

    let (pos, vel) = if s_pos.len() == n && s_vel.len() == n {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        if dt < EPS {
            (s_pos, s_vel) // loop-wrap / same tick → hold
        } else {
            step(&s_pos, &s_vel, anchor, &rest, dt, p)
        }
    } else {
        // Seed the rest mesh at the anchor, at rest (zero velocity).
        let seed: Vec<[f32; 2]> = rest
            .iter()
            .map(|q| [q[0] + anchor[0], q[1] + anchor[1]])
            .collect();
        (seed, vec![[0.0, 0.0]; n])
    };

    Stream::new(pos.len())
        .with("P", Column::Vec2(pos))
        .with("sb_vel", Column::Vec2(vel))
        .with("sim_t", Column::Scalar(vec![playhead; n]))
}

struct MotionSoftBody;

impl NodeOp for MotionSoftBody {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let side = |name: &str| (ctx.param(name).round() as i64).clamp(2, MAX_SIDE) as usize;
        let p = Params {
            rows: side("rows"),
            cols: side("cols"),
            spacing: ctx.param("spacing").max(1e-3),
            gravity: ctx.param("gravity"),
            stiffness: ctx.param("stiffness").clamp(0.0, 1.0),
            beta: ctx.param("stretch").clamp(0.0, 1.0),
            damping: ctx.param("damping").clamp(0.0, 0.99),
            pin: ctx.param("pin") >= 0.5,
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
    reg.register(Box::new(MotionSoftBody))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Soft Body",
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
        param: "rows",
        label: "Rows",
        min: 2.0,
        max: 40.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cols",
        label: "Cols",
        min: 2.0,
        max: 40.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: 0.1,
        max: 4.0,
        step: 0.05,
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
        param: "stiffness",
        label: "Stiffness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "stretch",
        label: "Stretch",
        min: 0.0,
        max: 1.0,
        step: 0.01,
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
        param: "pin",
        label: "Pin Top",
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

    fn params(rows: usize, cols: usize, gravity: f32, stiffness: f32, pin: bool) -> Params {
        Params {
            rows,
            cols,
            spacing: 0.7,
            gravity,
            stiffness,
            beta: 0.0,
            damping: 0.0,
            pin,
        }
    }

    fn run(
        anchor: impl Fn(f32) -> [f32; 2],
        p: &Params,
        ticks: usize,
        dt: f32,
    ) -> Vec<Vec<[f32; 2]>> {
        let mut state = Stream::new(0);
        let mut frames = Vec::new();
        for k in 0..ticks {
            let t = k as f32 * dt;
            let out = simulate(anchor(t), &state, t, p);
            state = out.clone();
            frames.push(vec2_col(&out, "P"));
        }
        frames
    }

    /// Tick 0 seeds the rest mesh at the anchor: `rows·cols` particles, the top row
    /// (indices `0..cols`) sitting at the anchor row.
    #[test]
    fn seeds_the_rest_mesh_at_the_anchor() {
        let p = params(3, 4, 9.0, 0.5, true);
        let out = simulate([2.0, 1.0], &Stream::new(0), 0.0, &p);
        let pos = vec2_col(&out, "P");
        assert_eq!(pos.len(), 12, "3×4 mesh");
        // Top row is at the top (max y); rows descend. Centre column near anchor x.
        let top_y = pos[0][1];
        let bottom_y = pos[11][1];
        assert!(top_y > bottom_y, "row 0 is the top");
        // The mesh is centred on the anchor: mean position ≈ anchor.
        let mean = pos
            .iter()
            .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
        let mean = [mean[0] / 12.0, mean[1] / 12.0];
        assert!((mean[0] - 2.0).abs() < 1e-4 && (mean[1] - 1.0).abs() < 1e-4);
    }

    // (The pure shape-match geometry — rigid invariance, shape recovery, and the
    // `beta` linear-deformation mode — is falsified in the `shape` module's own
    // tests. Here we exercise the full sequential simulation.)

    /// Gravity + pin: the pinned top row holds at the anchor while the free body
    /// sags below it over time. FALSIFIED by a dead body (no gravity) — it stays put.
    #[test]
    fn gravity_hangs_the_body_below_the_pinned_top() {
        let live = params(4, 3, 12.0, 0.5, true);
        let bottom_y = |p: &Params| {
            let last = run(|_| [0.0, 0.0], p, 180, 1.0 / 60.0);
            let pos = last.last().unwrap();
            // Bottom row's mean y.
            let (rows, cols) = (p.rows, p.cols);
            let start = (rows - 1) * cols;
            pos[start..].iter().map(|q| q[1]).sum::<f32>() / cols as f32
        };
        assert!(bottom_y(&live) < -0.5, "the body sags below the pin");
        // The pinned top row stays at the anchor height (y = +half).
        let top = run(|_| [0.0, 0.0], &live, 180, 1.0 / 60.0);
        let top_row = &top.last().unwrap()[0..live.cols];
        for q in top_row {
            assert!(q[1].abs() < 0.5 + 1.0, "top row pinned near the anchor row");
        }
    }

    /// A moving anchor drags the jelly: sliding the pin carries the whole body along.
    #[test]
    fn a_moving_anchor_drags_the_body() {
        let p = params(4, 3, 9.0, 0.5, true);
        let moved = run(|t| [3.0 * t, 0.0], &p, 120, 1.0 / 60.0);
        let still = run(|_| [0.0, 0.0], &p, 120, 1.0 / 60.0);
        let mean_x = |f: &[[f32; 2]]| f.iter().map(|q| q[0]).sum::<f32>() / f.len() as f32;
        assert!(
            mean_x(moved.last().unwrap()) > mean_x(still.last().unwrap()) + 0.5,
            "the sliding anchor carried the body in +x"
        );
    }

    /// Deterministic replay (HR-5: arithmetic + one `sqrt`): two runs match exactly.
    #[test]
    fn replay_is_deterministic() {
        let p = params(4, 4, 9.0, 0.4, true);
        let a = run(|t| [2.0 * t, 0.0], &p, 90, 1.0 / 60.0);
        let b = run(|t| [2.0 * t, 0.0], &p, 90, 1.0 / 60.0);
        assert_eq!(a, b);
    }

    /// Without the state loop it re-seeds every tick → the rest mesh at the anchor,
    /// never deforming (the "only simulates with feedback" footnote).
    #[test]
    fn without_the_state_loop_it_holds_the_rest_mesh() {
        let p = params(3, 3, 12.0, 0.5, true);
        let rest = rest_shape(p.rows, p.cols, p.spacing);
        for k in 0..20 {
            let out = simulate([1.0, 0.0], &Stream::new(0), k as f32 / 60.0, &p);
            let pos = vec2_col(&out, "P");
            for (q, r) in pos.iter().zip(&rest) {
                assert!((q[0] - (r[0] + 1.0)).abs() < 1e-5 && (q[1] - r[1]).abs() < 1e-5);
            }
        }
    }

    /// A poisoned (non-finite) state recovers on the rest frame instead of spreading.
    #[test]
    fn non_finite_state_recovers() {
        let p = params(2, 2, 9.0, 0.5, false);
        let rest = rest_shape(p.rows, p.cols, p.spacing);
        let state = Stream::new(4)
            .with(
                "P",
                Column::Vec2(vec![
                    [0.0, 0.0],
                    [f32::INFINITY, 0.0],
                    [0.0, 0.0],
                    [0.0, 0.0],
                ]),
            )
            .with("sb_vel", Column::Vec2(vec![[0.0, 0.0]; 4]))
            .with("sim_t", Column::Scalar(vec![0.0; 4]));
        let out = simulate([0.0, 0.0], &state, 1.0 / 60.0, &p);
        let _ = rest;
        assert!(
            vec2_col(&out, "P")
                .iter()
                .all(|q| q[0].is_finite() && q[1].is_finite()),
            "the diverged particle was reset, not propagated"
        );
    }

    /// Cooks through the registry with the `pre` self-loop, exactly as the editor
    /// wires it.
    #[test]
    fn registers_and_steps_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionSoftBody as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let sb = g.add_node("motion.soft_body");
        g.set_param(sb, "rows", 4.0);
        g.set_param(sb, "cols", 3.0);
        g.set_param(sb, "gravity", 12.0);
        g.connect(Edge {
            from: (sb, 0),
            to: (sb, 2),
            delayed: true,
        })
        .unwrap();

        let mut cook = Cook::new();
        let out0 = cook.cook(&g, &Ops, sb, 0.0).unwrap();
        assert!(matches!(out0[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 12));
        for k in 0..60 {
            let t = k as f64 / 60.0;
            cook.cook(&g, &Ops, sb, t).unwrap();
            cook.advance_tick(&g, &Ops, t).unwrap();
        }
        let out = cook.cook(&g, &Ops, sb, 1.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            // The free bottom row has sagged below the pinned top.
            Column::Vec2(v) => assert!(v[11][1] < v[0][1], "the body hangs after a second"),
            _ => panic!("P"),
        }
    }
}

#[cfg(test)]
mod cost_probe {
    use super::shape::{rest_shape, shape_goals};

    /// MEASUREMENT (a fila §2 do handoff da linha GPU): o cap do nó é
    /// `MAX_SIDE = 40` (1600 partículas). O custo por tick é o shape-matching —
    /// DUAS passadas lineares sobre a nuvem (centroide, depois `A_pq`/`A_qq`) —
    /// então a pergunta do §0.0 é: esse cap é de CUSTO ou é escolha?
    ///
    ///   cargo test -p ph2d-node-motion-soft-body --release -- --ignored --nocapture
    #[test]
    #[ignore = "measurement — run alone with --nocapture"]
    fn what_does_the_shape_match_cost_per_tick() {
        for &side in &[40usize, 100, 316, 1000] {
            let n = side * side;
            let rest = rest_shape(side, side, 1.0);
            let pred: Vec<[f32; 2]> = rest.iter().map(|p| [p[0] * 1.1, p[1] * 0.9]).collect();
            let t0 = std::time::Instant::now();
            const REPS: u32 = 20;
            for _ in 0..REPS {
                std::hint::black_box(shape_goals(&pred, &rest, 0.3));
            }
            let ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);
            eprintln!("  shape_match {side:>4}x{side:<4} = {n:>9} partículas: {ms:>8.3} ms/tick");
        }
    }
}
