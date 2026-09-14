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

mod accessors;
pub mod codegen;
mod count;
pub mod debug_read;
mod encode;
pub mod error;
mod estado;
pub mod field_name;
mod gather;
pub mod grid;
pub mod instances;
mod lifecycle;
pub mod lower;
pub mod plan;
pub mod reduce;
// PUBLIC for one reason, written here so it is not "tidied" back: the naga sweep
// needs `reduce_stage::map_module` to parse+validate the `value` expression of
// every registered `ReduceSpec` WITHOUT a device. Before that gate existed those
// expressions only ever met a compiler on a machine with an adapter.
pub mod reduce_stage;
pub mod ring;
pub mod scan;
pub mod shape;
pub mod stream;
mod stream_op;
pub mod tap;
mod tex_runs;
pub mod voronoi;

pub use debug_read::read_instances;
pub use error::GpuCookError;
pub use instances::GpuInstances;
pub use plan::{DrivenParams, GpuPlan, GpuSource, GpuStage, plan, plan_driven};
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

mod cook_clock;
pub use cook_clock::CookClock;

/// A compiled compute pipeline + the uniform buffer its dispatches write.
pub(crate) struct CachedPipeline {
    pub(crate) pipeline: wgpu::ComputePipeline,
}

pub use encode::UNIFORM_BYTES;
pub use estado::GpuCook;

/// Ceiling on a relaxation solver's sweeps (`GridSpec::sweeps_param`). It is the
/// SAME number the CPU reference clamps to (`motion.collide::MAX_ITERATIONS`),
/// because a divergent cap is a divergent answer: the artist drags the slider to
/// 200, the CPU runs 64 and the device runs 200, and the parity gate — which
/// tests at the default — stays green while the product disagrees with itself.
pub(crate) const MAX_SWEEPS: i64 = 64;

impl GpuCook {
    /// ⭐⭐⭐ **Os valores dos params dirigidos deste quadro** (doc 110 §3) — postos pelo chamador,
    /// que tem o cozedor da CPU em mão, e lidos por cada `resolve_param` do sequenciador.
    ///
    /// ⚠️ **Tem de ser o MESMO mapa que a [`plan_driven`] viu.** O planeador decide quem é
    /// encenado a partir das CHAVES dele; o encode lê os VALORES. Dois mapas diferentes dariam um
    /// nó encenado com o número de outro quadro — e é por isso que o chamador os deriva uma vez e
    /// passa os dois do mesmo `let`.
    pub fn set_driven(&mut self, driven: DrivenParams) {
        self.driven = driven;
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
        // The sink's STYLE — lowering decisions the HOST owns, like the two
        // defaults above. It comes from the one door, `sink_style`; the why is
        // written there (this crate keeps `ph2d-eval-motion` a DEV dep on
        // purpose, so it cannot ask that door itself — daí o tipo viver no
        // `ph2d-render`, que é de quem os quatro campos são).
        style: ph2d_render::SinkStyle,
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
        // The LUT transients (A1-gpu), same window as the reductions.
        self.lut_hold.clear();
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
                count::No {
                    graph,
                    node: stage.node,
                    manifest,
                    driven: &self.driven,
                },
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
                .resolve(&|name| resolve_param(graph, stage.node, manifest, name, &self.driven))
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
            // A neighbourhood kernel (ADR-0140 D2) gets its grid built into this
            // same encoder BEFORE its pass — over the position column the spec
            // names, on the port the spec names (port 0 for a per-element node,
            // the `pre` state port for a self-loop sim).
            let grid_spec = kernels.grid(stage.ty);
            // ⚠️ The budget is what the MODULE declares, not what its COLUMNS do: grid,
            // reductions and LUTs bind buffers too (`codegen::storage_buffers`).
            let needed = codegen::storage_buffers(
                bindings,
                |b| column_present(gather_port, count, &inputs, b),
                codegen::ExtraBuffers {
                    grid: grid_spec,
                    reduces: kernels.reduces(stage.ty),
                    luts: kernels.luts(stage.ty),
                },
            );
            let limit = gpu.device.limits().max_storage_buffers_per_shader_stage;
            if needed > limit {
                return Err(GpuCookError::TooManyBindings(stage.ty, needed, limit));
            }
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
                .map(|p| resolve_param(graph, stage.node, manifest, p, &self.driven))
                .map_or(1i64, |v| (v.round() as i64).clamp(0, MAX_SWEEPS))
                as u32;
            // This stage's lookup tables (A1-gpu), filled from the node's text
            // params ONCE — the curve shape is a function of a text param,
            // invariant to the columns a sweep moves, so it is built OUTSIDE the
            // loop (unlike the grid and the reductions, which a sweep invalidates).
            let lut_specs = kernels.luts(stage.ty);
            let mut lut_buffers = self.build_luts(gpu, lut_specs, graph, stage.node);
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
                let shared = kernels.wgsl_shared(stage.ty);
                let reduce_results = self.run_reduces(
                    gpu,
                    &mut encoder,
                    reduce_specs,
                    &inputs,
                    graph,
                    stage.node,
                    manifest,
                    shared,
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
                    (lut_specs, &lut_buffers),
                    shared,
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
            // Keep this stage's LUT buffers alive until this cook's submit — the
            // bind group referenced them and the pass is not encoded yet. Built
            // once above, so held once here (not per sweep).
            self.lut_hold.append(&mut lut_buffers);
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
        // The texture-run partition, from the CPU boundary — no readback (see
        // [`tex_runs`]). Empty for a non-object graph.
        self.tex_runs.clear();
        tex_runs::texture_runs_from_boundary(boundary_streams, count, &mut self.tex_runs);
        // The instance buffer is the one binding that can outgrow the device's
        // storage-binding limit below the id ceiling (184 B × count; every
        // stream column caps at 16 B × ID_WRAP ≈ 268 MB). Refuse BEFORE the
        // bind group turns it into a validation panic.
        let instance_bytes =
            u64::from(count) * std::mem::size_of::<ph2d_render::RenderInstance>() as u64;
        // wgpu 29: `max_storage_buffer_binding_size` is already `u64`.
        let binding_limit = gpu.device.limits().max_storage_buffer_binding_size;
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
            style,
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
