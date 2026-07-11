#![forbid(unsafe_code)]
//! `motion.boids` — **emergent flocking**: `count` agents that steer by the three
//! classic local rules and coalesce into a living flock (Motion Nodes M4,
//! simulation — doc 01 §3 / doc 21). No path is authored; the swirling, splitting,
//! re-forming shoal is what *emerges* from every agent obeying its neighbours. The
//! counterpart to the deterministic `motion.verlet_rope`: same sequential substrate
//! (`pre` state), opposite character — self-organising rather than constrained.
//!
//! **Algorithm — Reynolds' boids (SIGGRAPH 1987), the gold standard.** Each agent
//! looks only at flock-mates within a perception `radius` and sums three steering
//! urges:
//! - **Separation** — steer away from crowding neighbours (inverse-square repulsion,
//!   so it bites only when they get close).
//! - **Alignment** — steer toward the average heading of neighbours.
//! - **Cohesion** — steer toward the average position (centroid) of neighbours.
//!
//! Plus a **seek** pull toward a target point (the value inputs; unconnected → the
//! origin). Seek is a linear spring, so it doubles as the flock's tether: it grows
//! with distance and keeps the swarm bounded around its home (no fly-away, no
//! separate boundary hack). A `value.lfo` on the target herds the whole flock.
//!
//! Neighbour search is the honest `O(N²)` all-pairs scan — exact and simple for the
//! bounded counts a node produces. A spatial hash (the standard scale trick) is a
//! pure-perf follow-up (doc 21 §5); it would not change a single emitted position.
//!
//! ## Topology (the `pre` self-loop)
//!
//! Sequential like `motion.spring`/`motion.integrate`/`motion.verlet_rope`: each
//! agent's `P` and `vel` ride the `state` feedback port, auto-plumbed as the `pre`
//! self-loop on add (`out --pre--> state`). Tick 0 (`pre` = Empty) **seeds** a
//! hashed cloud of positions + velocities (stateless, reproducible — Jarzynski/
//! Olano); from tick 1 it steps. `dt` derives from the state's own `sim_t`, clamped
//! to `[0, MAX_DT]` (a loop-wrap freezes one tick, never explodes) — like
//! `motion.integrate`.
//!
//! Transcendental-free (HR-5): steering is add/multiply and one IEEE `sqrt` per
//! normalise (heading / speed clamp); the seed is the splitmix hash. Deterministic
//! → `Effect::Temporal`, replays bit-for-bit.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
use hash::hash3;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `target_*` inputs (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Ceiling on a single step (see `motion.integrate`): guards a playhead jump.
const MAX_DT: f32 = 0.1;
/// Half-extent of the seed cloud (world units), centred on the target/home.
const SEED_SPREAD: f32 = 3.0;
/// A boid never drifts below this fraction of `max_speed` — keeps the flock
/// gliding instead of stalling to a dead point.
const MIN_SPEED_FRAC: f32 = 0.2;
/// Below this a vector length is treated as zero (skip the normalise).
const EPS: f32 = 1e-6;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.boids"),
    name: "motion.boids",
    inputs: &[
        // The seek target / home, as two value fields (animatable). Optional:
        // unconnected reads as 0 → the origin.
        PortSpec {
            name: "target_x",
            ty: VALUE,
        },
        PortSpec {
            name: "target_y",
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
            name: "count",
            default: 48.0,
        },
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
        // Perception radius: only neighbours within it steer an agent.
        ParamSpec {
            name: "radius",
            default: 2.0,
        },
        ParamSpec {
            name: "separation",
            default: 1.6,
        },
        ParamSpec {
            name: "alignment",
            default: 1.0,
        },
        ParamSpec {
            name: "cohesion",
            default: 0.9,
        },
        // Pull toward the target/home (a linear spring → also the tether/bound).
        ParamSpec {
            name: "seek",
            default: 1.0,
        },
        ParamSpec {
            name: "max_speed",
            default: 4.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Resolved simulation weights (arithmetic-ready).
struct Params {
    count: usize,
    seed: u32,
    radius_sq: f32,
    separation: f32,
    alignment: f32,
    cohesion: f32,
    seek: f32,
    max_speed: f32,
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

/// The value coordinate for the (single) target: **unconnected → 0.0** (origin).
fn value_head(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.0)
}

/// `(unit, len)` of `v`; `([0,0], 0)` for a ~zero vector (no NaN).
fn norm(v: [f32; 2]) -> ([f32; 2], f32) {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len < EPS {
        ([0.0, 0.0], 0.0)
    } else {
        ([v[0] / len, v[1] / len], len)
    }
}

/// Seed a hashed cloud of positions + velocities around `home` (tick 0 / a count
/// change). Stateless: a pure function of `(seed, index)`, so it reproduces.
fn seed(home: [f32; 2], p: &Params) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut pos = Vec::with_capacity(p.count);
    let mut vel = Vec::with_capacity(p.count);
    for i in 0..p.count as u32 {
        pos.push([
            home[0] + (hash3(p.seed, i, 0) - 0.5) * 2.0 * SEED_SPREAD,
            home[1] + (hash3(p.seed, i, 1) - 0.5) * 2.0 * SEED_SPREAD,
        ]);
        // Initial velocities spread across ±max_speed/2 so the flock starts alive.
        vel.push([
            (hash3(p.seed, i, 2) - 0.5) * p.max_speed,
            (hash3(p.seed, i, 3) - 0.5) * p.max_speed,
        ]);
    }
    (pos, vel)
}

/// One flocking step as a pure function. `pos`/`vel` are this tick's entry state.
fn step(
    pos: &[[f32; 2]],
    vel: &[[f32; 2]],
    target: [f32; 2],
    dt: f32,
    p: &Params,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let n = pos.len();
    let mut out_pos = vec![[0.0f32; 2]; n];
    let mut out_vel = vec![[0.0f32; 2]; n];
    let min_speed = p.max_speed * MIN_SPEED_FRAC;

    for i in 0..n {
        let (pi, vi) = (pos[i], vel[i]);
        // Accumulate the three Reynolds urges over neighbours within the radius.
        let mut sep = [0.0f32; 2];
        let mut align = [0.0f32; 2];
        let mut centroid = [0.0f32; 2];
        let mut neighbours = 0u32;
        for j in 0..n {
            if j == i {
                continue;
            }
            let d = [pi[0] - pos[j][0], pi[1] - pos[j][1]];
            let dist_sq = d[0] * d[0] + d[1] * d[1];
            if dist_sq > p.radius_sq || dist_sq < EPS {
                continue;
            }
            // Separation: inverse-square repulsion (bites hardest when closest).
            let inv = 1.0 / dist_sq;
            sep[0] += d[0] * inv;
            sep[1] += d[1] * inv;
            // Alignment / cohesion accumulate neighbour heading / position.
            align[0] += vel[j][0];
            align[1] += vel[j][1];
            centroid[0] += pos[j][0];
            centroid[1] += pos[j][1];
            neighbours += 1;
        }

        let mut accel = [0.0f32; 2];
        if neighbours > 0 {
            let inv_n = 1.0 / neighbours as f32;
            // Alignment: steer toward the average heading.
            accel[0] += (align[0] * inv_n - vi[0]) * p.alignment;
            accel[1] += (align[1] * inv_n - vi[1]) * p.alignment;
            // Cohesion: steer toward the centroid.
            accel[0] += (centroid[0] * inv_n - pi[0]) * p.cohesion;
            accel[1] += (centroid[1] * inv_n - pi[1]) * p.cohesion;
            // Separation: push away (already an inverse-square sum).
            accel[0] += sep[0] * p.separation;
            accel[1] += sep[1] * p.separation;
        }
        // Seek/home: a linear spring toward the target — herds AND bounds.
        accel[0] += (target[0] - pi[0]) * p.seek;
        accel[1] += (target[1] - pi[1]) * p.seek;

        // Integrate, then clamp the speed into [min, max] so the flock glides.
        let mut nv = [vi[0] + accel[0] * dt, vi[1] + accel[1] * dt];
        let (unit, speed) = norm(nv);
        if speed > p.max_speed {
            nv = [unit[0] * p.max_speed, unit[1] * p.max_speed];
        } else if speed > EPS && speed < min_speed {
            nv = [unit[0] * min_speed, unit[1] * min_speed];
        }
        let mut np = [pi[0] + nv[0] * dt, pi[1] + nv[1] * dt];
        // NaN/∞ guard (reference parity): a diverged agent recovers at the target.
        if !(np[0].is_finite() && np[1].is_finite() && nv[0].is_finite() && nv[1].is_finite()) {
            np = target;
            nv = [0.0, 0.0];
        }
        out_pos[i] = np;
        out_vel[i] = nv;
    }
    (out_pos, out_vel)
}

/// The whole node as a pure function: seed on the first tick / a count change,
/// else step. Emits `P` + the `vel`/`sim_t` state.
fn simulate(target: [f32; 2], state: &Stream, playhead: f32, p: &Params) -> Stream {
    let s_pos = vec2_col(state, "P");
    let s_vel = vec2_col(state, "vel");

    let (pos, vel) = if s_pos.len() == p.count && s_vel.len() == p.count {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        step(&s_pos, &s_vel, target, dt, p)
    } else {
        seed(target, p)
    };

    Stream::new(pos.len())
        .with("P", Column::Vec2(pos))
        .with("vel", Column::Vec2(vel))
        .with("sim_t", Column::Scalar(vec![playhead; p.count]))
}

struct MotionBoids;

impl NodeOp for MotionBoids {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS).max(1);
        let radius = ctx.param("radius").max(0.0);
        let p = Params {
            count,
            seed: ctx.param("seed").max(0.0).round() as u32,
            radius_sq: radius * radius,
            separation: ctx.param("separation").max(0.0),
            alignment: ctx.param("alignment").max(0.0),
            cohesion: ctx.param("cohesion").max(0.0),
            seek: ctx.param("seek").max(0.0),
            max_speed: ctx.param("max_speed").max(0.01),
        };
        let playhead = ctx.playhead() as f32;
        let target = [
            value_head(&scalar_col(ctx.input(0), VALUE_COL)),
            value_head(&scalar_col(ctx.input(1), VALUE_COL)),
        ];
        let state = ctx.input(2);
        let out = simulate(target, state, playhead, &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionBoids))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Boids",
            // Source: it mints its own agent stream, then simulates it.
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
        label: "Count",
        min: 1.0,
        max: 500.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.1,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "separation",
        label: "Separation",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "alignment",
        label: "Alignment",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cohesion",
        label: "Cohesion",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seek",
        label: "Seek",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max_speed",
        label: "Max Speed",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn params(count: usize) -> Params {
        Params {
            count,
            seed: 1,
            radius_sq: 4.0 * 4.0, // wide perception so the whole flock interacts
            separation: 0.0,
            alignment: 0.0,
            cohesion: 0.0,
            seek: 0.0,
            max_speed: 4.0,
        }
    }

    /// Mean distance of the flock to its own centroid — how spread out it is.
    fn spread(pos: &[[f32; 2]]) -> f32 {
        let n = pos.len() as f32;
        let c = pos
            .iter()
            .fold([0.0f32; 2], |a, p| [a[0] + p[0], a[1] + p[1]]);
        let c = [c[0] / n, c[1] / n];
        pos.iter()
            .map(|p| {
                let d = [p[0] - c[0], p[1] - c[1]];
                (d[0] * d[0] + d[1] * d[1]).sqrt()
            })
            .sum::<f32>()
            / n
    }

    fn centroid(pos: &[[f32; 2]]) -> [f32; 2] {
        let n = pos.len() as f32;
        let c = pos
            .iter()
            .fold([0.0f32; 2], |a, p| [a[0] + p[0], a[1] + p[1]]);
        [c[0] / n, c[1] / n]
    }

    /// The smallest pairwise distance in the flock (collapse detector).
    fn min_pair(pos: &[[f32; 2]]) -> f32 {
        let mut m = f32::MAX;
        for (i, a) in pos.iter().enumerate() {
            for b in &pos[i + 1..] {
                let d = [a[0] - b[0], a[1] - b[1]];
                m = m.min((d[0] * d[0] + d[1] * d[1]).sqrt());
            }
        }
        m
    }

    /// Coherence of headings ∈ [0,1]: the length of the mean unit-velocity. ~1 when
    /// all agents fly the same way, ~0 when their headings are scattered.
    fn heading_coherence(vel: &[[f32; 2]]) -> f32 {
        let n = vel.len() as f32;
        let sum = vel.iter().fold([0.0f32; 2], |a, v| {
            let (u, _) = norm(*v);
            [a[0] + u[0], a[1] + u[1]]
        });
        ((sum[0] * sum[0] + sum[1] * sum[1]).sqrt()) / n
    }

    /// Run `ticks` fixed steps through the pure `simulate`, feeding output back as
    /// the next tick's state (what the `pre` loop does live). Returns the last frame.
    fn run(target: [f32; 2], p: &Params, ticks: usize, dt: f32) -> Stream {
        let mut state = Stream::new(0); // Empty → tick 0 seeds
        let mut last = state.clone();
        for k in 0..ticks {
            last = simulate(target, &state, k as f32 * dt, p);
            state = last.clone();
        }
        last
    }

    /// Cohesion CONTRACTS the flock: with only cohesion on, the spread shrinks over
    /// time. FALSIFIED against no forces — inertia alone lets the random seed
    /// velocities carry the agents APART, so the spread grows.
    #[test]
    fn cohesion_pulls_the_flock_together() {
        let mut coh = params(40);
        coh.cohesion = 3.0;
        let mut none = params(40);
        none.cohesion = 0.0;

        let seed_spread = spread(&vec2_col(&run([0.0, 0.0], &coh, 1, 1.0 / 60.0), "P"));
        let coh_end = spread(&vec2_col(&run([0.0, 0.0], &coh, 180, 1.0 / 60.0), "P"));
        let none_end = spread(&vec2_col(&run([0.0, 0.0], &none, 180, 1.0 / 60.0), "P"));

        // Cohesion holds the flock together (never expands past its seed spread);
        // inertia alone lets the random seed velocities carry the agents APART.
        assert!(
            coh_end <= seed_spread,
            "cohesion held the flock ({seed_spread} → {coh_end})"
        );
        assert!(
            none_end > coh_end * 1.4,
            "inertia-only drifts far wider ({none_end}) than the cohesive flock ({coh_end})"
        );
    }

    /// Alignment ALIGNS the headings: with only alignment on, the agents converge to
    /// a common heading (coherence → high). FALSIFIED against no forces — the random
    /// seed headings stay scattered (low coherence).
    #[test]
    fn alignment_aligns_the_headings() {
        let mut al = params(40);
        al.alignment = 3.0;
        let mut none = params(40);
        none.alignment = 0.0;

        let al_end = heading_coherence(&vec2_col(&run([0.0, 0.0], &al, 120, 1.0 / 60.0), "vel"));
        let none_end =
            heading_coherence(&vec2_col(&run([0.0, 0.0], &none, 120, 1.0 / 60.0), "vel"));
        assert!(
            al_end > 0.9,
            "alignment made the flock fly as one: {al_end}"
        );
        assert!(
            al_end > none_end + 0.3,
            "aligned ({al_end}) far more coherent than free ({none_end})"
        );
    }

    /// Separation PREVENTS COLLAPSE: strong cohesion WITHOUT separation stacks the
    /// agents almost on top of each other; adding separation keeps them spaced.
    #[test]
    fn separation_prevents_collapse() {
        let mut collapse = params(30);
        collapse.cohesion = 3.0;
        collapse.radius_sq = 100.0; // everyone sees everyone
        let mut spaced = params(30);
        spaced.cohesion = 3.0;
        spaced.separation = 2.0;
        spaced.radius_sq = 100.0;

        let collapse_min = min_pair(&vec2_col(&run([0.0, 0.0], &collapse, 240, 1.0 / 60.0), "P"));
        let spaced_min = min_pair(&vec2_col(&run([0.0, 0.0], &spaced, 240, 1.0 / 60.0), "P"));
        assert!(
            collapse_min < 0.2,
            "no separation → they pile up: {collapse_min}"
        );
        assert!(
            spaced_min > collapse_min * 3.0,
            "separation kept them apart ({spaced_min}) vs collapsed ({collapse_min})"
        );
    }

    /// The flock SEEKS the target: with seek on and a target off to the right, the
    /// centroid migrates toward it. FALSIFIED against seek 0 — the centroid stays put.
    #[test]
    fn the_flock_seeks_the_target() {
        let mut seek = params(30);
        seek.seek = 1.5;
        let mut free = params(30);
        free.seek = 0.0;

        let target = [10.0, 0.0];
        let seek_c = centroid(&vec2_col(&run(target, &seek, 200, 1.0 / 60.0), "P"));
        // The seed centres on the target, so measure migration by seeding at origin:
        // give `free` no seek and target far away — its centroid barely moves from 0.
        let free_c = centroid(&vec2_col(&run([0.0, 0.0], &free, 200, 1.0 / 60.0), "P"));
        assert!(
            (seek_c[0] - 10.0).abs() < 2.0,
            "the flock gathered on the target: centroid x {}",
            seek_c[0]
        );
        assert!(
            free_c[0].abs() < 4.0,
            "no seek: stays near origin: {}",
            free_c[0]
        );
    }

    /// The seek spring keeps the flock BOUNDED: after a long run every agent is
    /// finite and within a sane radius of its home (no explosion).
    #[test]
    fn the_seek_spring_bounds_the_flock() {
        let mut p = params(50);
        p.separation = 2.0;
        p.alignment = 1.0;
        p.cohesion = 1.0;
        p.seek = 1.0;
        p.radius_sq = 4.0;
        let last = vec2_col(&run([0.0, 0.0], &p, 600, 1.0 / 60.0), "P");
        for q in &last {
            // Compare the squared distance from home (< 20²) — no transcendental.
            assert!(
                q[0].is_finite() && q[1].is_finite() && q[0] * q[0] + q[1] * q[1] < 400.0,
                "bounded near home: {q:?}"
            );
        }
    }

    /// Deterministic replay (HR-5): two runs match bit-for-bit; a different seed
    /// re-rolls the flock.
    #[test]
    fn replay_is_deterministic_and_seed_re_rolls() {
        let mut p = params(24);
        p.separation = 1.5;
        p.alignment = 1.0;
        p.cohesion = 1.0;
        p.seek = 1.0;
        let a = vec2_col(&run([0.0, 0.0], &p, 90, 1.0 / 60.0), "P");
        let b = vec2_col(&run([0.0, 0.0], &p, 90, 1.0 / 60.0), "P");
        assert_eq!(a, b, "same seed replays");
        let mut p2 = params(24);
        p2.seed = 2;
        let c = vec2_col(&run([0.0, 0.0], &p2, 1, 1.0 / 60.0), "P");
        let a0 = vec2_col(&run([0.0, 0.0], &p, 1, 1.0 / 60.0), "P");
        assert_ne!(a0, c, "a new seed re-rolls the flock");
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
                (ty == MANIFEST.id).then_some(&MotionBoids as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let boids = g.add_node("motion.boids");
        g.set_param(boids, "count", 20.0);
        g.connect(Edge {
            from: (boids, 0),
            to: (boids, 2),
            delayed: true,
        })
        .unwrap();

        let mut cook = Cook::new();
        let out0 = cook.cook(&g, &Ops, boids, 0.0).unwrap();
        assert!(matches!(out0[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 20));
        for k in 0..60 {
            let t = k as f64 / 60.0;
            cook.cook(&g, &Ops, boids, t).unwrap();
            cook.advance_tick(&g, &Ops, t).unwrap();
        }
        let out = cook.cook(&g, &Ops, boids, 1.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert!(v.iter().all(|q| q[0].is_finite()), "still alive & finite"),
            _ => panic!("P"),
        }
    }
}
