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
//! Params: `rate` (particles/sec), `life` (sec), `speed`, `speed_random`,
//! `angle` (degrees CCW from +X; `90` is up in this Y-up world), `spread`
//! (degrees of cone), `x`/`y` (origin), `seed`, `max` (hard cap — the newest
//! `max` survive), `size` (world-space side of each particle quad) and
//! `size_random`.
//!
//! ## `speed_random` — the muzzle velocity varies per particle
//!
//! Every particle left the barrel at exactly the same speed, so a fountain read as a rigid
//! arc rather than a spray. The reference has the knob everywhere (Particular *Velocity
//! Random*, Niagara's *Add Velocity in Cone* taking `Velocity Strength` as a range, Cavalry's
//! per-particle behaviours on *Initial Speed*).
//!
//! ⚠️ **Nothing downstream could supply it, and that is why it is a param and not a chain.**
//! The emitter closes `P = origin + v·age` *inside* the node, so a `motion.drive` downstream
//! only adds a CONSTANT displacement; scaling the launch would mean re-deriving `v·age` per
//! particle, and `vel` is a **Vec2** column that the scalar `value.attribute` cannot take apart.
//!
//! The draw is `speed × (1 + (h − ½)·r·2)` on its own hash lane, so it is the same stateless,
//! scrub-exact identity the cone angle already uses — **not** a sequence, and **not**
//! `value.instance_field(Random)`, which hashes the *index*: in an emitter the alive window
//! slides, so a particle's index changes the moment an older one dies and anything built on it
//! **flickers**. `r = 0` is byte-identical (`speed × 1.0`).
//!
//! ⚠️ Above `r = 1` the multiplier goes negative and the particle launches into the opposite
//! half of the cone. That is not guarded — it is `spread`'s job, expressed better — but it IS
//! mirrored, so the two paths agree about it.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ID_WRAP, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod gpu;
mod hash;
mod params_ui;
mod trig;
use gpu::GPU_KERNEL;
use hash::rand01;
use params_ui::{
    PARAM_GATES, PARAM_GROUPS, PARAM_HARD_MAX, PARAM_HARD_MIN, PARAM_HINTS, PARAM_UNITS,
};
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
/// Hash lane for the launch SPEED draw. A lane of its own, never a re-use of
/// [`LANE_ANGLE`]: sharing one would tie a particle's speed to its direction, so the fast ones
/// would all sit on the same side of the cone and the spray would read as a fan.
const LANE_SPEED: u32 = 1;
/// The two hash lanes for the birth POSITION inside the emitter's shape. Two, because a filled
/// area is a two-dimensional draw and one number cannot fill one — reusing a single lane for
/// both would lay every particle on a diagonal.
const LANE_SHAPE_U: u32 = 2;
const LANE_SHAPE_V: u32 = 3;
/// Hash lane for the particle's SIZE draw. Its own, for the reason [`LANE_SPEED`] gives: sharing
/// one would tie a particle's size to another of its properties, and a spray whose big grains are
/// exactly the fast ones reads as a rule rather than as variety.
const LANE_SIZE: u32 = 4;

/// Where a particle is born, relative to the emitter's origin.
///
/// The **integer** the `shape_mode` param carries; anything else reads as [`Self::Point`], which
/// is what every graph that predates the param means.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Shape {
    /// All from one spot — the emitter that always shipped.
    Point,
    /// Anywhere inside the ellipse `shape_w × shape_h`.
    Disc,
    /// On its outline only.
    Ring,
    /// Anywhere inside the rectangle of half-extents `shape_w × shape_h`.
    Rect,
}

impl Shape {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Disc,
            2 => Self::Ring,
            3 => Self::Rect,
            _ => Self::Point,
        }
    }
}

/// The birth offset of particle `id` inside `shape`, in the emitter's own frame — or `None`
/// when the shape is a point and there is **no offset at all**.
///
/// ⚠️ **The `None` is what makes the default byte-identical rather than merely zero.** Returning
/// `[0.0, 0.0]` would have the caller add it, and `-0.0 + 0.0` is `+0.0`: an origin the artist
/// typed as `-0` would come back with its sign flipped. The type says *there is nothing to add*,
/// so there is one place that knows it and no unreachable arm anywhere.
///
/// The draws are **uniform over the AREA**, not over the parameter: a disc sampled with a raw
/// radius piles up in the middle, which reads as a bright core the artist did not author — hence
/// the `sqrt`. Both lanes are the particle's own identity, so a birth place is as scrub-exact as
/// its velocity. The kernel mirrors this term for term.
fn birth_offset(shape: Shape, w: f32, h: f32, seed: u32, id: u32) -> Option<[f32; 2]> {
    if shape == Shape::Point {
        return None;
    }
    let u = rand01(seed, id, LANE_SHAPE_U);
    let v = rand01(seed, id, LANE_SHAPE_V);
    Some(match shape {
        Shape::Rect => [(u - 0.5) * 2.0 * w, (v - 0.5) * 2.0 * h],
        // The outline: one draw is enough, and the second lane stays unread so the ring does
        // not silently depend on a number it has no use for.
        Shape::Ring => {
            let (c, s) = cos_sin_cycles(u);
            [c * w, s * h]
        }
        // `sqrt(u)` is the area-uniform radius; the affine `w`/`h` carries it to an ellipse.
        Shape::Disc | Shape::Point => {
            let r = u.sqrt();
            let (c, s) = cos_sin_cycles(v);
            [c * w * r, s * h * r]
        }
    })
}

/// **Which way a particle leaves** — along the artist's `angle`, or along its own radius.
///
/// `Outwards`/`Inwards` are only a question once a particle is born somewhere OTHER than the
/// origin, so this param arrived with the shape and not before it (the sheet marked it
/// *"not expressible without a shape"*, and the shape is what moved that number).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum DirMode {
    /// The cone `angle ± spread/2` — what always shipped.
    Angle,
    /// Away from the emitter's centre, through the birth place.
    Outwards,
    /// Towards it.
    Inwards,
}

impl DirMode {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Outwards,
            2 => Self::Inwards,
            _ => Self::Angle,
        }
    }
}

/// The unit axis a particle leaves along, BEFORE the cone's jitter — or `None` when there is
/// no radius to leave along and the artist's `angle` is the only direction there is.
///
/// ⚠️ **`None` is the whole reason `Angle` stays byte-identical.** The caller keeps its single
/// `cos_sin_cycles(angle + jitter·spread)` on that arm — the same expression, not an equivalent
/// one — and only the radial arms ROTATE. Two `cos_sin` calls composed are not the same `f32` as
/// one call on the summed angle, so folding the two arms together would have moved every existing
/// emitter by an ulp and taken the GPU parity with it.
///
/// ⚠️ **And the axis is ROTATED, never re-derived.** The birth offset is already a vector; asking
/// `atan2` for its angle, adding the jitter and calling `cos_sin` back would spend two
/// approximations to learn what a 2×2 rotation composes exactly — the lesson the radial array's
/// `align` paid a wave earlier.
///
/// A particle born exactly at the centre has no radial direction (a `Point` emitter, a zero-sized
/// shape, or a `Disc` draw that lands on the middle), so it falls back to the cone. That is the
/// honest answer, not a special case: "outwards" from a zero-length vector is not a direction.
fn radial_axis(dir: DirMode, offset: Option<[f32; 2]>) -> Option<[f32; 2]> {
    let sign = match dir {
        DirMode::Angle => return None,
        DirMode::Outwards => 1.0,
        DirMode::Inwards => -1.0,
    };
    let d = offset?;
    let len2 = d[0] * d[0] + d[1] * d[1];
    if len2 <= 0.0 {
        return None;
    }
    let inv = sign / len2.sqrt();
    Some([d[0] * inv, d[1] * inv])
}

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
        // How much the muzzle velocity varies per particle, as a fraction of `speed`:
        // `1` spreads it over `0 .. 2×`. APPENDED, and `0` is the single speed that
        // always shipped, byte for byte.
        ParamSpec {
            name: "speed_random",
            default: 0.0,
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
        // WHERE a particle is born — `0` is the point emitter that always shipped, and the two
        // extents are inert under it. APPENDED, so a saved graph reads all three as absent.
        ParamSpec {
            name: "shape_mode",
            default: 0.0,
        },
        ParamSpec {
            name: "shape_w",
            default: 1.0,
        },
        ParamSpec {
            name: "shape_h",
            default: 1.0,
        },
        // APPENDED, and `0` = Angle: a saved graph reads it as absent and launches down the
        // cone it always launched down.
        ParamSpec {
            name: "dir_mode",
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
        // How much each particle's size varies, as a fraction of `size` — the twin of
        // `speed_random`, one law for both. APPENDED, and `0` is the single size that always
        // shipped, byte for byte.
        ParamSpec {
            name: "size_random",
            default: 0.0,
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
            speed_random: ctx.param("speed_random"),
            angle: ctx.param("angle"),
            spread: ctx.param("spread"),
            origin: [ctx.param("x"), ctx.param("y")],
            shape: Shape::from_param(ctx.param("shape_mode")),
            shape_wh: [ctx.param("shape_w"), ctx.param("shape_h")],
            dir: DirMode::from_param(ctx.param("dir_mode")),
            seed: ctx.param("seed").max(0.0) as u32,
            max: (ctx.param("max").max(0.0) as usize).min(MAX_ALIVE),
            size: ctx.param("size").max(0.0),
            size_random: ctx.param("size_random"),
        };
        ctx.emit(emit(&spec, ctx.playhead() as f32));
    }
}

/// The emitter's resolved params (a struct so [`emit`] is a testable pure fn).
struct Spec {
    rate: f32,
    life: f32,
    speed: f32,
    /// Per-particle spread of the muzzle velocity, as a fraction of `speed`.
    speed_random: f32,
    angle: f32,
    spread: f32,
    origin: [f32; 2],
    /// WHERE inside the emitter a particle is born.
    shape: Shape,
    /// The shape's half-extents (radii for [`Shape::Disc`]/[`Shape::Ring`]), world units.
    shape_wh: [f32; 2],
    /// Which way a particle leaves (`Angle` unless a shape gives it a radius).
    dir: DirMode,
    seed: u32,
    max: usize,
    /// World-space side of each particle quad. Emitted as the `size` column, so
    /// particles are sized by the emitter rather than inheriting the caller's
    /// fallback (a grid dot's size): a fountain's grains want to be small enough
    /// that consecutive ones read as separate grains and not a poured ribbon.
    size: f32,
    /// Per-particle spread of `size`, as a fraction of it.
    size_random: f32,
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
    let mut sizes = Vec::with_capacity(n);
    for k in 0..n {
        let id = (w.first + k as u32) % ID_WRAP;
        // Where it is born — read BEFORE the direction now, because `Outwards`/`Inwards` are a
        // question about the offset. Pure, so hoisting it moves no value.
        let off = birth_offset(
            spec.shape,
            spec.shape_wh[0],
            spec.shape_wh[1],
            spec.seed,
            id,
        );
        // Launch direction: the cone `angle ± spread/2`, drawn from the
        // particle's identity (never from a sequence) — see `hash`.
        let jitter = rand01(spec.seed, id, LANE_ANGLE) - 0.5;
        let (cx, sy) = match radial_axis(spec.dir, off) {
            // The expression that always shipped, character for character — see `radial_axis`.
            None => cos_sin_cycles((spec.angle + jitter * spec.spread) / 360.0),
            // The cone is the SAME half-width here; only what it opens around changed, so
            // `spread` keeps meaning one thing in all three modes.
            Some(a) => {
                let (rc, rs) = cos_sin_cycles(jitter * spec.spread / 360.0);
                (a[0] * rc - a[1] * rs, a[0] * rs + a[1] * rc)
            }
        };
        // Its own lane, and the SAME term order the kernel runs: parity here is bit-exact by
        // construction, not by epsilon. With `speed_random = 0` the factor is `1.0` exactly.
        let js = rand01(spec.seed, id, LANE_SPEED) - 0.5;
        let speed = spec.speed * (1.0 + js * spec.speed_random * 2.0);
        p.push(match off {
            Some(d) => [spec.origin[0] + d[0], spec.origin[1] + d[1]],
            None => spec.origin,
        });
        vel.push([cx * speed, sy * speed]);
        ids.push(id as f32);
        // The SAME two-step the kernel runs (`age_first − k/rate`), not
        // `t − id/rate`: the kernel has to mirror this arithmetic exactly, and
        // the one-step form is the one that cancels.
        ages.push(w.age_first - k as f32 / spec.rate);
        index.push(k as f32);
        count.push(n as f32);
        // Same law as `speed_random`, same term order as the kernel — with `size_random = 0` the
        // factor is `1.0` exactly and `size × 1.0` is exact in IEEE-754, so the column is the
        // uniform one that always shipped, byte for byte.
        //
        // ⚠️ **The clamp is where this DIFFERS from its twin, and the difference is real.** Past
        // `r = 1` the multiplier turns negative; for `speed` that is a picture (the particle
        // launches into the opposite half of the cone, which `spread` says better), for a size it
        // is not — a quad of negative side is a winding flip or nothing at all, never something an
        // artist asked for. So the result honours the same floor the base `size` already carries.
        let jz = rand01(spec.seed, id, LANE_SIZE) - 0.5;
        let sz = (spec.size * (1.0 + jz * spec.size_random * 2.0)).max(0.0);
        sizes.push([sz, sz]);
    }
    Stream::new(n)
        .with("P", Column::Vec2(p))
        .with("vel", Column::Vec2(vel))
        .with("id", Column::Scalar(ids))
        .with("age", Column::Scalar(ages))
        .with("life", Column::Scalar(vec![spec.life; n]))
        .with("Index", Column::Scalar(index))
        .with("Count", Column::Scalar(count))
        .with("size", Column::Vec2(sizes))
}

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
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_hard_min(MANIFEST.id, PARAM_HARD_MIN);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
