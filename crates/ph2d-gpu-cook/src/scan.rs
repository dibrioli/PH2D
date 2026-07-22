//! **GPU exclusive prefix-sum (scan)** over a `u32` storage buffer — the reusable
//! primitive the spatial-hash counting-sort needs (ADR-0140, Phase 1a).
//!
//! The counting-sort that turns positions into a neighbour grid is
//! *clear → count → **scan** → scatter*, and the scan is the only step that is
//! not embarrassingly parallel: element `i`'s output depends on every element
//! before it. This is the classic **work-efficient block scan** (Blelloch's
//! shape, Hillis–Steele within a block): each workgroup exclusive-scans a block
//! of [`WG`] elements in workgroup memory and emits the block's total; those
//! totals are **recursively** scanned; then each block adds its predecessors'
//! total back. Depth is `log_WG(n)` — two levels already reach 16 M
//! (`256² = 65 536` blocks).
//!
//! It is built and gated ALONE (`tests/gpu_scan.rs`) before any grid or boids
//! logic, because it is the frailest piece: a scan that is off by one row
//! mis-pairs every neighbour downstream, silently.
//!
//! **Composes with the cook's single submit** (master plan §2): [`Scan::exclusive`]
//! encodes into a caller-owned [`wgpu::CommandEncoder`] and parks its transient
//! block-sum / uniform buffers in a [`ScanScratch`] the caller keeps alive until
//! it submits. The standalone gate drives its own encoder; Phase 2 hands it the
//! cook's.

use crate::create_pipeline;
use ph2d_gpu::GpuContext;
use std::sync::OnceLock;

/// Elements scanned per workgroup — one per invocation, so it is the workgroup
/// size the whole engine uses (`codegen::WORKGROUP_SIZE`).
const WG: u32 = crate::codegen::WORKGROUP_SIZE;

/// Per-block exclusive scan: each workgroup scans `WG` elements of `data` in
/// place and writes the block's TOTAL to `block_sums[workgroup]`.
static BLOCK_WGSL: OnceLock<String> = OnceLock::new();
/// Add each block's predecessor-offset (the scanned `block_sums`) back into `data`.
static ADD_WGSL: OnceLock<String> = OnceLock::new();

fn block_wgsl() -> &'static str {
    BLOCK_WGSL.get_or_init(|| {
        format!(
            "struct U {{ n: u32 }};\n\
             @group(0) @binding(0) var<uniform> u: U;\n\
             @group(0) @binding(1) var<storage, read_write> data: array<u32>;\n\
             @group(0) @binding(2) var<storage, read_write> block_sums: array<u32>;\n\
             var<workgroup> temp: array<u32, {WG}u>;\n\
             @compute @workgroup_size({WG})\n\
             fn main(@builtin(local_invocation_id) lid_v: vec3<u32>,\n\
                     @builtin(workgroup_id) wid_v: vec3<u32>,\n\
                     @builtin(num_workgroups) nw: vec3<u32>) {{\n\
             \x20   let lid = lid_v.x;\n\
             \x20   // 2-D dispatch (ADR-0136 line, tetos §0.0): a bucket scan at 8 M\n\
             \x20   // elements is 2^24+1 entries = 65 537 blocks, past the 65 535\n\
             \x20   // per-dimension limit — the workgroup id is linearised from the\n\
             \x20   // grid instead, and the padding blocks of the rectangle bail out.\n\
             \x20   let wid = wid_v.x + wid_v.y * nw.x;\n\
             \x20   if (wid >= (u.n + {WG}u - 1u) / {WG}u) {{ return; }}\n\
             \x20   let i = wid * {WG}u + lid;\n\
             \x20   // Guarded read (WGSL `select` is not short-circuit — it would read OOB).\n\
             \x20   var v = 0u;\n\
             \x20   if (i < u.n) {{ v = data[i]; }}\n\
             \x20   temp[lid] = v;\n\
             \x20   // Hillis-Steele INCLUSIVE scan over the block (double-barrier per step).\n\
             \x20   for (var d = 1u; d < {WG}u; d = d << 1u) {{\n\
             \x20       workgroupBarrier();\n\
             \x20       var t = 0u;\n\
             \x20       if (lid >= d) {{ t = temp[lid - d]; }}\n\
             \x20       workgroupBarrier();\n\
             \x20       temp[lid] = temp[lid] + t;\n\
             \x20   }}\n\
             \x20   workgroupBarrier();\n\
             \x20   // Exclusive = inclusive - own value; the last lane emits the block total.\n\
             \x20   if (i < u.n) {{ data[i] = temp[lid] - v; }}\n\
             \x20   if (lid == {WG}u - 1u) {{ block_sums[wid] = temp[{WG}u - 1u]; }}\n\
             }}\n"
        )
    })
}

fn add_wgsl() -> &'static str {
    ADD_WGSL.get_or_init(|| {
        format!(
            "struct U {{ n: u32 }};\n\
             @group(0) @binding(0) var<uniform> u: U;\n\
             @group(0) @binding(1) var<storage, read_write> data: array<u32>;\n\
             @group(0) @binding(2) var<storage, read> block_sums: array<u32>;\n\
             @compute @workgroup_size({WG})\n\
             fn main(@builtin(local_invocation_id) lid_v: vec3<u32>,\n\
                     @builtin(workgroup_id) wid_v: vec3<u32>,\n\
                     @builtin(num_workgroups) nw: vec3<u32>) {{\n\
             \x20   let wid = wid_v.x + wid_v.y * nw.x;\n\
             \x20   let i = wid * {WG}u + lid_v.x;\n\
             \x20   if (i < u.n) {{ data[i] = data[i] + block_sums[wid]; }}\n\
             }}\n"
        )
    })
}

/// Compiled scan pipelines — build once, reuse every tick.
pub struct Scan {
    block: wgpu::ComputePipeline,
    add: wgpu::ComputePipeline,
}

/// Transient buffers a scan run allocates (block-sum arrays + `n` uniforms). The
/// caller keeps it alive until it submits the encoder the scan wrote into, then
/// drops it. `Default::default()` is an empty run.
#[derive(Default)]
pub struct ScanScratch {
    buffers: Vec<wgpu::Buffer>,
}

impl Scan {
    pub fn new(gpu: &GpuContext) -> Self {
        Scan {
            block: create_pipeline(gpu, block_wgsl(), "ph2d-scan block"),
            add: create_pipeline(gpu, add_wgsl(), "ph2d-scan add"),
        }
    }

    /// Exclusive-scan `data[0..n]` **in place**. Encodes into `encoder`; the
    /// transient block-sum and uniform buffers land in `scratch` (keep it alive
    /// until the caller submits). `data` must be `STORAGE` and hold ≥ `n` `u32`s.
    pub fn exclusive(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        data: &wgpu::Buffer,
        n: u32,
        scratch: &mut ScanScratch,
    ) {
        if n == 0 {
            return;
        }
        let num_blocks = n.div_ceil(WG);

        // `n` in a uniform (padded to 16 B — the safe uniform-binding minimum).
        let uni = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-scan n"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&uni, 0, &n.to_le_bytes());

        // One total per block — recursively scanned when there is more than one.
        let block_sums = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-scan block_sums"),
            size: u64::from(num_blocks) * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        self.pass(
            gpu,
            encoder,
            &self.block,
            &uni,
            data,
            &block_sums,
            num_blocks,
        );

        if num_blocks > 1 {
            // Scan the block totals into per-block predecessor offsets, then fold
            // them back in. The recursion borrows `scratch` too — its buffers pile
            // in alongside ours; order does not matter, they only need to outlive
            // the submit.
            self.exclusive(gpu, encoder, &block_sums, num_blocks, scratch);
            self.pass(gpu, encoder, &self.add, &uni, data, &block_sums, num_blocks);
        }

        scratch.buffers.push(uni);
        scratch.buffers.push(block_sums);
    }

    /// Encode one pass: bind `{uniform, data, block_sums}` and dispatch
    /// `num_blocks` workgroups (the dispatch is over the ELEMENTS, `WG` per block).
    #[allow(clippy::too_many_arguments)]
    fn pass(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::ComputePipeline,
        uni: &wgpu::Buffer,
        data: &wgpu::Buffer,
        block_sums: &wgpu::Buffer,
        num_blocks: u32,
    ) {
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-scan"),
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
                    resource: block_sums.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ph2d-scan"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        // 2-D shape past the per-dimension limit; the kernels linearise the
        // workgroup id and bail out of the rectangle's padding blocks.
        let (x, y) = dispatch_2d(num_blocks);
        pass.dispatch_workgroups(x, y, 1);
    }
}

/// The 65 535-per-dimension limit is a DISPATCH-SHAPE limit, not a work limit:
/// fold `blocks` into an `(x, y)` rectangle that covers it (`x·y ≥ blocks`,
/// both ≤ 65 535). Kernels linearise `wid = x + y·nw.x` and guard the padding.
pub fn dispatch_2d(blocks: u32) -> (u32, u32) {
    const MAX_DIM: u32 = 65_535;
    if blocks <= MAX_DIM {
        (blocks.max(1), 1)
    } else {
        (MAX_DIM, blocks.div_ceil(MAX_DIM))
    }
}

/// The CPU oracle: exclusive prefix-sum, the canonical answer the gate reconciles
/// the device against (this one is INTEGER, so parity is bit-exact, not ε).
#[must_use]
pub fn cpu_exclusive(data: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(data.len());
    let mut acc = 0u32;
    for &v in data {
        out.push(acc);
        acc = acc.wrapping_add(v);
    }
    out
}
