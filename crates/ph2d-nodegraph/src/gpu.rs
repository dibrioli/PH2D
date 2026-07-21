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
            | ColumnAccess::GatherKey => false,
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

pub use crate::stream_op_meta::{KEEP_FLAG_COL, ROWS_COL, StreamOp};

/// Everything a node's **count law** may look at. Dispatch size must be known
/// host-side, so this is evaluated on the CPU at cook time.
///
/// The three fields are the three things a count has ever been observed to
/// depend on in this graph, and each earned its place by a node that needs it:
/// [`Self::param`] (`motion.grid`: rows × cols), [`Self::playhead`]
/// (`motion.emitter`: the alive window `n(t)` grows and shrinks as particles are
/// born and die, ADR-0130), and [`Self::inputs`] (`value.lfo`: unconnected it is
/// ONE global oscillation, connected it is one per instance).
pub struct CountLawCtx<'a> {
    /// The element count of each input port, **in port order** — empty for a
    /// generator. The counts and not the streams: a law may only ask how WIDE
    /// its inputs are, never what is in them (that would be a readback, and the
    /// readback is measured-negative — see `ph2d-gpu-cook`'s `shape` module).
    pub inputs: &'a [u32],
    /// Resolved params, override-else-default — exactly like `EvalCtx::param`.
    pub param: &'a dyn Fn(&str) -> f32,
    /// The cook's playhead, in seconds.
    pub playhead: f64,
    /// The ROOT clock's step since the previous cook — **the same expression as
    /// the CPU's `EvalCtx::dt`** (`prev_playhead.map_or(0.0, |p| playhead - p)`),
    /// computed by the sequencer from its own last cooked playhead and restored
    /// through the scrub ring exactly like the CPU checkpoint restores
    /// `prev_playhead`. `0.0` on the first cook after a seed, which is what makes
    /// a birth law emit nothing on the tick it has no history for (`sim.spawn`'s
    /// documented seed behaviour). Earned its place by `sim.spawn`, whose count
    /// is `floor(rate·t) − floor(rate·(t−dt))` (ADR-0136).
    pub dt: f64,
}

/// **How many elements does this node emit?** — asked once per stage, answered
/// in one place.
///
/// `None` on [`GpuKernel::count_law`] means *"as many as port 0"*: the output
/// rides its base input, which is what every instance transformer wants and is
/// why the overwhelming majority of kernels declare nothing here.
///
/// A kernel declares a law when its CPU `eval` computes `n` from something
/// else — and it must declare **the same expression**, because the count is not
/// a detail of the dispatch: it is the length of the field the artist sees. Two
/// laws that disagree do not crash, they render a different number of things.
pub type CountLawFn = fn(&CountLawCtx<'_>) -> SourceWindow;

/// What a generator emits at a given playhead: how many elements, and — for a
/// generator whose elements carry an integer IDENTITY — where that identity
/// window begins.
///
/// It is a window and not a count because **the identity arithmetic belongs to
/// the CPU**. `motion.emitter`'s spawn index passes 2²⁴ (the exactly
/// representable `f32` integers) after `2²⁴ / rate` seconds — 23 minutes at
/// 12.000/s, and **four seconds** at the millions-per-second a multi-million
/// particle sim needs. Past it `floor(t·rate)` recomputed in the kernel starts
/// skipping integers, two particles land on one id, and the gather hands both
/// the same prior state. Silently.
///
/// So the kernel stops re-deriving the window and is TOLD it, in integers the
/// CPU computed in `f64`. Parity used to hold "by construction because both
/// sides read the same `f32`"; it now holds because there is only one number
/// and it is an integer. That is strictly fewer things that can disagree.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SourceWindow {
    /// Element count — what `SourceCountFn` used to be, alone.
    pub count: usize,
    /// The first element's spawn index, **wrapped into the exactly representable
    /// `f32` integer range** (`< 2²⁴`). The wrap is invisible to every consumer
    /// because identity is only ever read as a DIFFERENCE inside one window, and
    /// a window is orders of magnitude smaller than the wrap period.
    ///
    /// `0` for a generator whose elements have no spawn identity — `motion.grid`
    /// element `i` simply IS index `i`.
    pub first: u32,
    /// The OLDEST element's age, in seconds. Carried rather than recomputed
    /// because the kernel's version would be `t − first/rate`: a small answer
    /// asked of two large numbers, which loses its significant digits to
    /// cancellation well before the ids themselves go inexact.
    ///
    /// `0.0` for a generator without ages.
    pub age_first: f32,
}

impl SourceWindow {
    /// The window of a generator whose elements have no spawn identity and no
    /// age — a static grid. `count` alone, which is all `SourceCountFn` ever
    /// said.
    #[must_use]
    pub const fn of_count(count: usize) -> Self {
        Self {
            count,
            first: 0,
            age_first: 0.0,
        }
    }
}

/// The spawn-index wrap period: the largest power of two below which every
/// integer is exactly representable as `f32`. Identity is stored as `f32`
/// (the stream's only scalar column type), so this is not a policy — it is the
/// last integer the representation can hold without collapsing two of them
/// into one.
pub const ID_WRAP: u32 = 1 << 24;

/// Picks a kernel VARIANT from the node's resolved params — see
/// [`GpuKernel::variant_by_param`], which explains why the whole kernel varies
/// and not merely its binding set.
pub type VariantFn = fn(&dyn Fn(&str) -> f32) -> &'static GpuKernel;

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
    /// How wide this node's output is — see [`CountLawFn`]. `None` (the common
    /// case) = as wide as port 0.
    pub count_law: Option<CountLawFn>,
    /// **A kernel whose SHAPE depends on its params** delegates to one of several
    /// static variants; `None` (the common case) = this kernel is itself.
    ///
    /// `motion.drive` and `motion.oscillator` are why this exists: they write a
    /// *different column* per `channel` — `P` for X/Y, `rot` for Rotation, `size`
    /// for Size, `tint` for Opacity — and materialise the target from its identity
    /// when the stream lacks it. With one static shape they had two bad options:
    /// bind all four (and then a drive on X emits `rot`/`size`/`tint` **that the
    /// CPU's output does not carry** — a different stream SHAPE, not an ε) or
    /// claim only the channels writing one column, which is what both did, at ⅖
    /// and ½ of the node.
    ///
    /// **A whole kernel and not merely a binding set**, because the two cannot be
    /// separated: the generated module only defines `write_<col>` for columns that
    /// are BOUND, so a body written against `write_P` cannot serve the `rot`
    /// variant — and it should not want to, since one writes a `vec2` and the
    /// other an `f32`. Varying the bindings alone would produce a body calling an
    /// accessor that does not exist.
    ///
    /// Resolution is **one level deep** by construction: [`Self::resolve`] does
    /// not recurse, so a variant's own `variant_by_param` is never read (and
    /// should be `None`).
    pub variant_by_param: Option<VariantFn>,
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
        count_law: None,
        variant_by_param: None,
        applicable: None,
    };

    /// **The kernel this node actually runs**, given its resolved params — the one
    /// door for a question that used to be a field read.
    ///
    /// Every consumer must come through here rather than use `self` directly: the
    /// generated WGSL, the bind group it is dispatched against and the cache key
    /// they are stored under have to describe the SAME variant, and a site that
    /// skipped this would silently describe the default one.
    ///
    /// Does not recurse — see [`Self::variant_by_param`].
    pub fn resolve(&self, param: &dyn Fn(&str) -> f32) -> &GpuKernel {
        match self.variant_by_param {
            Some(f) => f(param),
            None => self,
        }
    }

    /// `true` for [`Self::PASSTHROUGH`]-shaped kernels (no body, no bindings).
    pub const fn is_passthrough(&self) -> bool {
        self.wgsl.is_empty() && self.bindings.is_empty()
    }
}

/// A node whose GPU kernel needs a **spatial neighbourhood grid** (ADR-0134 D2)
/// declares one, registered on the side like the kernel itself. Before the
/// node's kernel pass the sequencer bins `column` (a `vec2` position stream) on
/// input `port` into a uniform grid of cell size `param(cell_param)`, and the
/// generated module gains `grid_cell_of` / `grid_bucket_of` plus the
/// `grid_starts` / `grid_sorted` arrays, so the body can sweep the 3×3
/// neighbouring cells (cell = radius ⇒ the sweep covers exactly the
/// within-radius set the CPU all-pairs sees).
///
/// **Pure data** (ADR-0126): the grid BUILD lives in the sequencer — identical
/// for boids, `sim.collide` and SPH — while the node authors only its *query*
/// (how it uses the neighbours). Declaring a grid is append-only side metadata;
/// no kernel that lacks one changes in any way.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GridSpec {
    /// The stream column carrying the `vec2` positions the grid bins. It must
    /// ALSO be a readable binding of the kernel, so the body can read a
    /// neighbour's position (`read_<column>(j)`) to reject it by distance.
    pub column: &'static str,
    /// Which input port that column is read from — `0` for a per-element node,
    /// the state/`pre` port for a self-loop sim like boids.
    pub port: usize,
    /// The param whose value is the grid cell size — the perception radius, so a
    /// 3×3 cell sweep is exactly the within-radius neighbourhood.
    pub cell_param: &'static str,
    /// **A relaxation SOLVER sweeps more than once.** `None` = one dispatch (a
    /// simulation *step*, like boids: the tick already is the iteration).
    /// `Some(param)` = read that param and run the pass that many times, with the
    /// kernel's own output fed back as the next sweep's input **on [`Self::port`]**
    /// and the **grid REBUILT from the moved positions** each time.
    ///
    /// Rebuilding is not optional: a sweep moves the very column the grid indexes,
    /// so a grid built once would, by sweep 2, be answering *"who was near you
    /// before you moved?"* — the neighbour set would quietly go stale and the
    /// packing would differ from the CPU's by more than an ε.
    ///
    /// The feedback lands on `port` because the column being relaxed is the column
    /// being indexed — that is what makes it a neighbourhood relaxation rather than
    /// two unrelated facts. A kernel that wants to iterate over something it does
    /// NOT spatially index has no client yet, and inventing the contract for it
    /// here would be speculating (ADR-0134 D2's deferral, held).
    pub sweeps_param: Option<&'static str>,
}

/// A **simulation-zone** container on the GPU (ADR-0135). A `sim.zone` holds
/// no per-element kernel; it is a **conditional passthrough** — it forwards its
/// INIT port until it has state, then its STATE port. That is the device mirror
/// of the CPU zone's `if ctx.started() { input(state) } else { input(init) }`,
/// where "started" is whether the node fed a `pre` edge last tick (its
/// [`GpuStream`](crate) prev exists), exactly the CPU's `prev_outputs.contains`.
///
/// **Pure data** (ADR-0126), declared on the side like a [`GridSpec`]: the node
/// registers [`GpuKernel::PASSTHROUGH`] (so the plan claims it, and no compute
/// pass is emitted) plus this, and the sequencer does the select. Append-only —
/// no node that lacks one changes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StateSelect {
    /// The port carrying the seed population — forwarded before the loop has run.
    pub init_port: usize,
    /// The port carrying last tick's evolved state — forwarded once started.
    pub state_port: usize,
    /// Scratch columns the state must NOT carry across the loop. A force
    /// accumulates `accel` each tick and the integrator consumes it, so a zone
    /// that kept it would re-apply a stale acceleration forever; `falloff` is the
    /// same kind of per-tick mask. The CPU zone's `store()` strips exactly these.
    pub transients: &'static [&'static str],
}

/// Resolves a node type id to its registered GPU kernel — the side-channel
/// mirror of [`crate::cook::OpResolver`], implemented by the node registry.
/// Kept as a trait so the GPU sequencer is decoupled from the registry crate
/// exactly like the cook engine is.
pub trait KernelResolver {
    fn gpu_kernel(&self, ty: NodeTypeId) -> Option<&GpuKernel>;

    /// The neighbourhood grid this node's kernel needs, if any (ADR-0134 D2).
    /// Default `None` — a kernel opts in by registering a [`GridSpec`], exactly
    /// as it opts into a kernel at all; the 32 shipping kernels declare nothing
    /// and get nothing.
    fn grid(&self, _ty: NodeTypeId) -> Option<&GridSpec> {
        None
    }

    /// The state-loop select this node is (a `sim.zone`), if any (ADR-0135).
    /// Default `None` — a node opts in by registering a [`StateSelect`] alongside
    /// its [`GpuKernel::PASSTHROUGH`]; every other node is a plain passthrough or
    /// a compute kernel and gets nothing.
    fn state_select(&self, _ty: NodeTypeId) -> Option<&StateSelect> {
        None
    }

    /// The structural stream operation this node is (ADR-0136), if any. Default
    /// `None` — a node opts in by registering a [`StreamOp`] alongside its
    /// kernel, exactly like a [`GridSpec`]; every per-element node declares
    /// nothing and nothing about it changes.
    fn stream_op(&self, _ty: NodeTypeId) -> Option<&StreamOp> {
        None
    }

    /// Whether this node type KEEPS the **dense id window** (ADR-0130): a source
    /// that emits one (`motion.emitter`, whose alive set is a contiguous id
    /// range) or a per-element transformer that preserves it without reordering,
    /// filtering, duplicating or rewriting `id` (`motion.integrate`, `spring`,
    /// the forces). The plan needs this to know when the `id`-gather is
    /// arithmetic (`current_id − prev_first`) rather than a hash/sort.
    ///
    /// **Default false, and that direction is the whole point.** A node keeps the
    /// window only by DECLARING it (`register_dense_window`) — never by an
    /// allowlist of the nodes that break it. A new structural node (`sort`,
    /// `cull`) that forgets to declare defaults to *breaking* the window, so the
    /// plan safely recedes; the alternative (default-keeps, list the breakers)
    /// makes a forgotten breaker silently mispair the gather
    /// ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
    fn keeps_dense_window(&self, _ty: NodeTypeId) -> bool {
        false
    }
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
            count_law: None,
            variant_by_param: None,
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
        for a in [
            Read,
            Write,
            ReadWrite,
            ReadWriteExisting,
            RefuseIfPresent,
            GatherKey,
        ] {
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
        for a in [
            Read,
            Write,
            ReadWrite,
            ReadWriteExisting,
            Consume,
            GatherKey,
        ] {
            assert!(!a.refuses(), "{a:?} must not refuse the node");
        }
    }

    #[test]
    fn a_gather_key_reads_the_id_but_never_writes_consumes_or_refuses() {
        use ColumnAccess::*;
        // ADR-0130: the gather key is a REAL read (the current element's id, so
        // the module can compute the row) — but it is a conditional refusal, not
        // a plain refusal, so `refuses()` (the unconditional-recede path) must
        // NOT catch it (else it would recede on a dense window too).
        assert!(GatherKey.reads(), "reads the current element's id");
        assert!(!GatherKey.writes(true) && !GatherKey.writes(false));
        assert!(!GatherKey.consumes(), "the id rides through the base");
        assert!(
            !GatherKey.refuses(),
            "it CLAIMS a dense window — not a plain refusal"
        );
        assert!(GatherKey.is_gather_key());
        for a in [
            Read,
            Write,
            ReadWrite,
            ReadWriteExisting,
            Consume,
            RefuseIfPresent,
        ] {
            assert!(!a.is_gather_key(), "{a:?} is not the gather key");
        }
    }
}
