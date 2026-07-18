#![forbid(unsafe_code)]
//! `motion.integrate` — the **one** sequential integrator of the Motion dynamics
//! chain (plan §1.6, M2). Forces are `Pure` nodes that accumulate into the
//! transient `accel` column; this node semi-implicit-Euler-integrates that
//! acceleration into `vel` and an accumulated displacement `sim_d`, and emits
//! `P = rest.P (live) + sim_d` — so upstream animation (oscillator, orbit, a
//! moving emitter) keeps composing with the physics, exactly the reference's
//! `x = in.x + s.dx` behaviour but with the state as visible stream columns
//! instead of hidden per-node maps (replayable, GPU-lowerable ping-pong).
//!
//! ## Topology (the `pre` loop)
//!
//! ```text
//! rest ──fwd──> integrate.rest                 (the live upstream chain)
//! integrate.out ──pre──> force₁ ─…─ forceₙ ──fwd──> integrate.forces
//! ```
//!
//! With no forces the loop is the plain self-loop `out --pre--> forces` (the
//! editor auto-wires it on add: the plumbing recognises a feedback input named
//! `state` OR `forces` with the output's type — `motion_bridge_plumbing.rs`;
//! this node names it `forces` because the force chain is what plugs in). At
//! tick 0 the `pre` reads Empty → the node **seeds** from `rest` (`sim_d = 0`,
//! `vel` = rest's `vel` column or zero) and nothing moves; from tick 1 on it
//! steps.
//!
//! ## Column contract
//!
//! - Reads from `forces`: `vel`, `sim_d`, `sim_t`, and the transient `accel`
//!   the in-loop forces accumulated. `accel` is **consumed** (never emitted) so
//!   every tick starts from zero acceleration.
//! - Emits: every non-sim column copied **live from `rest`** (tint/size/uv/
//!   falloff keep animating), plus `P = rest.P + sim_d`, `vel`, `sim_d`, and
//!   `sim_t` (= this eval's playhead, from which the next eval derives `dt`).
//! - `dt = clamp(playhead − sim_t, 0, MAX_DT)` — derived from the playhead the
//!   state itself carries, so there is no cross-crate timestep constant and a
//!   backwards playhead jump (loop wrap) freezes for one tick instead of
//!   exploding. The shell cooks once per fixed tick, so in steady state `dt`
//!   IS the fixed timestep (deterministic, HR-5: arithmetic only).
//! - A non-finite `vel`/`sim_d` element resets that instance (the reference's
//!   NaN guard: recover automatically instead of freezing a diverged sim).
//!
//! ## Identity: the `id` column vs position
//!
//! A stream carrying an **`id` column** (the particle emitter stamps one) has
//! *element identity*: the set churns every tick as particles are born and die,
//! yet each survivor must keep its velocity and displacement. State is matched
//! by id; an id with no prior state is **seeded** (taking `vel` from `rest`, so
//! the emitter's muzzle velocity is what launches it) and a state row whose id
//! vanished dies with it.
//!
//! A stream with **no `id`** (a grid, a cloner) has only positional identity, so
//! a changed count means the whole set was rebuilt: the state re-seeds wholesale
//! rather than pairing unrelated elements by index.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod columns;
use columns::{ids_of, scalar_to_n, vec2_to_n};
use std::collections::BTreeMap;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Ceiling on a single integration step, in seconds. In steady state `dt` is
/// the fixed timestep (~1/60); this only guards a pathological playhead jump
/// (a scrub / loop-wrap) from becoming one giant unstable step.
const MAX_DT: f32 = 0.1;

/// The inverse-mass column (PBD's `w = 1/m`) that `motion.pin_constraint`
/// writes: `1` = free (the default when absent — every pre-pin graph integrates
/// exactly as it did), `0` = pinned. It scales this node's whole contribution,
/// so a pinned element keeps `sim_d = 0` and rides its rest animation instead.
/// A string convention shared by the module's solvers, spelled locally by each
/// reader (like `P` / `falloff` / `accel`) rather than coupling the crates.
const INV_MASS: &str = "inv_mass";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.integrate"),
    name: "motion.integrate",
    inputs: &[
        PortSpec {
            name: "rest",
            ty: INST_VEC2,
        },
        // The feedback port. `forces` receives the force chain's output (last
        // tick's state with `accel` accumulated); the editor owns its `pre`
        // plumbing — self-loop when empty, `out --pre--> chain head` when a
        // chain is wired in (docs/Motion Nodes/03, O1). The user never draws
        // the loop.
        PortSpec {
            name: "forces",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Temporal: `eval` reads `ctx.playhead()` (it stamps `sim_t` and derives
    // `dt = playhead − sim_t`), and only a Temporal manifest folds the playhead
    // into the memo fingerprint. The consumed `pre` edge already forces a
    // re-cook per tick, which masks this during forward playback — but a
    // same-tick re-cook at a moved playhead (checkpoint/restore scrub, M2.N2)
    // would return a stale `sim_t`/trajectory under `Pure`. Convention: reads
    // playhead ⇒ Temporal (`motion.oscillator`, `pulse.beat`).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (GPU/M5 **Fase 3**, ADR-0126 side channel + ADR-0127/0130):
/// the WGSL port of [`step`], element for element.
///
/// **Pairing matches the CPU's, both arms** (ADR-0130). The CPU pairs state to
/// elements by `id` when the stream has identity (a gather) and positionally
/// otherwise:
///
/// - **Positional** (a grid): no `id` column, so `gather_row(i) = i` and
///   `gather_paired(i)` is always true — the sequencer's `count == dispatch`
///   presence rule is the other half (a stale-length state reads ABSENT →
///   `HAS_forces_sim_d` false → seed, the `_ if sn == n` arm).
/// - **Id-gather** (a `motion.emitter`): the `id` binding below is a
///   [`ColumnAccess::GatherKey`] — a conditional refusal that CLAIMS when `rest`
///   is a dense ascending id window (ADR-0130 D2). Then `gather_row(i) =
///   current_id − prev_first` is the prior-state row of this element's id (dense
///   state ids are `[prev_first, prev_first+prev_n)`, so id-X's row IS `X −
///   prev_first` — the arithmetic that equals the CPU's `BTreeMap<id,row>`
///   exactly), and `gather_paired(i)` is whether that row exists. A `rest` whose
///   id is present but NOT a dense window (a `sort`/`cull` broke it) still
///   recedes to the CPU.
///
/// The state port carries `id` too (a plain [`ColumnAccess::Read`]) so the module
/// can read `prev_first = state.id[0]`.
///
/// **`gather_paired(i)` is DISTINCT from `HAS_forces_sim_d`.** Two questions:
/// "is there ANY prior state?" (the global const, the seed vs step branch) and
/// "does THIS element have a prior row?" (the per-element gather, a fresh id
/// stays seeded) — [[feedback_layered_defenses_need_per_layer_gates]].
///
/// **The seed is not a zeroed step.** With no prior state (or a freshly-born id)
/// the CPU takes neither branch of the step: `vel` comes from `rest` (an
/// emitter's muzzle velocity) and `sim_d` is 0. Stepping from zeroed identities
/// would instead give `vel = 0` — the same for a grid, different the moment
/// `rest` carries `vel`.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let ig_w = read_rest_inv_mass(i);\n\
        var ig_vel = read_rest_vel(i) * ig_w;\n\
        var ig_d = vec2<f32>(0.0, 0.0);\n\
        let ig_row = gather_row(i);\n\
        if (HAS_forces_sim_d && gather_paired(i)) {\n\
        \x20   let ig_t_prev = select(params.playhead, read_forces_sim_t(0u), HAS_forces_sim_t);\n\
        \x20   let ig_dt = clamp(params.playhead - ig_t_prev, 0.0, INTEGRATE_MAX_DT);\n\
        \x20   var v = read_forces_vel(ig_row);\n\
        \x20   var d = read_forces_sim_d(ig_row);\n\
        \x20   let a = read_forces_accel(ig_row);\n\
        \x20   v = v + a * ig_dt * ig_w;\n\
        \x20   d = d + v * ig_dt * ig_w;\n\
        \x20   if (integrate_finite(v) && integrate_finite(d)) {\n\
        \x20       ig_vel = v;\n\
        \x20       ig_d = d;\n\
        \x20   }\n\
        }\n\
        write_P(i, read_rest_P(i) + ig_d);\n\
        write_vel(i, ig_vel);\n\
        write_sim_d(i, ig_d);\n\
        write_sim_t(i, params.playhead);\n",
    wgsl_lib: "\
        const INTEGRATE_MAX_DT: f32 = 0.1;\n\
        // `f32::MAX`. WGSL has no `isFinite`, but Rust's `is_finite()` is exactly\n\
        // \"not NaN and not ±inf\", and `abs(x) <= F32_MAX` answers both: every\n\
        // comparison against NaN is false, and ±inf exceeds the max. Written per\n\
        // lane rather than through `max()`, whose NaN behaviour WGSL leaves\n\
        // implementation-defined.\n\
        const INTEGRATE_F32_MAX: f32 = 3.4028235e38;\n\
        fn integrate_finite(v: vec2<f32>) -> bool {\n\
        \x20   return abs(v.x) <= INTEGRATE_F32_MAX && abs(v.y) <= INTEGRATE_F32_MAX;\n\
        }\n",
    bindings: &[
        // ── port 0: `rest`, the live chain. Also the BASE: every column this
        // kernel never mentions (tint/size/uv/falloff — and `id`, were it not
        // refused) rides through it, which is the CPU's "copy rest, rewrite the
        // sim columns".
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: INV_MASS,
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            // Absent = every element free (`w = 1`), so a pre-pin graph
            // integrates exactly as it did — and `·1.0` is exact.
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            // The SEED velocity — `rest`'s, not the state's (see the doc above).
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        // ── port 1: `forces`, last tick's state with `accel` accumulated.
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: "sim_d",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            // The state's own clock, from which `dt` is derived — element 0's,
            // like the CPU's `t_prev.first()`. Absent → the body falls back to
            // `params.playhead` (`dt = 0`), matching `scalar_to_n`'s fill.
            column: "sim_t",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            // Transient: eaten here so every tick starts from zero acceleration.
            column: "accel",
            dim: Dim::Vec2,
            access: ColumnAccess::Consume,
            identity: [0.0; 4],
            port: 1,
        },
        // The prior state's id, off the `pre` port — the module reads `id[0]`
        // for `prev_first`. Absent (tick 0 / a grid) → identity, no buffer.
        ColumnBinding {
            column: "id",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 1,
        },
        // ── ADR-0130: the id-gather key on the base port. Claims a dense id
        // window (the emitter), recedes a non-dense / unprovable one. Readable:
        // the current element's id feeds `gather_row`.
        ColumnBinding {
            column: "id",
            dim: Dim::Scalar,
            access: ColumnAccess::GatherKey,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct MotionIntegrate;

impl NodeOp for MotionIntegrate {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let playhead = ctx.playhead() as f32;
        let out = {
            let rest = ctx.input(0);
            let state = ctx.input(1);
            step(rest, state, playhead)
        };
        ctx.emit(out);
    }
}

/// One integration step (or a seed) as a pure function — the whole node.
fn step(rest: &Stream, state: &Stream, playhead: f32) -> Stream {
    let n = rest.count();
    // Every non-sim column flows LIVE from rest; the transient `accel` is
    // consumed (dropped) and the sim columns are rewritten below. `id` is NOT a
    // sim column — it rides through, so next tick's state carries it.
    let mut out = Stream::new(n);
    for (name, col) in rest.columns() {
        if !matches!(name.as_str(), "accel" | "vel" | "sim_d" | "sim_t") {
            out.set(name.clone(), col.clone());
        }
    }
    let rest_p = vec2_to_n(rest, "P", n, [0.0, 0.0]);
    // The pin weights ride the LIVE chain (rest), not the state: re-pinning
    // takes effect on the next tick without a re-seed. Absent = all free.
    let w = scalar_to_n(rest, INV_MASS, n, 1.0);

    // Every element starts at its seed (`vel` from rest — the emitter's muzzle
    // velocity — and no displacement); the ones the state knows then step. A
    // pinned element's seed velocity is scaled away too (`w = 0`), so it never
    // reports a phantom velocity to a downstream reader (`motion.look_at`).
    let mut vel = vec2_to_n(rest, "vel", n, [0.0, 0.0]);
    for (v, w) in vel.iter_mut().zip(&w) {
        v[0] *= w;
        v[1] *= w;
    }
    let mut sim_d = vec![[0.0f32; 2]; n];

    // Pair each rest element with its prior state row (`None` = seed it).
    if let Some(prev) = pairing(rest, state, n) {
        let sn = state.count();
        // dt from the state's own clock column — see the module docs.
        let t_prev = scalar_to_n(state, "sim_t", sn, playhead);
        let dt = (playhead - t_prev.first().copied().unwrap_or(playhead)).clamp(0.0, MAX_DT);
        let s_vel = vec2_to_n(state, "vel", sn, [0.0, 0.0]);
        let s_sim_d = vec2_to_n(state, "sim_d", sn, [0.0, 0.0]);
        let accel = vec2_to_n(state, "accel", sn, [0.0, 0.0]);
        for (i, slot) in prev.iter().enumerate() {
            let Some(j) = *slot else { continue }; // a fresh id: stays seeded
            let (mut v, mut d, a) = (s_vel[j], s_sim_d[j], accel[j]);
            // Semi-implicit Euler: velocity first, then position with the NEW
            // velocity (symplectic — stable for oscillatory forces).
            //
            // `w` is the inverse mass (PBD): the acceleration a force produces
            // is `F·w`, and the displacement it accumulates is scaled by the
            // same weight — so `w = 0` is a HARD pin (no force reaches it, and
            // no seed velocity can carry it off either: `sim_d` stays 0 and the
            // element sits exactly on its rest animation), `w = 1` is the free
            // element (`·1.0` is exact — the pre-pin trajectory is bit-identical)
            // and in between is a heavy, sluggish element.
            let w = w[i];
            v[0] += a[0] * dt * w;
            v[1] += a[1] * dt * w;
            d[0] += v[0] * dt * w;
            d[1] += v[1] * dt * w;
            // NaN/∞ guard (reference parity): a diverged instance recovers by
            // resetting instead of freezing the whole sim.
            if v.iter().chain(&d).all(|x| x.is_finite()) {
                vel[i] = v;
                sim_d[i] = d;
            }
        }
    }

    // Pure per-instance map → parallel above the threshold
    // (bit-identical, no reduction). GPU/M5 Fase 0.
    let p: Vec<[f32; 2]> = par_build(n, |i| {
        [rest_p[i][0] + sim_d[i][0], rest_p[i][1] + sim_d[i][1]]
    });
    out.set("P", Column::Vec2(p));
    out.set("vel", Column::Vec2(vel));
    out.set("sim_d", Column::Vec2(sim_d));
    out.set("sim_t", Column::Scalar(vec![playhead; n]));
    out
}

/// For each of the `n` rest elements, the row of `state` holding its previous
/// simulation state — `None` for a freshly-born element (it stays at its seed).
/// The whole result is `None` (seed everything) when the state carries no sim
/// columns yet (tick 0, `pre` = Empty) or when a count change on an id-less
/// stream says the set was rebuilt (see the module docs).
fn pairing(rest: &Stream, state: &Stream, n: usize) -> Option<Vec<Option<usize>>> {
    state.get("sim_d")?; // no state yet (tick 0, `pre` = Empty)
    let sn = state.count();
    match (ids_of(rest, n), ids_of(state, sn)) {
        // Element identity: match by id. `BTreeMap` keeps it deterministic and
        // O(n log n) rather than the O(n·sn) scan a naive `position()` would be.
        (Some(rest_ids), Some(state_ids)) => {
            let index: BTreeMap<u32, usize> = state_ids
                .iter()
                .enumerate()
                .map(|(j, id)| (*id, j))
                .collect();
            Some(rest_ids.iter().map(|id| index.get(id).copied()).collect())
        }
        // Positional identity: a stable count pairs row-for-row; a changed count
        // means the set was rebuilt, so re-seed rather than pair by index.
        _ if sn == n => Some((0..n).map(Some).collect()),
        _ => None,
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionIntegrate))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Integrate",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ADR-0130: per-element: accumulates forces, never reorders/rewrites id.
    reg.register_dense_window(MANIFEST.id);
    // No params → no param hints to register (the panel renders no rows).
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
