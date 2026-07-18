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
pub mod debug_read;
mod gather;
pub mod instances;
pub mod lower;
pub mod plan;
pub mod ring;
pub mod shape;
pub mod stream;

pub use instances::GpuInstances;
pub use plan::{GpuPlan, GpuSource, GpuStage, plan};
pub use ring::GpuCheckpointRing;
pub use stream::{BufferPool, GpuColumn, GpuStream};

use crate::gather::{column_present, gather_key_port, gather_prev_n};
use crate::plan::resolve_param;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::gpu::{ColumnBinding, GpuKernel, KernelResolver};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::{NodeManifest, NodeTypeId};
use std::collections::{BTreeMap, BTreeSet};

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
}

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
    /// The fixed tick [`Self::prev`] belongs to — the GPU sim's own clock,
    /// mirroring `MotionCookPump::last_cooked_tick`. A sequential cook owes one
    /// step per tick, so the caller needs to know which one it last took; a
    /// stateless plan never reads this.
    last_tick: Option<u64>,
    /// The backwards-scrub ring (D5): past states, held by refcount, on the
    /// device. See [`ring`].
    ring: GpuCheckpointRing,
    /// A **live edit** invalidated the sim: the next [`Self::rewind_for`] seeds AT
    /// the tick it is asked for instead of anchoring at 0. See
    /// [`Self::reseed_from_next_tick`].
    reseed: bool,
}

/// Uniform slot size: count + playhead + up to 14 params, pow2-rounded.
pub(crate) const UNIFORM_BYTES: u64 = 64;

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
            self.ring.record(t, &self.prev);
        }
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
            let inputs: Vec<GpuStream> = stage
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
            let base = inputs.first().cloned().unwrap_or_default();

            let Some(kernel) = kernels.gpu_kernel(stage.ty) else {
                // The registry changed under a stale plan; treat as pass-through
                // rather than dispatch garbage — the next frame replans.
                streams.insert(stage.node, base);
                continue;
            };
            if kernel.is_passthrough() {
                streams.insert(stage.node, base);
                continue;
            }
            let manifest = ops
                .resolve(stage.ty)
                .expect("planned nodes resolve")
                .manifest();
            let count = match kernel.source_window {
                Some(f) => {
                    let get = |name: &str| resolve_param(graph, stage.node, manifest, name);
                    f(&get, clock.playhead).count.min(u32::MAX as usize) as u32
                }
                None => base.count,
            };
            if count == 0 {
                streams.insert(stage.node, GpuStream::default());
                continue;
            }
            // Refuse an over-limit layout BEFORE wgpu turns it into a panic.
            // The presence rule must be the gather-aware one (own-length state
            // ports, ADR-0130), or this counts a different set of buffers than
            // the module `encode_kernel_stage` actually binds.
            let gather_port = gather_key_port(kernel, &inputs, count);
            let needed = codegen::storage_bindings(kernel, |b| {
                column_present(gather_port, count, &inputs, b)
            });
            let limit = gpu.device.limits().max_storage_buffers_per_shader_stage;
            if needed > limit {
                return Err(GpuCookError::TooManyBindings(stage.ty, needed, limit));
            }
            let out = self.encode_kernel_stage(
                gpu,
                &mut encoder,
                stage_idx,
                kernel,
                graph,
                stage.node,
                manifest,
                count,
                playhead,
                &inputs,
                base,
            );
            streams.insert(stage.node, out);
        }

        // What the panel gets to know about a GPU frame (see `last_counts`): the
        // host-side element count of every staged node, recorded once the walk is
        // done. Cheap (a map of `u32`) and honest — these ARE the dispatch sizes.
        self.shape.record(&streams);
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

        // Drop the frame's streams so the pool can reclaim next cook.
        drop(streams);
        Ok(count)
    }

    /// Forget the simulation state AND its scrub history — the next cook seeds
    /// from scratch, exactly as at tick 0 (when the `pre` edge reads Empty).
    ///
    /// Call this when the **graph changes**: a cached state is a function of the
    /// graph that produced it, so an edit invalidates every checkpoint (the
    /// Blender/Houdini semantics the CPU ring already follows — its `clear` has
    /// the same trigger). It also releases the ring's pinned VRAM.
    pub fn forget_state(&mut self) {
        self.prev.clear();
        self.last_tick = None;
        self.ring.clear();
        self.reseed = false;
    }

    /// Drop the sim state and **restart it AT the next tick cooked**, rather than
    /// re-deriving the history the old parameters produced.
    ///
    /// This is the **live-edit** invalidation (ADR-0130 D7), and it exists because
    /// [`Self::forget_state`] is the wrong one for an edit in flight.
    /// `forget_state` means *"this state is invalid, re-derive it"* — and
    /// [`Self::rewind_for`] honours that by anchoring an empty ring at tick 0, so
    /// the caller re-cooks `0..=target`. For a discrete edit that is one honest
    /// bake (Blender/Houdini re-bake a sim when you edit it). For a param a user
    /// is **holding and dragging**, it is `O(current tick)` re-simulated EVERY
    /// FRAME — which is not a bake, it is a freeze (the smoke: *"re-bake travado"*).
    ///
    /// An artist dragging an emitter's `rate` is asking *"what does it look like
    /// with THIS rate?"*, not *"replay the last forty seconds with it"*. So the
    /// honest answer is a fountain that RESTARTS: seed at the tick on screen and
    /// step forward from there, `O(1)` per edit. The scrub ring is dropped too —
    /// its checkpoints are the old params' sim, and a scrub through them would
    /// show a history this document never had.
    pub fn reseed_from_next_tick(&mut self) {
        self.prev.clear();
        self.ring.clear();
        self.last_tick = None;
        self.reseed = true;
    }

    /// Rewind (if needed) so that cooking `first..=target` in order stands the
    /// sim at `target`; returns that first tick. Call BEFORE the march.
    ///
    /// Forward — the overwhelming case, one tick per frame — this is
    /// `last_tick + 1` and touches nothing. Backwards (a scrub), it restores the
    /// newest checkpoint at or before the target and hands back its tick to
    /// re-sim from: **GGPO save/load/advance, without leaving the device**
    /// (ADR-0127 D5). A target the ring no longer covers anchors at the tick-0
    /// seed and re-sims forward, which is slow but always right — and is the CPU
    /// ring's own answer to the same question.
    ///
    /// Without this, a backwards scrub would cook `target` against the state of
    /// a LATER tick: `dt` would clamp to zero (the integrator's guard) so nothing
    /// would explode — it would just quietly show the future's pose and call it
    /// the past.
    pub fn rewind_for(&mut self, target: u64) -> u64 {
        // A LIVE EDIT invalidated the sim (D7): seed AT the target — do NOT
        // re-derive the history the old params produced. One cook, not `target`
        // of them; see [`Self::reseed_from_next_tick`] for why that distinction
        // is the difference between a bake and a freeze.
        if std::mem::take(&mut self.reseed) {
            self.prev.clear();
            self.last_tick = None;
            return target;
        }
        match self.last_tick {
            Some(t) if target > t => t + 1,
            _ => {
                let (anchor, state) = self.ring.anchor_at_or_before(target);
                self.prev = state;
                self.last_tick = None;
                anchor
            }
        }
    }

    /// Retune the scrub ring's VRAM cap (default [`ring::RING_BYTES`]).
    pub fn set_ring_budget(&mut self, bytes: u64) {
        self.ring.set_budget(bytes);
    }

    /// Checkpoints the scrub ring holds, and the VRAM they pin (an upper bound).
    pub fn ring_stats(&self) -> (usize, u64) {
        (self.ring.len(), self.ring.bytes())
    }

    /// Encode one kernel stage: resolve/compile the pipeline for the inputs'
    /// column sets, allocate output columns, write the uniform, dispatch, and
    /// return the node's output stream.
    #[allow(clippy::too_many_arguments)] // private seam of `cook`
    fn encode_kernel_stage(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        stage_idx: usize,
        kernel: &GpuKernel,
        graph: &Graph,
        node: NodeId,
        manifest: &'static NodeManifest,
        count: u32,
        playhead: f64,
        inputs: &[GpuStream],
        base: GpuStream,
    ) -> GpuStream {
        use codegen::BindingPlan;

        // Is the binding's column readable off its port? The CPU's re-seed rule
        // (a state whose count no longer matches the live set was rebuilt, so
        // pair nothing — `motion.integrate`'s `pairing`: `_ if sn == n`),
        // expressed once and generally by [`column_present`]. Under an active
        // id-gather (ADR-0130) the state ports are length-decoupled (paired by
        // id, not position); everywhere else it is the dispatch-length test the
        // Fase 1/2 kernels already had.
        let gather_port = gather_key_port(kernel, inputs, count);
        let present = |b: &ColumnBinding| column_present(gather_port, count, inputs, b);
        let port_names: Vec<&str> = manifest.inputs.iter().map(|p| p.name).collect();

        let ty_key = manifest.id.0;
        let sig = codegen::presence_signature(kernel, present);
        self.kernel_pipelines
            .entry((ty_key, sig))
            .or_insert_with(|| {
                let src = codegen::kernel_module(kernel, &port_names, present);
                CachedPipeline {
                    pipeline: create_pipeline(gpu, &src, manifest.name),
                }
            });

        // Uniform: [count, playhead, params…, gather_prev_n?] — the layout
        // `kernel_module` declared, zero-padded to the slot size. `gather_prev_n`
        // is appended AFTER the params (matching the generated struct) only when
        // the gather is active (ADR-0130).
        let mut uni = [0u8; UNIFORM_BYTES as usize];
        uni[0..4].copy_from_slice(&count.to_le_bytes());
        uni[4..8].copy_from_slice(&(playhead as f32).to_le_bytes());
        for (k, name) in kernel.params.iter().enumerate() {
            let v = resolve_param(graph, node, manifest, name);
            let at = 8 + k * 4;
            uni[at..at + 4].copy_from_slice(&v.to_le_bytes());
        }
        if let Some(key_port) = gather_port {
            let at = 8 + kernel.params.len() * 4;
            uni[at..at + 4].copy_from_slice(&gather_prev_n(inputs, key_port).to_le_bytes());
        }
        // The generator's window, in the layout `kernel_module` declared. A node
        // is never both a gatherer and a generator (a generator has no input to
        // gather FROM), but the offset accounts for both so the layout is total
        // rather than true-by-luck.
        if let Some(win) = kernel.source_window {
            let w = win(&|name| resolve_param(graph, node, manifest, name), playhead);
            let at = 8 + kernel.params.len() * 4 + usize::from(gather_port.is_some()) * 4;
            uni[at..at + 4].copy_from_slice(&w.first.to_le_bytes());
            uni[at + 4..at + 8].copy_from_slice(&w.age_first.to_le_bytes());
        }
        let uniform = self.uniform_slot(gpu, stage_idx);
        gpu.queue.write_buffer(uniform, 0, &uni);

        // Bind group: uniform, then read buffers, then fresh write buffers —
        // the exact order `codegen::kernel_module` assigned.
        let plans = codegen::plan_bindings(kernel, present);
        let mut new_cols: Vec<(String, GpuColumn)> = Vec::new();
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
        // (entries borrow buffers; collect the write buffers first)
        let mut write_buffers: Vec<(usize, GpuColumn)> = Vec::new();
        for (i, (b, (_, write))) in kernel.bindings.iter().zip(&plans).enumerate() {
            if matches!(write, Some(BindingPlan::WriteBuffer)) {
                let bytes = u64::from(count) * stream::element_stride(b.dim);
                let buffer = self.pool.acquire(gpu, bytes);
                write_buffers.push((i, GpuColumn { buffer, dim: b.dim }));
            }
        }
        let uniform = &self.uniforms[stage_idx];
        entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        });
        let mut slot = 1u32;
        for (b, (read, _)) in kernel.bindings.iter().zip(&plans) {
            if matches!(read, Some(BindingPlan::ReadBuffer)) {
                let col = inputs[b.port].cols.get(b.column).expect("presence checked");
                entries.push(wgpu::BindGroupEntry {
                    binding: slot,
                    resource: col.buffer.as_entire_binding(),
                });
                slot += 1;
            }
        }
        for (i, col) in &write_buffers {
            entries.push(wgpu::BindGroupEntry {
                binding: slot,
                resource: col.buffer.as_entire_binding(),
            });
            slot += 1;
            new_cols.push((kernel.bindings[*i].column.to_string(), col.clone()));
        }
        let pipeline = &self
            .kernel_pipelines
            .get(&(ty_key, sig))
            .expect("inserted above")
            .pipeline;
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-gpu-cook stage"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(manifest.name),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(count.div_ceil(codegen::WORKGROUP_SIZE), 1, 1);
        }

        // The node's output stream: the base (port 0) rides through, consumed
        // columns are dropped, written ones replace. The CPU nodes are written
        // the same way — copy the primary input, rewrite my channels — so this
        // threading IS the shared semantic, not a GPU convention.
        //
        // **Unless the node emits a different KIND of stream.** `value.lfo` and
        // friends take instances and emit a VALUE stream: one `v` column and
        // nothing else. Riding the base there would hand downstream a VALUE stream
        // still carrying `P`/`Index`, which the CPU's does not have — not an ε, a
        // different shape, surfacing far away as a node reading a column that
        // should be gone. The manifest already states it (port 0 in vs out), so no
        // kernel declares anything, and it is a NO-OP for every kernel so far.
        let rides_base = match (manifest.inputs.first(), manifest.outputs.first()) {
            (Some(i), Some(o)) => o.ty == i.ty,
            _ => true,
        };
        let mut out = if rides_base {
            base
        } else {
            GpuStream::default()
        };
        out.count = count;
        for b in kernel.bindings.iter().filter(|b| b.access.consumes()) {
            out.cols.remove(b.column);
        }
        for (name, col) in new_cols {
            out.cols.insert(name, col);
        }
        out
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

/// Read the cooked instances back to the CPU — **the deliberate boundary
/// crossing** for the parity gates and any future canonical need (ADR-0126:
/// if the replay ever wants a value, it comes from the CPU; this readback
/// exists to CHECK the GPU, not to feed the frame). Blocks on the GPU.
pub fn read_instances(
    gpu: &GpuContext,
    instances: &GpuInstances,
) -> Vec<ph2d_render::RenderInstance> {
    let n = instances.len as usize;
    if n == 0 {
        return Vec::new();
    }
    let bytes = (n * std::mem::size_of::<ph2d_render::RenderInstance>()) as u64;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-gpu-cook readback"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_buffer_to_buffer(&instances.buffer, 0, &staging, 0, bytes);
    gpu.queue.submit(Some(encoder.finish()));
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv()
        .expect("map_async callback ran")
        .expect("readback map succeeded");
    let out: Vec<ph2d_render::RenderInstance> =
        bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
    staging.unmap();
    out
}
