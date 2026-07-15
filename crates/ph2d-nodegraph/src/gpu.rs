//! GPU kernel **side-metadata** for node types (GPU/M5 Fase 1, ADR-0122).
//!
//! A node's optional WGSL compute kernel is registered NEXT TO its op (the
//! registry's `register_gpu_kernel`, mirroring `register_ui`), never inside the
//! frozen `NodeManifest` — `NodeOp = 2` / `OpResolver = 1` / `NodeManifest = 8`
//! stay intact (gate `architecture_contract_surface`). A node without a kernel
//! opts into nothing and changes in nothing: the CPU `eval` remains the
//! **canonical** path (the replay-hash / `cook_determinism` oracle); a kernel
//! is a performance lowering the GPU sequencer (`ph2d-gpu-cook`) may run,
//! reconciled against the CPU by tolerance (float on a GPU is not
//! bit-reproducible cross-vendor).
//!
//! This module is pure data — no wgpu types, no GPU dependency. The kernel is
//! a WGSL *body* plus a declaration of which stream columns it reads/writes
//! and which manifest params it consumes; the sequencer generates the full
//! compute module around it (bindings, uniforms, entry point) and owns all
//! GPU objects. That keeps the substrate leaf-level and the kernel authorable
//! inside each node's own drop-crate.

use crate::node::NodeTypeId;
use crate::port::Dim;

/// How a kernel touches one named stream column.
///
/// Writes always land in a **fresh** buffer (the sequencer never mutates a
/// cooked column in place — the implicit ping-pong), so `ReadWrite` means
/// "read the input stream's column, write this node's output column".
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColumnAccess {
    /// Read-only. If the input stream lacks the column, the generated module
    /// substitutes the binding's [`ColumnBinding::identity`] as a constant —
    /// the same absent-column fallback the CPU nodes apply (e.g. `falloff` → 1).
    Read,
    /// Write-only (a generator's output). The column is materialized on the
    /// node's output stream.
    Write,
    /// Read + write; an **absent** input column is materialized from
    /// [`ColumnBinding::identity`] — matching CPU behaviours that build their
    /// target channel from its identity when the stream lacks it
    /// (`apply_channel_delta`'s `base_vec2`, doc 39).
    ReadWrite,
    /// Read + write, but **only when the input stream carries the column** —
    /// when absent the write is dropped and the column stays absent, matching
    /// CPU modifiers that pattern-match the column and otherwise pass through
    /// (`motion.move` touches `P` only if `P` exists).
    ReadWriteExisting,
}

impl ColumnAccess {
    /// Does this access read the input stream's column?
    pub const fn reads(self) -> bool {
        !matches!(self, ColumnAccess::Write)
    }

    /// Does this access write an output column (given whether the input
    /// stream carries it)?
    pub const fn writes(self, present_on_input: bool) -> bool {
        match self {
            ColumnAccess::Read => false,
            ColumnAccess::Write | ColumnAccess::ReadWrite => true,
            ColumnAccess::ReadWriteExisting => present_on_input,
        }
    }
}

/// One stream column a kernel touches: its name (the stream convention —
/// `P`, `size`, `rot`, `tint`, `falloff`, …), its element type, how it is
/// accessed, and the per-element identity used when a readable column is
/// absent (only the first [`Dim`]-many lanes are meaningful).
#[derive(Copy, Clone, Debug)]
pub struct ColumnBinding {
    pub column: &'static str,
    pub dim: Dim,
    pub access: ColumnAccess,
    /// The value an absent readable column reads as, per element. `P`/`rot`
    /// read `0`, `falloff` reads `1`, `size` reads unit scale (`SIZE_IDENTITY`)
    /// — the same identities the CPU paths use, so absence means the same
    /// thing on both sides.
    pub identity: [f32; 4],
}

/// A signature for "how many elements does this generator emit", evaluated on
/// the CPU at plan time from the node's resolved params (the getter applies
/// override-else-default, exactly like `EvalCtx::param`). Dispatch size must
/// be known host-side; a generator's kernel body then writes `0..count`.
pub type SourceCountFn = fn(&dyn Fn(&str) -> f32) -> usize;

/// A param-dependent applicability test, evaluated at plan time. A kernel
/// whose static WGSL body only covers part of a node's param space (e.g. an
/// oscillator kernel that handles the X/Y channels but not Rotation/Size)
/// returns `false` outside it, and the sequencer falls back to the CPU `eval`
/// for that node — the explicit CPU↔GPU boundary, never a wrong answer.
pub type ApplicableFn = fn(&dyn Fn(&str) -> f32) -> bool;

/// A node type's WGSL compute kernel — pure `'static` data, registered on the
/// side (ADR-0122). The sequencer wraps [`Self::wgsl`] in a generated module:
///
/// ```wgsl
/// // generated: uniforms (count, playhead, one f32 per declared param) +
/// // one storage binding per present column + read_/write_ helpers.
/// @compute @workgroup_size(256)
/// fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
///     let i = gid.x;
///     if (i >= params.count) { return; }
///     // ── kernel body pasted here ──
/// }
/// ```
///
/// Inside the body the kernel sees: `i` (the element index), `params.count` /
/// `params.playhead` / `params.<name>` for each entry of [`Self::params`], and
/// `read_<column>(i)` / `write_<column>(i, v)` for each of [`Self::bindings`]
/// (an absent readable column reads its identity; an absent
/// [`ColumnAccess::ReadWriteExisting`] column's write is a no-op). HR-5
/// discipline: waveform/trig math in a body must use the same polynomial
/// approximations as the node's CPU `eval` (WGSL `sin`/`cos` carry no
/// cross-vendor guarantee), so GPU-vs-CPU parity holds within ε.
#[derive(Copy, Clone, Debug)]
pub struct GpuKernel {
    /// The per-element kernel body (see the type-level contract above). An
    /// **empty** body with no bindings is the pass-through kernel
    /// ([`Self::PASSTHROUGH`]): the sequencer emits no pass and the stream
    /// flows through untouched.
    pub wgsl: &'static str,
    /// Module-level WGSL helpers the body calls (functions/consts), pasted
    /// before the entry point — a kernel cannot define functions inside the
    /// body. Namespaced by convention (`<node>_…`) so two kernels' helpers
    /// could coexist in a future fused module. Empty when the body is
    /// self-contained.
    pub wgsl_lib: &'static str,
    /// The stream columns the body touches.
    pub bindings: &'static [ColumnBinding],
    /// The manifest params the body reads (`params.<name>`), resolved
    /// override-else-default at dispatch time. Order defines the uniform
    /// layout; names must be declared `ParamSpec`s of the node.
    pub params: &'static [&'static str],
    /// `Some` for a **generator** (no stream input): the element count of the
    /// emitted stream as a function of the resolved params. `None` for a
    /// transformer (count = input count).
    pub source_count: Option<SourceCountFn>,
    /// `Some` when the kernel only covers part of the node's param space;
    /// evaluated at plan time. `None` = always applicable.
    pub applicable: Option<ApplicableFn>,
}

impl GpuKernel {
    /// The pass-through kernel: the node's output stream IS its input stream
    /// (`motion.output` and any other pure copy). No compute pass is emitted.
    pub const PASSTHROUGH: GpuKernel = GpuKernel {
        wgsl: "",
        wgsl_lib: "",
        bindings: &[],
        params: &[],
        source_count: None,
        applicable: None,
    };

    /// `true` for [`Self::PASSTHROUGH`]-shaped kernels (no body, no bindings).
    pub const fn is_passthrough(&self) -> bool {
        self.wgsl.is_empty() && self.bindings.is_empty()
    }
}

/// Resolves a node type id to its registered GPU kernel — the side-channel
/// mirror of [`crate::cook::OpResolver`], implemented by the node registry.
/// Kept as a trait so the GPU sequencer is decoupled from the registry crate
/// exactly like the cook engine is.
pub trait KernelResolver {
    fn gpu_kernel(&self, ty: NodeTypeId) -> Option<&GpuKernel>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_is_recognized_and_emits_nothing() {
        assert!(GpuKernel::PASSTHROUGH.is_passthrough());
        let real = GpuKernel {
            wgsl: "write_P(i, read_P(i));",
            wgsl_lib: "",
            bindings: &[ColumnBinding {
                column: "P",
                dim: Dim::Vec2,
                access: ColumnAccess::ReadWrite,
                identity: [0.0; 4],
            }],
            params: &[],
            source_count: None,
            applicable: None,
        };
        assert!(!real.is_passthrough());
    }

    #[test]
    fn access_read_write_matrix() {
        use ColumnAccess::*;
        assert!(Read.reads() && !Read.writes(true) && !Read.writes(false));
        assert!(!Write.reads() && Write.writes(true) && Write.writes(false));
        assert!(ReadWrite.reads() && ReadWrite.writes(false));
        // The pass-through-when-absent modifier: writes only what exists.
        assert!(ReadWriteExisting.reads());
        assert!(ReadWriteExisting.writes(true));
        assert!(!ReadWriteExisting.writes(false));
    }
}
