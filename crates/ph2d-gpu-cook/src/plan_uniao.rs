//! **As portas do planeador** — [`plan`], [`plan_driven`] e a da UNIÃO, [`plan_driven_many`]
//! (doc 119 W2), partidas do `plan.rs` no tecto de LOC ao longo da costura que já lá estava: lá
//! vive *o que um nó é* (elegível, a caminhada, as colunas); aqui vive *que saídas se pedem e
//! quando o laço recua*.

use super::{DrivenParams, GpuPlan, Walk, eligible};
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::gpu::KernelResolver;
use ph2d_nodegraph::graph::{Graph, NodeId};
use std::collections::BTreeSet;

/// Claim the GPU-runnable part of `sink`'s chain (see the crate docs). Pure and
/// cheap — a walk of the graph, no GPU objects — so calling it every frame is
/// fine; compiled pipelines are cached by [`GpuCook`].
pub fn plan(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    sink: NodeId,
) -> GpuPlan {
    plan_driven(graph, ops, kernels, sink, &DrivenParams::new())
}

/// ⭐⭐⭐ [`plan`], **com os valores dos params dirigidos** (doc 110 §3).
///
/// ⚠️ **A [`plan`] é esta função com o mapa VAZIO**, e isso não é um atalho: sem valores nenhum nó
/// com fio é encenado, que é exactamente o que este planeador fazia antes desta wave. *Uma porta
/// nova cujo caso vazio reproduz a lei antiga não precisa que ninguém confie nela.*
pub fn plan_driven(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    sink: NodeId,
    driven: &DrivenParams,
) -> GpuPlan {
    plan_driven_many(graph, ops, kernels, &[sink], driven)
}

/// ⭐⭐⭐ **O plano da UNIÃO de várias saídas** (doc 119 W2) — [`plan_driven`] com N sinks.
///
/// ⚠️ **As saídas partilham UMA caminhada**, e é essa a lei inteira: um nó que duas saídas lêem
/// é encenado **uma vez** (o `claimed` é um só). Dois planos de um sink cozinhados lado a lado
/// correriam esse nó duas vezes por tique, e num nó que alimenta um laço `pre` isso é **avançar a
/// simulação duas vezes** — a trajectória deixaria de ser a da CPU, que cozinha cada nó uma vez.
///
/// ⚠️ **Com UM sink ela É a [`plan_driven`]**, byte a byte: aquela delega aqui, e é isso que põe
/// todos os gates de paridade desta crate a guardar o caso de uma saída.
///
/// ⚠️ **A ordem das saídas pode mudar o plano, e isso é declarado.** Um `pre` só é aceite se a
/// fonte já foi encenada (ver o `eligible`), logo um nó da saída B que lê o laço da saída A
/// encena-se se A foi caminhada antes, e é FRONTEIRA se não — e aí o recuo (abaixo) devolve o laço
/// à CPU. Os dois são correctos; o chamador passa a ordem do DOCUMENTO, que é estável.
pub fn plan_driven_many(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    sinks: &[NodeId],
    driven: &DrivenParams,
) -> GpuPlan {
    plan_forbidding(graph, ops, kernels, sinks, &BTreeSet::new(), driven)
}

/// [`plan`], with a set of nodes forced to be boundaries — the retreat mechanism
/// (see the `sim_state_on_gpu` handling below). The public entry is the empty set.
/// Is this boundary node's transitive input chain **STATIC** — every node
/// `Effect::Pure`, no delayed edge, no driven param anywhere in it? Pure is "no
/// state, no time: same inputs → same output" (the enum's contract, which every
/// playhead-reader honours by declaring `Temporal` — the LFO, the oscillator,
/// the wiggle all do), so a static chain is a CONSTANT per param set.
///
/// Driven params are refused **conservatively**: the driver is a wire into
/// another subtree that may be temporal, and chasing it buys nothing today — a
/// template with a driven param simply keeps the pre-ADR-0136 retreat.
fn boundary_is_static(graph: &Graph, ops: &dyn OpResolver, node: NodeId) -> bool {
    let mut seen: BTreeSet<NodeId> = BTreeSet::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if !seen.insert(n) {
            continue;
        }
        let Some(op) = graph.node(n).and_then(|i| ops.resolve(i.type_id())) else {
            return false; // unknown node: assume the worst, keep the retreat
        };
        if op.manifest().effect != ph2d_nodegraph::effect::Effect::Pure {
            return false;
        }
        if graph.param_sources(n).is_some_and(|s| !s.is_empty()) {
            return false;
        }
        for port in 0..op.manifest().inputs.len() {
            if let Some((src, _, delayed)) = graph.input_edge(n, port) {
                if delayed {
                    return false;
                }
                stack.push(src);
            }
        }
    }
    true
}

fn plan_forbidding(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    sinks: &[NodeId],
    forbidden: &BTreeSet<NodeId>,
    driven: &DrivenParams,
) -> GpuPlan {
    let mut walk = Walk {
        graph,
        ops,
        kernels,
        stages: Vec::new(),
        boundaries: Vec::new(),
        claimed: BTreeSet::new(),
        forbidden,
        driven,
    };
    let mut staged_sinks = Vec::with_capacity(sinks.len());
    for &sink in sinks {
        match graph.node(sink).map(|i| i.type_id()) {
            Some(ty) if eligible(graph, ops, kernels, sink, &walk.claimed, forbidden, driven) => {
                walk.accept(sink, ty);
                staged_sinks.push(sink);
            }
            // The sink itself is CPU: nothing to claim for it. (With one sink,
            // `dispatching_stages` is 0 and the caller's route recuses whole.)
            _ => walk.boundaries.push((sink, 0)),
        }
    }
    if staged_sinks.is_empty() {
        return GpuPlan {
            boundaries: walk.boundaries,
            stages: Vec::new(),
            sinks: Vec::new(),
        };
    }

    // A staged node that feeds a `pre` edge owns simulation state. If the plan
    // ALSO leaves a CPU boundary, the caller would cook that boundary on the
    // pump — and the pump, to do so, re-cooks this node's chain with ITS OWN
    // `prev`. That is two simulations of one state: the GPU would integrate an
    // `accel` computed from the CPU's divergent trajectory, every tick. The
    // node's own loop being GPU-resident is not enough (a force node without a
    // kernel puts the boundary INSIDE the loop).
    let pre_sources_on_gpu: BTreeSet<NodeId> = walk
        .graph
        .edges()
        .iter()
        .filter(|e| e.delayed && walk.claimed.contains(&e.from.0))
        .map(|e| e.from.0)
        .collect();
    // **A STATIC boundary does not evict the loop** (ADR-0136 §5). The retreat
    // below guards against two simulations of one state — the pump re-cooking a
    // boundary's chain with its own `prev`. A chain that is all `Effect::Pure`
    // with no delayed edges and no driven params HAS no prev and reads no clock
    // (`Pure` is "no state, no time" by the enum's own contract): it is a
    // constant, and the pump re-evaluating a constant is not a second simulation
    // of anything. This is what lets a `motion.distribute_poisson` template — an
    // inherently sequential algorithm that will never have a kernel — feed a
    // GPU-resident sim without dragging the whole loop back to the pump.
    let every_boundary_is_static = walk
        .boundaries
        .iter()
        .all(|(n, _)| boundary_is_static(graph, ops, *n));
    if !pre_sources_on_gpu.is_empty() && !walk.boundaries.is_empty() && !every_boundary_is_static {
        // RETREAT, not refuse-whole. Forbid the staged `pre`-sources and re-plan:
        // the sim loop recedes to the pump (its container — a `sim.zone` — becomes
        // a boundary), while any render suffix DOWNSTREAM of the loop stays on the
        // GPU. That preserves the hybrid a partially-covered sim document had
        // before its zone was claimable — the boot snow renders its final
        // population on the device even though the count-changing interior cooks
        // on the CPU. A refuse-to-`(sink, 0)` would drop that suffix for nothing.
        let mut next = forbidden.clone();
        let grew = pre_sources_on_gpu
            .iter()
            .fold(false, |g, n| next.insert(*n) | g);
        if grew {
            return plan_forbidding(graph, ops, kernels, sinks, &next, driven);
        }
        // No progress possible (a forbidden node still staged — cannot happen,
        // `eligible` refuses it): fall back to the whole refusal.
        return GpuPlan {
            boundaries: sinks.iter().map(|&s| (s, 0)).collect(),
            stages: Vec::new(),
            sinks: Vec::new(),
        };
    }

    GpuPlan {
        boundaries: walk.boundaries,
        stages: walk.stages,
        sinks: staged_sinks,
    }
}
