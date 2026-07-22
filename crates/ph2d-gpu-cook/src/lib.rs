#![forbid(unsafe_code)]
//! `ph2d-gpu-cook` — the GPU-resident node cook (GPU/M5 **Fase 1**, ADR-0126).
//!
//! Takes a motion chain whose nodes registered WGSL kernels (the registry's
//! side channel, `register_gpu_kernel`) and runs it as a sequence of compute
//! passes in a **single submit**: stream columns live in storage buffers
//! ([`GpuStream`]), each kernel writes fresh buffers (implicit ping-pong),
//! and the final **lowering** pass gathers the columns straight into a buffer
//! laid out as [`ph2d_render::RenderInstance`] — which the sprite renderer
//! binds as its instance vertex buffer. **Zero readback on the hot path.**
//!
//! ## The plan and the explicit CPU↔GPU boundary
//!
//! [`plan`] walks upstream from the sink on **every** input port and claims the
//! part of the chain that can run on the GPU (kernel registered + applicable to
//! the node's params + no driven params + a derivable column shape). The result
//! is a **DAG** in topological order where each input names its source
//! ([`GpuSource`]) — ADR-0127 D2. Whatever it cannot claim is cooked by the
//! ordinary CPU [`Cook`] — with ALL of its semantics (memo, `pre` feedback,
//! time scopes, driven params) — and its output stream is uploaded ONCE at the
//! seam ([`stream::upload_stream`]). The boundaries are a plan-time fact the
//! caller can see ([`GpuPlan::boundaries`]), never a silent per-node copy:
//! readback never happens on this path (the anti-pattern of
//! [[project_painter_fluid_4k_perf_architecture]]).
//!
//! ## Simulation: the state is a column, so `pre` is a refcount
//!
//! A `pre` (delayed) edge is a **stop**, not a refusal: that input reads last
//! tick's output, which this engine already holds — a `GpuStream` is
//! `Arc<wgpu::Buffer>` columns, and `motion.integrate` keeps its state in
//! visible columns (`vel`/`sim_d`/`sim_t`), so "last tick's state" is literally
//! last tick's buffers ([`GpuCook::prev`], ADR-0127 D1). No readback, no copy,
//! no barrier. The loop must be claimed WHOLE, though: a CPU boundary inside it
//! would make the pump re-cook the sim with its own `prev`, and two simulations
//! of one state diverge — [`plan`] refuses that outright.
//!
//! ## Determinism (ADR-0126/0127 — do not reopen)
//!
//! The CPU `eval` is the CANONICAL path: the replay-hash, `cook_determinism`
//! and `transform_determinism` all run on it, and anything that needs a
//! canonical value reads the CPU. This engine is **performance/preview**,
//! reconciled against the CPU by ε-tolerance parity gates (float on a GPU is
//! not bit-reproducible cross-vendor). Kernels port the HR-5 polynomial
//! approximations (see `motion.oscillator`'s parabolic sine) so ε stays tiny.
//!
//! A **sequential** node changes what that means: `x_{n+1} = f(x_n)` feeds ε
//! back, so after N ticks the GPU and the CPU are different animations, and
//! that is not a bug (ADR-0127 D4). Hence a sim's parity gate asserts **one
//! step** from a seeded state — never a trajectory with ε loosened until it
//! passes.

pub mod codegen;
mod count;
pub mod debug_read;
mod encode;
pub mod error;
pub mod field_name;
mod gather;
pub mod grid;
pub mod instances;
mod lifecycle;
pub mod lower;
pub mod plan;
pub mod reduce;
mod reduce_stage;
pub mod ring;
pub mod scan;
pub mod shape;
pub mod stream;
mod stream_op;
pub mod tap;
pub mod voronoi;

pub use debug_read::read_instances;
pub use error::GpuCookError;
pub use instances::GpuInstances;
pub use plan::{GpuPlan, GpuSource, GpuStage, plan};
pub use ring::GpuCheckpointRing;
pub use stream::{BufferPool, GpuColumn, GpuStream};

use crate::gather::{column_present, gather_key_port};
use crate::plan::resolve_param;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::gpu::KernelResolver;
use ph2d_nodegraph::graph::{Graph, NodeId};
use std::collections::{BTreeMap, BTreeSet};

/// When a cook happens: the continuous `playhead` the kernels see, and the
/// fixed `tick` it stands on.
///
/// They are not redundant. The playhead is what a kernel reads (and what a sim
/// derives its own `dt` from — the state carries `sim_t`); the tick is the
/// SEQUENCE number, which is how the caller knows whether it is continuing this
/// sim or jumping. A stateless plan has no sequence to keep, hence `Option`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CookClock {
    pub playhead: f64,
    /// The fixed tick, for a plan that [`GpuPlan::drives_a_loop`]. `None` — a
    /// stateless cook (`f(params, playhead)`, F1.1/Fase 2): nothing to sequence.
    pub tick: Option<u64>,
}

impl CookClock {
    /// A stateless cook at `playhead` — the F1.1/Fase 2 shape.
    pub fn at(playhead: f64) -> Self {
        Self {
            playhead,
            tick: None,
        }
    }
}

/// A compiled compute pipeline + the uniform buffer its dispatches write.
pub(crate) struct CachedPipeline {
    pub(crate) pipeline: wgpu::ComputePipeline,
}

/// The sequencer. Owns the buffer pool, the pipeline caches and the
/// persistent instance output; reuse ONE across frames (like the CPU pump).
#[derive(Default)]
pub struct GpuCook {
    pool: BufferPool,
    /// Kernel pipelines keyed by `(node type, column-presence signature)`.
    kernel_pipelines: BTreeMap<(u64, u64), CachedPipeline>,
    /// Lowering pipelines keyed by the 5-column presence signature.
    lower_pipelines: BTreeMap<u64, CachedPipeline>,
    /// Per-stage uniform buffers (index = stage position; last = lowering).
    uniforms: Vec<wgpu::Buffer>,
    /// The persistent instance output (grow-only, like `InstanceBuffer`).
    instances: Option<GpuInstances>,
    /// **Last tick's output** of each node that feeds a `pre` edge — the GPU
    /// mirror of `Cook::prev_outputs`, populated at the end of every cook by
    /// the same rule as `Cook::advance_tick_scoped` (ADR-0127 D1).
    ///
    /// This IS the simulation state, and holding it costs a refcount: a
    /// `GpuStream` is `Arc<wgpu::Buffer>` columns, so "last tick's output" is
    /// literally the buffers the last tick wrote, and [`BufferPool::reclaim`]
    /// skips anything still referenced. No readback, no copy, no barrier —
    /// the ping-pong falls out of the fact that state was always a column.
    prev: BTreeMap<NodeId, GpuStream>,
    /// What the host knows about the last cook — element counts and column sets
    /// per staged node, for the graph panel (see [`shape::CookShape`]).
    shape: shape::CookShape,
    /// Gate-only stream retention ([`debug_read`]); off in production, where it
    /// would pin every intermediate against [`BufferPool::reclaim`].
    pub(crate) debug_retain: bool,
    pub(crate) debug_streams: BTreeMap<NodeId, GpuStream>,
    /// Last cook's output streams, held for the frame-path [`tap`]. **Cleared at
    /// the top of every cook, BEFORE [`BufferPool::reclaim`]** — holding a
    /// `GpuStream` is a refcount on its buffers, so a hold that outlived the
    /// frame would defeat the pool exactly like `debug_streams` does. Held only
    /// across the window in which the buffers are alive anyway, it costs nothing.
    ///
    /// Populated unconditionally rather than behind a flag: the tap is a
    /// *frame-path* facility (0,5% of a 60 fps frame, measured), and a flag would
    /// mean the panel's first frame after enabling it shows the previous cook.
    tap_streams: BTreeMap<NodeId, GpuStream>,
    /// The tap's compute pipeline, built on first use and reused. `Option` and
    /// not built in `new()` because `GpuCook` is `Default` and has no device.
    tap_pipeline: Option<tap::TapPipeline>,
    /// The fixed tick [`Self::prev`] belongs to — the GPU sim's own clock,
    /// mirroring `MotionCookPump::last_cooked_tick`. A sequential cook owes one
    /// step per tick, so the caller needs to know which one it last took; a
    /// stateless plan never reads this.
    last_tick: Option<u64>,
    /// The playhead of the last cook — the GPU mirror of `Cook::prev_playhead`,
    /// and computed into a count law's `dt` by **the same expression** the CPU's
    /// `EvalCtx::dt` uses (`map_or(0.0, |p| playhead - p)`), so a birth law
    /// (`sim.spawn`, ADR-0136) counts the same births on both sides. `None`
    /// after a seed (`dt = 0` — nothing is born on a tick with no history);
    /// restored through the scrub ring like the CPU checkpoint restores its
    /// `prev_playhead`.
    last_playhead: Option<f64>,
    /// The backwards-scrub ring (D5): past states, held by refcount, on the
    /// device. See [`ring`].
    ring: GpuCheckpointRing,
    /// A **live edit** invalidated the sim: the next [`Self::rewind_for`] seeds AT
    /// the tick it is asked for instead of anchoring at 0. See
    /// [`Self::reseed_from_next_tick`].
    reseed: bool,
    /// The spatial-grid service (ADR-0140 D2), built on first use like the tap
    /// pipeline — `Option` because `GpuCook` is `Default` and has no device.
    grid: Option<grid::Grid>,
    /// Transients of THIS cook's grid builds (scan scratch + cursors). Cleared at
    /// the top of every cook, like [`Self::tap_streams`]; wgpu keeps the buffers
    /// alive across the submit even as the next cook drops them.
    grid_scratch: grid::GridScratch,
    /// The grid output buffers (`starts`/`sorted`) a kernel pass binds, held for
    /// the same window — one per grid-bearing stage this cook.
    grid_hold: Vec<grid::GridBuffers>,
    /// The whole-stream reduction service (the deformer channel), built on first
    /// use like the grid — `Option` because `GpuCook` is `Default`.
    reduce: Option<reduce::Reduce>,
    /// The reduce map passes' N-sized scratch, held until this cook's submit —
    /// the same window as [`Self::grid_hold`], for the same reason. Pooled
    /// column buffers are `Arc`, the per-pass uniforms are owned, so both shapes
    /// are held (mirroring `stream_op_hold` / `stream_op_hold_bufs`).
    reduce_hold: Vec<wgpu::Buffer>,
    reduce_hold_bufs: Vec<std::sync::Arc<wgpu::Buffer>>,
    /// The 4-byte reduction results a kernel pass binds, held for the same
    /// window — one set per reducing stage this cook.
    reduce_results_hold: Vec<reduce_stage::ReduceResults>,
    /// The structural stream-op pipelines (ADR-0136), built on first use like
    /// the tap and the grid — `Option` because `GpuCook` is `Default`.
    stream_op_pipes: Option<stream_op::StreamOpPipes>,
    /// Stream-op **and algorithm** transients that must outlive THIS cook's
    /// final submit (the post-compaction gathers' uniforms and rows buffers;
    /// the voronoi passes' uniforms, ADR-0139). Cleared at the top of every
    /// cook, like [`Self::grid_hold`].
    stream_op_hold: Vec<wgpu::Buffer>,
    stream_op_hold_bufs: Vec<std::sync::Arc<wgpu::Buffer>>,
    /// The engine-algorithm pipelines (ADR-0139), built on first use like the
    /// stream ops — `Option` because `GpuCook` is `Default`.
    voronoi_pipes: Option<voronoi::VoronoiPipes>,
}

/// Uniform slot size, pow2-rounded: `count` + `playhead` + one `f32` per param,
/// then the conditional engine fields (`gather_prev_n`, the generator's window,
/// the broadcast mask — appended in that order, each after everything that can
/// precede it, so adding one never moves an existing offset).
///
/// 64 held 14 params and nothing else; a node with many params AND a conditional
/// field would have run off the end, writing a param into the next field's bytes
/// and reading as plausible garbage. This is a slot, not an allocation per
/// element — the headroom is free.
///
/// **`pub` for the budget gate** (the shell's `motion_gpu_kernel_budgets`): the
/// packer (`encode_kernel_stage`) writes by offset arithmetic into a slice of
/// exactly this size, so a registered kernel whose declared layout exceeds it
/// PANICS at first dispatch in production — the gate refuses it at `cargo test`
/// instead, over every kernel the registry actually carries.
pub const UNIFORM_BYTES: u64 = 128;

/// Ceiling on a relaxation solver's sweeps (`GridSpec::sweeps_param`). It is the
/// SAME number the CPU reference clamps to (`motion.collide::MAX_ITERATIONS`),
/// because a divergent cap is a divergent answer: the artist drags the slider to
/// 200, the CPU runs 64 and the device runs 200, and the parity gate — which
/// tests at the default — stays green while the product disagrees with itself.
pub(crate) const MAX_SWEEPS: i64 = 64;

impl GpuCook {
    pub fn new() -> Self {
        Self::default()
    }

    /// The instance buffer the LAST [`Self::cook`] produced, if any — what the
    /// renderer binds. `None` before the first cook.
    pub fn instances(&self) -> Option<&GpuInstances> {
        self.instances.as_ref()
    }

    /// What the last [`Self::cook`] produced, per staged node — the graph panel's
    /// only window into a GPU-resident frame. See [`shape::CookShape`].
    pub fn shape(&self) -> &shape::CookShape {
        &self.shape
    }

    /// How many elements `node` carried on the last [`Self::cook`].
    pub fn node_count(&self, node: NodeId) -> Option<u32> {
        self.shape.count(node)
    }

    /// The column names `node`'s output carried on the last [`Self::cook`].
    pub fn node_columns(&self, node: NodeId) -> Option<&[String]> {
        self.shape.columns(node)
    }

    /// The fixed tick this sim's state ([`Self::prev`]) belongs to — the GPU
    /// mirror of `MotionCookPump::last_cooked_tick`, and the caller's input for
    /// "how many ticks do I owe?". `None` before the first sequential cook.
    ///
    /// A sequential trajectory is the SUM of its steps, so a caller must cook
    /// EVERY owed tick rather than one big jump — the same law the CPU pump
    /// states (`ticks_owed`: "forward: every tick, never a skip"), for the same
    /// reason: otherwise the motion depends on the frame rate.
    pub fn last_cooked_tick(&self) -> Option<u64> {
        self.last_tick
    }

    /// Column buffers the pool has ever created — flat across a steady scene,
    /// which is the whole claim of the ping-pong (D1) and is otherwise
    /// unobservable from outside.
    pub fn pool_allocations(&self) -> usize {
        self.pool.allocations()
    }

    /// Column buffers something still holds — for a sim, last tick's state.
    pub fn pool_retained(&self) -> usize {
        self.pool.retained()
    }

    /// Run `plan` at `clock`, producing the instance buffer. Each entry of
    /// `boundary_streams` is a [`GpuPlan::boundaries`] node's freshly cooked
    /// output stream (cook them with the ordinary
    /// [`ph2d_nodegraph::cook::Cook`] — that keeps every CPU semantic
    /// canonical); the set must match the plan's exactly, and is empty iff the
    /// plan is fully GPU. `default_uv_rect`/`default_size` are the CPU
    /// lowering's fallbacks, applied by the lowering kernel to absent columns.
    /// Returns the instance count. **One queue submit.**
    #[allow(clippy::too_many_arguments)] // the cook seam: graph + resolvers + plan + clock + defaults
    pub fn cook(
        &mut self,
        gpu: &GpuContext,
        graph: &Graph,
        ops: &dyn OpResolver,
        kernels: &dyn KernelResolver,
        plan: &GpuPlan,
        boundary_streams: &[(NodeId, &Stream)],
        clock: CookClock,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
    ) -> Result<u32, GpuCookError> {
        let CookClock { playhead, tick } = clock;
        let want: BTreeSet<NodeId> = plan.boundaries.iter().map(|(n, _)| *n).collect();
        let got: BTreeSet<NodeId> = boundary_streams.iter().map(|(n, _)| *n).collect();
        if want != got {
            return Err(GpuCookError::BoundaryMismatch);
        }
        // D5 — the scrub ring: `prev` right now IS the state this tick cooks
        // from, so record it BEFORE the cook overwrites it. Free in time (a map
        // of refcounts); the cost is the VRAM it pins, which is what the ring
        // caps. A stateless plan has no `tick` and records nothing.
        if let Some(t) = tick
            && plan.drives_a_loop()
            && self.ring.should_record(t)
        {
            self.ring.record(t, &self.prev, self.last_playhead);
        }
        // The ROOT clock's step — the same expression as the CPU's `EvalCtx::dt`
        // (`prev_playhead.map_or(0.0, |p| playhead - p)`), for the count laws
        // that need one (`sim.spawn`, ADR-0136).
        let dt = self.last_playhead.map_or(0.0, |p| playhead - p);
        // Drop the previous frame's tap hold BEFORE reclaiming, or its refcount
        // would keep every intermediate out of the pool for one extra frame.
        self.tap_streams.clear();
        // This cook's grid transients (ADR-0140): dropped like the tap hold, so
        // the buffers return once the prior submit that used them has completed.
        self.grid_hold.clear();
        self.grid_scratch = grid::GridScratch::default();
        // The reduce transients (the deformer channel), same window as the grid's.
        self.reduce_hold.clear();
        self.reduce_hold_bufs.clear();
        self.reduce_results_hold.clear();
        // The stream-op transients (ADR-0136), same window.
        self.stream_op_hold.clear();
        self.stream_op_hold_bufs.clear();
        self.pool.reclaim();

        // The CPU→GPU crossings: one upload per boundary node, before anything
        // is encoded (a node consumed twice uploads once).
        let mut uploaded: BTreeMap<NodeId, GpuStream> = BTreeMap::new();
        for (node, s) in boundary_streams {
            uploaded.insert(*node, stream::upload_stream(gpu, &mut self.pool, s));
        }

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-gpu-cook chain"),
            });

        // Every claimed node's output, threaded in topological order. The plan
        // named each input's source, so this is a lookup, never a search.
        let mut streams: BTreeMap<NodeId, GpuStream> = BTreeMap::new();
        for (stage_idx, stage) in plan.stages.iter().enumerate() {
            let mut inputs: Vec<GpuStream> = stage
                .inputs
                .iter()
                .map(|src| match src {
                    GpuSource::Stage(n) => streams.get(n).cloned().unwrap_or_default(),
                    GpuSource::Boundary(n, _) => uploaded.get(n).cloned().unwrap_or_default(),
                    // Tick 0 (or a re-plan): no state yet — the empty stream,
                    // which is exactly what the CPU's `pre` reads then, and what
                    // makes a kernel take its seed path.
                    GpuSource::Prev(n) => self.prev.get(n).cloned().unwrap_or_default(),
                    GpuSource::Empty => GpuStream::default(),
                })
                .collect();
            // Port 0 is the base the output rides on (`ColumnBinding::port`).
            let mut base = inputs.first().cloned().unwrap_or_default();

            // A `sim.zone` is a conditional passthrough (ADR-0135): forward the
            // INIT port until the loop has state, the STATE port after, stripping
            // the transients the state must not carry. "Started" is whether last
            // tick populated this node's `prev` — it feeds a `pre` edge, so after
            // tick 0 it always has — mirroring the CPU zone's `ctx.started()`
            // (`prev_outputs.contains_key`). Intercepted BEFORE the passthrough
            // branch, which would forward port 0 (init) unconditionally.
            if let Some(sel) = kernels.state_select(stage.ty) {
                let started = self.prev.contains_key(&stage.node);
                let port = if started {
                    sel.state_port
                } else {
                    sel.init_port
                };
                let mut out = inputs.get(port).cloned().unwrap_or_default();
                for t in sel.transients {
                    out.cols.remove(*t);
                }
                streams.insert(stage.node, out);
                continue;
            }

            let Some(kernel) = kernels.gpu_kernel(stage.ty) else {
                // The registry changed under a stale plan; treat as pass-through
                // rather than dispatch garbage — the next frame replans.
                streams.insert(stage.node, base);
                continue;
            };
            let manifest = ops
                .resolve(stage.ty)
                .expect("planned nodes resolve")
                .manifest();
            // A multi-pass engine ALGORITHM (ADR-0139), intercepted like the
            // stream ops and before the passthrough branch — the node registers
            // PASSTHROUGH so the plan claims it, and that branch would forward
            // port 0 (`motion.voronoi`'s relax VALUE) instead of the cloud.
            if let Some(alg) = kernels.algorithm(stage.ty) {
                let out = self.encode_algorithm(
                    gpu,
                    &mut encoder,
                    alg,
                    graph,
                    stage.node,
                    manifest,
                    &inputs,
                );
                streams.insert(stage.node, out);
                continue;
            }
            // The structural stream ops (ADR-0136), intercepted BEFORE the
            // passthrough branch — a Concat/Project registers PASSTHROUGH, and
            // that branch would forward port 0 instead.
            let mut source_port: Option<usize> = None;
            match kernels.stream_op(stage.ty) {
                Some(ph2d_nodegraph::gpu::StreamOp::Concat { ports }) => {
                    let out = self.encode_concat(gpu, &mut encoder, ports, &inputs);
                    streams.insert(stage.node, out);
                    continue;
                }
                Some(ph2d_nodegraph::gpu::StreamOp::Project {
                    text_param,
                    mode_param,
                }) => {
                    let out = self.encode_project(
                        gpu,
                        &mut encoder,
                        graph,
                        stage.node,
                        manifest,
                        text_param,
                        mode_param,
                        &inputs,
                    );
                    streams.insert(stage.node, out);
                    continue;
                }
                Some(ph2d_nodegraph::gpu::StreamOp::Compact { port, predicate }) => {
                    // Filter the port's stream BEFORE the node's own kernel — the
                    // kernel (`sim.lifetime`'s `life` writer) runs on survivors.
                    // The predicate gets its own uniform slot, disjoint from the
                    // stage range and the lowering's (`plan.stages.len()`).
                    let compacted = self.encode_compact(
                        gpu,
                        &mut encoder,
                        plan.stages.len() + 1 + stage_idx,
                        predicate,
                        graph,
                        stage.node,
                        manifest,
                        playhead,
                        &inputs,
                        *port,
                    )?;
                    if *port < inputs.len() {
                        inputs[*port] = compacted.clone();
                    }
                    if *port == 0 {
                        base = compacted;
                    }
                }
                Some(ph2d_nodegraph::gpu::StreamOp::SourceRows { port }) => {
                    // The kernel writes ROWS_COL + its own columns on a FRESH
                    // base — riding the template would hand the output the
                    // template's other columns un-gathered, at template length.
                    base = GpuStream::default();
                    source_port = Some(*port);
                }
                None => {}
            }
            if kernel.is_passthrough() {
                streams.insert(stage.node, base);
                continue;
            }
            let window = count::stage_window(
                kernel,
                graph,
                stage.node,
                manifest,
                &inputs,
                clock.playhead,
                dt,
            );
            let count = window.count.min(u32::MAX as usize) as u32;
            if count == 0 {
                streams.insert(stage.node, GpuStream::default());
                continue;
            }
            // Refuse an over-limit layout BEFORE wgpu turns it into a panic.
            // The presence rule must be the gather-aware one (own-length state
            // ports, ADR-0130), or this counts a different set of buffers than
            // the module `encode_kernel_stage` actually binds.
            // The limit must be counted against the variant this dispatch will
            // actually run, which for a param-dependent kernel is not `kernel`
            // itself (`GpuKernel::resolve`).
            let bindings = kernel
                .resolve(&|name| resolve_param(graph, stage.node, manifest, name))
                .bindings;
            let gather_port = gather_key_port(bindings, &inputs, count);
            // A broadcast port at a length the dispatch cannot pair (neither
            // per-element nor row-0) would be judged absent and read identity at
            // EVERY index while the CPU reads its real rows — a shape divergence.
            // Refuse the frame; the caller recedes to the canonical CPU (the same
            // door as the binding limit below).
            if let Some((port, len)) =
                gather::broadcast_length_mismatch(gather_port, count, bindings, |b| {
                    inputs
                        .get(b.port)
                        .map(|s| (s.count, s.cols.contains_key(b.column)))
                })
            {
                return Err(GpuCookError::BroadcastLengthMismatch {
                    ty: stage.ty,
                    port,
                    len,
                    count,
                });
            }
            let needed = codegen::storage_bindings(bindings, |b| {
                column_present(gather_port, count, &inputs, b)
            });
            let limit = gpu.device.limits().max_storage_buffers_per_shader_stage;
            if needed > limit {
                return Err(GpuCookError::TooManyBindings(stage.ty, needed, limit));
            }
            // A neighbourhood kernel (ADR-0140 D2) gets its grid built into this
            // same encoder BEFORE its pass — over the position column the spec
            // names, on the port the spec names (port 0 for a per-element node,
            // the `pre` state port for a self-loop sim).
            let grid_spec = kernels.grid(stage.ty);
            // **How many sweeps?** A simulation STEP dispatches once (the tick is
            // the iteration); a relaxation SOLVER runs its `iterations` param
            // (`GridSpec::sweeps_param`, ADR-0140 Fase 5). Clamped to at least one
            // so a zero/negative param is the identity dispatch, never a skipped
            // stage that would leave the node's output undefined.
            // ⚠️ The rounding and the clamp are the CPU's, to the letter
            // (`motion.collide`: `round() as i64).clamp(0, MAX_ITERATIONS)`) —
            // including that **zero sweeps is the IDENTITY**, not a skipped stage.
            // `out` therefore starts as the base, so a zero-iteration node emits
            // its input unchanged exactly as the reference does.
            let sweeps = grid_spec
                .and_then(|s| s.sweeps_param)
                .map(|p| resolve_param(graph, stage.node, manifest, p))
                .map_or(1i64, |v| (v.round() as i64).clamp(0, MAX_SWEEPS))
                as u32;
            let mut out = base.clone();
            for _ in 0..sweeps {
                // The grid is rebuilt from the CURRENT positions every sweep — a
                // sweep moves the column the grid indexes, so a grid built once
                // would answer "who was near you BEFORE you moved" (see
                // `GridSpec::sweeps_param`).
                let grid_buffers = grid_spec.map(|spec| {
                    self.build_grid(
                        gpu,
                        &mut encoder,
                        spec,
                        &inputs,
                        graph,
                        stage.node,
                        manifest,
                    )
                });
                // The node's declared whole-stream reductions (the deformer
                // channel), folded into this same encoder BEFORE its kernel pass.
                //
                // **Inside the sweep loop, next to the grid rebuild, and for the
                // same reason**: a sweep moves the very column a reduction reads,
                // so a fold hoisted out would answer "how wide was the layout
                // BEFORE you deformed it?" from sweep 2 onward. Reductions are
                // per-element-cheap and today's clients run one sweep, so this
                // costs nothing and cannot go stale.
                let reduce_specs = kernels.reduces(stage.ty);
                let reduce_results = self.run_reduces(
                    gpu,
                    &mut encoder,
                    reduce_specs,
                    &inputs,
                    graph,
                    stage.node,
                    manifest,
                );
                out = self.encode_kernel_stage(
                    gpu,
                    &mut encoder,
                    stage_idx,
                    0,
                    kernel,
                    graph,
                    stage.node,
                    manifest,
                    window,
                    playhead,
                    &inputs,
                    base.clone(),
                    grid_spec.zip(grid_buffers.as_ref()),
                    (reduce_specs, &reduce_results.buffers),
                );
                self.reduce_results_hold.push(reduce_results);
                if let Some(gb) = grid_buffers {
                    self.grid_hold.push(gb);
                }
                // Feed this sweep's result into the next one, on the port the grid
                // indexes. `base` follows it when that port is the output's base
                // (port 0), so the pass-through columns ride the fresh stream
                // instead of the stale one.
                if sweeps > 1
                    && let Some(port) = grid_spec.map(|s| s.port)
                {
                    if port < inputs.len() {
                        inputs[port] = out.clone();
                    }
                    if port == 0 {
                        base = out.clone();
                    }
                }
            }
            // A SourceRows kernel wrote its rows; gather the template's other
            // columns at them so the newborns inherit the whole vocabulary
            // (ADR-0136 — the CPU's `newborns` copies every column but `id`).
            if let Some(p) = source_port {
                out = self.encode_source_gather(gpu, &mut encoder, out, inputs.get(p), count);
            }
            streams.insert(stage.node, out);
        }

        // What the panel gets to know about a GPU frame (see `last_counts`): the
        // host-side element count of every staged node, recorded once the walk is
        // done. Cheap (a map of `u32`) and honest — these ARE the dispatch sizes.
        self.shape.record(&streams);
        self.tap_streams = streams.clone();
        if self.debug_retain {
            self.debug_streams = streams.clone();
        }

        // The sink is the walk's post-order root, so it is the last stage.
        let sink_stream = plan
            .stages
            .last()
            .and_then(|s| streams.get(&s.node))
            .cloned()
            .unwrap_or_default();
        let count = sink_stream.count;
        // The instance buffer is the one binding that can outgrow the device's
        // storage-binding limit below the id ceiling (184 B × count; every
        // stream column caps at 16 B × ID_WRAP ≈ 268 MB). Refuse BEFORE the
        // bind group turns it into a validation panic.
        let instance_bytes =
            u64::from(count) * std::mem::size_of::<ph2d_render::RenderInstance>() as u64;
        let binding_limit = u64::from(gpu.device.limits().max_storage_buffer_binding_size);
        if instance_bytes > binding_limit {
            return Err(GpuCookError::BindingTooLarge {
                bytes: instance_bytes,
                limit: binding_limit,
            });
        }
        self.encode_lowering(
            gpu,
            &mut encoder,
            plan.stages.len(),
            &sink_stream,
            default_uv_rect,
            default_size,
        );
        gpu.queue.submit(Some(encoder.finish()));

        // D1 — the ping-pong: hold the `Arc`s of every node a `pre` edge reads,
        // by the SAME rule the CPU pump uses (`Cook::advance_tick_scoped`
        // snapshots exactly the sources of delayed edges). Assigning drops last
        // tick's streams, so their buffers return to the pool on the next
        // `reclaim` — the state is double-buffered and nothing else is.
        let pre_sources: BTreeSet<NodeId> = graph
            .edges()
            .iter()
            .filter(|e| e.delayed)
            .map(|e| e.from.0)
            .collect();
        self.prev = streams
            .iter()
            .filter(|(node, _)| pre_sources.contains(node))
            .map(|(node, s)| (*node, s.clone()))
            .collect();
        self.last_tick = tick;
        self.last_playhead = Some(playhead);

        // Drop the frame's streams so the pool can reclaim next cook.
        drop(streams);
        Ok(count)
    }
}

pub(crate) fn create_pipeline(gpu: &GpuContext, wgsl: &str, label: &str) -> wgpu::ComputePipeline {
    let module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });
    gpu.device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
}
