//! GPU-resident cook routing (GPU/M5 Fase 1 F1.1 + F1.2, ADR-0126).
//!
//! The per-frame decision — does this document's chain cook on the GPU, and if
//! so fully or from a CPU boundary — is a **pure function** of the plan and a
//! few flags, extracted here so it is unit-testable without a device (the bridge
//! tests are headless; the ε-parity of the actual dispatch is gated in the
//! motor, `ph2d-gpu-cook`'s `gpu_cpu_parity`). The bridge's `dispatch` reads the
//! route and drives the pump / `GpuCook` accordingly.

use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::NodeId;

/// Whether the GPU produced this frame (the caller skips the CPU pump) or the
/// frame fell through to it (GPU off / no useful GPU work / a fully-GPU cook that
/// errored). A hybrid frame is always `Handled` — the pump was already marched
/// to the boundary, so re-running the sink loop would corrupt its clock.
pub(super) enum GpuOutcome {
    Handled,
    FellThrough,
}

/// Which cook path this frame takes.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum GpuRoute {
    /// The CPU pump renders the sinks (no GPU, or the GPU can't claim useful work).
    Cpu,
    /// The whole chain is kernel-covered: cook it 100% on the GPU, no CPU pump,
    /// no readback (F1.1).
    FullyGpu,
    /// A CPU prefix cooks up to the plan's boundary nodes; the GPU runs the
    /// suffix. **The nodes are not carried here**: `plan.boundaries` already
    /// names them, and copying them into the route would be a second list to keep
    /// in step with the first ([[feedback_two_doors_to_the_same_question_diverge]]).
    Hybrid,
}

/// Choose the cook route from the plan and this frame's flags — the one place
/// the "fully vs hybrid vs CPU" policy lives.
///
/// - GPU is opt-in (`gpu_enabled`, `PH2D_GPU_COOK=1`) and only for a **single**
///   sink with **no time scopes** — multi-sink and `motion.time_remap` recuse to
///   the CPU whole (F1.1's scope; F2+ territory).
/// - Fully-GPU when the plan claims the whole chain (no boundaries).
/// - Hybrid when the plan leaves **any** CPU boundaries **and** the GPU suffix
///   has at least one dispatching stage — a boundary whose only GPU stage is the
///   pass-through `output` would upload the sink stream just to lower it (no
///   compute win), so that recuses to the CPU.
///
/// **N boundaries used to recuse here**, on the reasoning that the pump handed
/// over one cooked node per tick and *"marching it twice would advance its clock
/// twice"*. That described the CALLER: the march and the `pre` feedback are per
/// call, so the pump now takes the whole set and marches once
/// (`advance_or_scrub_to_nodes_scoped`). The reasoning was also unreachable when
/// it was written — no kernel had two stream inputs — and it silently swallowed
/// the two-seam case the day `motion.look_at` made it reachable.
pub(super) fn gpu_route(
    gpu_enabled: bool,
    n_sinks: usize,
    scopes_empty: bool,
    boundaries: &[(NodeId, usize)],
    dispatching_stages: usize,
) -> GpuRoute {
    if !gpu_enabled || n_sinks != 1 || !scopes_empty {
        return GpuRoute::Cpu;
    }
    match boundaries {
        [] => GpuRoute::FullyGpu,
        _ if dispatching_stages >= 1 => GpuRoute::Hybrid,
        _ => GpuRoute::Cpu,
    }
}

/// The GPU-resident cook for this frame (GPU/M5 Fase 1 + F1.2, ADR-0126).
///
/// With `PH2D_GPU_COOK=1` a single-sink, unscoped document cooks on the GPU:
/// compute passes in one submit, the lowering writes the renderer's instance
/// buffer directly, zero readback. **Fully-GPU** when the plan claims the whole
/// chain; **hybrid** when a node has no kernel — the CPU prefix cooks up to that
/// boundary on the persistent pump (memo + `pre` feedback + the tick march, so a
/// sequential prefix sims correctly and a scrub is bit-exact), and only its
/// output stream crosses to the GPU, which runs the covered suffix. Anything the
/// plan can't usefully claim returns [`GpuOutcome::FellThrough`] to the CPU pump.
///
/// Trade-off while the flag is on: the graph panel's readouts/probe read the CPU
/// memo, which the fully-GPU path doesn't feed — wiring those is Fase 4.
pub(super) fn cook_gpu(
    motion: &mut MotionState,
    gpu: &ph2d_gpu::GpuContext,
    target: u64,
    fixed_dt: f64,
    scopes: &TimeScopes,
) -> GpuOutcome {
    motion.gpu_live = false;
    // Fast-path guard so a GPU-off or multi-sink document never plans.
    if !motion.gpu_enabled || motion.sinks.len() != 1 {
        return GpuOutcome::FellThrough;
    }
    let plan = ph2d_gpu_cook::plan(
        &motion.doc.graph,
        &motion.registry,
        &motion.registry,
        motion.sinks[0],
    );
    let route = gpu_route(
        motion.gpu_enabled,
        motion.sinks.len(),
        scopes.is_empty(),
        &plan.boundaries,
        plan.dispatching_stages(&motion.registry),
    );
    match route {
        GpuRoute::Cpu => GpuOutcome::FellThrough,
        GpuRoute::FullyGpu => {
            // A stateless plan is `f(params, playhead)` — one cook at the target
            // tick's time, as F1.1 always did. A plan that drives a `pre` loop
            // (ADR-0127) is SEQUENTIAL: its trajectory is the sum of its steps,
            // so it owes one cook per fixed tick, under the same law as the CPU
            // pump (`ticks_owed`) and for the same reason — one big jump would
            // make the motion depend on the frame rate.
            //
            // `rewind_for` is the scrub (D5): forward it just answers
            // `last + 1`; backwards it restores the newest checkpoint at or
            // before the target, on the device, and says which tick to re-sim
            // from. It replaces `ticks_owed` here rather than wrapping it —
            // `ticks_owed` answers `target..=target` for a backwards jump, which
            // for a sim means "cook the past against the future's state".
            let ticks: Vec<u64> = match plan.drives_a_loop() {
                true => (motion.gpu_cook.rewind_for(target)..=target).collect(),
                false => vec![target],
            };
            motion.gpu_live = ticks.iter().all(|&tick| {
                motion
                    .gpu_cook
                    .cook(
                        gpu,
                        &motion.doc.graph,
                        &motion.registry,
                        &motion.registry,
                        &plan,
                        &[],
                        ph2d_gpu_cook::CookClock {
                            playhead: tick as f64 * fixed_dt,
                            tick: plan.drives_a_loop().then_some(tick),
                        },
                        motion.default_uv_rect,
                        motion.default_size,
                    )
                    .is_ok()
            });
            // A cook that errored leaves `gpu_live` false → let the CPU pump draw.
            if motion.gpu_live {
                GpuOutcome::Handled
            } else {
                GpuOutcome::FellThrough
            }
        }
        GpuRoute::Hybrid => {
            // Cook the CPU prefix up to EVERY boundary on the pump, marching each
            // owed tick (a sequential prefix must sim each), then upload their
            // streams to the GPU suffix. Time scopes are empty here (the route
            // gate refused otherwise).
            //
            // The node list comes from the plan, and the whole set goes in one
            // call: the march and the `pre` advance are per CALL, so one call per
            // boundary would advance the clock once per boundary and re-simulate
            // the shared prefix.
            let boundary_nodes: Vec<NodeId> = plan.boundaries.iter().map(|(n, _)| *n).collect();
            for tick in super::ticks_owed(motion.pump.last_cooked_tick(), target) {
                motion.pump.advance_or_scrub_to_nodes_scoped(
                    &motion.doc.graph,
                    &motion.registry,
                    &boundary_nodes,
                    tick,
                    |t| t as f64 * fixed_dt,
                    scopes,
                );
            }
            // Borrow-splitting: the cook takes `&motion.gpu_cook` mutably while the
            // streams live in `motion.pump`, so the hand-off is materialised first.
            let handed: Vec<(NodeId, &ph2d_nodegraph::attr::Stream)> = motion
                .pump
                .boundary_streams()
                .iter()
                .map(|(n, s)| (*n, s))
                .collect();
            // A boundary that failed to cook is simply missing, and the sequencer
            // validates the set against `plan.boundaries` — so an incomplete
            // hand-off is REFUSED (`BoundaryMismatch`) rather than dispatched
            // against an empty stream. Letting it try is deliberate: the check
            // lives in one place, and duplicating it here would be a second
            // opinion about what a complete hand-off is.
            if !handed.is_empty() {
                motion.gpu_live = motion
                    .gpu_cook
                    .cook(
                        gpu,
                        &motion.doc.graph,
                        &motion.registry,
                        &motion.registry,
                        &plan,
                        &handed,
                        // A hybrid plan never drives a loop: a staged `pre`
                        // source plus a boundary is refused whole at plan time
                        // (it would be two sims of one state), so there is no
                        // sequence to keep here.
                        ph2d_gpu_cook::CookClock::at(target as f64 * fixed_dt),
                        motion.default_uv_rect,
                        motion.default_size,
                    )
                    .is_ok();
            }
            // The pump was marched to the boundary this frame regardless of the
            // GPU result — do NOT also run the sink loop (it would early-return on
            // the same tick and leave `instances` holding the boundary lowering).
            // A GPU failure here renders nothing this frame rather than corrupt
            // the pump's clock.
            GpuOutcome::Handled
        }
    }
}

/// Does editing `param` on a node of `type_name` **re-number** the emitter's ids
/// — ADR-0130 D7, the trigger for restarting the GPU sim?
///
/// **Only `rate` does**, and the distinction is the whole point. Particle `k` is
/// born at `k / rate` (`emit`), so `rate` is the ONLY param in the emitter's
/// count law that re-defines what id `k` *names*: the same id becomes a different
/// particle, and the paired state (`gather_row = current_id − prev_first`) would
/// hand the new ids the old window's rows — a silent mispair as the window jumps.
/// There is no state to carry, because "the state of id `k`" no longer refers to
/// the same thing; so the sim restarts at the tick on screen
/// ([`GpuCook::reseed_from_next_tick`]).
///
/// `life` and `max` were in this list and **should not have been** (the smoke
/// question *"o que impede que as atualizações sejam feitas em tempo real?"*).
/// They do not touch `birth(k) = k/rate` at all — they only move the window's
/// LEFT edge (`first = newest + 1 − n`), so id `k` keeps its identity and every
/// survivor keeps its row. Shrinking is therefore live and **exact**: the
/// survivors' trajectories are bit-identical to a run that never changed. Growing
/// reveals ids the previous frame did not carry, and those are seeded by the
/// per-element `gather_paired` bounds check that already exists for newborns
/// (ADR-0130 D4) — no restart needed, and no read of a row that isn't there.
///
/// The launch params (`speed`/`angle`/`spread`/`seed`) and the geometry
/// (`x`/`y`/`size`) do not re-number either — the gather pairs id-for-id, the
/// running sim KEEPS, and only newborns take the new launch.
///
/// It is a tiny allowlist tied to ONE node's numbering law, not a general rule
/// that could rot ([[feedback_a_condition_that_enumerates_its_readers_rots]]): a
/// new numbering param on the emitter is a deliberate change to `emit` that
/// revisits this line.
pub(super) fn edit_renumbers_emitter(type_name: &str, param: &str) -> bool {
    type_name == "motion.emitter" && param == "rate"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion_state::MotionState;

    // A stand-in boundary node id (the routing never dereferences it).
    fn node() -> NodeId {
        NodeId(7)
    }

    #[test]
    fn only_the_emitters_renumbering_params_invalidate_the_gpu_sim() {
        // ADR-0130 D7. `rate` alone moves the id↔particle map (`birth(k) = k/rate`)
        // → restart; everything else keeps the running sim; and NO other node type
        // restarts. The mutation that drops the `motion.emitter` guard makes a
        // grid's or a force's "rate" restart too — the last loop below catches it.
        assert!(
            edit_renumbers_emitter("motion.emitter", "rate"),
            "rate re-numbers: `birth(k) = k/rate`"
        );
        // `life`/`max` are the ones this gate got WRONG at first. They move the
        // window's left edge, never `birth(k)` — survivors keep their rows, and
        // ids the window newly reveals are seeded by the per-element bounds check.
        // Listed by NAME, not folded into the loop below, because their absence
        // here is a decision and not an omission (the GPU gate
        // `shrinking_the_life_of_a_live_emitter_leaves_the_survivors_untouched`
        // is what proves the decision is safe).
        for p in ["life", "max"] {
            assert!(
                !edit_renumbers_emitter("motion.emitter", p),
                "{p} resizes the window; it does not re-number"
            );
        }
        for p in ["speed", "angle", "spread", "seed", "x", "y", "size"] {
            assert!(
                !edit_renumbers_emitter("motion.emitter", p),
                "{p} keeps the sim (the gather still pairs id-for-id)"
            );
        }
        for ty in [
            "motion.grid",
            "force.wind",
            "motion.integrate",
            "motion.spring",
        ] {
            assert!(
                !edit_renumbers_emitter(ty, "rate"),
                "{ty} is not the emitter"
            );
        }
    }

    #[test]
    fn a_real_emitter_nodes_type_name_drives_the_policy() {
        // The seam reads `inst.type_name`; pin that a REAL emitter node resolves to
        // exactly the string the policy matches (and a grid does not), so the wire
        // from the param edit to the forget can never miss by a renamed type.
        let mut motion = MotionState::new();
        let em = motion.doc.graph.add_node("motion.emitter");
        let grid = motion.doc.graph.add_node("motion.grid");
        let ty = |m: &MotionState, n| m.doc.graph.node(n).expect("added").type_name.clone();
        assert!(edit_renumbers_emitter(&ty(&motion, em), "rate"));
        assert!(!edit_renumbers_emitter(&ty(&motion, em), "speed"));
        assert!(!edit_renumbers_emitter(&ty(&motion, grid), "rate"));
    }

    #[test]
    fn disabled_or_multi_sink_or_scoped_is_always_cpu() {
        // Every gate is independent: flip one and the GPU is refused even when a
        // fully-claimed plan is on offer.
        assert_eq!(gpu_route(false, 1, true, &[], 3), GpuRoute::Cpu);
        assert_eq!(gpu_route(true, 2, true, &[], 3), GpuRoute::Cpu);
        assert_eq!(gpu_route(true, 0, true, &[], 3), GpuRoute::Cpu);
        assert_eq!(gpu_route(true, 1, false, &[], 3), GpuRoute::Cpu);
    }

    /// **Two seams take the GPU now** — the assertion that flipped when the pump
    /// went plural, and the reason it was pinned before it could.
    ///
    /// A multi-input kernel (`motion.look_at`) leaves a plan with two CPU
    /// boundaries and real GPU work behind them (measured in `ph2d-gpu-cook`'s
    /// `boundary_arity`, item (d)). This used to assert `Cpu`: the pump took ONE
    /// boundary, so the route forfeited the GPU for the whole frame — never
    /// wrong, and never fast either.
    ///
    /// It was written as a gate rather than left implicit because
    /// `_ => GpuRoute::Cpu` is a catch-all: it swallowed the two-seam case the
    /// day it became reachable **without a single gate changing colour**. Being
    /// pinned is what made the change visible when it came.
    #[test]
    fn two_seams_take_the_hybrid_route() {
        // Two DISTINCT nodes — `node()` is a single stand-in id, and reusing it
        // would model one node consumed on two ports (the duplicate entry the
        // pump dedupes), which is a different shape.
        let (a, b) = (NodeId(7), NodeId(8));
        assert_eq!(
            gpu_route(true, 1, true, &[(a, 0), (b, 0)], 3),
            GpuRoute::Hybrid
        );
        // Still no compute win, still the CPU — the dispatching-stage rule is
        // independent of how many seams there are.
        assert_eq!(
            gpu_route(true, 1, true, &[(a, 0), (b, 0)], 0),
            GpuRoute::Cpu
        );
    }

    #[test]
    fn fully_claimed_plan_runs_fully_on_the_gpu() {
        assert_eq!(gpu_route(true, 1, true, &[], 3), GpuRoute::FullyGpu);
    }

    #[test]
    fn a_boundary_with_gpu_work_is_hybrid() {
        assert_eq!(
            gpu_route(true, 1, true, &[(node(), 0)], 2),
            GpuRoute::Hybrid
        );
        // One dispatching stage is enough.
        assert_eq!(
            gpu_route(true, 1, true, &[(node(), 0)], 1),
            GpuRoute::Hybrid
        );
    }

    #[test]
    fn a_boundary_with_no_dispatching_suffix_recuses_to_cpu() {
        // A lone pass-through `output` above the boundary is no compute win —
        // uploading the sink stream just to lower it — so it stays on the CPU.
        assert_eq!(gpu_route(true, 1, true, &[(node(), 0)], 0), GpuRoute::Cpu);
    }
}
