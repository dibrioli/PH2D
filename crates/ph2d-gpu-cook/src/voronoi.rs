//! **`motion.voronoi` on the device** (ADR-0139): Lloyd relaxation toward a
//! CVT via **Jump Flooding** — the engine's first [`GpuAlgorithm`], the
//! multi-pass sibling of the grid build and the stream ops.
//!
//! The CPU node's `nearest` is a linear scan per grid sample —
//! `O(iterations · res² · count)` — and the 600-point cap exists only to keep
//! that affordable (§0.0: the slow path was defining the product). The device
//! replaces the scan with the **Jump Flooding Algorithm** (Rong & Tan 2006):
//! seed a `res²` grid with each point's id, then `log₂(res)+1` passes where
//! every texel adopts, among its 9 neighbours at the current offset, the owner
//! nearest to its own centre. `O(res² · log res)` — **independent of the
//! count**. All passes land in the caller's encoder: zero readback, the
//! count is a host-side param law.
//!
//! ## Integer centroids (ADR-0139 §2)
//!
//! WGSL has no `f32` atomics, and the centroid does not need them: a sample's
//! position is an **affine function of its texel index** (`sample_pos`), so
//! accumulating `Σgx, Σgy, n` in `u32` is exact and order-independent (integer
//! addition commutes), and the centroid is the same affine map applied to the
//! mean index. The device is deterministic by construction. Overflow bound:
//! `Σgx ≤ res³`, so `res ≤ 1625` keeps every sum under `2³²`
//! ([`INT_CENTROID_RES_CEILING`]).
//!
//! ## Where the two paths honestly differ (measured in the gates)
//!
//! - **JFA misses rare boundary texels** (~10⁻⁴ of the grid): a texel whose
//!   two nearest owners are nearly equidistant can keep the farther one. The
//!   assignment gate counts these and requires them to BE near-ties.
//! - **A seed collision hides a point for one round**: two points in one texel
//!   leave one of them owning nothing (it holds still that iteration — the
//!   CPU's empty-cell rule), while the CPU's exact scan still hands it the
//!   texels it is nearest to. The winner moves away, the pair separates, and
//!   the next iteration both relax normally — transient and self-healing, but
//!   it is why full-trajectory parity carries a looser, measured bound while
//!   the **one-step** gate (ADR-0127 D4: sequential systems gate one step) is
//!   tight.
//!
//! Seeding stores `count − id` under `atomicMax` (0 = empty, so
//! `clear_buffer` empties the grid): deterministic, and the **lower id wins**
//! a contested texel — the same preference the CPU's `nearest` keep-first
//! gives an exact-distance tie, which the JFA passes also apply.

use crate::plan::resolve_param;
use crate::{GpuColumn, GpuCook, GpuStream, create_pipeline};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::gpu::GpuAlgorithm;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::{NodeManifest, param_as_count};
use std::collections::BTreeMap;

const WG: u32 = crate::codegen::WORKGROUP_SIZE;

/// The largest grid side whose integer centroid sums provably fit `u32`:
/// `Σgx ≤ (res−1)·res² < res³`, and `1625³ < 2³² < 1626³`. The node's own
/// `max_res` sits far below; this is the engine's guard, not a tuning knob.
pub const INT_CENTROID_RES_CEILING: usize = 1625;

/// The value column the relax port carries (the node's `VALUE_COL`).
const VALUE_COL: &str = "v";

/// The uniform every voronoi pass reads — one 32-byte layout so a single
/// helper drives all six pipelines. `n` is the DISPATCH length (count or
/// res²); `count` is always the point count (the accumulator sections and the
/// seed-key inversion need it even in grid-sized passes).
fn uniform_bytes(n: u32, res: u32, step: u32, count: u32, w: f32, h: f32, seed: u32) -> [u8; 32] {
    let mut b = [0u8; 32];
    b[0..4].copy_from_slice(&n.to_le_bytes());
    b[4..8].copy_from_slice(&res.to_le_bytes());
    b[8..12].copy_from_slice(&step.to_le_bytes());
    b[12..16].copy_from_slice(&count.to_le_bytes());
    b[16..20].copy_from_slice(&w.to_le_bytes());
    b[20..24].copy_from_slice(&h.to_le_bytes());
    b[24..28].copy_from_slice(&seed.to_le_bytes());
    b
}

fn module(bindings: &str, body: &str, lib: &str) -> String {
    format!(
        "struct U {{ n: u32, res: u32, step: u32, count: u32, w: f32, h: f32, seed: u32, pad: u32 }}\n\
         @group(0) @binding(0) var<uniform> u: U;\n\
         {bindings}\n\
         {lib}\
         @compute @workgroup_size({WG})\n\
         fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
         \x20   let i = gid.x;\n\
         \x20   if (i >= u.n) {{ return; }}\n\
         {body}\n\
         }}\n"
    )
}

/// The integer avalanche of the node's `hash3` — the emitter's `em_hash3`
/// port, **bit-exact** in WGSL (`u32` wraps mod 2³² like `wrapping_mul`), so
/// the raw seed cloud is the CPU's to the bit and `relax = 0` is byte-parity.
const HASH_LIB: &str = "\
    fn vo_hash3(a: u32, b: u32, lane: u32) -> f32 {\n\
        var x: u32 = a * 0x9e3779b9u + b * 0x85ebca6bu + lane * 0xc2b2ae35u;\n\
        x = x ^ (x >> 16u);\n\
        x = x * 0x7feb352du;\n\
        x = x ^ (x >> 15u);\n\
        x = x * 0x846ca68bu;\n\
        x = x ^ (x >> 16u);\n\
        return f32(x >> 8u) / f32(16777216u);\n\
    }\n";

/// The six fixed pipelines, compiled on first use and reused every cook (the
/// stream-op pattern — `GpuCook` is `Default` and has no device).
pub(crate) struct VoronoiPipes {
    /// `raw[i] = cur[i] = hashed seed point i` — dispatch over `count`.
    seed: wgpu::ComputePipeline,
    /// `atomicMax(&grid[cell(cur[i])], count − i)` — dispatch over `count`
    /// into a grid `clear_buffer`ed to 0 (= empty).
    grid_init: wgpu::ComputePipeline,
    /// One jump-flood step at `u.step` — dispatch over `res²`, ping-pong
    /// src→dst. Strictly-closer wins; an exact tie prefers the lower id.
    jfa: wgpu::ComputePipeline,
    /// `Σgx/Σgy/n` per owner in `u32` atomics — dispatch over `res²` into an
    /// accumulator cleared to 0 (three `count`-sized sections).
    reduce: wgpu::ComputePipeline,
    /// `cur[i] = affine(mean index)` (owners with no texels hold still — the
    /// CPU's empty-cell rule) — dispatch over `count`.
    mv: wgpu::ComputePipeline,
    /// `out[i] = raw[i] + (cur[i] − raw[i])·clamp(relax[0], 0, 1)` — the
    /// relax VALUE read at **row 0 on the device**; dispatch over `count`.
    lerp: wgpu::ComputePipeline,
}

/// The six WGSL modules, by name — one source of truth for [`VoronoiPipes::new`]
/// and the no-device naga validation test (the `sprite_wgsl_valid` pattern).
pub(crate) fn sources() -> [(&'static str, String); 6] {
    let seed = module(
        "@group(0) @binding(1) var<storage, read_write> raw: array<vec2<f32>>;\n\
             @group(0) @binding(2) var<storage, read_write> cur: array<vec2<f32>>;",
        "\x20   let p = vec2<f32>((vo_hash3(u.seed, i, 0u) - 0.5) * u.w,\n\
             \x20                     (vo_hash3(u.seed, i, 1u) - 0.5) * u.h);\n\
             \x20   raw[i] = p;\n\
             \x20   cur[i] = p;",
        HASH_LIB,
    );
    let grid_init = module(
        "@group(0) @binding(1) var<storage, read> cur: array<vec2<f32>>;\n\
             @group(0) @binding(2) var<storage, read_write> grid: array<atomic<u32>>;",
        "\x20   let p = cur[i];\n\
             \x20   let r = f32(u.res);\n\
             \x20   let gx = clamp(i32(floor((p.x / u.w + 0.5) * r)), 0, i32(u.res) - 1);\n\
             \x20   let gy = clamp(i32(floor((p.y / u.h + 0.5) * r)), 0, i32(u.res) - 1);\n\
             \x20   atomicMax(&grid[u32(gy) * u.res + u32(gx)], u.count - i);",
        "",
    );
    let jfa = module(
        "@group(0) @binding(1) var<storage, read> pts: array<vec2<f32>>;\n\
             @group(0) @binding(2) var<storage, read> src: array<u32>;\n\
             @group(0) @binding(3) var<storage, read_write> dst: array<u32>;",
        "\x20   let gx = i32(i % u.res);\n\
             \x20   let gy = i32(i / u.res);\n\
             \x20   let r = f32(u.res);\n\
             \x20   let tc = vec2<f32>(((f32(gx) + 0.5) / r - 0.5) * u.w,\n\
             \x20                      ((f32(gy) + 0.5) / r - 0.5) * u.h);\n\
             \x20   var best = 0u;\n\
             \x20   var bd = 0.0;\n\
             \x20   let s = i32(u.step);\n\
             \x20   for (var dy = -1; dy <= 1; dy = dy + 1) {\n\
             \x20       for (var dx = -1; dx <= 1; dx = dx + 1) {\n\
             \x20           let nx = gx + dx * s;\n\
             \x20           let ny = gy + dy * s;\n\
             \x20           if (nx < 0 || ny < 0 || nx >= i32(u.res) || ny >= i32(u.res)) { continue; }\n\
             \x20           let cand = src[u32(ny) * u.res + u32(nx)];\n\
             \x20           if (cand == 0u) { continue; }\n\
             \x20           let dv = tc - pts[u.count - cand];\n\
             \x20           let d = dot(dv, dv);\n\
             \x20           // Strictly closer wins; an exact tie prefers the LOWER id —\n\
             \x20           // the CPU nearest's keep-first on strict `<`. The stored key\n\
             \x20           // is count−id, so the lower id is the HIGHER key.\n\
             \x20           if (best == 0u || d < bd || (d == bd && cand > best)) {\n\
             \x20               best = cand;\n\
             \x20               bd = d;\n\
             \x20           }\n\
             \x20       }\n\
             \x20   }\n\
             \x20   dst[i] = best;",
        "",
    );
    let reduce = module(
        "@group(0) @binding(1) var<storage, read> grid: array<u32>;\n\
             @group(0) @binding(2) var<storage, read_write> acc: array<atomic<u32>>;",
        "\x20   let own = grid[i];\n\
             \x20   if (own != 0u) {\n\
             \x20       let j = u.count - own;\n\
             \x20       atomicAdd(&acc[j], i % u.res);\n\
             \x20       atomicAdd(&acc[u.count + j], i / u.res);\n\
             \x20       atomicAdd(&acc[2u * u.count + j], 1u);\n\
             \x20   }",
        "",
    );
    let mv = module(
        "@group(0) @binding(1) var<storage, read> acc: array<u32>;\n\
             @group(0) @binding(2) var<storage, read_write> cur: array<vec2<f32>>;",
        "\x20   let cnt = acc[2u * u.count + i];\n\
             \x20   if (cnt != 0u) {\n\
             \x20       let r = f32(u.res);\n\
             \x20       let mx = f32(acc[i]) / f32(cnt);\n\
             \x20       let my = f32(acc[u.count + i]) / f32(cnt);\n\
             \x20       cur[i] = vec2<f32>(((mx + 0.5) / r - 0.5) * u.w,\n\
             \x20                          ((my + 0.5) / r - 0.5) * u.h);\n\
             \x20   }",
        "",
    );
    let lerp = module(
        "@group(0) @binding(1) var<storage, read> raw: array<vec2<f32>>;\n\
             @group(0) @binding(2) var<storage, read> cur: array<vec2<f32>>;\n\
             @group(0) @binding(3) var<storage, read> relax: array<f32>;\n\
             @group(0) @binding(4) var<storage, read_write> outp: array<vec2<f32>>;",
        "\x20   let t = clamp(relax[0], 0.0, 1.0);\n\
             \x20   outp[i] = raw[i] + (cur[i] - raw[i]) * t;",
        "",
    );
    [
        ("ph2d-voronoi seed", seed),
        ("ph2d-voronoi grid-init", grid_init),
        ("ph2d-voronoi jfa", jfa),
        ("ph2d-voronoi reduce", reduce),
        ("ph2d-voronoi move", mv),
        ("ph2d-voronoi lerp", lerp),
    ]
}

impl VoronoiPipes {
    fn new(gpu: &GpuContext) -> Self {
        let [seed, grid_init, jfa, reduce, mv, lerp] =
            sources().map(|(label, src)| create_pipeline(gpu, &src, label));
        VoronoiPipes {
            seed,
            grid_init,
            jfa,
            reduce,
            mv,
            lerp,
        }
    }

    /// Encode one pass: the shared uniform + `buffers`, dispatched over `n`.
    /// The uniform is a fresh 32-byte buffer per dispatch (the stream-op
    /// pattern); it lands in `hold` so it outlives the cook's final submit.
    #[allow(clippy::too_many_arguments)] // private seam, mirrors StreamOpPipes::pass
    fn pass(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::ComputePipeline,
        uni_bytes: [u8; 32],
        n: u32,
        buffers: &[&wgpu::Buffer],
        hold: &mut Vec<wgpu::Buffer>,
    ) {
        let uni = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-voronoi u"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&uni, 0, &uni_bytes);
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: uni.as_entire_binding(),
        }];
        for (k, b) in buffers.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry {
                binding: (k + 1) as u32,
                resource: b.as_entire_binding(),
            });
        }
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-voronoi"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-voronoi"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(n.div_ceil(WG), 1, 1);
        }
        hold.push(uni);
    }

    /// The jump-flood offset schedule for a `res`-sided grid: a **1+JFA**
    /// prologue (one step-1 pass, the standard cheap error reducer) then the
    /// halving sequence `P/2, P/4, …, 1` over the next power of two.
    fn jfa_steps(res: usize) -> Vec<u32> {
        let mut steps = vec![1u32];
        let mut s = (res.next_power_of_two() / 2).max(1) as u32;
        loop {
            steps.push(s);
            if s == 1 {
                break;
            }
            s /= 2;
        }
        steps
    }
}

impl GpuCook {
    /// Dispatch on the algorithm family (ADR-0139) — the single seam `cook`
    /// calls, so a second algorithm is one match arm, not a second seam.
    #[allow(clippy::too_many_arguments)] // private seam of `cook`
    pub(crate) fn encode_algorithm(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        alg: &GpuAlgorithm,
        graph: &Graph,
        node: NodeId,
        manifest: &'static NodeManifest,
        inputs: &[GpuStream],
    ) -> GpuStream {
        match alg {
            GpuAlgorithm::LloydVoronoi { .. } => {
                self.encode_lloyd_voronoi(gpu, encoder, alg, graph, node, manifest, inputs)
            }
        }
    }

    /// The Lloyd/JFA cook (module docs). Every pass in the caller's encoder;
    /// the output is one `P` column at the param law's count.
    #[allow(clippy::too_many_arguments)] // private seam of `cook`
    fn encode_lloyd_voronoi(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        alg: &GpuAlgorithm,
        graph: &Graph,
        node: NodeId,
        manifest: &'static NodeManifest,
        inputs: &[GpuStream],
    ) -> GpuStream {
        let GpuAlgorithm::LloydVoronoi {
            count_param,
            width_param,
            height_param,
            seed_param,
            iterations_param,
            relax_port,
            max_points,
            min_res,
            max_res,
            samples_per_point,
            max_iterations,
        } = alg;
        // The node's own param laws, to the letter (`motion.voronoi::eval`).
        let count = param_as_count(
            resolve_param(graph, node, manifest, count_param),
            *max_points,
        );
        if count == 0 {
            return GpuStream::default();
        }
        let w = resolve_param(graph, node, manifest, width_param).max(1e-3);
        let h = resolve_param(graph, node, manifest, height_param).max(1e-3);
        let seed = resolve_param(graph, node, manifest, seed_param)
            .max(0.0)
            .round() as u32;
        let iterations = (resolve_param(graph, node, manifest, iterations_param).round() as i64)
            .clamp(0, *max_iterations) as usize;
        let res = GpuAlgorithm::lloyd_resolution(count, *samples_per_point, *min_res, *max_res)
            .min(INT_CENTROID_RES_CEILING);

        let n = count as u32;
        let cells = (res * res) as u32;
        let raw = self.pool.acquire(gpu, u64::from(n) * 8);
        let cur = self.pool.acquire(gpu, u64::from(n) * 8);
        let grid_a = self.pool.acquire(gpu, u64::from(cells) * 4);
        let grid_b = self.pool.acquire(gpu, u64::from(cells) * 4);
        let acc = self.pool.acquire(gpu, u64::from(n) * 12);
        let out = self.pool.acquire(gpu, u64::from(n) * 8);

        let mut hold: Vec<wgpu::Buffer> = Vec::new();
        let uni =
            |dispatch: u32, step: u32| uniform_bytes(dispatch, res as u32, step, n, w, h, seed);
        let pipes = self
            .voronoi_pipes
            .get_or_insert_with(|| VoronoiPipes::new(gpu));

        pipes.pass(
            gpu,
            encoder,
            &pipes.seed,
            uni(n, 0),
            n,
            &[&raw, &cur],
            &mut hold,
        );
        for _ in 0..iterations {
            // (a) Seed the grid from the CURRENT positions (0 = empty).
            encoder.clear_buffer(&grid_a, 0, Some(u64::from(cells) * 4));
            pipes.pass(
                gpu,
                encoder,
                &pipes.grid_init,
                uni(n, 0),
                n,
                &[&cur, &grid_a],
                &mut hold,
            );
            // (b) Flood: ping-pong A→B→A…; `src` tracks who holds the answer.
            let mut src = &grid_a;
            let mut dst = &grid_b;
            for step in VoronoiPipes::jfa_steps(res) {
                pipes.pass(
                    gpu,
                    encoder,
                    &pipes.jfa,
                    uni(cells, step),
                    cells,
                    &[&cur, src, dst],
                    &mut hold,
                );
                std::mem::swap(&mut src, &mut dst);
            }
            // (c) Integer centroid reduce over the final assignment.
            encoder.clear_buffer(&acc, 0, Some(u64::from(n) * 12));
            pipes.pass(
                gpu,
                encoder,
                &pipes.reduce,
                uni(cells, 0),
                cells,
                &[src, &acc],
                &mut hold,
            );
            // (d) Move each point to its cell's centroid (in place: the
            //     dispatch-order guarantee sequences these passes).
            pipes.pass(
                gpu,
                encoder,
                &pipes.mv,
                uni(n, 0),
                n,
                &[&acc, &cur],
                &mut hold,
            );
        }

        // The relax VALUE at row 0, on the device — absent (or empty) reads
        // 1.0, the CPU's `v.first().unwrap_or(1.0)`.
        let relax = match inputs
            .get(*relax_port)
            .filter(|s| s.count > 0)
            .and_then(|s| s.cols.get(VALUE_COL))
        {
            Some(c) => c.buffer.clone(),
            None => {
                let one = self.pool.acquire(gpu, 4);
                gpu.queue.write_buffer(&one, 0, &1.0f32.to_le_bytes());
                one
            }
        };
        pipes.pass(
            gpu,
            encoder,
            &pipes.lerp,
            uni(n, 0),
            n,
            &[&raw, &cur, &relax, &out],
            &mut hold,
        );
        self.stream_op_hold.append(&mut hold);

        let mut cols = BTreeMap::new();
        cols.insert(
            "P".to_string(),
            GpuColumn {
                buffer: out,
                dim: ph2d_nodegraph::port::Dim::Vec2,
            },
        );
        GpuStream { count: n, cols }
    }
}

/// **Gates only** — the assignment oracle's window (the `debug_read` role):
/// run grid-init + the flood over `points` at `res` and read the owner grid
/// back. Returns one owner id per texel, row-major (`u32::MAX` = unowned,
/// which a flooded grid never leaves behind). The product path never calls
/// this; the parity gate compares it against the CPU's exact `nearest`.
pub fn jfa_assignment(
    gpu: &GpuContext,
    points: &[[f32; 2]],
    w: f32,
    h: f32,
    res: usize,
) -> Vec<u32> {
    let n = points.len() as u32;
    let cells = (res * res) as u32;
    let pipes = VoronoiPipes::new(gpu);
    let mk = |bytes: u64, usage: wgpu::BufferUsages| {
        gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-voronoi debug"),
            size: bytes,
            usage,
            mapped_at_creation: false,
        })
    };
    let storage =
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
    let pts = mk(u64::from(n) * 8, storage);
    gpu.queue
        .write_buffer(&pts, 0, bytemuck::cast_slice(points));
    let grid_a = mk(u64::from(cells) * 4, storage);
    let grid_b = mk(u64::from(cells) * 4, storage);
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.clear_buffer(&grid_a, 0, None);
    let mut hold = Vec::new();
    let uni = |dispatch: u32, step: u32| uniform_bytes(dispatch, res as u32, step, n, w, h, 0);
    pipes.pass(
        gpu,
        &mut encoder,
        &pipes.grid_init,
        uni(n, 0),
        n,
        &[&pts, &grid_a],
        &mut hold,
    );
    let mut src = &grid_a;
    let mut dst = &grid_b;
    for step in VoronoiPipes::jfa_steps(res) {
        pipes.pass(
            gpu,
            &mut encoder,
            &pipes.jfa,
            uni(cells, step),
            cells,
            &[&pts, src, dst],
            &mut hold,
        );
        std::mem::swap(&mut src, &mut dst);
    }
    let staging = mk(
        u64::from(cells) * 4,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    );
    encoder.copy_buffer_to_buffer(src, 0, &staging, 0, u64::from(cells) * 4);
    gpu.queue.submit(Some(encoder.finish()));
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let keys: Vec<u32> = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
    staging.unmap();
    keys.iter()
        .map(|&k| if k == 0 { u32::MAX } else { n - k })
        .collect()
}

#[cfg(test)]
mod tests {
    /// Every voronoi module parses and validates without a device (the
    /// `sprite_wgsl_valid` pattern) — a WGSL typo otherwise only surfaces on
    /// the RTX gates, which CI lanes without an adapter never run.
    #[test]
    fn every_voronoi_module_is_valid_wgsl() {
        for (label, src) in super::sources() {
            let module = naga::front::wgsl::parse_str(&src)
                .unwrap_or_else(|e| panic!("{label}: parse failed:\n{e}\n---\n{src}"));
            naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            )
            .validate(&module)
            .unwrap_or_else(|e| panic!("{label}: validation failed: {e:?}"));
        }
    }
}
