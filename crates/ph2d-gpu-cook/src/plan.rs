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
use ph2d_nodegraph::gpu::KernelResolver;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::{NodeManifest, NodeTypeId};
use std::collections::BTreeSet;

/// Where one input port of a [`GpuStage`] gets its stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuSource {
    /// Another planned stage's output, threaded GPU-side (the common case).
    Stage(NodeId),
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
    /// Stages in **topological** (source→sink) order, each node once; the sink
    /// is last. Pass-throughs are included and emit no pass.
    pub stages: Vec<GpuStage>,
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
) -> Option<Shape> {
    let budget = budget.checked_sub(1)?;
    let inst = graph.node(node)?;
    let manifest = ops.resolve(inst.type_id())?.manifest();
    let kernel = kernels.gpu_kernel(inst.type_id())?;
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
            Some((src, _, false)) => {
                let s = output_shape(graph, ops, kernels, src, budget)?;
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
                    Some((src, _, false)) => {
                        cols.extend(output_shape(graph, ops, kernels, src, budget)?.cols);
                    }
                }
            }
            return Some(Shape { cols, dense: false });
        }
        // Compact: the base's columns survive (filtered, not reshaped) and the
        // node's own kernel writes ride on top — the plain rules below.
        // SourceRows: the base IS the template whose columns the newborns
        // inherit, plus the kernel's writes; `cp_rows` is popped after the loop.
        Some(_) | None => {}
    }
    // The set this node will ACTUALLY bind — a param-dependent kernel writes a
    // different column per param, and deriving the shape from the default set
    // would predict columns the dispatch never produces.
    for b in kernel
        .resolve(&|name| resolve_param(graph, node, manifest, name))
        .bindings
    {
        let present = cols.contains(b.column);
        if b.access.consumes() {
            cols.remove(b.column);
        } else if b.access.writes(present) {
            cols.insert(b.column);
        }
    }
    // A SourceRows kernel's rows column is machinery, never output (ADR-0136).
    cols.remove(ph2d_nodegraph::gpu::ROWS_COL);
    // The dense id window (ADR-0130) flows on the port-0 base: a source that
    // emits one keeps it unconditionally (`inputs.is_empty()`), a transformer
    // only if it preserves AND its base already was dense. Anything that does not
    // declare `keeps_dense_window` clears it — the safe default, so a structural
    // node (`sort`/`cull`) breaks the window the instant it gains a kernel.
    let dense =
        kernels.keeps_dense_window(inst.type_id()) && (manifest.inputs.is_empty() || dense_in);
    Some(Shape { cols, dense })
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
    output_shape(graph, ops, kernels, node, SHAPE_DEPTH).map(|s| s.dense)
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
    // A driven param is a live wire into another subtree — the CPU cook
    // resolves those; a GPU stage has no lane for them (F1.2+).
    if graph.param_sources(node).is_some_and(|s| !s.is_empty()) {
        return false;
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
    if !kernel
        .params
        .iter()
        .all(|p| manifest.param_default(p).is_some())
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
        .resolve(&|name| resolve_param(graph, node, manifest, name))
        .bindings;
    for b in bindings.iter().filter(|b| b.access.refuses()) {
        let known = match graph.input_edge(node, b.port) {
            None => Some(BTreeSet::new()),
            Some((_, _, true)) => None,
            Some((src, _, false)) => {
                output_shape(graph, ops, kernels, src, SHAPE_DEPTH).map(|s| s.cols)
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
            Some((src, _, false)) => {
                output_shape(graph, ops, kernels, src, SHAPE_DEPTH).map(|s| (s.cols, s.dense))
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
        Some(f) => f(&|name| resolve_param(graph, node, manifest, name)),
        None => true,
    }
}

/// Recursion bound for [`output_shape`] — far above any real chain; it exists
/// so a malformed (forward-cyclic) graph is a refusal, not a stack overflow.
const SHAPE_DEPTH: u32 = 256;

/// The node's live param value: per-instance override else manifest default —
/// the same override>default resolution as `EvalCtx::param` (driven params
/// were excluded by [`eligible`], so the wire tier cannot apply here).
pub(crate) fn resolve_param(
    graph: &Graph,
    node: NodeId,
    manifest: &NodeManifest,
    name: &str,
) -> f32 {
    graph
        .node_param_overrides(node)
        .and_then(|m| m.get(name).copied())
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
                ) =>
            {
                self.accept(src, ty);
                GpuSource::Stage(src)
            }
            _ => {
                self.boundaries.push((src, out_port));
                GpuSource::Boundary(src, out_port)
            }
        }
    }
}

/// Claim the GPU-runnable part of `sink`'s chain (see the crate docs). Pure and
/// cheap — a walk of the graph, no GPU objects — so calling it every frame is
/// fine; compiled pipelines are cached by [`GpuCook`].
pub fn plan(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    sink: NodeId,
) -> GpuPlan {
    plan_forbidding(graph, ops, kernels, sink, &BTreeSet::new())
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
    sink: NodeId,
    forbidden: &BTreeSet<NodeId>,
) -> GpuPlan {
    let claimed = BTreeSet::new();
    let sink_ty = match graph.node(sink).map(|i| i.type_id()) {
        Some(ty) if eligible(graph, ops, kernels, sink, &claimed, forbidden) => ty,
        // The sink itself is CPU: nothing to claim. (`dispatching_stages` is 0,
        // so the caller's route recuses whole.)
        _ => {
            return GpuPlan {
                boundaries: vec![(sink, 0)],
                stages: Vec::new(),
            };
        }
    };
    let mut walk = Walk {
        graph,
        ops,
        kernels,
        stages: Vec::new(),
        boundaries: Vec::new(),
        claimed,
        forbidden,
    };
    walk.accept(sink, sink_ty);

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
            return plan_forbidding(graph, ops, kernels, sink, &next);
        }
        // No progress possible (a forbidden node still staged — cannot happen,
        // `eligible` refuses it): fall back to the whole refusal.
        return GpuPlan {
            boundaries: vec![(sink, 0)],
            stages: Vec::new(),
        };
    }

    GpuPlan {
        boundaries: walk.boundaries,
        stages: walk.stages,
    }
}
