//! **A FORMA da saída de um nó** — que colunas ela PROVAVELMENTE traz, por porta. Irmão do
//! [`super`] pelo tecto de LOC, e o corte é por responsabilidade: lá vive *se a placa pode
//! reivindicar um nó*; aqui, *o que sai dele* (que as recusas de coluna, o id-gather e o segundo
//! nascimento perguntam).

use super::{DrivenParams, SHAPE_DEPTH, resolve_param};
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::gpu::{Complement, FiredBirth, KernelResolver, StreamOp};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use std::collections::BTreeSet;

/// The columns a node's output stream provably carries, or `None` when that is
/// **unknowable** — a CPU boundary (or a `pre` stop) feeds its base, so only the
/// CPU `eval` knows what comes out.
///
/// Derived from the same rule [`GpuCook::cook`] threads at runtime: the output
/// starts as port 0's stream, written columns are added and
/// [`ColumnAccess::Consume`]d ones dropped. It exists for the plan-time
/// refusals ([`ColumnAccess::RefuseIfPresent`], ADR-0127 D3) — those must be
/// *provable*, so unknown means refuse.
///
/// `budget` bounds the recursion: a well-formed graph's forward edges are
/// acyclic (a loop must go through a `pre`, which stops here), but a plan is not
/// the place to discover otherwise by overflowing the stack.
pub(super) fn output_shape(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    node: NodeId,
    budget: u32,
    driven: &DrivenParams,
) -> Option<Shape> {
    let budget = budget.checked_sub(1)?;
    let inst = graph.node(node)?;
    let manifest = ops.resolve(inst.type_id())?.manifest();
    let kernel = kernels.gpu_kernel(inst.type_id())?;
    // An engine ALGORITHM (ADR-0139) reshapes the output wholesale, like a
    // structural stream op: the node registers PASSTHROUGH, so the binding
    // rules below would answer with the base's columns — for `motion.voronoi`
    // that is the relax VALUE stream, not the point cloud it actually emits.
    if let Some(alg) = kernels.algorithm(inst.type_id()) {
        match alg {
            ph2d_nodegraph::gpu::GpuAlgorithm::LloydVoronoi { .. } => {
                let mut cols = BTreeSet::new();
                cols.insert("P");
                return Some(Shape { cols, dense: false });
            }
        }
    }
    // The base (port 0) the output rides on. A generator starts from nothing;
    // an unconnected input is the empty stream. Note this deliberately does NOT
    // ask `eligible` — a node the plan refuses still emits the columns its
    // kernel declares (that is the parity contract), and asking would recurse.
    let (mut cols, dense_in) = match manifest.inputs.first() {
        None => (BTreeSet::new(), false),
        Some(_) => match graph.input_edge(node, 0) {
            None => (BTreeSet::new(), false),
            // A `pre` stop: last tick's stream, which the walk has not derived.
            Some((_, _, true)) => return None,
            Some((src, sp, false)) => {
                let s = edge_shape(graph, ops, kernels, (src, sp), budget, driven)?;
                (s.cols, s.dense)
            }
        },
    };
    // A structural stream op (ADR-0136) reshapes the output wholesale — the
    // binding rules below describe a per-element MAP, which these are not.
    match kernels.stream_op(inst.type_id()) {
        Some(ph2d_nodegraph::gpu::StreamOp::Project { .. }) => {
            // One `v` column, whatever the input carried (the CPU emits exactly
            // that, zeros included). Never a dense window: `v` is a value field.
            let mut cols = BTreeSet::new();
            cols.insert("v");
            return Some(Shape { cols, dense: false });
        }
        Some(ph2d_nodegraph::gpu::StreamOp::Concat { ports }) => {
            // The union of the connected ports' provable shapes. An input the
            // plan cannot derive refuses the whole answer — a zero-filled column
            // is still a column, so guessing under-reports. This OVER-approximates
            // at runtime (an empty input's columns are skipped by the CPU's
            // non-empty rule), which errs in the refusing direction.
            let mut cols = BTreeSet::new();
            for p in *ports {
                match graph.input_edge(node, *p) {
                    None => {}
                    Some((_, _, true)) => return None,
                    Some((src, sp, false)) => {
                        cols.extend(
                            edge_shape(graph, ops, kernels, (src, sp), budget, driven)?.cols,
                        );
                    }
                }
            }
            return Some(Shape { cols, dense: false });
        }
        // SourceRows: the base IS the template whose columns the newborns inherit — minus the
        // ones they do NOT inherit —, plus the kernel's writes (and, with the second birth
        // active, its kernel's too — doc 110 §14.1 (7)); `cp_rows` is popped after the loop.
        Some(StreamOp::SourceRows {
            fired,
            not_inherited,
            ..
        }) => {
            for c in *not_inherited {
                cols.remove(c);
            }
            if let Some(f) = fired.filter(|f| fired_active(graph, ops, kernels, node, f, driven)) {
                let param = |name: &str| resolve_param(graph, node, manifest, name, driven);
                cols.extend(
                    f.kernel
                        .resolve(&param)
                        .bindings
                        .iter()
                        .filter_map(|b| b.access.writes(false).then_some(b.column)),
                );
            }
        }
        // Compact: the base's columns survive (filtered, not reshaped) and the
        // node's own kernel writes ride on top — the plain rules below.
        Some(_) | None => {}
    }
    // The set this node will ACTUALLY bind — a param-dependent kernel writes a
    // different column per param, and deriving the shape from the default set
    // would predict columns the dispatch never produces.
    for b in kernel
        .resolve(&|name| resolve_param(graph, node, manifest, name, driven))
        .bindings
    {
        let present = cols.contains(b.column);
        if b.access.consumes() {
            cols.remove(b.column);
        } else if b.access.writes(present) {
            cols.insert(b.column);
        }
    }
    // A SourceRows kernel's rows column is machinery, never output (ADR-0136); a projected
    // column leaves on its own port.
    cols.remove(ph2d_nodegraph::gpu::ROWS_COL);
    for pp in kernels.projected_ports(inst.type_id()) {
        cols.remove(pp.column);
    }
    // The dense id window (ADR-0130) flows on the port-0 base: a source that
    // emits one keeps it unconditionally (`inputs.is_empty()`), a transformer
    // only if it preserves AND its base already was dense. Anything that does not
    // declare `keeps_dense_window` clears it — the safe default, so a structural
    // node (`sort`/`cull`) breaks the window the instant it gains a kernel.
    let dense =
        kernels.keeps_dense_window(inst.type_id()) && (manifest.inputs.is_empty() || dense_in);
    Some(Shape { cols, dense })
}

/// A forma da saída `port` de `node` — a porta 0 é a [`output_shape`]; uma porta ≠ 0 só é
/// provável quando é o complemento declarado de um `Compact` (doc 110 §14.1 (1)): `Rows` leva as
/// colunas da porta filtrada tal e qual, `Event(c)` só a coluna `c`. Nunca uma janela densa.
pub(super) fn edge_shape(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    (node, port): (NodeId, usize),
    budget: u32,
    driven: &DrivenParams,
) -> Option<Shape> {
    if port == 0 {
        return output_shape(graph, ops, kernels, node, budget, driven);
    }
    let budget = budget.checked_sub(1)?;
    let ty = graph.node(node)?.type_id();
    if let Some(pp) = kernels
        .projected_ports(ty)
        .iter()
        .find(|p| usize::from(p.port) == port)
    {
        return Some(Shape {
            cols: BTreeSet::from([pp.as_name]),
            dense: false,
        });
    }
    let Some(StreamOp::Compact {
        port: filtrada,
        complement,
        ..
    }) = kernels.stream_op(ty)
    else {
        return None;
    };
    let cols = match complement
        .iter()
        .find(|c| usize::from(c.port) == port)?
        .carries
    {
        Complement::Event(col) => BTreeSet::from([col]),
        Complement::Rows => match graph.input_edge(node, *filtrada) {
            None => BTreeSet::new(),
            Some((_, _, true)) => return None,
            Some((src, sp, false)) => {
                edge_shape(graph, ops, kernels, (src, sp), budget, driven)?.cols
            }
        },
    };
    Some(Shape { cols, dense: false })
}

/// **O segundo nascimento está activo?** — a porta lateral traz, PROVAVELMENTE, a coluna dele (o
/// mesmo facto que a CPU pergunta à corrente: com produtores que emitem sempre as colunas que
/// declaram, o provável e o presente coincidem). Público ao sequenciador, que escolhe a variante.
pub(crate) fn fired_active(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    node: NodeId,
    f: &FiredBirth,
    driven: &DrivenParams,
) -> bool {
    match graph.input_edge(node, f.port) {
        Some((src, sp, false)) => edge_shape(graph, ops, kernels, (src, sp), SHAPE_DEPTH, driven)
            .is_some_and(|s| s.cols.contains(f.column)),
        _ => false,
    }
}

/// As portas ≠ 0 que o estágio de `ty` sabe produzir — o complemento de um `Compact` ou uma
/// projecção do kernel.
pub(super) fn declares_port(kernels: &dyn KernelResolver, ty: NodeTypeId, port: u16) -> bool {
    kernels.projected_ports(ty).iter().any(|p| p.port == port)
        || matches!(
            kernels.stream_op(ty),
            Some(StreamOp::Compact { complement, .. }) if complement.iter().any(|c| c.port == port)
        )
}

/// The proven shape of a node's output stream: which columns it carries, and
/// whether its `id` column is a dense ascending window (ADR-0130). `None` when
/// the walk cannot prove it (a `pre` stop, an unkernelled node, budget).
pub(super) struct Shape {
    pub(super) cols: BTreeSet<&'static str>,
    pub(super) dense: bool,
}

/// Whether `node`'s output is a dense id window (ADR-0130) — `None` when the plan
/// cannot prove the shape (a `pre` stop, an unkernelled boundary). Public for the
/// gates that pin the property's propagation through per-element stages and its
/// clearing by a structural one.
pub fn output_dense_window(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    node: NodeId,
) -> Option<bool> {
    // ⚠️ Sem params dirigidos: esta porta responde sobre a FORMA da saída, e um fio só a
    // mudaria através de um kernel dependente de param — caso que nenhum consumidor desta
    // função tem. Um mapa vazio é a resposta de sempre.
    output_shape(graph, ops, kernels, node, SHAPE_DEPTH, &DrivenParams::new()).map(|s| s.dense)
}
