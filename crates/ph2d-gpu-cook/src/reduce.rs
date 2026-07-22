//! **GPU whole-stream reduction** over an `f32` storage buffer — the reusable
//! primitive the *deformer* family needs, and the sibling of [`crate::scan`].
//!
//! ## Why this exists
//!
//! Every deformer in the library has the same shape, and it is not the
//! per-element shape the rest of the engine is built for:
//!
//! ```text
//!     reduce (one number about the WHOLE layout) → broadcast → per-element map
//! ```
//!
//! `motion.bend` wraps the layout onto an arc scaled to its **X extent**;
//! `motion.twist` turns the rim by an angle scaled to the **max radius**;
//! `motion.spherize` bulges around the **centroid**. None of them can be written
//! as a per-element kernel, because element `i`'s answer depends on a number that
//! only exists once every element has been looked at — which is exactly why the
//! whole family shipped CPU-only while 43 per-element nodes got kernels.
//!
//! The engine already owns the *other* non-embarrassingly-parallel primitive
//! ([`crate::scan`], built for the counting-sort). The only reduction in the
//! crate was **private inside `voronoi.rs`** (an integer centroid over the JFA
//! assignment), so it could not serve anyone else. This is that primitive, made
//! reusable — and, exactly like the scan, **built and gated ALONE** before any
//! node is wired to it, because a reduction that is wrong in the block seam is
//! invisible on one block and fatal on many.
//!
//! ## The operator lives in the pure-data crate
//!
//! [`ReduceOp`] is `ph2d_nodegraph::gpu::ReduceOp`, re-exported here. Its WGSL
//! combine, its WGSL identity and its CPU fold are three answers to ONE question
//! (*"what does this operator mean?"*), and a second copy on this side of the
//! fence is how the device's `max` and the host's `max` come to disagree about
//! an identity with nobody noticing. Its docs carry the parity contract:
//! `Max`/`Min` are **bit-exact** in any evaluation order, `Sum` is a documented
//! ε, and NaN is out of contract on both paths.
//!
//! ## Composes with the cook's single submit
//!
//! [`Reduce::reduce_into`] encodes into a caller-owned [`wgpu::CommandEncoder`]
//! and parks its transient level buffers in a [`ReduceScratch`] the caller keeps
//! alive until it submits — the same contract [`crate::scan::Scan::exclusive`]
//! has, for the same reason (master plan §2: one submit per cook).

use crate::create_pipeline;
use crate::scan::dispatch_2d;
use ph2d_gpu::GpuContext;
use std::sync::OnceLock;

pub use ph2d_nodegraph::gpu::ReduceOp;

/// Elements reduced per workgroup — one per invocation, so it is the workgroup
/// size the whole engine uses (`codegen::WORKGROUP_SIZE`).
const WG: u32 = crate::codegen::WORKGROUP_SIZE;

/// The reduction kernel for one operator: each workgroup folds a block of [`WG`]
/// elements in workgroup memory and writes the block's result to `out[wid]`.
///
/// One `OnceLock` per operator — the shape `scan.rs` uses for its two modules.
/// A keyed cache would have been a map, and a `HashMap` is a disallowed type
/// here (iteration order is the spine of this engine's determinism); with
/// exactly three operators, three slots is also simply less machinery.
fn reduce_wgsl(op: ReduceOp) -> &'static str {
    static MAX_WGSL: OnceLock<String> = OnceLock::new();
    static MIN_WGSL: OnceLock<String> = OnceLock::new();
    static SUM_WGSL: OnceLock<String> = OnceLock::new();
    let slot = match op {
        ReduceOp::Max => &MAX_WGSL,
        ReduceOp::Min => &MIN_WGSL,
        ReduceOp::Sum => &SUM_WGSL,
    };
    slot.get_or_init(|| {
        let combine = op.wgsl_combine();
        let identity = op.wgsl_identity();
        format!(
            "struct U {{ n: u32 }};\n\
             @group(0) @binding(0) var<uniform> u: U;\n\
             @group(0) @binding(1) var<storage, read> data: array<f32>;\n\
             @group(0) @binding(2) var<storage, read_write> out: array<f32>;\n\
             var<workgroup> temp: array<f32, {WG}u>;\n\
             fn combine(a: f32, b: f32) -> f32 {{ return {combine}; }}\n\
             @compute @workgroup_size({WG})\n\
             fn main(@builtin(local_invocation_id) lid_v: vec3<u32>,\n\
                     @builtin(workgroup_id) wid_v: vec3<u32>,\n\
                     @builtin(num_workgroups) nw: vec3<u32>) {{\n\
             \x20   let lid = lid_v.x;\n\
             \x20   // 2-D dispatch (the 65 535 per-dimension limit is a SHAPE limit,\n\
             \x20   // not a work limit): linearise the workgroup id exactly as the\n\
             \x20   // scan does, and bail out of the rectangle's padding blocks.\n\
             \x20   // The bail is UNIFORM across the workgroup (`wid` is a\n\
             \x20   // workgroup-wide value), which is what keeps the barriers below\n\
             \x20   // legal — a per-lane early return here would be undefined.\n\
             \x20   let wid = wid_v.x + wid_v.y * nw.x;\n\
             \x20   if (wid >= (u.n + {WG}u - 1u) / {WG}u) {{ return; }}\n\
             \x20   let i = wid * {WG}u + lid;\n\
             \x20   // Guarded read (WGSL `select` is not short-circuit — it would\n\
             \x20   // read out of bounds). A lane past the end contributes the\n\
             \x20   // operator's IDENTITY, so a partial final block is exact.\n\
             \x20   var v = {identity};\n\
             \x20   if (i < u.n) {{ v = data[i]; }}\n\
             \x20   temp[lid] = v;\n\
             \x20   // Tree fold, halving the active lanes each step. The barrier is\n\
             \x20   // OUTSIDE the `lid < d` guard: every lane reaches it, which is\n\
             \x20   // what makes it uniform control flow.\n\
             \x20   for (var d = {WG}u >> 1u; d > 0u; d = d >> 1u) {{\n\
             \x20       workgroupBarrier();\n\
             \x20       if (lid < d) {{ temp[lid] = combine(temp[lid], temp[lid + d]); }}\n\
             \x20   }}\n\
             \x20   workgroupBarrier();\n\
             \x20   if (lid == 0u) {{ out[wid] = temp[0]; }}\n\
             }}\n"
        )
    })
}

/// Compiled reduction pipelines — build once, reuse every tick.
pub struct Reduce {
    max: wgpu::ComputePipeline,
    min: wgpu::ComputePipeline,
    sum: wgpu::ComputePipeline,
}

/// Transient buffers a reduction run allocates (the per-level partial arrays and
/// the `n` uniforms). The caller keeps it alive until it submits the encoder the
/// reduction wrote into, then drops it. `Default::default()` is an empty run.
#[derive(Default)]
pub struct ReduceScratch {
    buffers: Vec<wgpu::Buffer>,
}

impl Reduce {
    pub fn new(gpu: &GpuContext) -> Self {
        Reduce {
            max: create_pipeline(gpu, reduce_wgsl(ReduceOp::Max), "ph2d-reduce max"),
            min: create_pipeline(gpu, reduce_wgsl(ReduceOp::Min), "ph2d-reduce min"),
            sum: create_pipeline(gpu, reduce_wgsl(ReduceOp::Sum), "ph2d-reduce sum"),
        }
    }

    fn pipeline(&self, op: ReduceOp) -> &wgpu::ComputePipeline {
        match op {
            ReduceOp::Max => &self.max,
            ReduceOp::Min => &self.min,
            ReduceOp::Sum => &self.sum,
        }
    }

    /// Reduce `data[0..n]` with `op`, writing the single result to `out[0]`.
    ///
    /// Encodes into `encoder`; the transient level buffers land in `scratch`
    /// (keep it alive until the caller submits). `data` must be `STORAGE` and
    /// hold ≥ `n` `f32`s; `out` must be `STORAGE` and ≥ 4 bytes.
    ///
    /// **`n == 0` writes nothing** — there is no identity to publish that a
    /// caller could not have supplied itself, and silently writing one would let
    /// an empty stream masquerade as a measured extent. The deformers seed `out`
    /// with their own degenerate-case value before the pass for exactly this.
    /// (8 args: the same shape `scan`'s `pass` carries, and for the same reason —
    /// device, encoder, operator, source, length, destination, scratch. Bundling
    /// them into a struct would name a thing that exists for one call.)
    #[allow(clippy::too_many_arguments)]
    pub fn reduce_into(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        op: ReduceOp,
        data: &wgpu::Buffer,
        n: u32,
        out: &wgpu::Buffer,
        scratch: &mut ReduceScratch,
    ) {
        if n == 0 {
            return;
        }
        let num_blocks = n.div_ceil(WG);

        // `n` in a uniform (padded to 16 B — the safe uniform-binding minimum).
        let uni = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-reduce n"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&uni, 0, &n.to_le_bytes());

        if num_blocks == 1 {
            // One block folds the whole thing — write straight to the caller's
            // slot and stop. This is the recursion's base case, and it is also
            // the ONLY case for every stream up to 256 elements.
            self.pass(gpu, encoder, op, &uni, data, out, num_blocks);
            scratch.buffers.push(uni);
            return;
        }

        // One partial per block, then reduce THOSE the same way. Depth is
        // `log_WG(n)`: two levels already reach 65 536² = 4 G elements.
        let level = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-reduce level"),
            size: u64::from(num_blocks) * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        self.pass(gpu, encoder, op, &uni, data, &level, num_blocks);
        self.reduce_into(gpu, encoder, op, &level, num_blocks, out, scratch);

        scratch.buffers.push(uni);
        scratch.buffers.push(level);
    }

    /// Encode one level: bind `{uniform, data, out}` and dispatch `num_blocks`
    /// workgroups (the dispatch is over the ELEMENTS, `WG` per block).
    #[allow(clippy::too_many_arguments)]
    fn pass(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        op: ReduceOp,
        uni: &wgpu::Buffer,
        data: &wgpu::Buffer,
        out: &wgpu::Buffer,
        num_blocks: u32,
    ) {
        let pipeline = self.pipeline(op);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-reduce"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uni.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: data.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: out.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ph2d-reduce"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let (x, y) = dispatch_2d(num_blocks);
        pass.dispatch_workgroups(x, y, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The operator's own properties (associativity, identity, the CPU oracle)
    // are pinned where the operator LIVES — `ph2d_nodegraph::reduce_meta`. A
    // second copy here would be two gates for one fact, free to drift apart the
    // day someone edits one of them. What this file still owns is the generated
    // MODULE, and that is what is gated below.

    #[test]
    fn every_operator_generates_a_distinct_module_with_its_own_combine() {
        // Cheap structural guard: the cache is keyed by op, so a copy-paste that
        // gave two operators the same body would be invisible without this.
        let (mx, mn, sm) = (
            reduce_wgsl(ReduceOp::Max),
            reduce_wgsl(ReduceOp::Min),
            reduce_wgsl(ReduceOp::Sum),
        );
        assert!(mx.contains("return max(a, b);"));
        assert!(mn.contains("return min(a, b);"));
        assert!(sm.contains("return a + b;"));
        assert_ne!(mx, mn);
        assert_ne!(mn, sm);
        // The barrier must sit INSIDE the loop but OUTSIDE the `lid < d` guard:
        // every lane has to reach it, which is what makes it uniform control flow
        // (a barrier under the guard is undefined behaviour, and the symptom is a
        // wrong answer on some drivers and none on others). Asserted as an ORDER
        // between the two landmarks, never as literal whitespace — an indentation
        // change must not be able to make this gate lie in either direction.
        let loop_body = mx
            .split_once("for (var d =")
            .expect("the tree loop is generated")
            .1;
        let barrier = loop_body
            .find("workgroupBarrier();")
            .expect("barrier in loop");
        let guard = loop_body.find("if (lid < d)").expect("guard in loop");
        assert!(
            barrier < guard,
            "the barrier must precede (and sit outside) the lane guard"
        );
    }
}
