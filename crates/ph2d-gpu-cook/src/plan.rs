//! The **plan**: which part of a sink's chain the GPU can claim, and where each
//! claimed node's inputs come from.
//!
//! Pure and cheap — a walk of the graph, no GPU objects, no streams — so the
//! caller re-derives it every frame and the sequencer ([`crate::GpuCook`])
//! merely executes it. Keeping the decision here, in one testable function, is
//! what lets `plan_analysis` gate the CPU↔GPU boundary on every CI lane: a
//! wrong decision is a silent wrong render (a node skipped on both paths) or a
//! wasted fallback (a coverable chain cooked on the CPU).

use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::gpu::{Complement, FiredBirth, KernelResolver, StreamOp};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::{NodeManifest, NodeTypeId};
use std::collections::{BTreeMap, BTreeSet};

/// Where one input port of a [`GpuStage`] gets its stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuSource {
    /// Another planned stage's output, threaded GPU-side (the common case).
    Stage(NodeId),
    /// A saída ≠ 0 de um estágio — o COMPLEMENTO de um `Compact` que a declara (doc 110 §14.1
    /// (1)). `Stage(n)` continua a ser a porta 0.
    StagePort(NodeId, usize),
    /// A CPU-cooked stream, uploaded once at the seam: `(node, out_port)`.
    Boundary(NodeId, usize),
    /// The **previous tick's** output of a planned node — a `pre` edge (D1).
    /// The sequencer simply held that tick's `Arc`s, so this costs nothing.
    Prev(NodeId),
    /// An unconnected input: the empty stream (the CPU cook's value for it).
    Empty,
}

/// One GPU stage of a plan: a node whose kernel runs as a compute pass
/// (pass-through kernels are planned but dispatch nothing).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuStage {
    pub node: NodeId,
    pub ty: NodeTypeId,
    /// Where each of the node's manifest input ports reads from, in port order.
    pub inputs: Vec<GpuSource>,
}

/// The result of [`plan`]: the GPU-runnable part of a sink's chain — a **DAG**
/// (ADR-0127 D2), not a chain: a node's inputs are walked on every port, so the
/// stages come out in topological order and each input names its own source.
#[derive(Clone, Debug, Default)]
pub struct GpuPlan {
    /// One per input the walk could not claim: cook that node with the ordinary
    /// [`Cook`] and hand its output stream to [`GpuCook::cook`]. Empty — every
    /// input bottoms out at a generator, an unconnected port or a `pre` stop:
    /// the whole chain is GPU-resident.
    pub boundaries: Vec<(NodeId, usize)>,
    /// Stages in **topological** (source→sink) order, each node once. With ONE sink
    /// the sink is last; with several (doc 119 W2) every sink comes after its own
    /// chain, and a node two sinks share appears ONCE. Pass-throughs are included
    /// and emit no pass.
    pub stages: Vec<GpuStage>,
    /// ⭐⭐ **As saídas que o plano ENCENOU, na ordem em que foram pedidas** (doc 119 W2) — cada
    /// uma é um dos [`Self::stages`] e o cozimento baixa-as por esta ordem. Com um sink só ela é
    /// `[o último estágio]`; ⚠️ um sink que a placa não pode encenar **não está aqui**, está em
    /// [`Self::boundaries`] como `(sink, 0)` — *pedir uma saída não é garantir que ela chega à
    /// placa*, e quem escolhe a rota compara esta lista com a que pediu.
    pub sinks: Vec<NodeId>,
}

impl GpuPlan {
    /// `true` when the entire chain runs on the GPU (no CPU prefix).
    pub fn is_fully_gpu(&self) -> bool {
        self.boundaries.is_empty()
    }

    /// Number of stages that actually dispatch a compute pass (excludes
    /// pass-throughs) — what a "the optimization FIRES" gate should assert.
    pub fn dispatching_stages(&self, kernels: &dyn KernelResolver) -> usize {
        self.stages
            .iter()
            .filter(|s| {
                kernels
                    .gpu_kernel(s.ty)
                    .is_some_and(|k| !k.is_passthrough())
            })
            .count()
    }

    /// `true` when any stage feeds a `pre` edge — i.e. the plan owns a
    /// simulation loop, and its state lives GPU-side across ticks (D1).
    pub fn drives_a_loop(&self) -> bool {
        self.stages
            .iter()
            .any(|s| s.inputs.iter().any(|i| matches!(i, GpuSource::Prev(_))))
    }

    /// `true` when the GPU suffix REORDERS or CHANGES THE COUNT of the principal
    /// (object) stream — a structural [`StreamOp`](ph2d_nodegraph::gpu::StreamOp)
    /// (Compact / SourceRows / Concat / Project), an engine
    /// [`algorithm`](KernelResolver::algorithm) (voronoi reshapes wholesale), or
    /// a `count_law` (a birth law, `sim.spawn`) — on the path the object flows.
    /// A per-element node (deformer, force, integrator, oscillator) declares
    /// none of these and preserves position.
    ///
    /// The texture-run partition for a `source.object` graph
    /// ([`GpuCook::texture_runs`](crate::GpuCook::texture_runs)) reads the
    /// boundary stream's `texture_id` column and assumes position `i` of the
    /// boundary is position `i` of the sink — true iff the suffix preserves
    /// position. When this returns `true`, the object graph must recuse to the
    /// CPU render (the boundary column no longer aligns with the device
    /// buffer), which the shell checks with the object-source signal. No node
    /// name is enumerated: the answer is each node's own declaration.
    ///
    /// ⚠️ **Only the PORT-0 lineage** (sink → boundary) is walked — the path the
    /// object stream takes. A generator on a SIDE port (a `value.lfo` driving a
    /// deformer's amount) sets its OWN count on its own port; it never reshapes
    /// the object stream, so scanning every stage would over-recuse the common
    /// case of an animated object deformer.
    ///
    /// ⚠️ Com várias saídas (doc 119 W2) a pergunta é feita a **cada uma**: a partição de
    /// texturas de uma saída desalinha-se pelo sufixo DELA, e basta uma para a rota recusar.
    pub fn suffix_changes_count(&self, kernels: &dyn KernelResolver) -> bool {
        self.sinks
            .iter()
            .any(|&sink| self.lineage_changes_count(sink, kernels))
    }

    /// ⭐⭐ **A fronteira em que a linhagem da porta 0 de UMA saída acaba** (doc 119 W3) — o fluxo
    /// de objectos que ESTA saída desenha, e portanto a única fronteira cuja coluna `texture_id`
    /// se alinha com as linhas dela. `None` quando a linhagem acaba num gerador, num `pre` ou numa
    /// porta por ligar (a saída é toda de átlas).
    ///
    /// ⚠️ Com uma saída a partição procura a fronteira pelo COMPRIMENTO entre todas (a lei de
    /// sempre); com várias, duas fronteiras do mesmo comprimento seriam indistinguíveis — e é aí
    /// que esta pergunta, que é a mesma caminhada da cerca do sufixo, dá o endereço certo.
    pub fn lineage_boundary(&self, sink: NodeId) -> Option<NodeId> {
        let stage_of = |node: NodeId| self.stages.iter().find(|s| s.node == node);
        let mut budget = self.stages.len() + 1;
        let mut cur = stage_of(sink);
        while let Some(s) = cur {
            budget = budget.checked_sub(1)?;
            cur = match s.inputs.first() {
                Some(GpuSource::Stage(node) | GpuSource::StagePort(node, _)) => stage_of(*node),
                Some(GpuSource::Boundary(node, _)) => return Some(*node),
                _ => None,
            };
        }
        None
    }

    /// A linhagem da porta 0 de UMA saída — a lei de [`Self::suffix_changes_count`].
    fn lineage_changes_count(&self, sink: NodeId, kernels: &dyn KernelResolver) -> bool {
        let stage_of = |node: NodeId| self.stages.iter().find(|s| s.node == node);
        let Some(sink) = stage_of(sink) else {
            return false;
        };
        // Acyclic forward path (a loop must cross a `pre`, which is a Prev stop);
        // the budget is a guard against a malformed plan, not a real limit.
        let mut budget = self.stages.len() + 1;
        let mut cur = Some(sink);
        while let Some(s) = cur {
            if kernels.stream_op(s.ty).is_some()
                || kernels.algorithm(s.ty).is_some()
                || kernels
                    .gpu_kernel(s.ty)
                    .is_some_and(|k| k.count_law.is_some())
            {
                return true;
            }
            budget = match budget.checked_sub(1) {
                Some(b) => b,
                None => return true, // conservative: a cycle can only mis-bind
            };
            cur = match s.inputs.first() {
                // Port 0 threads GPU-side to another stage — keep walking.
                Some(GpuSource::Stage(node) | GpuSource::StagePort(node, _)) => stage_of(*node),
                // Boundary / Prev / Empty / no input — the object path ends here.
                _ => None,
            };
        }
        false
    }
}

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
fn output_shape(
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
fn edge_shape(
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
fn declares_port(kernels: &dyn KernelResolver, ty: NodeTypeId, port: u16) -> bool {
    kernels.projected_ports(ty).iter().any(|p| p.port == port)
        || matches!(
            kernels.stream_op(ty),
            Some(StreamOp::Compact { complement, .. }) if complement.iter().any(|c| c.port == port)
        )
}

/// The proven shape of a node's output stream: which columns it carries, and
/// whether its `id` column is a dense ascending window (ADR-0130). `None` when
/// the walk cannot prove it (a `pre` stop, an unkernelled node, budget).
struct Shape {
    cols: BTreeSet<&'static str>,
    dense: bool,
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

/// Can the GPU claim this node? Every reason a node breaks the GPU claim lives
/// here — plan-time diagnostics (the editor can badge the boundary).
///
/// `claimed` is the set of nodes the in-flight walk is already staging; it
/// answers the one question a `pre` edge asks (see below).
fn eligible(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    node: NodeId,
    claimed: &BTreeSet<NodeId>,
    forbidden: &BTreeSet<NodeId>,
    driven: &DrivenParams,
) -> bool {
    // A node the RETREAT forbade (`plan`): a partially-claimed sim loop drops its
    // `pre`-source here so the loop recedes to the pump while a render suffix
    // stays on the GPU. Ineligible = a boundary, exactly what the zone was before
    // it had a kernel.
    if forbidden.contains(&node) {
        return false;
    }
    let Some(inst) = graph.node(node) else {
        return false;
    };
    let Some(op) = ops.resolve(inst.type_id()) else {
        return false;
    };
    let manifest = op.manifest();
    let Some(kernel) = kernels.gpu_kernel(inst.type_id()) else {
        return false;
    };
    // ⭐⭐⭐ **UM PARAM DIRIGIDO JÁ NÃO DERRUBA O NÓ** (doc 110 §3 — a recusa cega que este
    // planeador tinha desde a F1.2). Ela custava o dispositivo à razão de existir de uma família
    // inteira: **medido, 6 de 6 cadeias caíam, e a primeira delas era uma CONSTANTE**
    // (`value.number`), trocando `3,85 ms` por `195,9 ms` (doc 98).
    //
    // ⚠️ **O que a substitui não é «aceitar sempre»: é aceitar quando o VALOR está na mão.** Um
    // param dirigido cujo número não veio no mapa continua a ser recusado — e é essa metade que
    // impede a única falha grave possível aqui, a das duas rotas a desenharem documentos
    // diferentes (ver [`DrivenParams`]). Sem mapa nenhum, esta função responde exactamente o que
    // respondia antes desta wave.
    if let Some(fios) = graph.param_sources(node) {
        let mapa = driven.get(&node);
        if !fios.keys().all(|p| mapa.is_some_and(|m| m.contains_key(p))) {
            return false;
        }
    }
    // **Uma porta ≠ 0 só é encenável quando o nó a DECLARA** (o complemento de um `Compact`,
    // doc 110 §14.1 (1)): qualquer outra seria o buffer da porta 0 entregue em silêncio. A
    // recusa vive AQUI e não num nó, para o próximo nascer coberto (`pulse.counter`: o kernel
    // não calcula o `carry`, e recua). E um fio ATRASADO de porta ≠ 0 recua sempre: o estado
    // do quadro anterior (`GpuSource::Prev`) só guarda a porta 0.
    if graph.edges().iter().any(|e| {
        e.from.0 == node
            && e.from.1 != 0
            && (e.delayed || !declares_port(kernels, inst.type_id(), e.from.1))
    }) {
        return false;
    }
    // O segundo nascimento decide-se pela FORMA da porta lateral (`fired_active`): ligada, ela
    // tem de ser provável, e um `pre` nela recua (o estado só guarda a porta 0 — e a forma dele
    // não se deriva).
    if let Some(StreamOp::SourceRows { fired: Some(f), .. }) = kernels.stream_op(inst.type_id()) {
        match graph.input_edge(node, f.port) {
            None => {}
            Some((_, _, true)) => return false,
            Some((src, sp, false)) => {
                if edge_shape(graph, ops, kernels, (src, sp), SHAPE_DEPTH, driven).is_none() {
                    return false;
                }
            }
        }
    }
    // A generator must say how many elements it emits (dispatch is host-sized).
    if manifest.inputs.is_empty() && kernel.count_law.is_none() && !kernel.is_passthrough() {
        return false;
    }
    // Kernel params must be declared manifest params. They no longer have to avoid
    // the built-in uniform names: `codegen::wgsl_field` gives a colliding param its
    // own field (`count` → `count_`), so the shadowing this used to refuse cannot
    // happen. `motion.pin_constraint` has a param called `count`, and renaming it
    // was never an option — it is the artist's vocabulary and it is in saved docs.
    //
    // ⚠️ **Ou são um uniform DERIVADO que só a derivação conhece** (ciclo 7, `DerivedUniform`: a
    // matriz de cor do `motion.trail` não tem nove params no manifesto para lhe emprestar).
    let derivados = kernels.derived_uniforms(inst.type_id());
    if !kernel
        .params
        .iter()
        .all(|p| manifest.param_default(p).is_some() || derivados.iter().any(|d| d.param == *p))
    {
        return false;
    }
    // A `pre` edge is a STOP, not a refusal (D1/D2) — but only when it closes a
    // loop onto a node this plan also stages, i.e. either an ancestor the walk
    // is already committed to (`integrate.out --pre--> force₁ … --> integrate`)
    // or the node ITSELF (the bare `out --pre--> forces` self-loop the editor
    // auto-wires when no force chain is present — the default shape, and the
    // node is not in `claimed` yet because eligibility is settled BEFORE the
    // walk commits to it). A `pre` from anywhere else wants the CPU's previous
    // output, and this engine has no seam that hands one over; claiming the node
    // would silently feed it an empty stream. Recede.
    for port in 0..manifest.inputs.len() {
        if let Some((src, _, true)) = graph.input_edge(node, port)
            && src != node
            && !claimed.contains(&src)
        {
            return false;
        }
    }
    // Column-shape refusals (D3): the kernel names a column whose presence it
    // cannot answer for — `motion.integrate` + `id` is a gather. The shape must
    // be PROVABLE, so an input the plan cannot derive refuses too.
    let bindings = kernel
        .resolve(&|name| resolve_param(graph, node, manifest, name, driven))
        .bindings;
    for b in bindings.iter().filter(|b| b.access.refuses()) {
        let known = match graph.input_edge(node, b.port) {
            None => Some(BTreeSet::new()),
            Some((_, _, true)) => None,
            Some((src, sp, false)) => {
                edge_shape(graph, ops, kernels, (src, sp), SHAPE_DEPTH, driven).map(|s| s.cols)
            }
        };
        match known {
            None => return false,
            Some(cols) if cols.contains(b.column) => return false,
            Some(_) => {}
        }
    }
    // The `id`-gather (ADR-0130 D2): a conditional refusal. The node pairs its
    // per-element state by the `id` column, which is a plain row offset
    // (`current_id − prev_first`) ONLY when the port's stream is a dense
    // ascending id window — the property the bare emitter creates and a
    // `sort`/`cull` destroys. So:
    //   • the column is ABSENT  → positional pairing, claim (a grid);
    //   • present AND the window is provably DENSE → the arithmetic gather, claim;
    //   • present but NOT dense → the offset would mispair in silence, recede;
    //   • the shape is UNPROVABLE (a CPU boundary feeds the port) → recede.
    // Default-recede is the whole point: a future structural node that forgets to
    // declare `keeps_dense_window` clears the window, so the gather never claims
    // a stream it cannot pair ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
    for b in bindings.iter().filter(|b| b.access.is_gather_key()) {
        let shape = match graph.input_edge(node, b.port) {
            None => Some((BTreeSet::new(), false)),
            Some((_, _, true)) => None,
            Some((src, sp, false)) => {
                edge_shape(graph, ops, kernels, (src, sp), SHAPE_DEPTH, driven)
                    .map(|s| (s.cols, s.dense))
            }
        };
        match shape {
            None => return false,
            Some((cols, dense)) if cols.contains(b.column) && !dense => return false,
            Some(_) => {}
        }
    }
    // Param-dependent coverage (e.g. the oscillator's X/Y-only kernel).
    match kernel.applicable {
        Some(f) => f(&|name| resolve_param(graph, node, manifest, name, driven)),
        None => true,
    }
}

/// Recursion bound for [`output_shape`] — far above any real chain; it exists
/// so a malformed (forward-cyclic) graph is a refusal, not a stack overflow.
const SHAPE_DEPTH: u32 = 256;

/// ⭐⭐⭐ **O VALOR DE CADA PARAM DIRIGIDO neste quadro** (doc 110 §3 · [doc 102 W1]).
///
/// Um param dirigido é um fio para outra sub-árvore, e o número dele **não está no grafo**: quem o
/// sabe é quem cozeu o condutor. O sequenciador não coze nada, logo o valor **entra** — e a
/// ausência dele tem consequência declarada em [`eligible`]: *sem valor, o nó não é encenado*.
///
/// ⚠️ **A ausência não pode cair no default em silêncio.** A CPU, para um condutor que ainda não
/// produziu número nenhum, deixa o param a CAIR no override/default (a lei do `driven_value`); mas
/// um condutor que produziu `7` e um mapa que não o trouxe leem-se **iguais** aqui — e aí as duas
/// rotas desenhavam documentos diferentes sem nada acusar. Por isso a regra é sobre a CHAVE, não
/// sobre o valor: quem tem fio e não tem entrada no mapa fica na CPU, como sempre esteve.
///
/// [doc 102 W1]: ../../../docs/Motion%20Nodes/102_o_outro_patamar_plano_dos_nos_2026-09-04.md
/// ⚠️⚠️ **O valor é um `Option`, e as duas metades dele são perguntas DIFERENTES:**
/// - a **chave ausente** = *«ninguém consultou este fio»* ⇒ o nó não é encenado (a lei de sempre);
/// - a chave presente com **`None`** = *«consultei, e o condutor não deu número»* ⇒ o param CAI no
///   override/default, que é exactamente o que a CPU faz (`driven_value`), e o nó **fica**.
///
/// ⛔⛔ **Colapsar as duas faria a cena saltar entre a placa e o processador de quadro para
/// quadro:** um `pulse.*` só dá número no instante em que dispara, e sem esta distinção o plano
/// mudava de rota a cada tique — um engasgo visível, causado por uma escolha de tipo.
pub type DrivenParams = BTreeMap<NodeId, BTreeMap<String, Option<f32>>>;

/// The node's live param value: **driven** (the wire tier) else per-instance
/// override else manifest default — a escada do `EvalCtx::param`, agora com os
/// três degraus, e não dois.
pub(crate) fn resolve_param(
    graph: &Graph,
    node: NodeId,
    manifest: &NodeManifest,
    name: &str,
    driven: &DrivenParams,
) -> f32 {
    driven
        .get(&node)
        .and_then(|m| m.get(name).copied())
        .flatten()
        .or_else(|| {
            graph
                .node_param_overrides(node)
                .and_then(|m| m.get(name).copied())
        })
        .or_else(|| manifest.param_default(name))
        .unwrap_or(0.0)
}

/// The in-flight DFS of [`plan`].
struct Walk<'a> {
    graph: &'a Graph,
    ops: &'a dyn OpResolver,
    kernels: &'a dyn KernelResolver,
    stages: Vec<GpuStage>,
    boundaries: Vec<(NodeId, usize)>,
    /// Os valores dos params dirigidos — ver [`DrivenParams`].
    driven: &'a DrivenParams,
    /// Nodes this walk is staging — inserted on ENTRY, so a `pre` edge that
    /// closes a loop back onto an ancestor still on the stack sees it. Since
    /// eligibility is settled before `accept`, "claimed" is never withdrawn:
    /// at the end this set is exactly `stages`.
    claimed: BTreeSet<NodeId>,
    /// Nodes the retreat forced to be boundaries (see [`plan`]). Empty on the
    /// first pass.
    forbidden: &'a BTreeSet<NodeId>,
}

impl Walk<'_> {
    /// Stage `node` (already found [`eligible`]) after its inputs — post-order,
    /// so `stages` comes out topologically sorted and the sink lands last.
    /// Re-entry is a no-op: a diamond stages its shared ancestor once.
    fn accept(&mut self, node: NodeId, ty: NodeTypeId) {
        if !self.claimed.insert(node) {
            return;
        }
        let manifest = self.ops.resolve(ty).expect("eligible checked").manifest();
        let inputs = (0..manifest.inputs.len())
            .map(|port| self.source_of(node, port))
            .collect();
        self.stages.push(GpuStage { node, ty, inputs });
    }

    /// Resolve one input port to its stream source, recursing into a claimable
    /// upstream and recording a boundary otherwise.
    fn source_of(&mut self, node: NodeId, port: usize) -> GpuSource {
        let Some((src, out_port, delayed)) = self.graph.input_edge(node, port) else {
            return GpuSource::Empty;
        };
        if delayed {
            // `eligible` already proved the source is one we stage (else this
            // node would not be here) — the loop closes GPU-side.
            return GpuSource::Prev(src);
        }
        match self.graph.node(src).map(|i| i.type_id()) {
            Some(ty)
                if eligible(
                    self.graph,
                    self.ops,
                    self.kernels,
                    src,
                    &self.claimed,
                    self.forbidden,
                    self.driven,
                ) =>
            {
                self.accept(src, ty);
                // `eligible` só admitiu um `src` com fio de porta ≠ 0 se ele a declara.
                if out_port == 0 {
                    GpuSource::Stage(src)
                } else {
                    GpuSource::StagePort(src, out_port)
                }
            }
            _ => {
                self.boundaries.push((src, out_port));
                GpuSource::Boundary(src, out_port)
            }
        }
    }
}

#[path = "plan_uniao.rs"]
mod uniao;
pub use uniao::{plan, plan_driven, plan_driven_many};
