//! **Running a node's declared whole-stream reductions** before its kernel pass
//! — the deformer channel's build, the sibling of [`crate::grid`]'s.
//!
//! A [`ReduceSpec`] names a column, an operator, and a WGSL expression turning
//! one column element into the `f32` to fold. This runs that in two passes:
//!
//! ```text
//!     map: column[i] -> scratch[i]   (one dispatch, the spec's `value`)
//!     fold: scratch  -> result[0]    (crate::reduce, unchanged)
//! ```
//!
//! ⚠️ **Two passes, on purpose.** The `value` expression could be folded into the
//! reduction's first level, saving one dispatch and one N-sized scratch buffer.
//! It is not, because [`crate::reduce`] is *gated* — bit-exact against a CPU
//! oracle across every block seam and recursion depth — and templating a
//! caller's expression into it would fork the proven primitive into one variant
//! per node. The map pass is O(N) with trivial arithmetic, the same order as the
//! fold's own first level, so the cost is a constant factor on a pass that is
//! already not the frame's bottleneck. If it ever measures, fusing is a local
//! change behind this same door.

use crate::plan::resolve_param;
use crate::reduce::{Reduce, ReduceScratch};
use crate::{CachedPipeline, GpuCook, GpuStream, UNIFORM_BYTES, codegen, create_pipeline};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::gpu::ReduceSpec;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;

/// The pipeline-cache key for a spec's map module.
///
/// **Derived from the spec's CONTENT**, not from the node type: the module is a
/// pure function of exactly these fields, so two specs that agree on them can
/// share a pipeline and two that differ anywhere cannot — the same discipline as
/// `codegen::presence_signature`, and the reason a node's `k`-th spec is not
/// simply keyed by `(type, k)` (editing a spec's expression would then silently
/// reuse the pipeline compiled from the old one).
fn map_cache_key(spec: &ReduceSpec, present: bool) -> (u64, u64) {
    let fnv = |h: u64, bytes: &[u8]| {
        bytes
            .iter()
            .fold(h, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3))
    };
    let mut h = fnv(0xcbf2_9ce4_8422_2325, spec.value.as_bytes());
    h = fnv(h, spec.name.as_bytes());
    h = fnv(h, spec.column.as_bytes());
    h = fnv(h, &[spec.dim as u8, spec.op as u8, u8::from(present)]);
    for p in spec.params {
        h = fnv(h, p.as_bytes());
    }
    // A salt disjoint from every node type id, so a map module can never collide
    // with a kernel's `(ty_key, sig)` entry in the shared cache.
    (h, REDUCE_MAP_SALT)
}

/// Marks a cache entry as a reduce **map** module rather than a node kernel.
const REDUCE_MAP_SALT: u64 = 0x7265_6475_6365_6d70; // "reducemp"

/// The 4-byte result buffers of one stage's reductions, in spec order — what the
/// kernel pass binds and the body reads through `reduce_<name>()`.
#[derive(Default)]
pub struct ReduceResults {
    pub(crate) buffers: Vec<wgpu::Buffer>,
    /// The map-pass scratch, held until the caller submits (the same contract
    /// [`ReduceScratch`] has, for the same reason: one submit per cook).
    pub(crate) scratch: Vec<ReduceScratch>,
}

/// The WGSL module for a spec's **map** pass: read the column, apply `value`,
/// write one `f32`. A pure function of `(spec, present)`, so it caches like a
/// kernel.
///
/// `present = false` is the absent-column form: the source binding stays (one
/// bind-group shape, one code path) but is never read — every element folds
/// [`ReduceSpec::identity`], which is exactly what the CPU reduces over when it
/// materialises the missing column. Cheaper and more honest than uploading `n`
/// copies of a constant.
fn map_module(spec: &ReduceSpec, present: bool) -> String {
    let ty = codegen::wgsl_type(spec.dim);
    let mut src = String::with_capacity(512);
    src.push_str("struct MapParams {\n    count: u32,\n");
    for p in spec.params {
        src.push_str(&format!("    {}: f32,\n", codegen::wgsl_field(p)));
    }
    src.push_str(
        "}\n\
         @group(0) @binding(0) var<uniform> params: MapParams;\n\
         @group(0) @binding(1) var<storage, read> src: array<",
    );
    src.push_str(ty);
    src.push_str(
        ">;\n\
         @group(0) @binding(2) var<storage, read_write> dst: array<f32>;\n\n",
    );
    src.push_str(&format!(
        "fn reduce_value(v: {ty}) -> f32 {{ return {}; }}\n\n",
        spec.value
    ));
    let element = match present {
        true => "src[i]".to_string(),
        false => codegen::identity_literal(spec.dim, spec.identity),
    };
    src.push_str(&format!(
        "@compute @workgroup_size({wg})\n\
         fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
         \x20   let i = gid.x;\n\
         \x20   if (i >= params.count) {{ return; }}\n\
         \x20   dst[i] = reduce_value({element});\n\
         }}\n",
        wg = codegen::WORKGROUP_SIZE
    ));
    src
}

impl GpuCook {
    /// Encode every reduction a stage declares, into the cook's own encoder,
    /// before its kernel pass. Returns one result buffer per spec, in spec order.
    ///
    /// ⚠️ **Each map pass allocates its OWN uniform buffer** rather than taking a
    /// slot from [`GpuCook::uniform_slot`]'s indexed pool. The pool's contract is
    /// that two dispatches must never share a slot — `queue.write_buffer` is
    /// staged at submit, so the second write silently reaches BOTH — and a
    /// reduction is called from *inside* the sweep loop, where the same stage
    /// dispatches an unbounded number of times. Any slot arithmetic here would be
    /// a latent trap the first time a node declares both a grid and a reduction.
    /// The buffers are parked in `reduce_hold` for the cook's window, exactly as
    /// the post-compaction gathers park theirs.
    #[allow(clippy::too_many_arguments)] // private seam of `cook`, mirrors `build_grid`
    pub(crate) fn run_reduces(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        specs: &'static [ReduceSpec],
        inputs: &[GpuStream],
        graph: &Graph,
        node: NodeId,
        manifest: &NodeManifest,
    ) -> ReduceResults {
        let mut out = ReduceResults::default();
        for spec in specs {
            let result = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-reduce result"),
                size: 4,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            // **Seed with the operator's empty answer.** The fold below always
            // writes when `n > 0` (and the cook never reaches a stage with
            // `count == 0`), so this is not load-bearing for a healthy frame —
            // but the buffer would otherwise be uninitialised, and an
            // uninitialised 4 bytes read as a layout extent is a deformer folding
            // the scene around a number nobody computed. Costs one 4-byte write.
            gpu.queue
                .write_buffer(&result, 0, &spec.op.empty().to_le_bytes());

            let n = inputs.get(spec.port).map_or(0, |s| s.count);
            // An EMPTY port has nothing to fold and no elements to deform; the
            // seed above already publishes the operator's empty answer, which
            // lands the consumer in the same degenerate branch the CPU's fold
            // over zero elements does.
            if n == 0 {
                out.buffers.push(result);
                out.scratch.push(ReduceScratch::default());
                continue;
            }
            let src = inputs
                .get(spec.port)
                .and_then(|s| s.cols.get(spec.column))
                .map(|c| c.buffer.clone());
            let present = src.is_some();
            // Absent column: the pass still RUNS, folding the identity `n` times
            // (see [`map_module`]) — because the CPU materialises the column and
            // reduces over it, and skipping would answer a different question.
            let src = src.unwrap_or_else(|| {
                std::sync::Arc::new(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ph2d-reduce absent src"),
                    size: 16,
                    usage: wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }))
            });

            let scratch_buf = self.pool.acquire(gpu, u64::from(n) * 4);

            // --- map pass -------------------------------------------------
            let key = map_cache_key(spec, present);
            self.kernel_pipelines.entry(key).or_insert_with(|| {
                let src = map_module(spec, present);
                CachedPipeline {
                    pipeline: create_pipeline(gpu, &src, "ph2d-reduce map"),
                }
            });
            let mut uni = [0u8; UNIFORM_BYTES as usize];
            uni[0..4].copy_from_slice(&n.to_le_bytes());
            for (j, name) in spec.params.iter().enumerate() {
                let v = resolve_param(graph, node, manifest, name);
                let at = 4 + j * 4;
                uni[at..at + 4].copy_from_slice(&v.to_le_bytes());
            }
            let uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-reduce map uniform"),
                size: UNIFORM_BYTES,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu.queue.write_buffer(&uniform, 0, &uni);

            let pipeline = &self
                .kernel_pipelines
                .get(&key)
                .expect("inserted above")
                .pipeline;
            let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-reduce map"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: src.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: scratch_buf.as_entire_binding(),
                    },
                ],
            });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("ph2d-reduce map"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(n.div_ceil(codegen::WORKGROUP_SIZE), 1, 1);
            }

            // --- fold pass (the gated primitive, unchanged) ----------------
            let reduce = self.reduce.get_or_insert_with(|| Reduce::new(gpu));
            let mut sc = ReduceScratch::default();
            reduce.reduce_into(gpu, encoder, spec.op, &scratch_buf, n, &result, &mut sc);

            out.buffers.push(result);
            out.scratch.push(sc);
            // The map scratch and its uniform must outlive the submit too;
            // parking them is what `grid_hold` does for the grid's arrays.
            self.reduce_hold_bufs.push(scratch_buf);
            self.reduce_hold.push(uniform);
        }
        out
    }
}
