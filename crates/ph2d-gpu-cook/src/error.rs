//! The cook's refusal vocabulary — every reason a GPU frame recedes to the
//! canonical CPU instead of dispatching (split from `lib.rs` at the workspace
//! LOC cap; semantics unchanged). The common shape: each variant is a fact the
//! PLAN cannot see (device limits, stream lengths) discovered at cook time,
//! where the alternative to a `Result` is a validation panic.

use ph2d_nodegraph::node::NodeTypeId;

#[derive(Debug, PartialEq, Eq)]
pub enum GpuCookError {
    /// `plan.boundaries` and the `boundary_streams` argument disagree — the
    /// caller cooked (or skipped) the CPU prefix against a stale plan.
    BoundaryMismatch,
    /// A stage's kernel needs more storage bindings than the device allows
    /// (`max_storage_buffers_per_shader_stage`): `(node type, needed, limit)`.
    ///
    /// A kernel binds one buffer per column it touches, which is a fact about
    /// the STREAM, not the graph — so [`plan`], which never sees a device or a
    /// stream, cannot refuse it. The check lives here because the alternative is
    /// not a wrong answer but a **crash**: `create_compute_pipeline` reports an
    /// over-limit layout through the device's error scope, i.e. a panic, not a
    /// `Result`. The caller falls back to the CPU for the frame.
    TooManyBindings(NodeTypeId, u32, u32),
    /// A broadcast port carries its column at a length the dispatch can neither
    /// pair per-element nor pin to row 0 — neither the dispatch length, nor 1,
    /// nor empty (e.g. a 3-element value field aimed at a 5-element flock).
    ///
    /// Judged ABSENT, the kernel would read the identity at EVERY index while
    /// the CPU reads the real rows it has (`target_at`'s `_` arm) — a SHAPE
    /// divergence, not an ε. Like the binding limit, it is a fact about the
    /// STREAM (lengths exist only at cook time; the plan's `applicable` sees
    /// params alone), so the refusal lives here and the caller falls back to
    /// the CPU, which is canonical.
    BroadcastLengthMismatch {
        ty: NodeTypeId,
        port: usize,
        len: u32,
        count: u32,
    },
    /// The sink's instance buffer would exceed the device's storage-binding
    /// size limit — measured on the line's RTX: the adapter advertises
    /// **2 GiB − 4** (`max_storage_buffer_binding_size`, already requested at
    /// the adapter's own max by `ph2d-gpu`), which at 184 B per
    /// `RenderInstance` is **≈ 11,67 M instances**. Un-guarded this was a
    /// PRODUCTION PANIC (`create_bind_group` validation, found by the 12,58 M
    /// row of the scale sweep), not a fallback; the refusal takes the same door
    /// as [`Self::TooManyBindings`] — the caller recedes to the CPU. Raising it
    /// means SPLITTING the lowering binding (chunked draws), a named follow-up,
    /// not a requestable limit.
    BindingTooLarge { bytes: u64, limit: u64 },
    /// A registered [`ph2d_nodegraph::gpu::StreamOp`] and its kernel disagree —
    /// a compact predicate that wrote no `cp_keep`, a source-rows kernel that
    /// wrote no `cp_rows`. An authoring bug in the node's own crate (its gates
    /// catch it); production refuses the frame and the CPU stays canonical.
    MalformedStreamOp(NodeTypeId),
}
