//! GPU kernel **side-metadata** for node types (GPU/M5 Fase 1, ADR-0126).
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
/// "read the bound input port's column, write this node's output column".
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
    /// Read + **consume**: the body reads the column, and the node's output
    /// does NOT carry it — the transient-channel convention
    /// (`motion.integrate` eats the `accel` the in-loop forces accumulated, so
    /// every tick starts from zero acceleration). Without this the column would
    /// ride through from the base input (see [`ColumnBinding::port`]) and the
    /// next tick's forces would add onto a stale value, which is exactly what
    /// the CPU's `add_accel` does — divergence, not ε.
    Consume,
    /// **Not an access — a plan-time refusal.** The kernel declares that it
    /// cannot answer correctly for a stream carrying this column on this port,
    /// so [`crate::gpu`]'s sequencer refuses to claim the node and the CPU
    /// `eval` (the canonical path) owns it.
    ///
    /// The case that named it: `motion.integrate` pairs state to elements by
    /// **`id`** when the stream has identity (a gather) and **positionally**
    /// otherwise; the GPU kernel only does positional, so an `id` column on its
    /// `rest` port is a refusal (ADR-0127 D3). The eligibility answer must be
    /// provable, so an input whose columns the plan cannot derive (a CPU
    /// boundary feeds it) is refused too — **the default is to recede, never to
    /// answer wrong**.
    ///
    /// [`ColumnBinding::dim`] / [`ColumnBinding::identity`] are meaningless
    /// here (nothing is read): the binding declares only *which column on which
    /// port* is disqualifying.
    RefuseIfPresent,
}

impl ColumnAccess {
    /// Does this access read the bound port's column?
    pub const fn reads(self) -> bool {
        matches!(
            self,
            ColumnAccess::Read
                | ColumnAccess::ReadWrite
                | ColumnAccess::ReadWriteExisting
                | ColumnAccess::Consume
        )
    }

    /// Does this access write an output column (given whether the input
    /// stream carries it)?
    pub const fn writes(self, present_on_input: bool) -> bool {
        match self {
            ColumnAccess::Read | ColumnAccess::Consume | ColumnAccess::RefuseIfPresent => false,
            ColumnAccess::Write | ColumnAccess::ReadWrite => true,
            ColumnAccess::ReadWriteExisting => present_on_input,
        }
    }

    /// Does the node's output stream **drop** this column (rather than let the
    /// base input's copy ride through)?
    pub const fn consumes(self) -> bool {
        matches!(self, ColumnAccess::Consume)
    }

    /// Is this binding a plan-time refusal rather than an access?
    pub const fn refuses(self) -> bool {
        matches!(self, ColumnAccess::RefuseIfPresent)
    }
}

/// One stream column a kernel touches: its name (the stream convention —
/// `P`, `size`, `rot`, `tint`, `falloff`, …), its element type, how it is
/// accessed, which input port it is read from, and the per-element identity
/// used when a readable column is absent (only the first [`Dim`]-many lanes
/// are meaningful).
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
    /// Which **input port** of the node the column is read from (writes always
    /// land on the node's single output, so this only qualifies the read).
    /// `0` for every single-input node — the overwhelming case, and the reason
    /// this is spelled on each binding rather than inferred: a multi-input
    /// kernel that got it wrong would read the right column off the wrong
    /// stream and answer plausibly.
    ///
    /// **Port 0 is the base:** the node's output stream starts as port 0's
    /// (columns the kernel never mentions ride through it, exactly like the CPU
    /// nodes that copy their primary input and rewrite a channel), then written
    /// columns replace and [`ColumnAccess::Consume`]d ones are dropped.
    /// `motion.integrate` is the shape that named this: `rest` (port 0) carries
    /// the live animation forward while `forces` (port 1) carries last tick's
    /// state — and `vel` is read from BOTH (the seed from `rest`, the step from
    /// `forces`), which is why the port cannot be a property of the column name.
    pub port: usize,
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
/// side (ADR-0126). The sequencer wraps [`Self::wgsl`] in a generated module:
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
///
/// **Multi-input nodes** (`manifest.inputs.len() > 1`) get readers named by
/// **port** instead — `read_<port>_<column>(i)`, e.g. `read_rest_P(i)` /
/// `read_forces_vel(i)` — because the same column can be bound on two ports and
/// a bare `read_vel` would silently mean one of them. Writers stay
/// `write_<column>` (there is one output).
///
/// Every reader also gets a companion `const HAS_<same-suffix>: bool` telling
/// the body whether the column was actually **present** on that port, as
/// opposed to reading its identity. Presence is fixed per compiled pipeline
/// (the cache is keyed on exactly that signature), so branching on it is free —
/// and it is the only way to express a CPU behaviour that *branches* on absence
/// rather than substituting a value: `motion.integrate` seeds a fresh element
/// when there is no prior state at all, which is not the same as stepping it
/// with a zeroed one.
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
                port: 0,
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

    #[test]
    fn consume_reads_but_never_writes_and_drops_the_column() {
        use ColumnAccess::*;
        // The transient channel: the body sees it, the output does not carry it
        // — whether or not the base input had one.
        assert!(Consume.reads());
        assert!(!Consume.writes(true) && !Consume.writes(false));
        assert!(Consume.consumes());
        // Nothing else drops a column: an unmentioned column rides the base.
        for a in [Read, Write, ReadWrite, ReadWriteExisting, RefuseIfPresent] {
            assert!(!a.consumes(), "{a:?} must not drop the base's column");
        }
    }

    #[test]
    fn refuse_is_inert_as_an_access_and_only_answers_eligibility() {
        use ColumnAccess::*;
        // It binds no buffer and generates no accessor: a refusal that read or
        // wrote would cost a slot in every pipeline that never fires.
        assert!(!RefuseIfPresent.reads());
        assert!(!RefuseIfPresent.writes(true) && !RefuseIfPresent.writes(false));
        assert!(RefuseIfPresent.refuses());
        for a in [Read, Write, ReadWrite, ReadWriteExisting, Consume] {
            assert!(!a.refuses(), "{a:?} must not refuse the node");
        }
    }
}
