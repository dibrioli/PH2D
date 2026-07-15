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
    let p: Vec<[f32; 2]> =
        par_build(n, |i| [rest_p[i][0] + sim_d[i][0], rest_p[i][1] + sim_d[i][1]]);
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
    // No params → no param hints to register (the panel renders no rows).
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
