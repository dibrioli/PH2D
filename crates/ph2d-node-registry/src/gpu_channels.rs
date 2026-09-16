//! **Como o dispositivo coze este nó** — ver o doc do `mod gpu_channels` no `lib.rs`.
//!
//! Os campos ficam no `lib.rs` (são o estado do registry); aqui vivem a metade que os ESCREVE
//! (`register_*`) e a que os LÊ (`impl KernelResolver`), movidas verbatim — mais o canal novo do
//! [`DerivedUniform`].

use crate::NodeRegistry;
use ph2d_nodegraph::gpu::{
    DerivedUniform, GpuAlgorithm, GpuKernel, GridSpec, KernelResolver, LutSpec, ReduceSpec,
    StateSelect, StreamOp,
};
use ph2d_nodegraph::node::NodeTypeId;

impl NodeRegistry {
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

    /// Declare WGSL that this node's kernel **and** its reductions both see — the SHARED
    /// channel. Additive, `'static`, like [`Self::register_reduces`]; o porquê inteiro (e a
    /// recusa que ele desfez) vive em [`KernelResolver::wgsl_shared`].
    pub fn register_wgsl_shared(&mut self, id: NodeTypeId, wgsl: &'static str) {
        self.wgsl_shared.insert(id, wgsl);
    }

    /// Declare the lookup tables this node's kernel samples — the LUT channel
    /// (A1-gpu). Additive, last-write-wins, pure `'static` data, like
    /// [`Self::register_reduces`]. The sequencer fills each table from the named
    /// text param before the node's kernel pass and gives the body
    /// `<name>_sample(t)`.
    pub fn register_luts(&mut self, id: NodeTypeId, specs: &'static [LutSpec]) {
        self.luts.insert(id, specs);
    }

    /// Declare os params cujo slot do uniform é DERIVADO no hospedeiro ([`DerivedUniform`]).
    /// Aditivo, a última escrita ganha, dados `'static` — como [`Self::register_luts`].
    pub fn register_derived_uniforms(&mut self, id: NodeTypeId, specs: &'static [DerivedUniform]) {
        self.derived_uniforms.insert(id, specs);
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

    fn wgsl_shared(&self, ty: NodeTypeId) -> &'static str {
        self.wgsl_shared.get(&ty).copied().unwrap_or("")
    }

    fn derived_uniforms(&self, ty: NodeTypeId) -> &'static [DerivedUniform] {
        self.derived_uniforms.get(&ty).copied().unwrap_or(&[])
    }
}
