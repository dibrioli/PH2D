#![forbid(unsafe_code)]
//! Runtime registry of node operations (ADR-0031, ADR-0032).
//!
//! Mirrors `ph2d-tool-registry` for the node world: each `ph2d-node-*` crate
//! registers its [`NodeOp`]; the registry resolves a [`NodeTypeId`] to that op
//! — it **is** the [`OpResolver`] the cook engine consumes — and rejects id
//! collisions at registration (so a duplicate canonical name is caught at boot,
//! not silently at cook time).
//!
//! The aggregation point (`register_all_nodes`, which calls each node crate's
//! `register`) lives in `ph2d-node-registry-init` and is codegen'd from a scan
//! of `crates/ph2d-node-*` (W1.T3 sync tool + staleness gate) — so adding a
//! node is dropping a crate, never hand-editing a central list.

use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::gpu::{
    GpuAlgorithm, GpuKernel, GridSpec, KernelResolver, LutSpec, ReduceSpec, StateSelect, StreamOp,
};
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId};
use std::collections::BTreeMap;

mod ui;
pub use ui::{
    NodeSilhouette, NodeUiCategory, NodeUiManifest, ParamHardMax, ParamUiHint, ParamWidget,
    ReadChannel,
};

/// A registered set of node operations, keyed by their stable type id.
/// Deterministic iteration (`BTreeMap`, ADR-0022 / HR-5).
#[derive(Default)]
pub struct NodeRegistry {
    ops: BTreeMap<NodeTypeId, Box<dyn NodeOp>>,
    /// M1.R1 — per-type UI side-metadata, keyed the same way as `ops` and kept
    /// separate so the frozen `NodeManifest` is untouched. A node registers its
    /// entry in `register` (alongside its op).
    ui: BTreeMap<NodeTypeId, NodeUiManifest>,
    /// M1.P1 — per-type param UI hints (editable range + widget + label for each
    /// declared `ParamSpec`), keyed the same way. A `&'static` slice per type so
    /// registration is a single insert; the params panel reads it to build rows.
    param_ui: BTreeMap<NodeTypeId, &'static [ParamUiHint]>,
    param_hard_max: BTreeMap<NodeTypeId, &'static [ParamHardMax]>,
    /// GPU/M5 Fase 1 (ADR-0126) — per-type WGSL compute kernel, keyed the same
    /// way and kept OUT of the frozen `NodeManifest` exactly like the UI
    /// metadata. Opt-in: a node without one runs its CPU `eval` (the canonical
    /// path); the GPU sequencer (`ph2d-gpu-cook`) resolves kernels through
    /// [`KernelResolver`].
    gpu_kernels: BTreeMap<NodeTypeId, GpuKernel>,
    /// GPU/M5 (ADR-0140 D2) — per-type spatial-grid requirement, keyed the same
    /// way and kept OUT of the frozen manifest like every other side channel. A
    /// node with a neighbourhood kernel (boids, `sim.collide`, SPH) registers a
    /// [`GridSpec`]; the sequencer builds the grid before its kernel pass.
    grids: BTreeMap<NodeTypeId, GridSpec>,
    /// GPU/M5 (ADR-0135) — per-type state-loop select (a `sim.zone`), the
    /// device mirror of the zone's `started ? state : init`. Same side-channel
    /// shape as `grids`; a node opts in with a [`StateSelect`].
    state_selects: BTreeMap<NodeTypeId, StateSelect>,
    /// Node types that KEEP the dense id window (ADR-0130): the emitter (source)
    /// and the per-element sim transformers (integrate/spring/forces). Opt-in and
    /// default-empty, so a node keeps the window only by declaring it — see
    /// [`KernelResolver::keeps_dense_window`].
    dense_window_keepers: std::collections::BTreeSet<NodeTypeId>,
    /// GPU/M5 (ADR-0136) — per-type structural stream operation (compact /
    /// source-rows / concat / project), the count-changing family's side channel.
    /// Same shape as `grids`; a node opts in with a [`StreamOp`].
    stream_ops: BTreeMap<NodeTypeId, StreamOp>,
    /// GPU/M5 (ADR-0139) — per-type multi-pass engine algorithm (Lloyd/JFA).
    /// Same side-channel shape; a node opts in with a [`GpuAlgorithm`].
    algorithms: BTreeMap<NodeTypeId, GpuAlgorithm>,
    /// GPU/M5 (ADR-0126) — per-type whole-stream reductions, the DEFORMER
    /// channel. Same side-channel shape; a node opts in with a slice of
    /// [`ReduceSpec`]s the sequencer folds before its kernel pass.
    reduces: BTreeMap<NodeTypeId, &'static [ReduceSpec]>,
    /// A1-gpu — per-type lookup tables the kernel samples (the LUT channel, a
    /// transfer curve / colour ramp whose shape lives in a text param). Same
    /// side-channel shape; a node opts in with a slice of [`LutSpec`]s the
    /// sequencer fills from that text param before its kernel pass.
    luts: BTreeMap<NodeTypeId, &'static [LutSpec]>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// Two node types resolve to the same id (same canonical name, or — far
    /// less likely — an FNV-1a collision of distinct names). Reports the id and
    /// the name of the second registrant.
    Collision { id: NodeTypeId, name: &'static str },
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a node op. Errors if its id collides with one already present.
    pub fn register(&mut self, op: Box<dyn NodeOp>) -> Result<(), RegistryError> {
        let manifest = op.manifest();
        let id = manifest.id;
        if self.ops.contains_key(&id) {
            return Err(RegistryError::Collision {
                id,
                name: manifest.name,
            });
        }
        self.ops.insert(id, op);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Deterministic iteration over registered manifests (by id order).
    pub fn manifests(&self) -> impl Iterator<Item = &'static NodeManifest> + '_ {
        self.ops.values().map(|op| op.manifest())
    }

    /// Register a node type's UI metadata (M1.R1). Additive to [`Self::register`]
    /// — a node registers both its op and its UI manifest; last write wins.
    pub fn register_ui(&mut self, id: NodeTypeId, ui: NodeUiManifest) {
        self.ui.insert(id, ui);
    }

    /// The UI metadata for `id`, if registered (M1.R1).
    pub fn ui_manifest(&self, id: NodeTypeId) -> Option<&NodeUiManifest> {
        self.ui.get(&id)
    }

    /// Register a node type's param UI hints (M1.P1). Additive; last write wins.
    pub fn register_param_ui(&mut self, id: NodeTypeId, hints: &'static [ParamUiHint]) {
        self.param_ui.insert(id, hints);
    }

    /// The param UI hints for `id`, if registered (M1.P1). The params panel pairs
    /// each with the manifest's `ParamSpec` (falling back to a plain slider when a
    /// param has no hint).
    pub fn param_ui(&self, id: NodeTypeId) -> Option<&'static [ParamUiHint]> {
        self.param_ui.get(&id).copied()
    }

    /// Register the params whose typed entry reaches past their slider
    /// ([`ParamHardMax`]). Additive; last write wins.
    pub fn register_param_hard_max(&mut self, id: NodeTypeId, limits: &'static [ParamHardMax]) {
        self.param_hard_max.insert(id, limits);
    }

    /// The typed-entry ceiling for `(id, param)`, if one was registered. `None`
    /// means "the slider's `max`", which is what every param without an entry
    /// has always meant.
    pub fn param_hard_max(&self, id: NodeTypeId, param: &str) -> Option<f32> {
        self.param_hard_max
            .get(&id)?
            .iter()
            .find(|l| l.param == param)
            .map(|l| l.max)
    }

    /// Register a node type's GPU compute kernel (GPU/M5 Fase 1, ADR-0126).
    /// Additive to [`Self::register`], exactly like [`Self::register_ui`]; last
    /// write wins. The kernel is pure `'static` data (`ph2d_nodegraph::gpu`) —
    /// registering one adds no GPU dependency here.
    pub fn register_gpu_kernel(&mut self, id: NodeTypeId, kernel: GpuKernel) {
        self.gpu_kernels.insert(id, kernel);
    }

    /// Declare that a node type's kernel needs a spatial neighbourhood grid
    /// (ADR-0140 D2). Additive, last-write-wins, pure `'static` data — like
    /// [`Self::register_gpu_kernel`]. The sequencer builds the grid over the
    /// [`GridSpec`]'s column before the node's kernel pass.
    pub fn register_grid(&mut self, id: NodeTypeId, grid: GridSpec) {
        self.grids.insert(id, grid);
    }

    /// Declare a node a **state-loop select** (a `sim.zone`, ADR-0135). Pair it
    /// with `register_gpu_kernel(id, GpuKernel::PASSTHROUGH)` so the plan claims
    /// the node; the sequencer then forwards `init` before the loop has state and
    /// `state` after, stripping the [`StateSelect::transients`].
    pub fn register_state_select(&mut self, id: NodeTypeId, select: StateSelect) {
        self.state_selects.insert(id, select);
    }

    /// Declare that a node type KEEPS the dense id window (ADR-0130): a source
    /// that emits one, or a per-element transformer that preserves it. Additive,
    /// idempotent; a node NOT declared here breaks the window and the `id`-gather
    /// recedes (the safe default — see [`KernelResolver::keeps_dense_window`]).
    pub fn register_dense_window(&mut self, id: NodeTypeId) {
        self.dense_window_keepers.insert(id);
    }

    /// Declare this node type's structural stream operation (ADR-0136), paired
    /// with `register_gpu_kernel` exactly like a grid: a `Compact` node's kernel
    /// is its post-filter epilogue (or `PASSTHROUGH`), a `SourceRows` node's
    /// kernel writes the rows column, `Concat`/`Project` pair with `PASSTHROUGH`
    /// so the plan claims them.
    pub fn register_stream_op(&mut self, id: NodeTypeId, op: StreamOp) {
        self.stream_ops.insert(id, op);
    }

    /// Declare this node type's engine algorithm (ADR-0139), paired with
    /// `register_gpu_kernel(id, GpuKernel::PASSTHROUGH)` so the plan claims it.
    pub fn register_gpu_algorithm(&mut self, id: NodeTypeId, alg: GpuAlgorithm) {
        self.algorithms.insert(id, alg);
    }

    /// Declare the whole-stream reductions this node's kernel reads — the
    /// DEFORMER channel (ADR-0126). Additive, last-write-wins, pure `'static`
    /// data, like [`Self::register_grid`]. The sequencer folds each spec before
    /// the node's kernel pass and gives the body `reduce_<name>()`.
    pub fn register_reduces(&mut self, id: NodeTypeId, specs: &'static [ReduceSpec]) {
        self.reduces.insert(id, specs);
    }

    /// Declare the lookup tables this node's kernel samples — the LUT channel
    /// (A1-gpu). Additive, last-write-wins, pure `'static` data, like
    /// [`Self::register_reduces`]. The sequencer fills each table from the named
    /// text param before the node's kernel pass and gives the body
    /// `<name>_sample(t)`.
    pub fn register_luts(&mut self, id: NodeTypeId, specs: &'static [LutSpec]) {
        self.luts.insert(id, specs);
    }
}

impl KernelResolver for NodeRegistry {
    fn gpu_kernel(&self, ty: NodeTypeId) -> Option<&GpuKernel> {
        self.gpu_kernels.get(&ty)
    }

    fn keeps_dense_window(&self, ty: NodeTypeId) -> bool {
        self.dense_window_keepers.contains(&ty)
    }

    fn state_select(&self, ty: NodeTypeId) -> Option<&StateSelect> {
        self.state_selects.get(&ty)
    }

    fn grid(&self, ty: NodeTypeId) -> Option<&GridSpec> {
        self.grids.get(&ty)
    }

    fn stream_op(&self, ty: NodeTypeId) -> Option<&StreamOp> {
        self.stream_ops.get(&ty)
    }

    fn algorithm(&self, ty: NodeTypeId) -> Option<&GpuAlgorithm> {
        self.algorithms.get(&ty)
    }

    fn reduces(&self, ty: NodeTypeId) -> &'static [ReduceSpec] {
        self.reduces.get(&ty).copied().unwrap_or(&[])
    }

    fn luts(&self, ty: NodeTypeId) -> &'static [LutSpec] {
        self.luts.get(&ty).copied().unwrap_or(&[])
    }
}

impl OpResolver for NodeRegistry {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        self.ops.get(&ty).map(|boxed| boxed.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, EvalCtx};
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_nodegraph::node::{LoweringKind, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const T: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
    const fn port(name: &'static str) -> PortSpec {
        PortSpec { name, ty: T }
    }

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("reg.src"),
        name: "reg.src",
        inputs: &[],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    static PASS_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("reg.pass"),
        name: "reg.pass",
        inputs: &[port("in")],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };

    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(2).with("v", Column::Scalar(vec![10.0, 20.0])));
        }
    }

    struct Pass;
    impl NodeOp for Pass {
        fn manifest(&self) -> &'static NodeManifest {
            &PASS_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let passthrough = ctx.input(0).clone();
            ctx.emit(passthrough);
        }
    }

    /// Shares SRC_MAN's id — a collision.
    struct Dup;
    impl NodeOp for Dup {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, _ctx: &mut EvalCtx<'_>) {}
    }

    #[test]
    fn register_and_resolve() {
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Src)).unwrap();
        reg.register(Box::new(Pass)).unwrap();
        assert_eq!(reg.len(), 2);
        assert!(reg.resolve(SRC_MAN.id).is_some());
        assert!(reg.resolve(NodeTypeId::of("nope")).is_none());
    }

    #[test]
    fn param_ui_hints_round_trip() {
        static HINTS: &[ParamUiHint] = &[ParamUiHint {
            param: "rows",
            label: "Rows",
            min: 1.0,
            max: 20.0,
            step: 1.0,
            widget: ParamWidget::IntSlider,
        }];
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Src)).unwrap();
        assert!(reg.param_ui(SRC_MAN.id).is_none()); // none until registered
        reg.register_param_ui(SRC_MAN.id, HINTS);
        let got = reg.param_ui(SRC_MAN.id).expect("registered");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].param, "rows");
        assert!(got[0].widget.is_integer());
        assert!(reg.param_ui(NodeTypeId::of("nope")).is_none());
    }

    #[test]
    fn duplicate_id_is_rejected() {
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Src)).unwrap();
        assert_eq!(
            reg.register(Box::new(Dup)),
            Err(RegistryError::Collision {
                id: SRC_MAN.id,
                name: "reg.src"
            })
        );
    }

    #[test]
    fn registry_is_a_resolver_the_cook_can_use() {
        // The registry IS the OpResolver — validate + cook a real graph through it.
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Src)).unwrap();
        reg.register(Box::new(Pass)).unwrap();

        let mut g = Graph::new();
        let s = g.add_node("reg.src");
        let p = g.add_node("reg.pass");
        g.connect(Edge {
            from: (s, 0),
            to: (p, 0),
            delayed: false,
        })
        .unwrap();
        g.validate(&reg).expect("graph is well-typed");

        let mut cook = Cook::new();
        let out = cook.cook(&g, &reg, p, 0.0).unwrap();
        match out[0].as_stream().get("v") {
            Some(Column::Scalar(v)) => assert_eq!(v, &vec![10.0, 20.0]),
            other => panic!("expected scalar column, got {other:?}"),
        }
    }
}
