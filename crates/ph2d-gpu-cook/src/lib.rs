#![forbid(unsafe_code)]
//! `ph2d-gpu-cook` — the GPU-resident node cook (GPU/M5 **Fase 1**, ADR-0122).
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
//! [`plan`] walks upstream from the sink and claims the longest **suffix** of
//! the chain that can run on the GPU (kernel registered + applicable to the
//! node's params + single forward input + no `pre` edge + no driven params).
//! Whatever sits above that suffix is cooked by the ordinary CPU [`Cook`] —
//! with ALL of its semantics (memo, `pre` feedback, time scopes, driven
//! params) — and its output stream is uploaded ONCE at the seam
//! ([`stream::upload_stream`]). The boundary is a plan-time fact the caller
//! can see ([`GpuPlan::boundary`]), never a silent per-node copy: readback
//! never happens on this path (the anti-pattern of
//! [[project_painter_fluid_4k_perf_architecture]]).
//!
//! ## Determinism (ADR-0122 — do not reopen)
//!
//! The CPU `eval` is the CANONICAL path: the replay-hash, `cook_determinism`
//! and `transform_determinism` all run on it, and anything that needs a
//! canonical value reads the CPU. This engine is **performance/preview**,
//! reconciled against the CPU by ε-tolerance parity gates (float on a GPU is
//! not bit-reproducible cross-vendor). Kernels port the HR-5 polynomial
//! approximations (see `motion.oscillator`'s parabolic sine) so ε stays tiny.

pub mod codegen;
pub mod lower;
pub mod stream;

pub use stream::{BufferPool, GpuColumn, GpuStream};

use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::gpu::{GpuKernel, KernelResolver};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::{NodeManifest, NodeTypeId};
use std::collections::BTreeMap;

/// One GPU stage of a plan: a node whose kernel runs as a compute pass
/// (pass-through kernels are planned but dispatch nothing).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuStage {
    pub node: NodeId,
    pub ty: NodeTypeId,
}

/// The result of [`plan`]: the GPU-runnable suffix of a sink's chain.
#[derive(Clone, Debug, Default)]
pub struct GpuPlan {
    /// `Some((node, out_port))` — the chain continues above the suffix on the
    /// CPU: cook this node with the ordinary [`Cook`] and hand its output
    /// stream to [`GpuCook::cook`]. `None` — the suffix reaches a generator
    /// (or an unconnected input): the whole chain is GPU-resident.
    pub boundary: Option<(NodeId, usize)>,
    /// Stages in source→sink order (pass-throughs included; they emit no pass).
    pub stages: Vec<GpuStage>,
}

impl GpuPlan {
    /// `true` when the entire chain runs on the GPU (no CPU prefix).
    pub fn is_fully_gpu(&self) -> bool {
        self.boundary.is_none()
    }

    /// Number of stages that actually dispatch a compute pass (excludes
    /// pass-throughs) — what a "the optimization FIRES" gate should assert.
    pub fn dispatching_stages(&self, kernels: &dyn KernelResolver) -> usize {
        self.stages
            .iter()
            .filter(|s| kernels.gpu_kernel(s.ty).is_some_and(|k| !k.is_passthrough()))
            .count()
    }
}

/// Why a node broke the GPU suffix — plan-time diagnostics (the editor can
/// badge the boundary; the handoff's F1.2 wires that).
fn eligible(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    node: NodeId,
) -> bool {
    let Some(inst) = graph.node(node) else {
        return false;
    };
    let Some(op) = ops.resolve(inst.type_id()) else {
        return false;
    };
    let manifest = op.manifest();
    let Some(kernel) = kernels.gpu_kernel(inst.type_id()) else {
        return false;
    };
    // A driven param is a live wire into another subtree — the CPU cook
    // resolves those; a GPU stage has no lane for them (F1.2+).
    if graph.param_sources(node).is_some_and(|s| !s.is_empty()) {
        return false;
    }
    // Multi-input nodes (merge/lattice deform) are F2 territory.
    if manifest.inputs.len() > 1 {
        return false;
    }
    // A `pre` edge means sequential state over the outer tick — CPU-only here.
    if manifest.inputs.len() == 1
        && let Some((_, _, delayed)) = graph.input_edge(node, 0)
        && delayed
    {
        return false;
    }
    // A generator must say how many elements it emits (dispatch is host-sized).
    if manifest.inputs.is_empty() && kernel.source_count.is_none() && !kernel.is_passthrough() {
        return false;
    }
    // Kernel params must be declared manifest params and must not shadow the
    // built-in uniform fields the generated module owns.
    if !kernel.params.iter().all(|p| {
        *p != "count" && *p != "playhead" && manifest.param_default(p).is_some()
    }) {
        return false;
    }
    // Param-dependent coverage (e.g. the oscillator's X/Y-only kernel).
    match kernel.applicable {
        Some(f) => f(&|name| resolve_param(graph, node, manifest, name)),
        None => true,
    }
}

/// The node's live param value: per-instance override else manifest default —
/// the same override>default resolution as `EvalCtx::param` (driven params
/// were excluded by [`eligible`], so the wire tier cannot apply here).
fn resolve_param(graph: &Graph, node: NodeId, manifest: &NodeManifest, name: &str) -> f32 {
    graph
        .node_param_overrides(node)
        .and_then(|m| m.get(name).copied())
        .or_else(|| manifest.param_default(name))
        .unwrap_or(0.0)
}

/// Claim the longest GPU-runnable suffix of `sink`'s chain (see the crate
/// docs). Pure and cheap — a walk of the chain, no GPU objects — so calling
/// it every frame is fine; compiled pipelines are cached by [`GpuCook`].
pub fn plan(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    sink: NodeId,
) -> GpuPlan {
    let mut stages: Vec<GpuStage> = Vec::new();
    let mut cur = sink;
    let mut cur_port = 0usize;
    let boundary = loop {
        if !eligible(graph, ops, kernels, cur) {
            break Some((cur, cur_port));
        }
        let ty = graph.node(cur).expect("eligible checked").type_id();
        stages.push(GpuStage { node: cur, ty });
        let manifest = ops.resolve(ty).expect("eligible checked").manifest();
        if manifest.inputs.is_empty() {
            break None; // a generator roots the chain
        }
        match graph.input_edge(cur, 0) {
            // Unconnected input = the empty stream (the CPU cook's value for
            // it) — nothing above to run, the chain is fully resident.
            None => break None,
            Some((src, port, _forward)) => {
                cur = src;
                cur_port = port;
            }
        }
    };
    stages.reverse();
    GpuPlan { boundary, stages }
}

/// The GPU-resident instance output of a cook: a buffer laid out as
/// `[RenderInstance; len]`, usable directly as the sprite renderer's instance
/// vertex buffer (usage VERTEX) and mappable for the parity gates (COPY_SRC).
pub struct GpuInstances {
    buffer: wgpu::Buffer,
    len: u32,
    capacity: u32,
}

impl GpuInstances {
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn len(&self) -> u32 {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GpuCookError {
    /// `plan.boundary` and the `boundary_stream` argument disagree — the
    /// caller cooked (or skipped) the CPU prefix against a stale plan.
    BoundaryMismatch,
}

/// A compiled compute pipeline + the uniform buffer its dispatches write.
struct CachedPipeline {
    pipeline: wgpu::ComputePipeline,
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
}

/// Uniform slot size: count + playhead + up to 14 params, pow2-rounded.
const UNIFORM_BYTES: u64 = 64;

impl GpuCook {
    pub fn new() -> Self {
        Self::default()
    }

    /// The instance buffer the LAST [`Self::cook`] produced, if any — what the
    /// renderer binds. `None` before the first cook.
    pub fn instances(&self) -> Option<&GpuInstances> {
        self.instances.as_ref()
    }

    /// Run `plan` at `playhead`, producing the instance buffer. When the plan
    /// has a CPU boundary, `boundary_stream` is that node's freshly cooked
    /// output stream (cook it with the ordinary [`ph2d_nodegraph::cook::Cook`]
    /// — that keeps every CPU semantic canonical); `None` iff the plan is
    /// fully GPU. `default_uv_rect`/`default_size` are the CPU lowering's
    /// fallbacks, applied by the lowering kernel to absent columns. Returns
    /// the instance count. **One queue submit.**
    #[allow(clippy::too_many_arguments)] // the cook seam: graph + resolvers + plan + clock + defaults
    pub fn cook(
        &mut self,
        gpu: &GpuContext,
        graph: &Graph,
        ops: &dyn OpResolver,
        kernels: &dyn KernelResolver,
        plan: &GpuPlan,
        boundary_stream: Option<&Stream>,
        playhead: f64,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
    ) -> Result<u32, GpuCookError> {
        if plan.boundary.is_some() != boundary_stream.is_some() {
            return Err(GpuCookError::BoundaryMismatch);
        }
        self.pool.reclaim();

        let mut stream = match boundary_stream {
            Some(s) => stream::upload_stream(gpu, &mut self.pool, s),
            None => GpuStream::default(),
        };

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-gpu-cook chain"),
            });

        for (stage_idx, stage) in plan.stages.iter().enumerate() {
            let Some(kernel) = kernels.gpu_kernel(stage.ty) else {
                // The registry changed under a stale plan; treat as pass-through
                // rather than dispatch garbage — the next frame replans.
                continue;
            };
            if kernel.is_passthrough() {
                continue;
            }
            let manifest = ops
                .resolve(stage.ty)
                .expect("planned nodes resolve")
                .manifest();
            let count = match kernel.source_count {
                Some(f) => {
                    let get = |name: &str| resolve_param(graph, stage.node, manifest, name);
                    f(&get).min(u32::MAX as usize) as u32
                }
                None => stream.count,
            };
            if count == 0 {
                stream = GpuStream::default();
                continue;
            }
            self.encode_kernel_stage(
                gpu,
                &mut encoder,
                stage_idx,
                kernel,
                graph,
                stage.node,
                manifest,
                count,
                playhead,
                &mut stream,
            );
        }

        let count = stream.count;
        self.encode_lowering(
            gpu,
            &mut encoder,
            plan.stages.len(),
            &stream,
            default_uv_rect,
            default_size,
        );
        gpu.queue.submit(Some(encoder.finish()));
        // Drop the frame's streams so the pool can reclaim next cook.
        drop(stream);
        Ok(count)
    }

    /// Encode one kernel stage: resolve/compile the pipeline for the stream's
    /// column set, allocate output columns, write the uniform, dispatch, and
    /// advance `stream` to the node's output.
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
        stream: &mut GpuStream,
    ) {
        use codegen::BindingPlan;

        let ty_key = manifest.id.0;
        let sig = codegen::presence_signature(kernel, |b| stream.cols.contains_key(b.column));
        self.kernel_pipelines
            .entry((ty_key, sig))
            .or_insert_with(|| {
                let src = codegen::kernel_module(kernel, |b| stream.cols.contains_key(b.column));
                CachedPipeline {
                    pipeline: create_pipeline(gpu, &src, manifest.name),
                }
            });

        // Uniform: [count, playhead, params…] — the layout `kernel_module`
        // declared, zero-padded to the slot size.
        let mut uni = [0u8; UNIFORM_BYTES as usize];
        uni[0..4].copy_from_slice(&count.to_le_bytes());
        uni[4..8].copy_from_slice(&(playhead as f32).to_le_bytes());
        for (k, name) in kernel.params.iter().enumerate() {
            let v = resolve_param(graph, node, manifest, name);
            let at = 8 + k * 4;
            uni[at..at + 4].copy_from_slice(&v.to_le_bytes());
        }
        let uniform = self.uniform_slot(gpu, stage_idx);
        gpu.queue.write_buffer(uniform, 0, &uni);

        // Bind group: uniform, then read buffers, then fresh write buffers —
        // the exact order `codegen::kernel_module` assigned.
        let plans = codegen::plan_bindings(kernel, |b| stream.cols.contains_key(b.column));
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
                let col = stream.cols.get(b.column).expect("presence checked");
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

        // The node's output stream: written columns replace, the rest share.
        stream.count = count;
        for (name, col) in new_cols {
            stream.cols.insert(name, col);
        }
    }

    /// Encode the final lowering pass into the persistent instance buffer.
    fn encode_lowering(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        uniform_slot: usize,
        stream: &GpuStream,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
    ) {
        let count = stream.count;
        self.ensure_instance_capacity(gpu, count.max(1));
        let instances = self.instances.as_mut().expect("just ensured");
        instances.len = count;
        if count == 0 {
            return;
        }

        let present: [bool; 5] = std::array::from_fn(|i| {
            stream
                .cols
                .get(lower::LOWER_COLUMNS[i])
                .is_some_and(|c| c.dim == expected_lower_dim(i))
        });
        let sig = lower::lower_signature(present);
        self.lower_pipelines.entry(sig).or_insert_with(|| {
            let src = lower::lower_module(present);
            CachedPipeline {
                pipeline: create_pipeline(gpu, &src, "ph2d-gpu-cook lowering"),
            }
        });

        // Uniform: count, pad, default_size (vec2 @ 8), default_uv (vec4 @ 16).
        let mut uni = [0u8; 32];
        uni[0..4].copy_from_slice(&count.to_le_bytes());
        uni[8..12].copy_from_slice(&default_size[0].to_le_bytes());
        uni[12..16].copy_from_slice(&default_size[1].to_le_bytes());
        for (k, v) in default_uv_rect.iter().enumerate() {
            uni[16 + k * 4..20 + k * 4].copy_from_slice(&v.to_le_bytes());
        }
        let uniform = self.uniform_slot(gpu, uniform_slot);
        gpu.queue.write_buffer(uniform, 0, &uni);
        let uniform = &self.uniforms[uniform_slot];

        let instances = self.instances.as_ref().expect("ensured above");
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: instances.buffer.as_entire_binding(),
            },
        ];
        let mut slot = 2u32;
        for (i, name) in lower::LOWER_COLUMNS.iter().enumerate() {
            if present[i] {
                let col = stream.cols.get(*name).expect("presence checked");
                entries.push(wgpu::BindGroupEntry {
                    binding: slot,
                    resource: col.buffer.as_entire_binding(),
                });
                slot += 1;
            }
        }
        let pipeline = &self
            .lower_pipelines
            .get(&sig)
            .expect("inserted above")
            .pipeline;
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-gpu-cook lowering"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ph2d-gpu-cook lowering"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(count.div_ceil(codegen::WORKGROUP_SIZE), 1, 1);
    }

    /// The persistent uniform buffer for stage slot `idx` (created on demand).
    fn uniform_slot(&mut self, gpu: &GpuContext, idx: usize) -> &wgpu::Buffer {
        while self.uniforms.len() <= idx {
            self.uniforms
                .push(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ph2d-gpu-cook uniforms"),
                    size: UNIFORM_BYTES,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
        &self.uniforms[idx]
    }

    /// Grow (never shrink) the instance output to hold `count` instances —
    /// `InstanceBuffer`'s policy, plus STORAGE (the lowering writes it) and
    /// COPY_SRC (the parity gates read it back, deliberately off-path).
    fn ensure_instance_capacity(&mut self, gpu: &GpuContext, count: u32) {
        let needs_grow = match &self.instances {
            Some(gi) => gi.capacity < count,
            None => true,
        };
        if !needs_grow {
            return;
        }
        let mut capacity = self
            .instances
            .as_ref()
            .map(|gi| gi.capacity.max(1))
            .unwrap_or(1);
        while capacity < count {
            capacity = capacity.saturating_mul(2);
        }
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-gpu-cook instances"),
            size: u64::from(capacity) * u64::from(lower::INSTANCE_WORDS) * 4,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        self.instances = Some(GpuInstances {
            buffer,
            len: 0,
            capacity,
        });
    }
}

/// The dim each lowering column must have to be gathered (a column with the
/// wrong type is ignored, exactly like the CPU's typed `*_at` accessors).
fn expected_lower_dim(i: usize) -> ph2d_nodegraph::port::Dim {
    use ph2d_nodegraph::port::Dim;
    match i {
        0 | 1 => Dim::Vec2,  // P, size
        2 => Dim::Scalar,    // rot
        _ => Dim::Vec4,      // tint, uv_rect
    }
}

fn create_pipeline(gpu: &GpuContext, wgsl: &str, label: &str) -> wgpu::ComputePipeline {
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
/// crossing** for the parity gates and any future canonical need (ADR-0122:
/// if the replay ever wants a value, it comes from the CPU; this readback
/// exists to CHECK the GPU, not to feed the frame). Blocks on the GPU.
pub fn read_instances(gpu: &GpuContext, instances: &GpuInstances) -> Vec<ph2d_render::RenderInstance> {
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
