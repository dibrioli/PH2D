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
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ID_WRAP, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
mod params_ui;
mod trig;
use hash::rand01;
use params_ui::{PARAM_HARD_MAX, PARAM_HINTS};
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Hard ceiling on the alive set, whatever `rate × life` asks for.
///
/// **It is a MEMORY budget, and nothing else.** Every earlier number here was
/// something else wearing that name: 4096 guarded "a stream no renderer would
/// draw anyway" (the renderer draws 2M instances at 4,02 ms), and 16.384 was the
/// CPU fallback's frame time — which let the SLOWEST path set the ceiling of the
/// fastest one, in a module whose entire reason to exist is the fastest one.
///
/// Measured on the RTX (`emitter_sim_ceiling_probe`, emitter → wind → drag →
/// integrate, ms per tick):
///
/// | window    | GPU   | CPU     |
/// |-----------|-------|---------|
/// | 262.144   | 0,277 | 13,060  |
/// | 1.048.576 | 0,984 | 52,608  |
/// | 4.194.304 | 3,636 | 227,800 |
///
/// **4,19 million particles simulate in 3,6 ms** — 22% of a 60 fps frame. The
/// CPU cannot follow, and that is the correct division of labour, not a reason
/// to cap the GPU: the CPU is the canonical REFERENCE (it defines the answer the
/// parity gates hold the GPU to, and it may take as long as a test needs), while
/// the product path is the device.
///
/// So the number comes from what the device must hold: at `4 × 1024 × 1024`
/// elements the emitter's eight columns are ~44 B each, double-buffered across
/// the sim's `pre` loop ⇒ ≈ 370 MB of GPU residency for a maxed-out emitter.
/// That is a deliberate ceiling on an artist who asks for everything, not an
/// accident.
///
/// ⚠️ **The ceiling stays SHARED between the two paths.** ADR-0130's parity is
/// by construction from one count law; a GPU-only ceiling would give the two
/// sides different `n` for the same document, and the scrub, the hybrid seam and
/// the gather would each be reading a window the other half never had. A CPU too
/// slow to PLAY a 4M sim is a performance fact; it still computes the same
/// answer, which is all a reference has to do.
const MAX_ALIVE: usize = 4 * 1024 * 1024;
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

/// **The count law, and the only copy of it** — in `f64`, because the spawn index
/// is what has to stay exact.
///
/// `emit` and the GPU `count_law` used to compute this independently in
/// `f32`, and parity held only because both read the same `f32`. Two doors to
/// one question ([[feedback_two_doors_to_the_same_question_diverge]]), and both
/// of them wrong at scale: `floor(t·rate)` in `f32` starts skipping integers at
/// 2²⁴, so a rate in the millions loses the window after four seconds. Now there
/// is one door, it answers in integers, and the kernel is TOLD the answer.
fn window(rate: f32, life: f32, max: usize, t: f32) -> SourceWindow {
    if rate <= 0.0 || life <= 0.0 || t < 0.0 {
        return SourceWindow::of_count(0);
    }
    let (t, rate, life) = (f64::from(t), f64::from(rate), f64::from(life));
    // Particle k is born at k/rate. Alive at t iff `0 ≤ t − k/rate < life`.
    let newest = (t * rate).floor();
    let oldest = ((t - life) * rate).ceil().max(0.0);
    if newest < oldest {
        return SourceWindow::of_count(0);
    }
    // The cap keeps the NEWEST particles: an emitter whose rate outruns the cap
    // should look like a dense young jet, not a frozen ancient cloud.
    let span = (newest - oldest) as u64 + 1;
    let count = span.min(max as u64);
    let first = newest as u64 + 1 - count;
    SourceWindow {
        count: count as usize,
        // Wrapped into the exact `f32` integer range. Below 2²⁴ this is the
        // identity, so every scene that works today is BYTE-identical.
        first: (first % u64::from(ID_WRAP)) as u32,
        // The oldest particle's age, from the one place that can compute it
        // without cancellation.
        age_first: (t - first as f64 / rate) as f32,
    }
}

/// The alive set at `t`, as a stream. Pure: same `t` → same particles.
fn emit(spec: &Spec, t: f32) -> Stream {
    let w = window(spec.rate, spec.life, spec.max, t);
    let n = w.count;
    if n == 0 {
        return Stream::new(0);
    }

    let (mut p, mut vel) = (Vec::with_capacity(n), Vec::with_capacity(n));
    let (mut ids, mut ages) = (Vec::with_capacity(n), Vec::with_capacity(n));
    let (mut index, mut count) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for k in 0..n {
        let id = (w.first + k as u32) % ID_WRAP;
        // Launch direction: the cone `angle ± spread/2`, drawn from the
        // particle's identity (never from a sequence) — see `hash`.
        let jitter = rand01(spec.seed, id, LANE_ANGLE) - 0.5;
        let deg = spec.angle + jitter * spec.spread;
        let (cx, sy) = cos_sin_cycles(deg / 360.0);
        p.push(spec.origin);
        vel.push([cx * spec.speed, sy * spec.speed]);
        ids.push(id as f32);
        // The SAME two-step the kernel runs (`age_first − k/rate`), not
        // `t − id/rate`: the kernel has to mirror this arithmetic exactly, and
        // the one-step form is the one that cancels.
        ages.push(w.age_first - k as f32 / spec.rate);
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
        let em_id = (params.window_first + i) % 16777216u;\n\
        let em_seed = u32(max(params.seed, 0.0));\n\
        let em_jitter = em_rand01(em_seed, em_id, 0u) - 0.5;\n\
        let em_deg = params.angle + em_jitter * params.spread;\n\
        let em_cs = em_cos_sin_cycles(em_deg / 360.0);\n\
        write_P(i, vec2<f32>(params.x, params.y));\n\
        write_vel(i, vec2<f32>(em_cs.x * params.speed, em_cs.y * params.speed));\n\
        write_id(i, f32(em_id));\n\
        write_age(i, params.window_age - f32(i) / params.rate);\n\
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
    count_law: Some(|c| {
        window(
            (c.param)("rate"),
            (c.param)("life"),
            ((c.param)("max").max(0.0) as usize).min(MAX_ALIVE),
            c.playhead as f32,
        )
    }),
    variant_by_param: None,
    applicable: None,
};

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionEmitter))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ADR-0130: the emitter is the SOURCE of the dense window.
    reg.register_dense_window(MANIFEST.id);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Emitter",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
