#![forbid(unsafe_code)]
//! `motion.wave` — a **wave-equation ripple field**: a `rows×cols` grid whose
//! heights obey the discrete 2D wave equation, so a driven centre radiates
//! concentric ripples outward (Motion Nodes M4, simulation — doc 01 §3 / doc 22).
//! The field-dynamics counterpart to the particle sims: the disturbance PROPAGATES
//! at a finite speed (a real hyperbolic PDE), which no per-element oscillator can
//! do. The height maps to each dot's `size`, so the grid reads as expanding rings.
//!
//! **Algorithm — the classic finite-difference wave equation, leapfrog in time.**
//! `∂²h/∂t² = c²∇²h` discretises to
//! `h_next = 2·h − h_prev + C·∇²h` (leapfrog / Verlet in time), with the 5-point
//! Laplacian `∇²h[i,j] = h[i−1,j]+h[i+1,j]+h[i,j−1]+h[i,j+1] − 4·h[i,j]` and
//! reflecting (Neumann) edges. `C = (c·dt)²` is clamped below the CFL limit `0.5`
//! so it never explodes. A `damping` factor bleeds energy each step. The centre cell
//! is driven to the `drive` value input (a Dirichlet source — wire a `value.lfo`
//! and it emits continuous ripples). Pure arithmetic (HR-5: no `sqrt`, no trig).
//!
//! ## Topology (the `pre` self-loop)
//!
//! Sequential like the other sims: the height field `wave_h` and its previous frame
//! `wave_prev` ride the `state` feedback port (the `pre` self-loop). Tick 0
//! (`pre` = Empty) **seeds** a flat grid; from tick 1 it steps. `dt` derives from
//! the state's own `sim_t`, clamped to `[0, MAX_DT]` (a loop-wrap freezes one tick).
//!
//! Deterministic → `Effect::Temporal`, replays bit-for-bit.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `drive` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Ceiling on a single step (see `motion.integrate`): guards a playhead jump.
const MAX_DT: f32 = 0.1;
/// CFL stability bound for the 2D leapfrog wave: `C = (c·dt)² ≤ 0.5`.
const CFL_MAX: f32 = 0.49;
/// Grid side clamp (field cost is O(rows·cols)).
const MAX_SIDE: i64 = 60;
/// Baseline dot size (a flat field), and how much a unit of height swells it.
const SIZE_BASE: f32 = 0.22;
const SIZE_GAIN: f32 = 1.4;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.wave"),
    name: "motion.wave",
    inputs: &[
        // The source amplitude driven into the centre cell (animatable). Optional:
        // unconnected reads as 0 → a still field.
        PortSpec {
            name: "drive",
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
            default: 11.0,
        },
        ParamSpec {
            name: "cols",
            default: 11.0,
        },
        ParamSpec {
            name: "spacing",
            default: 0.5,
        },
        // Propagation coefficient ∈ [0, CFL_MAX]: how far a ripple travels per step.
        ParamSpec {
            name: "speed",
            default: 0.35,
        },
        ParamSpec {
            name: "damping",
            default: 0.02,
        },
        // Grid centre in world units (so several fields can sit side by side).
        ParamSpec {
            name: "center_x",
            default: 0.0,
        },
        ParamSpec {
            name: "center_y",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Resolved parameters (arithmetic-ready).
struct Params {
    rows: usize,
    cols: usize,
    spacing: f32,
    speed: f32,
    damping: f32,
    center: [f32; 2],
}

impl Params {
    fn count(&self) -> usize {
        self.rows * self.cols
    }
    /// The centre cell index (the driven source).
    fn source(&self) -> usize {
        (self.rows / 2) * self.cols + self.cols / 2
    }
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn value_head(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.0)
}

/// The fixed world positions of the grid, centred on `center`, row 0 at the top.
fn grid_positions(p: &Params) -> Vec<[f32; 2]> {
    let (w, h) = (
        (p.cols as f32 - 1.0) * p.spacing,
        (p.rows as f32 - 1.0) * p.spacing,
    );
    let mut out = Vec::with_capacity(p.count());
    for r in 0..p.rows {
        for c in 0..p.cols {
            out.push([
                p.center[0] + c as f32 * p.spacing - w * 0.5,
                p.center[1] + h * 0.5 - r as f32 * p.spacing,
            ]);
        }
    }
    out
}

/// One leapfrog wave step over the height field. Reflecting (Neumann) edges: an
/// out-of-grid neighbour reads as the centre cell, so it contributes no gradient.
fn step(h: &[f32], h_prev: &[f32], drive: f32, p: &Params) -> (Vec<f32>, Vec<f32>) {
    let (rows, cols) = (p.rows, p.cols);
    let coeff = p.speed.clamp(0.0, CFL_MAX);
    let keep = 1.0 - p.damping;
    let mut next = vec![0.0f32; h.len()];
    for r in 0..rows {
        for c in 0..cols {
            let i = r * cols + c;
            let at = |rr: usize, cc: usize| h[rr * cols + cc];
            // 5-point Laplacian with reflecting edges (missing neighbour = centre).
            let up = if r > 0 { at(r - 1, c) } else { h[i] };
            let down = if r + 1 < rows { at(r + 1, c) } else { h[i] };
            let left = if c > 0 { at(r, c - 1) } else { h[i] };
            let right = if c + 1 < cols { at(r, c + 1) } else { h[i] };
            let lap = up + down + left + right - 4.0 * h[i];
            let mut v = (2.0 * h[i] - h_prev[i] + coeff * lap) * keep;
            if !v.is_finite() {
                v = 0.0; // NaN guard: reset a diverged cell
            }
            next[i] = v;
        }
    }
    // Dirichlet source: the centre cell is driven to `drive`.
    next[p.source()] = drive;
    (next, h.to_vec())
}

/// The whole node as a pure function: seed a flat field on the first tick / a grid
/// change, else step. Emits `P` (fixed grid) + `size` (from |height|) + the
/// `wave_h`/`wave_prev`/`sim_t` state.
fn simulate(drive: f32, state: &Stream, playhead: f32, p: &Params) -> Stream {
    let n = p.count();
    let s_h = scalar_col(state, "wave_h");
    let s_prev = scalar_col(state, "wave_prev");

    let (h, prev) = if s_h.len() == n && s_prev.len() == n {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        if dt < 1e-6 {
            (s_h, s_prev) // loop-wrap / same tick → hold
        } else {
            step(&s_h, &s_prev, drive, p)
        }
    } else {
        (vec![0.0; n], vec![0.0; n]) // seed a flat field
    };

    let size: Vec<[f32; 2]> = h
        .iter()
        .map(|&z| {
            let s = SIZE_BASE + SIZE_GAIN * z.abs();
            [s, s]
        })
        .collect();
    Stream::new(n)
        .with("P", Column::Vec2(grid_positions(p)))
        .with("size", Column::Vec2(size))
        .with("wave_h", Column::Scalar(h))
        .with("wave_prev", Column::Scalar(prev))
        .with("sim_t", Column::Scalar(vec![playhead; n]))
}

struct MotionWave;

impl NodeOp for MotionWave {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let side = |name: &str| (ctx.param(name).round() as i64).clamp(2, MAX_SIDE) as usize;
        let p = Params {
            rows: side("rows"),
            cols: side("cols"),
            spacing: ctx.param("spacing").max(1e-3),
            speed: ctx.param("speed"),
            damping: ctx.param("damping").clamp(0.0, 0.99),
            center: [ctx.param("center_x"), ctx.param("center_y")],
        };
        let playhead = ctx.playhead() as f32;
        let drive = value_head(&scalar_col(ctx.input(0), VALUE_COL));
        let state = ctx.input(1);
        let out = simulate(drive, state, playhead, &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionWave))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Wave",
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
        max: 60.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cols",
        label: "Cols",
        min: 2.0,
        max: 60.0,
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
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: CFL_MAX,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "damping",
        label: "Damping",
        min: 0.0,
        max: 0.3,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_x",
        label: "Center X",
        min: -20.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "Center Y",
        min: -20.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn params(rows: usize, cols: usize, speed: f32, damping: f32) -> Params {
        Params {
            rows,
            cols,
            spacing: 0.5,
            speed,
            damping,
            center: [0.0, 0.0],
        }
    }

    /// Run `drive(t)` for `ticks` fixed steps, returning each frame's height field.
    fn run(drive: impl Fn(f32) -> f32, p: &Params, ticks: usize, dt: f32) -> Vec<Vec<f32>> {
        let mut state = Stream::new(0);
        let mut frames = Vec::new();
        for k in 0..ticks {
            let t = k as f32 * dt;
            let out = simulate(drive(t), &state, t, p);
            state = out.clone();
            frames.push(scalar_col(&out, "wave_h"));
        }
        frames
    }

    /// Tick 0 seeds a flat field: `rows·cols` cells, every height 0 and every dot at
    /// the baseline size.
    #[test]
    fn seeds_a_flat_field() {
        let p = params(5, 5, 0.3, 0.0);
        let out = simulate(0.0, &Stream::new(0), 0.0, &p);
        assert_eq!(out.count(), 25);
        assert!(scalar_col(&out, "wave_h").iter().all(|&z| z == 0.0), "flat");
        match out.get("size").unwrap() {
            Column::Vec2(v) => assert!(v.iter().all(|s| (s[0] - SIZE_BASE).abs() < 1e-6)),
            _ => panic!("size"),
        }
    }

    /// The disturbance PROPAGATES outward: a driven centre eventually lifts a cell far
    /// from the source. FALSIFIED at speed 0 — only the driven centre ever moves, the
    /// rim stays flat (no propagation).
    #[test]
    fn a_driven_source_propagates_outward() {
        let p = params(11, 11, 0.35, 0.0);
        let corner = 0usize; // top-left, far from the centre
        let live = run(|t| (t * 30.0).clamp(-1.0, 1.0), &p, 120, 1.0 / 60.0);
        let dead = run(
            |t| (t * 30.0).clamp(-1.0, 1.0),
            &params(11, 11, 0.0, 0.0),
            120,
            1.0 / 60.0,
        );
        let live_corner = live.last().unwrap()[corner].abs();
        let dead_corner = dead.last().unwrap()[corner].abs();
        assert!(
            live_corner > 0.02,
            "the ripple reached the corner: {live_corner}"
        );
        assert!(
            dead_corner < 1e-6,
            "speed 0 never propagates: {dead_corner}"
        );
    }

    /// The wave travels at a FINITE speed: one tick after the first impulse only the
    /// source's immediate neighbours have moved — a cell two rings out is still flat.
    /// FALSIFIED by instantaneous coupling (the far cell would already be non-zero).
    #[test]
    fn propagation_is_finite_speed_not_instant() {
        let p = params(11, 11, 0.4, 0.0);
        // Tick 0 seeds flat; tick 1 drives the centre; tick 2 spreads to the 4
        // immediate neighbours only — a cell three away is still untouched.
        let frames = run(|_| 1.0, &p, 3, 1.0 / 60.0);
        let h2 = &frames[2];
        let src = p.source();
        let neighbour = src - 1; // same row, one cell left (adjacent to the source)
        let two_out = src - 3; // three cells left — beyond the first ring's reach
        assert!(h2[neighbour].abs() > 1e-4, "the near neighbour moved");
        assert!(
            h2[two_out].abs() < 1e-9,
            "a far cell has not been reached yet"
        );
    }

    /// Damping keeps a continuously-driven field BOUNDED and shrinks the far-field:
    /// heavier damping leaves the rim quieter. FALSIFIED if damping did nothing (the
    /// two rim amplitudes would match).
    #[test]
    fn damping_bounds_and_quiets_the_field() {
        // A bounded sawtooth oscillation in [-1, 1] (no trig, no TAU).
        let drive = |t: f32| 2.0 * (t * 5.0 - (t * 5.0).floor()) - 1.0;
        let soft = run(drive, &params(11, 11, 0.35, 0.12), 300, 1.0 / 60.0);
        let hard = run(drive, &params(11, 11, 0.35, 0.28), 300, 1.0 / 60.0);
        let rim = 0usize;
        let soft_rim = soft.iter().map(|f| f[rim].abs()).fold(0.0, f32::max);
        let hard_rim = hard.iter().map(|f| f[rim].abs()).fold(0.0, f32::max);
        // Everything stays finite (bounded), and heavier damping is quieter.
        assert!(
            soft.last().unwrap().iter().all(|z| z.is_finite()),
            "bounded"
        );
        assert!(
            hard_rim < soft_rim,
            "heavier damping quiets the rim (hard {hard_rim} < soft {soft_rim})"
        );
    }

    /// Deterministic replay (HR-5: pure arithmetic): two runs match bit-for-bit.
    #[test]
    fn replay_is_deterministic() {
        let p = params(9, 9, 0.35, 0.02);
        let a = run(|t| (t * 5.0).sin_cos_free(), &p, 90, 1.0 / 60.0);
        let b = run(|t| (t * 5.0).sin_cos_free(), &p, 90, 1.0 / 60.0);
        assert_eq!(a, b);
    }

    /// Without the state loop it re-seeds every tick → a flat field forever.
    #[test]
    fn without_the_state_loop_it_stays_flat() {
        let p = params(7, 7, 0.35, 0.0);
        for k in 0..20 {
            let out = simulate(1.0, &Stream::new(0), k as f32 / 60.0, &p);
            // Only the driven centre is non-zero; everything else is flat (no history).
            let h = scalar_col(&out, "wave_h");
            for (i, &z) in h.iter().enumerate() {
                if i != p.source() {
                    assert_eq!(z, 0.0, "no propagation without feedback at {i}");
                }
            }
        }
    }

    /// Cooks through the registry with the `pre` self-loop, emitting `P` + `size`.
    #[test]
    fn registers_and_ripples_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionWave as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();

        let mut g = Graph::new();
        let wave = g.add_node("motion.wave");
        g.set_param(wave, "rows", 9.0);
        g.set_param(wave, "cols", 9.0);
        g.connect(Edge {
            from: (wave, 0),
            to: (wave, 1),
            delayed: true,
        })
        .unwrap();

        // Drive stays unconnected (→ 0); the field still steps through the pre loop.
        let mut cook = Cook::new();
        let out0 = cook.cook(&g, &Ops, wave, 0.0).unwrap();
        assert!(matches!(out0[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 81));
        assert!(
            out0[0].as_stream().get("size").is_some(),
            "emits a size column"
        );
        for k in 0..30 {
            let t = k as f64 / 60.0;
            cook.cook(&g, &Ops, wave, t).unwrap();
            cook.advance_tick(&g, &Ops, t).unwrap();
        }
        let out = cook.cook(&g, &Ops, wave, 0.5).unwrap();
        assert!(matches!(out[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 81));
    }

    // A tiny transcendental-free stand-in for a bounded oscillator in the determinism
    // test (keeps the test itself HR-5-clean).
    trait BoundedOsc {
        fn sin_cos_free(self) -> f32;
    }
    impl BoundedOsc for f32 {
        fn sin_cos_free(self) -> f32 {
            // Triangle wave in [-1, 1] from the fractional part — no trig.
            let f = self - self.floor();
            (2.0 * (2.0 * f - 1.0).abs() - 1.0).clamp(-1.0, 1.0)
        }
    }
}
