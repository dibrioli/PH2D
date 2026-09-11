//! **Spatial-hash grid parity** (ADR-0140, Phase 1b/2) — the neighbourhood
//! structure boids/collide/SPH share, gated standalone before any node wiring.
//!
//! The intra-bucket order is a non-deterministic atomic race (and unspecified on
//! purpose — the query sums over a bucket), so the oracle checks the three facts
//! that DON'T depend on order and together pin the whole count→scan→scatter:
//!   1. `starts` is bit-exact the CPU's exclusive scan of the bucket counts;
//!   2. `sorted` is a permutation of `0..n` (every element placed exactly once);
//!   3. every element in bucket `b`'s slot range hashes to `b` (scatter is sound).
//!
//! `#[ignore]`: needs an adapter. Run on the GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_grid --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::grid::{Grid, GridScratch, cpu_bucket, num_buckets};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// Deterministic positions in a ±20 box (so cell=1 yields ~1600 cells → many
/// multi-element buckets and hash collisions, the cases that break a naive build).
fn positions(n: usize) -> Vec<[f32; 2]> {
    let h = |i: u64, k: u64| -> f32 {
        let mut x = i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ k.wrapping_mul(0xD1B5_4A32_D192_ED03);
        x ^= x >> 33;
        x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
        x ^= x >> 33;
        (x as f64 / u64::MAX as f64) as f32
    };
    (0..n)
        .map(|i| [(h(i as u64, 0) - 0.5) * 40.0, (h(i as u64, 1) - 0.5) * 40.0])
        .collect()
}

fn readback_u32(gpu: &GpuContext, buf: &wgpu::Buffer, count: u32) -> Vec<u32> {
    if count == 0 {
        return Vec::new();
    }
    let bytes = u64::from(count) * 4;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("grid readback"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_buffer_to_buffer(buf, 0, &staging, 0, bytes);
    gpu.queue.submit(Some(enc.finish()));
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().unwrap().unwrap();
    let out: Vec<u32> = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
    staging.unmap();
    out
}

/// The CPU oracle: bucket counts → exclusive scan (`num_buckets+1` offsets).
fn cpu_starts(pos: &[[f32; 2]], cell: f32, nb: u32) -> (Vec<u32>, Vec<u32>) {
    let buckets: Vec<u32> = pos.iter().map(|&p| cpu_bucket(p, cell, nb)).collect();
    let mut starts = vec![0u32; nb as usize + 1];
    for &b in &buckets {
        starts[b as usize] += 1;
    }
    let mut acc = 0u32;
    for s in starts.iter_mut() {
        let c = *s;
        *s = acc;
        acc += c;
    }
    (starts, buckets)
}

#[test]
#[ignore = "needs a GPU adapter"]
fn the_grid_bins_every_element_into_its_bucket() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_grid");
        return;
    };
    let cell = 1.0f32;
    let grid = Grid::new(&gpu);

    // 300_000 forces the scan's recursion (num_buckets = 1_048_576).
    for &n in &[1usize, 5, 100, 1000, 50_000, 300_000] {
        let pos = positions(n);
        let nb = num_buckets(n as u32);
        let (cpu_off, cpu_buckets) = cpu_starts(&pos, cell, nb);

        // Upload positions and build on the device.
        let mut raw = Vec::with_capacity(n * 8);
        for p in &pos {
            raw.extend_from_slice(&p[0].to_le_bytes());
            raw.extend_from_slice(&p[1].to_le_bytes());
        }
        let pos_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grid pos"),
            size: (n.max(1) * 8) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if n > 0 {
            gpu.queue.write_buffer(&pos_buf, 0, &raw);
        }
        let mut scratch = GridScratch::default();
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let g = grid.build(&gpu, &mut enc, &pos_buf, n as u32, cell, &mut scratch);
        gpu.queue.submit(Some(enc.finish()));

        let gpu_starts = readback_u32(&gpu, &g.starts, nb + 1);
        let sorted = readback_u32(&gpu, &g.sorted, n as u32);

        // (1) offsets are exactly the CPU exclusive scan.
        assert_eq!(gpu_starts, cpu_off, "starts mismatch at n={n}");
        assert_eq!(
            gpu_starts[nb as usize], n as u32,
            "total offset is n, at n={n}"
        );

        // (2) sorted is a permutation of 0..n.
        let mut seen = sorted.clone();
        seen.sort_unstable();
        assert_eq!(
            seen,
            (0..n as u32).collect::<Vec<_>>(),
            "sorted is not a permutation at n={n}"
        );

        // (3) every element sits in its own bucket's range.
        for b in 0..nb as usize {
            for &e in &sorted[gpu_starts[b] as usize..gpu_starts[b + 1] as usize] {
                assert_eq!(
                    cpu_buckets[e as usize], b as u32,
                    "element {e} in bucket {b}'s range but hashes to {} (n={n})",
                    cpu_buckets[e as usize]
                );
            }
        }
    }
}
