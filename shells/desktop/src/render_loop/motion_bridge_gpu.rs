//! GPU-resident cook routing (GPU/M5 Fase 1 F1.1 + F1.2, ADR-0126).
//!
//! The per-frame decision — does this document's chain cook on the GPU, and if
//! so fully or from a CPU boundary — is a **pure function** of the plan and a
//! few flags, extracted here so it is unit-testable without a device (the bridge
//! tests are headless; the ε-parity of the actual dispatch is gated in the
//! motor, `ph2d-gpu-cook`'s `gpu_cpu_parity`). The bridge's `dispatch` reads the
//! route and drives the pump / `GpuCook` accordingly.

use crate::motion_state::MotionState;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;

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
pub(crate) enum GpuRoute {
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

/// ⭐⭐ **A ROTA, DITA EM VOZ ALTA** — `PH2D_MOTION_ROUTE_LOG=1`.
///
/// ⛔⛔ **Por que ela existe** (auditoria de performance, [doc 98 §2.3](../../../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md)):
/// o device faz **4,19 M objectos em 3,85 ms** contra **195,9 ms da CPU** — `50,9×` — e
/// **69,7% das cenas que este produto expõe caem para a CPU**. Até esta linha existir,
/// [`GpuOutcome::FellThrough`] era consumido pela ponte e **não acendia nada**: um grafo no
/// device e o mesmo grafo num núcleo só têm exactamente a mesma aparência na UI. *Um custo de
/// 50× que nenhuma superfície nomeia não é uma escolha de ninguém — é um acidente que se repete.*
///
/// ⚠️ **Disparo por BORDA, e a borda regista-se mesmo com o log desligado** — senão ligar a
/// variável a meio de uma sessão ficaria calado até à próxima mudança de rota. O que a variável
/// governa é a IMPRESSÃO, não o registo. (Um aviso chaveado por conteúdo imprimiria a 60 Hz;
/// esta linha pagou-se noutra wave deste módulo.)
pub(super) fn say_route(motion: &mut MotionState, reason: &'static str) {
    if motion.route_said == Some(reason) {
        return;
    }
    motion.route_said = Some(reason);
    if std::env::var_os("PH2D_MOTION_ROUTE_LOG").is_some_and(|v| v != "0") {
        eprintln!("[motion-route] {reason}");
    }
}

/// **Toda saída para a CPU passa por AQUI e nomeia-se.** ⛔ Um `return
/// GpuOutcome::FellThrough` cru é a forma que tornou o §2.3 possível: cinco recusas, cada uma
/// com o seu motivo, e nenhuma delas alcançável de fora.
fn fell(motion: &mut MotionState, reason: &'static str) -> GpuOutcome {
    say_route(motion, reason);
    GpuOutcome::FellThrough
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
pub(crate) fn gpu_route(
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

/// Does this document bring in a live vector SHAPE (`source.shape`)? (ADR-0154)
///
/// A live vector is drawn by the vector pass (`geometry_id`), which the
/// GPU-resident cook has NO route for — so a document carrying one draws as
/// blank atlas quads the moment a GPU stage runs (`source → duplicator → … `
/// is Hybrid). Recuse it to the CPU render (which draws it) at PLAN time, so the
/// CPU pump owns the tick from scratch and no sequential prefix is marched
/// twice. The signal is a registry flag `source.shape` sets
/// (`is_live_vector_source`), not a node-name match.
///
/// ⚠️ An OBJECT source (`source.object`, `texture_id`) is NOT here: the GPU cook
/// now draws it (the lowering carries the id, the renderer binds the texture per
/// run). It recuses only when its GPU suffix reorders / changes count — see
/// [`graph_has_object_source`] + [`ph2d_gpu_cook::GpuPlan::suffix_changes_count`].
pub(super) fn graph_has_live_vector_source(graph: &Graph, reg: &NodeRegistry) -> bool {
    graph
        .nodes()
        .iter()
        .any(|n| reg.is_live_vector_source(NodeTypeId::of(n.type_name.as_str())))
}

/// Os relógios que o device marcha: um tique vira `sub` sub-passadas.
///
/// ⚠️ **O TIQUE não se subdivide, só o PLAYHEAD** — e as duas metades disso são load-bearing.
/// O device avança o ping-pong do `pre` a cada CHAMADA de `cook` (o `self.prev` é reatribuído no
/// fim dela), então `sub` chamadas são `sub` sub-tiques sem renumerar nada; e o ring de scrub
/// chaveia pelo tique, com `should_record` a deduplicar — então a 1ª sub-passada grava o estado
/// de ENTRADA do quadro e as seguintes não o sobrescrevem com um estado do meio.
///
/// O `dt` que as leis de contagem leem sai de `playhead − last_playhead` no próprio device, então
/// ele subdivide sozinho e os nascimentos telescopam, exactamente como na CPU.
///
/// `sub <= 1` devolve o que este código sempre devolveu, termo a termo.
fn substep_clocks(
    ticks: &[u64],
    sub: u32,
    fixed_dt: f64,
    drives_loop: bool,
) -> Vec<(f64, Option<u64>)> {
    ticks
        .iter()
        .flat_map(|&t| {
            let clock = drives_loop.then_some(t);
            (1..=sub.max(1)).map(move |k| {
                let frac = f64::from(k) / f64::from(sub.max(1));
                // O quadro `t` cobre `((t-1)·dt, t·dt]`; a última sub-passada cai em `t·dt`.
                ((t as f64 - 1.0 + frac) * fixed_dt, clock)
            })
        })
        .collect()
}

/// Does this document bring in an engine OBJECT (`source.object`, `texture_id`)?
/// Read together with [`ph2d_gpu_cook::GpuPlan::suffix_changes_count`] for the
/// count-changing cerca: an object graph whose GPU suffix reorders / changes
/// count would mis-bind the texture-run partition (the boundary `texture_id`
/// column no longer aligns with the device buffer), so it recuses to the CPU
/// render. The signal is the registry flag `source.object` sets.
pub(super) fn graph_has_object_source(graph: &Graph, reg: &NodeRegistry) -> bool {
    graph
        .nodes()
        .iter()
        .any(|n| reg.is_object_source(NodeTypeId::of(n.type_name.as_str())))
}

/// Does the cook's external table carry a LIVE VECTOR (`geometry_id > 0`)? — the
/// CONTENT-aware half of the object recusal (ADR-0154 reused for objects).
///
/// Whether a `source.object` resolves to a vector depends on what the artist NAMED
/// (a sprite → `texture_id`, a vector → `geometry_id`), which the node-type registry
/// cannot see. The membrane publishes the externals BEFORE the cook runs (post-drain,
/// pre-cook), so this per-frame scan answers the real question and lets a pure-sprite
/// object graph stay on the GPU stamp while a vector-bearing one recuses. Cheap: a
/// handful of externals, one scalar-column probe each.
fn cook_publishes_live_geometry(cook: &ph2d_nodegraph::cook::Cook) -> bool {
    use ph2d_nodegraph::attr::Column;
    cook.externals().values().any(|e| {
        matches!(e.value.get("geometry_id"), Some(Column::Scalar(v)) if v.iter().any(|&g| g > 0.5))
    })
}

/// The GPU-resident cook for this frame (GPU/M5 Fase 1 + F1.2, ADR-0126).
///
/// Unless `PH2D_GPU_COOK=0`, a single-sink, unscoped document cooks on the GPU:
/// compute passes in one submit, the lowering writes the renderer's instance
/// buffer directly, zero readback. **Fully-GPU** when the plan claims the whole
/// chain; **hybrid** when a node has no kernel — the CPU prefix cooks up to that
/// boundary on the persistent pump (memo + `pre` feedback + the tick march, so a
/// sequential prefix sims correctly and a scrub is bit-exact), and only its
/// output stream crosses to the GPU, which runs the covered suffix. Anything the
/// plan can't usefully claim returns [`GpuOutcome::FellThrough`] to the CPU pump.
///
/// The graph panel reads a GPU frame through the bounded **tap** (Fase 4,
/// `readout::take_tap` — readouts, digest, probe), so a fully-GPU document is
/// no longer blind in the editor; the tap is one frame behind the cook it
/// samples (the documented ordering asymmetry vs the CPU memo).
pub(super) fn cook_gpu(
    motion: &mut MotionState,
    gpu: &ph2d_gpu::GpuContext,
    target: u64,
    fixed_dt: f64,
    scopes: &TimeScopes,
) -> GpuOutcome {
    motion.gpu_live = false;
    // Fast-path guard so a GPU-off or multi-sink document never plans.
    // ⚠️ **Os dois motivos separam-se aqui de propósito:** eles leem-se iguais numa recusa
    // («a CPU desenhou») e um deles é uma ESCOLHA do artista (`PH2D_GPU_COOK=0`) enquanto o
    // outro é uma escada que ele não pediu e cujo preço é `50,9×`.
    if !motion.gpu_enabled {
        return fell(motion, "CPU: o device esta desligado (PH2D_GPU_COOK=0)");
    }
    if motion.sinks.len() != 1 {
        return fell(
            motion,
            "CPU: mais de UM sink -- a escada do doc 98 §2, ~50x a contagem de objectos",
        );
    }
    // A document that brings in a live vector SHAPE (`source.shape`) recuses to
    // the CPU render — the GPU cook has no `geometry_id` route and would draw it
    // as blank atlas quads once a GPU stage runs. Checked before planning so the
    // CPU pump owns the tick from scratch (no double-march of a sequential
    // prefix). An OBJECT source (`source.object`) is NOT recused here — the GPU
    // cook draws it; the count-changing cerca below is its only guard.
    if graph_has_live_vector_source(&motion.doc.graph, &motion.registry) {
        return fell(
            motion,
            "CPU: o grafo traz uma FORMA vectorial viva (source.shape)",
        );
    }
    // A `source.object` that resolves to a live VECTOR publishes a `geometry_id`
    // external (ADR-0154 reused for objects, so a stamped vector stays crisp). The
    // GPU cook has no `geometry_id` route — it would draw a blank atlas quad — so
    // recuse to the CPU render, which draws it. CONTENT-aware, with the externals in
    // hand: a pure-sprite object graph publishes only `texture_id` and stays on the
    // GPU stamp (this wave's point). A node-type flag would recuse EVERY object graph.
    if graph_has_object_source(&motion.doc.graph, &motion.registry)
        && cook_publishes_live_geometry(&motion.pump.cook)
    {
        return fell(
            motion,
            "CPU: um source.object resolveu para geometria vectorial viva",
        );
    }
    // **O ritmo deste grafo** (doc 89, folha 13): de quantas sub-passadas o plano marcha. ⚠️ NÃO
    // há recusa aqui, e é por construção: o ritmo é do GRAFO — a mesma porta que o pump da CPU
    // pergunta —, então marchar o plano inteiro `n` vezes dá a cada ilha exactamente os `n`
    // sub-tiques que o bracket da CPU lhe daria. Os dois produtores concordam sem ninguém ter de
    // escolher entre acelerar e estar certo.
    let sub = ph2d_nodegraph::cook::graph_substeps(&motion.doc.graph, &motion.registry);
    let plan = ph2d_gpu_cook::plan(
        &motion.doc.graph,
        &motion.registry,
        &motion.registry,
        motion.sinks[0],
    );
    // Como este sink DESENHA (doc 89, folha 17): blend · pivô · filtro · ordem. Lido da
    // porta ÚNICA — a MESMA que o pump da CPU pergunta no laço de sinks —, e resolvido
    // AQUI, ao lado do sink que o plano escolheu: um segundo leitor teria liberdade de
    // arredondar diferente, e as duas rotas desenhariam o mesmo documento de maneiras
    // diferentes, que nenhum gate que olha para uma rota consegue ver.
    let blend = ph2d_eval_motion::sink_style(&motion.doc.graph, motion.sinks[0]);
    // The count-changing cerca (this wave): an OBJECT graph whose GPU suffix
    // reorders / changes count would mis-bind the texture-run partition — the
    // boundary `texture_id` column aligns with the sink ONLY when the suffix is
    // per-element. Recuse it to the CPU render (which draws it correctly). A
    // non-object graph is unaffected: no `texture_id`, no partition to mis-bind.
    if graph_has_object_source(&motion.doc.graph, &motion.registry)
        && plan.suffix_changes_count(&motion.registry)
    {
        return fell(
            motion,
            "CPU: sob um source.object, o sufixo de GPU muda a contagem",
        );
    }
    let route = gpu_route(
        motion.gpu_enabled,
        motion.sinks.len(),
        scopes.is_empty(),
        &plan.boundaries,
        plan.dispatching_stages(&motion.registry),
    );
    match route {
        GpuRoute::Cpu if !scopes.is_empty() => fell(
            motion,
            "CPU: escopo de tempo (motion.time_remap) -- a escada do doc 98 §2",
        ),
        GpuRoute::Cpu => fell(
            motion,
            "CPU: fronteira sem estagio de GPU que despache (so o output passa-through)",
        ),
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
            let ticks = substep_clocks(&ticks, sub, fixed_dt, plan.drives_a_loop());
            motion.gpu_live = ticks.iter().all(|&(playhead, tick)| {
                motion
                    .gpu_cook
                    .cook(
                        gpu,
                        &motion.doc.graph,
                        &motion.registry,
                        &motion.registry,
                        &plan,
                        &[],
                        ph2d_gpu_cook::CookClock { playhead, tick },
                        motion.default_uv_rect,
                        motion.default_size,
                        blend,
                    )
                    .is_ok()
            });
            // A cook that errored leaves `gpu_live` false → let the CPU pump draw.
            if motion.gpu_live {
                say_route(motion, "device: o plano inteiro (fully-GPU)");
                GpuOutcome::Handled
            } else {
                fell(motion, "CPU: o cook no device ERROU -- o pump desenha")
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
                if plan.drives_a_loop() {
                    // A hybrid plan CAN drive a loop since ADR-0136 §5 — but only
                    // when every boundary is STATIC (a temporal one still retreats
                    // at plan time, because that would be two sims of one state).
                    // A static boundary is a constant, so the SAME hand-off serves
                    // every marched tick, and the loop keeps its sequence exactly
                    // like the FullyGpu arm: rewind if owed, then march.
                    let ticks: Vec<u64> = (motion.gpu_cook.rewind_for(target)..=target).collect();
                    let ticks = substep_clocks(&ticks, sub, fixed_dt, true);
                    motion.gpu_live = ticks.iter().all(|&(playhead, tick)| {
                        motion
                            .gpu_cook
                            .cook(
                                gpu,
                                &motion.doc.graph,
                                &motion.registry,
                                &motion.registry,
                                &plan,
                                &handed,
                                ph2d_gpu_cook::CookClock { playhead, tick },
                                motion.default_uv_rect,
                                motion.default_size,
                                blend,
                            )
                            .is_ok()
                    });
                } else {
                    motion.gpu_live = motion
                        .gpu_cook
                        .cook(
                            gpu,
                            &motion.doc.graph,
                            &motion.registry,
                            &motion.registry,
                            &plan,
                            &handed,
                            // A stateless hybrid: nothing to sequence.
                            ph2d_gpu_cook::CookClock::at(target as f64 * fixed_dt),
                            motion.default_uv_rect,
                            motion.default_size,
                            blend,
                        )
                        .is_ok();
                }
            }
            // The pump was marched to the boundary this frame regardless of the
            // GPU result — do NOT also run the sink loop (it would early-return on
            // the same tick and leave `instances` holding the boundary lowering).
            // A GPU failure here renders nothing this frame rather than corrupt
            // the pump's clock.
            say_route(
                motion,
                "device: HIBRIDO -- prefixo na CPU, sufixo no device",
            );
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
#[path = "motion_bridge_gpu_tests.rs"]
mod tests;
