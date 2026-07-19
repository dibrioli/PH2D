//! **The spatial-hash grid** — the reusable neighbourhood structure the whole
//! O(N²) sim class is built on (ADR-0134, Phase 1b/2).
//!
//! Boids, `sim.collide` and a future SPH all ask the same question — *who is
//! within `radius` of element `i`?* — and all answer it the same way: bin every
//! element into a uniform grid of cell size `radius`, then look only at the 3×3
//! cells around each element. So the grid is a **service**, not a per-node kernel:
//! the node authors its *query* (how it uses the neighbours) as pure metadata,
//! and this builds the structure the query reads.
//!
//! **A hash, not a bounded grid.** The cell of a position is `⌊p / cell⌋`, hashed
//! to `num_buckets = pow2(2·n)` buckets. Hashing avoids a min/max **readback** of
//! the positions (the §0.0 anti-pattern) — `num_buckets` is a pure function of
//! `n`, known host-side. Hash collisions (two cells → one bucket) only add
//! candidates the query's distance test rejects, so the neighbour SET is exactly
//! the CPU all-pairs' — parity, not a different flock.
//!
//! **The build is a counting sort:** clear → count (atomic per bucket) →
//! [`scan`](crate::scan) (bucket offsets) → scatter (atomic slot per bucket). It
//! yields `starts` (`num_buckets+1` offsets; bucket `b` owns
//! `sorted[starts[b]..starts[b+1]]`) and `sorted` (a permutation of `0..n`). The
//! intra-bucket order is non-deterministic (atomic race) and **deliberately
//! unspecified** — the query sums over a bucket and the ε-parity absorbs the
//! order, exactly as it absorbs float non-associativity.
//!
//! Standalone and gated (`tests/gpu_grid.rs`) before any node wiring, like the
//! scan: a grid that mis-bins is a silent wrong flock.

use crate::create_pipeline;
use crate::scan::{Scan, ScanScratch};
use ph2d_gpu::GpuContext;
use std::sync::OnceLock;

const WG: u32 = crate::codegen::WORKGROUP_SIZE;

/// The bucket count for `n` elements: `pow2(2·n)` (load factor ½ — good
/// occupancy without a bounds pass), floored at a workgroup. A pure function of
/// `n` so the CPU oracle and the device agree exactly.
#[must_use]
pub fn num_buckets(n: u32) -> u32 {
    (n.saturating_mul(2)).max(WG).next_power_of_two()
}

/// The spatial-hash key of a 2D cell — the SAME expression on both sides
/// (integer, wrapping, so bit-exact CPU↔GPU). `num_buckets` is a power of two, so
/// the modulo is a mask.
#[must_use]
pub fn cpu_bucket(p: [f32; 2], cell: f32, num_buckets: u32) -> u32 {
    let cx = (p[0] / cell).floor() as i32;
    let cy = (p[1] / cell).floor() as i32;
    let h = cx.wrapping_mul(73_856_093) ^ cy.wrapping_mul(19_349_663);
    (h as u32) & (num_buckets - 1)
}

/// The WGSL `bucket_of` helper, shared by the count and scatter passes so the
/// two never disagree on where an element lands (mirror of [`cpu_bucket`]).
const BUCKET_LIB: &str = "\
fn bucket_of(p: vec2<f32>) -> u32 {\n\
\x20   let cx = i32(floor(p.x / u.cell));\n\
\x20   let cy = i32(floor(p.y / u.cell));\n\
\x20   let h = (cx * 73856093) ^ (cy * 19349663);\n\
\x20   return bitcast<u32>(h) & (u.num_buckets - 1u);\n\
}\n";

/// Uniform shared by both passes: `[n, num_buckets, cell, _pad]` (16 B).
const UNIFORM_STRUCT: &str = "struct U { n: u32, num_buckets: u32, cell: f32, _pad: u32 };\n@group(0) @binding(0) var<uniform> u: U;\n";

static COUNT_WGSL: OnceLock<String> = OnceLock::new();
static SCATTER_WGSL: OnceLock<String> = OnceLock::new();

fn count_wgsl() -> &'static str {
    COUNT_WGSL.get_or_init(|| {
        format!(
            "{UNIFORM_STRUCT}\
             @group(0) @binding(1) var<storage, read> pos: array<vec2<f32>>;\n\
             @group(0) @binding(2) var<storage, read_write> starts: array<atomic<u32>>;\n\
             {BUCKET_LIB}\
             @compute @workgroup_size({WG})\n\
             fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
             \x20   let i = gid.x;\n\
             \x20   if (i >= u.n) {{ return; }}\n\
             \x20   atomicAdd(&starts[bucket_of(pos[i])], 1u);\n\
             }}\n"
        )
    })
}

fn scatter_wgsl() -> &'static str {
    SCATTER_WGSL.get_or_init(|| {
        format!(
            "{UNIFORM_STRUCT}\
             @group(0) @binding(1) var<storage, read> pos: array<vec2<f32>>;\n\
             @group(0) @binding(2) var<storage, read> starts: array<u32>;\n\
             @group(0) @binding(3) var<storage, read_write> cursor: array<atomic<u32>>;\n\
             @group(0) @binding(4) var<storage, read_write> sorted: array<u32>;\n\
             {BUCKET_LIB}\
             @compute @workgroup_size({WG})\n\
             fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
             \x20   let i = gid.x;\n\
             \x20   if (i >= u.n) {{ return; }}\n\
             \x20   let b = bucket_of(pos[i]);\n\
             \x20   let slot = starts[b] + atomicAdd(&cursor[b], 1u);\n\
             \x20   sorted[slot] = i;\n\
             }}\n"
        )
    })
}

/// The grid's output buffers: `starts` (`num_buckets+1` offsets) and `sorted`
/// (`n` element indices, bucket-partitioned). A query binds both.
pub struct GridBuffers {
    pub starts: wgpu::Buffer,
    pub sorted: wgpu::Buffer,
    pub num_buckets: u32,
}

/// Transients a build allocates (the cursor + the scan's scratch), kept alive by
/// the caller until it submits.
#[derive(Default)]
pub struct GridScratch {
    scan: ScanScratch,
    buffers: Vec<wgpu::Buffer>,
}

/// Compiled grid pipelines — build once, reuse every tick.
pub struct Grid {
    count: wgpu::ComputePipeline,
    scatter: wgpu::ComputePipeline,
    scan: Scan,
}

impl Grid {
    pub fn new(gpu: &GpuContext) -> Self {
        Grid {
            count: create_pipeline(gpu, count_wgsl(), "ph2d-grid count"),
            scatter: create_pipeline(gpu, scatter_wgsl(), "ph2d-grid scatter"),
            scan: Scan::new(gpu),
        }
    }

    /// Build the grid over `pos` (`n` × `vec2<f32>`, cell size `cell`). Encodes
    /// into `encoder`; transients land in `scratch` (keep it alive until submit).
    /// The returned `starts`/`sorted` buffers are the caller's to bind or read.
    pub fn build(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        pos: &wgpu::Buffer,
        n: u32,
        cell: f32,
        scratch: &mut GridScratch,
    ) -> GridBuffers {
        let num_buckets = num_buckets(n);

        let uni = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-grid uniform"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut u = [0u8; 16];
        u[0..4].copy_from_slice(&n.to_le_bytes());
        u[4..8].copy_from_slice(&num_buckets.to_le_bytes());
        u[8..12].copy_from_slice(&cell.to_le_bytes());
        gpu.queue.write_buffer(&uni, 0, &u);

        // `starts` is num_buckets+1: the count writes [0, num_buckets), the last
        // slot stays 0, and the exclusive scan turns [c0,…,c_{m-1}, 0] into
        // [0, c0, …, total] — so bucket b owns [starts[b], starts[b+1]) and
        // starts[num_buckets] = n falls out for free.
        let starts = storage(gpu, u64::from(num_buckets + 1) * 4, "ph2d-grid starts");
        let cursor = storage(gpu, u64::from(num_buckets) * 4, "ph2d-grid cursor");
        let sorted = storage(gpu, u64::from(n.max(1)) * 4, "ph2d-grid sorted");
        encoder.clear_buffer(&starts, 0, None);
        encoder.clear_buffer(&cursor, 0, None);

        // Count: one atomic increment per element into its bucket.
        self.dispatch(
            gpu,
            encoder,
            &self.count,
            &[
                uni.as_entire_binding(),
                pos.as_entire_binding(),
                starts.as_entire_binding(),
            ],
            n,
        );
        // Offsets: exclusive scan turns the counts into per-bucket bases.
        self.scan
            .exclusive(gpu, encoder, &starts, num_buckets + 1, &mut scratch.scan);
        // Scatter: each element claims a slot in its bucket's range.
        self.dispatch(
            gpu,
            encoder,
            &self.scatter,
            &[
                uni.as_entire_binding(),
                pos.as_entire_binding(),
                starts.as_entire_binding(),
                cursor.as_entire_binding(),
                sorted.as_entire_binding(),
            ],
            n,
        );

        scratch.buffers.push(uni);
        scratch.buffers.push(cursor);
        GridBuffers {
            starts,
            sorted,
            num_buckets,
        }
    }

    /// Encode one pass over `n` elements with the given bindings (uniform first).
    fn dispatch(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::ComputePipeline,
        resources: &[wgpu::BindingResource],
        n: u32,
    ) {
        let entries: Vec<wgpu::BindGroupEntry> = resources
            .iter()
            .enumerate()
            .map(|(i, r)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: r.clone(),
            })
            .collect();
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-grid"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ph2d-grid"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(n.div_ceil(WG), 1, 1);
    }
}

fn storage(gpu: &GpuContext, bytes: u64, label: &str) -> wgpu::Buffer {
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes,
        // COPY_DST for `clear_buffer` (starts/cursor); COPY_SRC for the gate readback.
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
