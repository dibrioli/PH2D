#![forbid(unsafe_code)]
//! **`sim.step`** — one integration step INSIDE a simulation zone (Motion Nodes O4, doc 48).
//!
//! ## Why this is not `motion.integrate`
//!
//! `motion.integrate` is a stateful node: it holds the simulation itself, pairs each element of
//! a LIVE rest chain with its own prior row by `id`, and adds the accumulated displacement to a
//! position the rest of the graph keeps re-authoring. It is the right node when the sim is a
//! *deviation from an animation*.
//!
//! Inside a zone there is no rest chain. **The stream IS the state** — it came out of the zone,
//! it goes back into the zone, and its positions are the truth. So the step is *stateless*:
//! read `vel`/`accel` off the stream, write `P`/`vel` back. It is Blender's `Set Position` with
//! `velocity × Delta Time`, which is exactly what their simulation zone makes you write by hand.
//!
//! Putting `motion.integrate` inside a zone would give the sim TWO memories — the zone's and the
//! integrator's own `pre` self-loop — and they would disagree the moment a kill node removed a
//! row. One state, one owner: the zone.
//!
//! ## Semi-implicit Euler, and the clock the state carries
//!
//! `vel += accel·dt·w` first, then `P += vel·dt·w` with the NEW velocity (symplectic — stable
//! under oscillatory forces; the same integrator `motion.integrate` uses, for the same reason).
//! `w` is `inv_mass`, so `w = 0` is a hard pin.
//!
//! **`dt` comes from a clock column the state carries** (`sim_t`), not from a frame counter: an
//! element that has never been stepped has no `sim_t`, so its `dt` is 0 and it simply starts.
//! Without that, a zone dropped onto a graph at playhead = 8 s would take one 8-second step and
//! fling everything to infinity on the frame it was created. The clamp (`MAX_DT`) does the same
//! job for a scrub.
//!
//! `accel` is CONSUMED (dropped): it is the transient the `force.*` nodes accumulate, and a step
//! that left it on the stream would re-apply the same force forever, one tick out of date.
//!
//! ## It also keeps the `age`
//!
//! The step owns the clock, so the step owns the ageing: every element's `age` grows by the same
//! `dt` its motion did (doc 50). An element with no `age` yet is newborn — it starts at zero.
//! Nothing else in the library could do this honestly: a node without the sim's own clock would
//! have to guess the frame rate, and the age is what `sim.lifetime` kills by and what
//! `value.attribute` hands to a colour ramp.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The largest step the integrator will take, in seconds. A pause, a scrub or a dropped frame
/// otherwise arrives as one enormous `dt` and the sim explodes on the frame it resumes.
const MAX_DT: f32 = 1.0 / 20.0;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("sim.step"),
    name: "sim.step",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead (it is the clock the step is measured against), holds no state of its
    // own — the zone holds it.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Velocity retained per second: 1 = frictionless (the default, and bit-identical to no
        // damping at all), 0.1 = molasses. Applied linearly per step (`1 - (1-damping)·dt`)
        // rather than as `damping^dt`: same thing to first order, and transcendental-free (HR-5).
        ParamSpec {
            name: "damping",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The WGSL port of [`step`], element for element (ADR-0135 — the sim-zone
/// family on the GPU). Unlike `motion.integrate` there is no `rest` chain and no
/// id-gather: inside a zone **the stream IS the state**, so this is a single-port
/// kernel reading its own columns and writing them back.
///
/// **The clock is per-element** (`read_sim_t(i)`, not element 0's): a zone that
/// spawns carries newborns with no `sim_t` beside veterans that have one, and the
/// step ages each by *its own* `dt`. When the column is absent entirely,
/// `HAS_sim_t` is false and every element falls back to `params.playhead` ⇒
/// `dt = 0` ⇒ a fresh element STARTS rather than leaping (the `scalar_col` →
/// `None` → `0.0` path on the CPU).
///
/// **The finiteness guard keeps the OLD value**, not a zero: a diverged element
/// resets to where it was rather than NaN-poisoning the zone — so `out_*` seed
/// from `read_*(i)` and are only overwritten when the new state is finite, exactly
/// as the CPU's `if …all(is_finite) { vel[i] = v; p[i] = q; }`.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let st_t_prev = select(params.playhead, read_sim_t(i), HAS_sim_t);\n\
        let st_dt = clamp(params.playhead - st_t_prev, 0.0, STEP_MAX_DT);\n\
        let st_w = read_inv_mass(i);\n\
        var st_v = read_vel(i);\n\
        var st_q = read_P(i);\n\
        let st_a = read_accel(i);\n\
        // Semi-implicit: velocity first, then position with the NEW velocity.\n\
        st_v = st_v + st_a * st_dt * st_w;\n\
        // Linear damping — first-order equivalent of `damping^dt`, transcendental-free.\n\
        let st_keep = 1.0 - (1.0 - params.damping) * st_dt;\n\
        st_v = st_v * st_keep;\n\
        st_q = st_q + st_v * st_dt * st_w;\n\
        var st_out_v = read_vel(i);\n\
        var st_out_q = read_P(i);\n\
        if (step_finite(st_v) && step_finite(st_q)) {\n\
        \x20   st_out_v = st_v;\n\
        \x20   st_out_q = st_q;\n\
        }\n\
        write_P(i, st_out_q);\n\
        write_vel(i, st_out_v);\n\
        // The step owns the clock, so the step owns the ageing (doc 50).\n\
        write_age(i, read_age(i) + st_dt);\n\
        write_sim_t(i, params.playhead);\n",
    wgsl_lib: "\
        const STEP_MAX_DT: f32 = 0.05;\n\
        // WGSL has no `isFinite`; `abs(x) <= F32_MAX` is exactly \"not NaN and not\n\
        // infinite\" (every compare against NaN is false, an inf exceeds the max),\n\
        // per-lane rather than through `max()` whose NaN behaviour is undefined.\n\
        const STEP_F32_MAX: f32 = 3.4028235e38;\n\
        fn step_finite(v: vec2<f32>) -> bool {\n\
        \x20   return abs(v.x) <= STEP_F32_MAX && abs(v.y) <= STEP_F32_MAX;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // The state's own clock, from which `dt` is derived — per element, like
        // the CPU's `t_prev[i]`. Absent → `HAS_sim_t` false → `params.playhead`
        // (`dt = 0`), matching `scalar_col`'s `None`.
        ColumnBinding {
            column: "sim_t",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // A row with no `age` is newborn: identity 0.
        ColumnBinding {
            column: "age",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // Transient: eaten here so every tick starts from zero acceleration —
        // without this the same force would re-apply forever, one tick out of date.
        ColumnBinding {
            column: "accel",
            dim: Dim::Vec2,
            access: ColumnAccess::Consume,
            identity: [0.0; 4],
            port: 0,
        },
        // Absent = every element free (`w = 1`); `·1.0` is exact, so a pre-pin
        // graph integrates identically.
        ColumnBinding {
            column: "inv_mass",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &["damping"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

fn vec2_col(s: &Stream, name: &str, n: usize) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}

fn scalar_col(s: &Stream, name: &str, n: usize, default: f32) -> Option<Vec<f32>> {
    match s.get(name) {
        Some(Column::Scalar(v)) if v.len() == n => Some(v.clone()),
        Some(_) => Some(vec![default; n]),
        None => None,
    }
}

/// One step of the whole zone's state, as a pure function — the node.
fn step(state: &Stream, playhead: f32, damping: f32) -> Stream {
    let n = state.count();
    let mut out = Stream::new(n);
    // Everything the sim does not own rides through untouched — `id` above all, so a kill node
    // downstream can tell the survivors apart next tick. `accel` is consumed.
    for (name, col) in state.columns() {
        if !matches!(name.as_str(), "accel" | "P" | "vel" | "sim_t" | "age") {
            out.set(name.clone(), col.clone());
        }
    }
    // The age grows by the same `dt` the motion did — the step owns the clock, so the step owns
    // the ageing. A row with no `age` is newborn: it starts at zero.
    let age_prev = scalar_col(state, "age", n, 0.0).unwrap_or_else(|| vec![0.0; n]);

    let mut p = vec2_col(state, "P", n);
    let mut vel = vec2_col(state, "vel", n);
    let accel = vec2_col(state, "accel", n);
    let w = scalar_col(state, "inv_mass", n, 1.0).unwrap_or_else(|| vec![1.0; n]);
    // No clock on the state = these elements have never been stepped: `dt` is 0, they start.
    let t_prev = scalar_col(state, "sim_t", n, playhead);

    for i in 0..n {
        let dt = t_prev
            .as_ref()
            .map(|t| (playhead - t[i]).clamp(0.0, MAX_DT)) // CLAMP-OK: const bounds, min < max
            .unwrap_or(0.0);
        let (mut v, mut q, a, wi) = (vel[i], p[i], accel[i], w[i]);
        // Semi-implicit: velocity first, then position with the NEW velocity.
        v[0] += a[0] * dt * wi;
        v[1] += a[1] * dt * wi;
        // Linear damping — first-order equivalent of `damping^dt`, and transcendental-free
        // (HR-5). At `damping = 1` it is exactly `·1.0`: an undamped sim is bit-identical.
        let keep = 1.0 - (1.0 - damping) * dt;
        v[0] *= keep;
        v[1] *= keep;
        q[0] += v[0] * dt * wi;
        q[1] += v[1] * dt * wi;
        // A diverged element resets rather than freezing (or NaN-poisoning) the whole zone.
        if v.iter().chain(&q).all(|x| x.is_finite()) {
            vel[i] = v;
            p[i] = q;
        }
    }

    let age: Vec<f32> = (0..n)
        .map(|i| {
            let dt = t_prev
                .as_ref()
                .map(|t| (playhead - t[i]).clamp(0.0, MAX_DT)) // CLAMP-OK: const bounds, min < max
                .unwrap_or(0.0);
            age_prev[i] + dt
        })
        .collect();

    out.set("P", Column::Vec2(p));
    out.set("vel", Column::Vec2(vel));
    out.set("age", Column::Scalar(age));
    out.set("sim_t", Column::Scalar(vec![playhead; n]));
    out
}

struct SimStep;

impl NodeOp for SimStep {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let playhead = ctx.playhead() as f32;
        let damping = ctx.param("damping");
        let out = step(ctx.input(0), playhead, damping);
        ctx.emit(out);
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SimStep))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Simulation Step",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

/// Param UI hints.
static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "damping",
    label: "Damping",
    min: 0.0,
    max: 1.0,
    step: 0.01,
    widget: ParamWidget::Slider,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
