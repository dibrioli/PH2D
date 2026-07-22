//! **How a kernel touches a stream column** — the [`ColumnAccess`] verb and the
//! [`ColumnBinding`] bundle, split from `gpu.rs` at the LOC cap. Pure data; the
//! sequencer reads these to decide what buffer each binding gets and how the
//! generated body reads/writes it.

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
    /// Read-only, and a **length-1 port BROADCASTS**: element `i` reads row `i`
    /// when the port is the dispatch length, and row `0` when the port carries
    /// exactly one value — one number held across the whole field.
    ///
    /// This is the third length rule in the engine, and it is the artist-visible
    /// one. `motion.look_at`'s target is two VALUE fields: wire a bare
    /// `value.lfo` and the ENTIRE flock turns toward one moving point; wire a
    /// per-element field and each element aims somewhere else. The CPU expresses
    /// it as `target_at` (`0 => identity, 1 => vals[0], _ => vals[i]`), and this
    /// is that function, declared.
    ///
    /// Plain [`Self::Read`] cannot express it: presence there means *"this port
    /// is the dispatch length"*, so a length-1 target would be judged ABSENT and
    /// silently read its identity — the flock would face the origin instead of
    /// the point the artist animated. Absent still reads the identity (the
    /// `0 =>` arm); only the length-1 case is new.
    ReadBroadcast,
    /// **The `id`-gather key** (ADR-0130), the conditional successor to
    /// [`Self::RefuseIfPresent`] for `motion.integrate`/`motion.spring`. Names
    /// the column this node pairs its per-element state by — an `id` on the
    /// **base** port.
    ///
    /// It is BOTH a plan-time guard and a real read:
    ///
    /// - **Plan:** like [`Self::RefuseIfPresent`], the node recedes when this
    ///   column is present on its port — **except** when the plan can prove that
    ///   port's stream is a **dense ascending id window**
    ///   ([`crate::gpu::KernelResolver::keeps_dense_window`]). A dense window is
    ///   exactly the case where the gather is arithmetic (`current_id −
    ///   prev_first`, a row offset) rather than a hash/sort — so the node CLAIMS
    ///   it. A present-but-not-dense `id` (a `sort`/`cull` broke the window) or
    ///   an unprovable shape still recedes: the default is to answer right on the
    ///   CPU, never wrong on the GPU.
    /// - **Codegen:** it [`reads`](Self::reads) the current element's id (so the
    ///   generated module can compute the gather row). The column rides through
    ///   the base like any unwritten column; nothing is written or consumed.
    ///
    /// The generated module gives the body `gather_row(i)` (the paired state row
    /// — `i` when the input is not a dense window, `current_id − prev_first`
    /// when it is) and `gather_paired(i)` (whether that row exists in the prior
    /// state, distinct from the global "is there any prior state?" —
    /// [[feedback_layered_defenses_need_per_layer_gates]]). Reading `prev_first`
    /// needs the SAME column bound on the state port too (a plain [`Self::Read`]).
    GatherKey,
    /// **A read from a TEMPLATE port whose length is decoupled from the dispatch**
    /// (ADR-0136, the `StreamOp::SourceRows` family). A count-changing node
    /// dispatches at its OUTPUT length (its `count_law`) while its template port
    /// carries the SOURCE length — so a plain [`Self::Read`], which is present
    /// only at the dispatch length, would judge the column ABSENT and hand back
    /// its identity at every element.
    ///
    /// The read index is **the kernel's own arithmetic**, not `i`: element `i`
    /// reads `read_<col>(row)` for a `row` the body computes (`motion.kaleidoscope`
    /// reads source row `i % window_src_n` — the slice-major fan-out). This is the
    /// same length-decouple the [`Self::GatherKey`] state ports have, without an
    /// id: the mapping is arithmetic, exactly like `sim.spawn`'s `cp_rows`.
    ///
    /// Present iff the port carries the column at ANY non-empty length; the
    /// generated `read_<col>` is a plain buffer read (the body supplies the
    /// index), so a SourceRead adds nothing the kernel could not already write —
    /// it only tells the sequencer the port is length-decoupled. Reads, never
    /// writes: the node's OUTPUT column is a separate [`Self::Write`] binding on
    /// the same port (`kaleidoscope` reads the source `P` and writes a transformed
    /// output `P` — two bindings, because they live in different buffers).
    SourceRead,
}

impl ColumnAccess {
    /// Does this access read the bound port's column?
    pub const fn reads(self) -> bool {
        matches!(
            self,
            ColumnAccess::Read
                | ColumnAccess::ReadBroadcast
                | ColumnAccess::ReadWrite
                | ColumnAccess::ReadWriteExisting
                | ColumnAccess::Consume
                // The gather key reads the current element's id.
                | ColumnAccess::GatherKey
                // A source-mapped read reads its template at the body's own index.
                | ColumnAccess::SourceRead
        )
    }

    /// Does this access write an output column (given whether the input
    /// stream carries it)?
    pub const fn writes(self, present_on_input: bool) -> bool {
        match self {
            ColumnAccess::Read
            | ColumnAccess::ReadBroadcast
            | ColumnAccess::Consume
            | ColumnAccess::RefuseIfPresent
            | ColumnAccess::GatherKey
            | ColumnAccess::SourceRead => false,
            ColumnAccess::Write | ColumnAccess::ReadWrite => true,
            ColumnAccess::ReadWriteExisting => present_on_input,
        }
    }

    /// Does the node's output stream **drop** this column (rather than let the
    /// base input's copy ride through)?
    pub const fn consumes(self) -> bool {
        matches!(self, ColumnAccess::Consume)
    }

    /// Does a length-1 port broadcast on this access ([`Self::ReadBroadcast`])?
    pub const fn broadcasts(self) -> bool {
        matches!(self, ColumnAccess::ReadBroadcast)
    }

    /// Is this binding a plan-time refusal rather than an access?
    pub const fn refuses(self) -> bool {
        matches!(self, ColumnAccess::RefuseIfPresent)
    }

    /// Is this binding the [`Self::GatherKey`] — a conditional refusal that
    /// claims when its port's stream is a dense id window (ADR-0130)?
    pub const fn is_gather_key(self) -> bool {
        matches!(self, ColumnAccess::GatherKey)
    }

    /// Is this a [`Self::SourceRead`] — a read from a length-decoupled template
    /// port (`StreamOp::SourceRows`), present at the port's own length?
    pub const fn is_source_read(self) -> bool {
        matches!(self, ColumnAccess::SourceRead)
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
