#![forbid(unsafe_code)]
//! `motion.emitter` — a particle source: a continuous stream of short-lived
//! instances launched from a point, to be moved by the force loop
//! (`motion.integrate` + `force.*`).
//!
//! ## The alive set is a pure function of the playhead — no state at all
//!
//! Particle `k` is born at exactly `k / rate` and lives `life` seconds, so at
//! playhead `t` the alive set is the closed id range
//! `[ceil((t − life)·rate), floor(t·rate)]` — computed, never accumulated. Its
//! launch velocity is `hash(seed, k)` ([`hash`]), not a draw from a sequence.
//!
//! That is what the reference (MiniCavalryV2 `particleEmitter.js`) could not do:
//! it kept a live particle array, so scrubbing backwards or replaying gave a
//! different scene. Here **scrub is free and replay is bit-exact** (HR-5): ask
//! for `t` and you get the same particles, in the same order, whatever happened
//! before. There is no `state` port and no `pre` loop on this node.
//!
//! Downstream, `motion.integrate` matches its per-particle simulation state by
//! the `id` column this node stamps, so a particle keeps its velocity while its
//! neighbours are born and die around it.
//!
//! ## Columns emitted
//!
//! `P` (the emitter's origin — displacement is the integrator's job), `vel` (the
//! muzzle velocity the integrator seeds from), `id` (the spawn index — stable
//! identity), `age`/`life` (seconds; a fade/size node can read them), `size`
//! (a `Vec2` column `[size, size]` — the square particle quad; a column, NOT
//! the scalar param it is set from), and `Index`/`Count` (positional,
//! oldest-first — so `motion.tint` in Gradient mode paints the stream by age
//! for free).
//!
//! Params: `rate` (particles/sec), `life` (sec), `speed`, `angle` (degrees CCW
//! from +X; `90` is up in this Y-up world), `spread` (degrees of cone),
//! `x`/`y` (origin), `seed`, `max` (hard cap — the newest `max` survive), and
//! `size` (world-space side of each particle quad).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
mod trig;
use hash::rand01;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Hard ceiling on the alive set, whatever `rate × life` asks for. Guards the
/// per-frame cook (and a hand-authored `p rate 1e9`) from allocating a stream
/// that no renderer would draw anyway.
const MAX_ALIVE: usize = 4096;
/// Hash lane for the launch angle draw (see [`hash::rand01`]).
const LANE_ANGLE: u32 = 0;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.emitter"),
    name: "motion.emitter",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead; holds no state (the alive set is derived from it).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "rate",
            default: 40.0,
        },
        ParamSpec {
            name: "life",
            default: 3.0,
        },
        ParamSpec {
            name: "speed",
            default: 4.0,
        },
        ParamSpec {
            name: "angle",
            default: 90.0, // up (Y-up world)
        },
        ParamSpec {
            name: "spread",
            default: 30.0,
        },
        ParamSpec {
            name: "x",
            default: 0.0,
        },
        ParamSpec {
            name: "y",
            default: 0.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        ParamSpec {
            name: "max",
            default: 512.0,
        },
        ParamSpec {
            name: "size",
            default: 0.15,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct MotionEmitter;

impl NodeOp for MotionEmitter {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let spec = Spec {
            rate: ctx.param("rate"),
            life: ctx.param("life"),
            speed: ctx.param("speed"),
            angle: ctx.param("angle"),
            spread: ctx.param("spread"),
            origin: [ctx.param("x"), ctx.param("y")],
            seed: ctx.param("seed").max(0.0) as u32,
            max: (ctx.param("max").max(0.0) as usize).min(MAX_ALIVE),
            size: ctx.param("size").max(0.0),
        };
        ctx.emit(emit(&spec, ctx.playhead() as f32));
    }
}

/// The emitter's resolved params (a struct so [`emit`] is a testable pure fn).
struct Spec {
    rate: f32,
    life: f32,
    speed: f32,
    angle: f32,
    spread: f32,
    origin: [f32; 2],
    seed: u32,
    max: usize,
    /// World-space side of each particle quad. Emitted as the `size` column, so
    /// particles are sized by the emitter rather than inheriting the caller's
    /// fallback (a grid dot's size): a fountain's grains want to be small enough
    /// that consecutive ones read as separate grains and not a poured ribbon.
    size: f32,
}

/// The alive set at `t`, as a stream. Pure: same `t` → same particles.
fn emit(spec: &Spec, t: f32) -> Stream {
    if spec.rate <= 0.0 || spec.life <= 0.0 || t < 0.0 {
        return Stream::new(0);
    }
    // Particle k is born at k/rate. Alive at t iff `0 ≤ t − k/rate < life`.
    let newest = (t * spec.rate).floor();
    let oldest = ((t - spec.life) * spec.rate).ceil().max(0.0);
    if newest < oldest {
        return Stream::new(0);
    }
    // The cap keeps the NEWEST particles: an emitter whose rate outruns the cap
    // should look like a dense young jet, not a frozen ancient cloud.
    let span = (newest - oldest) as usize + 1;
    let n = span.min(spec.max);
    let first = newest as u32 + 1 - n as u32;

    let (mut p, mut vel) = (Vec::with_capacity(n), Vec::with_capacity(n));
    let (mut ids, mut ages) = (Vec::with_capacity(n), Vec::with_capacity(n));
    let (mut index, mut count) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for k in 0..n {
        let id = first + k as u32;
        // Launch direction: the cone `angle ± spread/2`, drawn from the
        // particle's identity (never from a sequence) — see `hash`.
        let jitter = rand01(spec.seed, id, LANE_ANGLE) - 0.5;
        let deg = spec.angle + jitter * spec.spread;
        let (cx, sy) = cos_sin_cycles(deg / 360.0);
        p.push(spec.origin);
        vel.push([cx * spec.speed, sy * spec.speed]);
        ids.push(id as f32);
        ages.push(t - id as f32 / spec.rate);
        index.push(k as f32);
        count.push(n as f32);
    }
    Stream::new(n)
        .with("P", Column::Vec2(p))
        .with("vel", Column::Vec2(vel))
        .with("id", Column::Scalar(ids))
        .with("age", Column::Scalar(ages))
        .with("life", Column::Scalar(vec![spec.life; n]))
        .with("Index", Column::Scalar(index))
        .with("Count", Column::Scalar(count))
        .with("size", Column::Vec2(vec![[spec.size; 2]; n]))
}

/// The GPU kernel (ADR-0126/0130): **side metadata**, not a lowering — the manifest is
/// untouched, `eval`/`emit` stays canonical. The emitter is a **generator**, so the kernel is
/// all-Write (like `motion.grid`), and its dispatch size is playhead-dependent (`source_count`
/// takes the playhead, ADR-0130): `n(t)` is the alive-window length.
///
/// **The kernel derives `first` from `params.count`, it does NOT recompute the cap.** The CPU
/// (`emit`) computes `n = min(span, max)` then `first = newest + 1 − n`; the cook runs that
/// same `source_count` and writes `n` into `params.count`, so the kernel takes `first =
/// u32(floor(t·rate)) + 1 − count` — one `floor`, no `ceil`/`span`/cap to diverge on. Parity is
/// by construction: `params.playhead` is `clock.playhead as f32` (`lib.rs`, the uniform pack),
/// which is the exact `f32` the CPU `emit` reads (`ctx.playhead() as f32`), and `source_count`
/// truncates the same way — so `newest`, `n` and `first` are computed from one shared `f32`
/// ([[feedback_test_with_product_numbers_not_convenient_ones]] — the number that must match is
/// the one the product uses).
///
/// The hash (`em_hash3`) is the integer avalanche of [`hash`], **bit-exact** in WGSL (u32
/// wraps mod 2³² like `wrapping_mul`); the wave is the sibling `trig.rs`, byte-identical to the
/// force kernels' corrected parabolic sine (HR-5). `max` is not a kernel param — `count`
/// already carries the cap.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let em_t = params.playhead;\n\
        let em_first = u32(floor(em_t * params.rate)) + 1u - params.count;\n\
        let em_id = em_first + i;\n\
        let em_seed = u32(max(params.seed, 0.0));\n\
        let em_jitter = em_rand01(em_seed, em_id, 0u) - 0.5;\n\
        let em_deg = params.angle + em_jitter * params.spread;\n\
        let em_cs = em_cos_sin_cycles(em_deg / 360.0);\n\
        write_P(i, vec2<f32>(params.x, params.y));\n\
        write_vel(i, vec2<f32>(em_cs.x * params.speed, em_cs.y * params.speed));\n\
        write_id(i, f32(em_id));\n\
        write_age(i, em_t - f32(em_id) / params.rate);\n\
        write_life(i, params.life);\n\
        write_Index(i, f32(i));\n\
        write_Count(i, f32(params.count));\n\
        write_size(i, vec2<f32>(params.size, params.size));\n",
    wgsl_lib: "\
        fn em_hash3(a: u32, b: u32, lane: u32) -> f32 {\n\
            var h: u32 = a * 0x9e3779b9u + b * 0x85ebca6bu + lane * 0xc2b2ae35u;\n\
            h = h ^ (h >> 16u);\n\
            h = h * 0x7feb352du;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x846ca68bu;\n\
            h = h ^ (h >> 16u);\n\
            return f32(h >> 8u) / f32(16777216u);\n\
        }\n\
        fn em_rand01(seed: u32, id: u32, lane: u32) -> f32 {\n\
            return em_hash3(seed, id, lane);\n\
        }\n\
        fn em_sin_cycles(phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn em_cos_sin_cycles(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(em_sin_cycles(phase + 0.25), em_sin_cycles(phase));\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "id",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "age",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "life",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "Index",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "Count",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[
        "rate", "life", "speed", "angle", "spread", "x", "y", "seed", "size",
    ],
    // Playhead-dependent (ADR-0130): the alive-window length `n(t)`, mirroring `emit`'s count.
    source_count: Some(|param, playhead| {
        let t = playhead as f32;
        let rate = param("rate");
        let life = param("life");
        let max = (param("max").max(0.0) as usize).min(MAX_ALIVE);
        if rate <= 0.0 || life <= 0.0 || t < 0.0 {
            return 0;
        }
        let newest = (t * rate).floor();
        let oldest = ((t - life) * rate).ceil().max(0.0);
        if newest < oldest {
            return 0;
        }
        let span = (newest - oldest) as usize + 1;
        span.min(max)
    }),
    applicable: None,
};

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionEmitter))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Emitter",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rate",
        label: "Rate",
        min: 0.0,
        max: 200.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "life",
        label: "Life",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "spread",
        label: "Spread",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "x",
        label: "Origin X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "y",
        label: "Origin Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 100.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "max",
        label: "Max Particles",
        min: 1.0,
        max: 4096.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "size",
        label: "Size",
        min: 0.01,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> Spec {
        Spec {
            rate: 10.0,
            life: 1.0,
            speed: 2.0,
            angle: 90.0,
            spread: 0.0, // a pencil beam: the launch is exactly `angle`
            origin: [1.0, -1.0],
            seed: 0,
            max: 1024,
            size: 0.2,
        }
    }

    fn ids_of(s: &Stream) -> Vec<f32> {
        match s.get("id").unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("id"),
        }
    }

    #[test]
    fn the_alive_set_is_the_id_window_born_within_life() {
        // rate 10, life 1 → at t = 1.55 the alive ids are those born in
        // (0.55, 1.55]: k/10 ∈ (0.55, 1.55] → k ∈ 6..=15.
        let s = emit(&spec(), 1.55);
        assert_eq!(ids_of(&s).first().copied(), Some(6.0));
        assert_eq!(ids_of(&s).last().copied(), Some(15.0));
        assert_eq!(s.count(), 10);
        // Ids are strictly ascending (oldest first) so Index reads as age order.
        assert!(ids_of(&s).windows(2).all(|w| w[1] > w[0]));
    }

    #[test]
    fn particles_are_born_and_die_on_schedule() {
        // At t=0 only particle 0 exists; by t just under 1 life later it still
        // does; a hair past, it is gone (its successors remain).
        assert_eq!(ids_of(&emit(&spec(), 0.0)), vec![0.0]);
        assert!(ids_of(&emit(&spec(), 0.99)).contains(&0.0), "still alive");
        assert!(!ids_of(&emit(&spec(), 1.01)).contains(&0.0), "died at life");
    }

    #[test]
    fn scrubbing_backwards_reproduces_the_scene_exactly() {
        // The reference's stateful emitter could not do this. Ask for a time,
        // get the same particles — no matter what was asked before.
        let forward = emit(&spec(), 2.5);
        for t in [0.3, 7.1, 0.0, 2.5] {
            let _ = emit(&spec(), t);
        }
        let revisited = emit(&spec(), 2.5);
        assert_eq!(ids_of(&forward), ids_of(&revisited));
        assert_eq!(forward.get("vel"), revisited.get("vel"));
    }

    #[test]
    fn launch_velocity_follows_angle_and_speed() {
        // Zero spread, angle 90 (up in this Y-up world) → vel = (0, speed).
        let s = emit(&spec(), 0.5);
        match s.get("vel").unwrap() {
            Column::Vec2(v) => {
                for q in v {
                    assert!(q[0].abs() < 1e-4, "no lateral component");
                    assert!((q[1] - 2.0).abs() < 1e-4, "speed straight up");
                }
            }
            _ => panic!("vel"),
        }
        // And they all start at the origin — displacement is the integrator's.
        match s.get("P").unwrap() {
            Column::Vec2(v) => assert!(v.iter().all(|p| *p == [1.0, -1.0])),
            _ => panic!("P"),
        }
    }

    #[test]
    fn spread_fans_the_cone_but_never_past_its_half_angle() {
        let mut sp = spec();
        sp.spread = 60.0;
        sp.rate = 200.0; // plenty of samples
        let s = emit(&sp, 1.0);
        let Column::Vec2(v) = s.get("vel").unwrap() else {
            panic!("vel")
        };
        // angle 90 ± 30 → the x-component is bounded by speed·sin(30°) = 1.0.
        let max_lateral = v.iter().map(|q| q[0].abs()).fold(0.0f32, f32::max);
        assert!(
            max_lateral <= 1.01,
            "cone half-angle respected: {max_lateral}"
        );
        assert!(max_lateral > 0.5, "the cone actually fans out");
        // All still flying upward.
        assert!(v.iter().all(|q| q[1] > 0.0));
    }

    #[test]
    fn the_cap_keeps_the_newest_particles() {
        let mut sp = spec();
        sp.max = 3;
        let s = emit(&sp, 1.55); // would be 10 alive
        assert_eq!(s.count(), 3);
        assert_eq!(ids_of(&s), vec![13.0, 14.0, 15.0], "the newest three");
    }

    #[test]
    fn a_dead_or_absurd_emitter_yields_an_empty_stream_not_a_panic() {
        let mut sp = spec();
        sp.rate = 0.0;
        assert_eq!(emit(&sp, 5.0).count(), 0);
        let mut sp = spec();
        sp.life = 0.0;
        assert_eq!(emit(&sp, 5.0).count(), 0);
        // A negative playhead (a scrub past zero) emits nothing rather than
        // hashing negative ids.
        assert_eq!(emit(&spec(), -1.0).count(), 0);
        // A rate that outruns MAX_ALIVE is capped by the caller (`eval`), and
        // `emit` honours whatever cap it is handed.
        let mut sp = spec();
        sp.rate = 1e6;
        sp.max = MAX_ALIVE;
        assert_eq!(emit(&sp, 10.0).count(), MAX_ALIVE);
    }

    #[test]
    fn age_and_index_track_the_stream_order() {
        let s = emit(&spec(), 1.55);
        let Column::Scalar(age) = s.get("age").unwrap() else {
            panic!("age")
        };
        // Oldest first: age descends, and every age is within [0, life).
        assert!(age.windows(2).all(|w| w[1] < w[0]));
        assert!(age.iter().all(|a| *a >= 0.0 && *a < 1.0));
        let Column::Scalar(idx) = s.get("Index").unwrap() else {
            panic!("Index")
        };
        assert_eq!(idx.first().copied(), Some(0.0));
        assert_eq!(idx.last().copied(), Some(9.0));
    }

    #[test]
    fn particles_carry_their_own_size() {
        // Without this column the lowering would fall back to the caller's
        // default (a grid dot), and a dense jet would read as a poured ribbon.
        let s = emit(&spec(), 0.5);
        match s.get("size").unwrap() {
            Column::Vec2(v) => assert!(v.iter().all(|q| *q == [0.2, 0.2])),
            _ => panic!("size"),
        }
    }

    /// The `eval` seam itself (audit 2026-07-10: every behavioural test above
    /// drives `emit()` directly, so the ctx.param/ctx.playhead → `Spec` wiring
    /// was untested): cook the REAL node with overridden params and assert the
    /// alive set reflects them — rate·window rows, all at the (x, y) origin,
    /// sized by the `size` param as a Vec2 column.
    #[test]
    fn eval_maps_params_and_playhead_into_the_spec() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::Graph;
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionEmitter as &dyn NodeOp)
            }
        }
        let mut g = Graph::new();
        let em = g.add_node("motion.emitter");
        for (name, v) in [
            ("rate", 2.0),
            ("life", 1.0),
            ("speed", 0.0),
            ("x", 3.0),
            ("y", 4.0),
            ("size", 0.25),
        ] {
            g.set_param(em, name, v);
        }
        let mut cook = Cook::new();
        // t = 0.9, rate 2 → births at 0 and 0.5, both younger than life 1.
        let out = cook.cook(&g, &Ops, em, 0.9).unwrap();
        let s = out[0].as_stream();
        assert_eq!(s.count(), 2, "rate × window alive");
        match s.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[3.0, 4.0]; 2], "the (x,y) origin"),
            _ => panic!("P"),
        }
        match s.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.25, 0.25]; 2], "size param → Vec2 column"),
            _ => panic!("size"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        use ph2d_nodegraph::cook::OpResolver;
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
