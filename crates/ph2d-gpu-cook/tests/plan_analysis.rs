//! Plan-level gates — no GPU device needed, so they run on every CI lane.
//!
//! The plan is the CPU↔GPU **boundary decision**, and a wrong decision is a
//! silent wrong render (a node skipped on both paths) or a wasted fallback (a
//! coverable chain cooked on the CPU). These pin the decision for the F1.1
//! chain and for each refusal reason.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// A node whose kernel covers **half of its param space** — and nothing else.
///
/// It exists because `applicable` needs a live subject and **every shipping node
/// has outgrown one**. The gate below used the oscillator until
/// `GpuKernel::variant_by_param` let it claim every channel, then `motion.spring`
/// for the same reason, and then the variants reached the spring too. Both
/// rebases were the coverage work doing its job, and both were the *fixture*
/// dissolving under the gate rather than the gate finding anything
/// ([[feedback_a_seam_fixture_must_rest_on_something_uncoverable]]).
///
/// Borrowing a third node would buy the same amount of time. So the subject is
/// SYNTHETIC: `mode >= 0.5` is refused here by construction, which is not a
/// backlog item anybody can close. The mechanism under test — *`applicable`
/// refuses → the boundary lands at this node → the prefix cooks on the CPU* — is
/// engine behaviour, and it is the engine, not a node's coverage, that this gate
/// is about.
struct HalfCovered;

static HALF_COVERED: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.half_covered"),
    name: "test.half_covered",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "mode",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

impl NodeOp for HalfCovered {
    fn manifest(&self) -> &'static NodeManifest {
        &HALF_COVERED
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let out: Stream = ctx.input(0).clone();
        ctx.emit(out);
    }
}

/// The covered half writes `P` and nothing more; the refused half is refused.
static HALF_COVERED_KERNEL: GpuKernel = GpuKernel {
    wgsl: "        write_P(i, read_P(i));\n",
    wgsl_lib: "",
    bindings: &[ColumnBinding {
        column: "P",
        dim: Dim::Vec2,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["mode"],
    count_law: None,
    variant_by_param: None,
    applicable: Some(|param| param("mode") < 0.5),
};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    reg.register(Box::new(HalfCovered)).unwrap();
    reg.register_gpu_kernel(HALF_COVERED.id, HALF_COVERED_KERNEL);
    reg
}

/// `grid → oscillator → move → output`, the F1.1 chain. Returns the graph and
/// `(grid, osc, mv, out)`.
fn chain(reg: &NodeRegistry) -> (Graph, [NodeId; 4]) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let osc = g.add_node("motion.oscillator");
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    for (a, b) in [(grid, osc), (osc, mv), (mv, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(reg).expect("chain is well-typed");
    (g, [grid, osc, mv, out])
}

#[test]
fn the_covered_chain_is_claimed_whole() {
    let reg = registry();
    let (g, [grid, osc, mv, out]) = chain(&reg);
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu(), "every node has a kernel → no boundary");
    let nodes: Vec<NodeId> = plan.stages.iter().map(|s| s.node).collect();
    assert_eq!(nodes, vec![grid, osc, mv, out], "source→sink order");
    // Output is a pass-through: 3 passes dispatch, not 4. This is also the
    // "the optimization FIRES" assertion the parity gate re-checks.
    assert_eq!(plan.dispatching_stages(&reg), 3);
}

#[test]
fn an_uncovered_param_space_puts_the_boundary_at_that_node() {
    // [`HalfCovered`] with `mode = 1` — its `applicable` refuses → the CPU cooks
    // the prefix, the GPU runs the suffix from the uploaded boundary stream.
    //
    // ⚠️ The subject is SYNTHETIC on purpose, and it is the third one. This gate
    // used the OSCILLATOR until `GpuKernel::variant_by_param` let it claim every
    // channel, then `motion.spring` for exactly the same restriction, and then
    // the variants reached the spring too — twice the fixture dissolved because
    // the coverage work succeeded. See [`HalfCovered`] for why borrowing a fourth
    // node would only reset the clock.
    let reg = registry();
    let (mut g, [grid, osc, mv, out]) = chain(&reg);
    let sp = g.add_node("test.half_covered");
    g.set_param(sp, "mode", 1.0);
    g.disconnect(osc, 0).expect("the chain wired grid → osc");
    for (a, b) in [(grid, sp), (sp, osc)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(plan.boundaries, vec![(sp, 0)]);
    let nodes: Vec<NodeId> = plan.stages.iter().map(|s| s.node).collect();
    assert_eq!(nodes, vec![osc, mv, out]);

    // The PRESENCE sibling: the SAME graph on the covered half is claimed whole.
    // Without it the assertion above holds just as well for a kernel that
    // refuses unconditionally — or for a `plan` that refuses everything — and
    // the gate would be measuring nothing
    // ([[feedback_absence_gate_needs_a_presence_sibling]]).
    g.set_param(sp, "mode", 0.0);
    let claimed = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(
        claimed.is_fully_gpu(),
        "the covered half must be claimed: {:?}",
        claimed.boundaries
    );
    let nodes: Vec<NodeId> = claimed.stages.iter().map(|s| s.node).collect();
    assert_eq!(nodes, vec![grid, sp, osc, mv, out]);
}

#[test]
fn a_driven_param_puts_the_boundary_at_the_driven_node() {
    // Wire a driver into `move.dx` (doc 58): the GPU stage has no lane for a
    // live wire, so `move` itself must cook on the CPU; only `output` (a
    // pass-through) stays GPU-side — a lowering-only plan.
    let reg = registry();
    let (mut g, [grid, _, mv, out]) = chain(&reg);
    g.drive_param(mv, "dx", (grid, 0)).unwrap();
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(plan.boundaries, vec![(mv, 0)]);
    assert_eq!(
        plan.dispatching_stages(&reg),
        0,
        "only the pass-through sink"
    );
}

#[test]
fn a_node_without_a_kernel_breaks_the_suffix_exactly_there() {
    // A node type the kernel registry doesn't cover, sitting mid-chain: the
    // suffix stops at it — upstream coverage is moot. `test.nokernel` has an
    // op (so the CPU can cook it) but never called `register_gpu_kernel`.
    use ph2d_nodegraph::attr::Stream;
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
    const T: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
    static NOKERNEL: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.nokernel"),
        name: "test.nokernel",
        inputs: &[PortSpec { name: "in", ty: T }],
        outputs: &[PortSpec { name: "out", ty: T }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct NoKernel;
    impl NodeOp for NoKernel {
        fn manifest(&self) -> &'static NodeManifest {
            &NOKERNEL
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(ctx.input(0).count()));
        }
    }

    let mut reg = registry();
    reg.register(Box::new(NoKernel)).unwrap();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let alien = g.add_node("test.nokernel");
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    for (a, b) in [(grid, alien), (alien, mv), (mv, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(&reg).expect("well-typed");
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(plan.boundaries, vec![(alien, 0)], "the CPU cooks up to it");
    let nodes: Vec<NodeId> = plan.stages.iter().map(|s| s.node).collect();
    assert_eq!(nodes, vec![mv, out], "the GPU runs the suffix below it");
}

#[test]
fn an_unconnected_transformer_input_is_a_fully_gpu_empty_chain() {
    // `move → output` with nothing feeding move: the CPU cook's value for the
    // unconnected input is the empty stream; the plan claims the chain and the
    // cook dispatches nothing (count 0) — mirrored semantics, no boundary.
    let reg = registry();
    let mut g = Graph::new();
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (mv, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(plan.is_fully_gpu());
    assert_eq!(plan.stages.len(), 2);
}
