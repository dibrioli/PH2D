//! Plan-level gates — no GPU device needed, so they run on every CI lane.
//!
//! The plan is the CPU↔GPU **boundary decision**, and a wrong decision is a
//! silent wrong render (a node skipped on both paths) or a wasted fallback (a
//! coverable chain cooked on the CPU). These pin the decision for the F1.1
//! chain and for each refusal reason.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
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
    // Oscillator on the Rotation channel (2): its kernel only covers X/Y →
    // `applicable` refuses → the CPU cooks grid+oscillator, the GPU runs the
    // suffix (move, output) from the uploaded boundary stream.
    let reg = registry();
    let (mut g, [_, osc, mv, out]) = chain(&reg);
    g.set_param(osc, "channel", 2.0);
    let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(plan.boundary, Some((osc, 0)));
    let nodes: Vec<NodeId> = plan.stages.iter().map(|s| s.node).collect();
    assert_eq!(nodes, vec![mv, out]);
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
    assert_eq!(plan.boundary, Some((mv, 0)));
    assert_eq!(plan.dispatching_stages(&reg), 0, "only the pass-through sink");
}

#[test]
fn a_node_without_a_kernel_breaks_the_suffix_exactly_there() {
    // A node type the kernel registry doesn't cover, sitting mid-chain: the
    // suffix stops at it — upstream coverage is moot. `test.nokernel` has an
    // op (so the CPU can cook it) but never called `register_gpu_kernel`.
    use ph2d_nodegraph::attr::Stream;
    use ph2d_nodegraph::cook::EvalCtx;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::node::{
        LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec,
    };
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
    assert_eq!(plan.boundary, Some((alien, 0)), "the CPU cooks up to it");
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
