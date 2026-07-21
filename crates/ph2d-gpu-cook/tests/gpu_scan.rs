//! **GPU exclusive-scan parity** (ADR-0134, Phase 1a) — the reusable prefix-sum
//! the spatial-hash counting-sort is built on, gated ALONE before any grid.
//!
//! The scan is INTEGER, so unlike the float kernels this is **bit-exact**, not
//! ε: the device must produce EXACTLY the CPU's exclusive prefix-sum. The lengths
//! straddle every structural boundary — a partial final block, exactly one full
//! block (no recursion), the two-level boundary at `256²`, and a million (three
//! recursion levels) — because an off-by-one in the block seam or the recursion
//! is invisible on a single block and fatal on many.
//!
//! `#[ignore]`: needs a real adapter. Run on the GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_scan --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::scan::{Scan, ScanScratch, cpu_exclusive};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// A deterministic small-valued input (0..256 per element, so the total of a
/// million stays well under `u32::MAX` — the scan is exact, not modular, in range).
fn sample(len: usize) -> Vec<u32> {
    (0..len)
        .map(|i| ((i as u64).wrapping_mul(2_654_435_761) >> 24) as u32 & 0xFF)
        .collect()
}

/// Run the GPU exclusive scan and read the result back.
fn gpu_scan(gpu: &GpuContext, input: &[u32]) -> Vec<u32> {
    let n = input.len() as u32;
    if n == 0 {
        return Vec::new();
    }
    let bytes = u64::from(n) * 4;
    let data = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scan data"),
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    gpu.queue
        .write_buffer(&data, 0, bytemuck::cast_slice(input));

    let scan = Scan::new(gpu);
    let mut scratch = ScanScratch::default();
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    scan.exclusive(gpu, &mut encoder, &data, n, &mut scratch);

    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scan readback"),
        size: bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&data, 0, &staging, 0, bytes);
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
    let out: Vec<u32> = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
    staging.unmap();
    // `scratch` drops here — after the GPU has finished (we polled to completion).
    out
}

#[test]
#[ignore = "needs a GPU adapter"]
fn gpu_exclusive_scan_matches_the_cpu_bit_for_bit() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_scan");
        return;
    };
    // 255/257 straddle the first block seam; 256 is exactly one block (no
    // recursion); 65 536/65 537 straddle the two-level boundary (256²); a million
    // forces three recursion levels.
    for &len in &[
        0usize, 1, 2, 5, 255, 256, 257, 512, 1000, 65_536, 65_537, 300_000, 1_000_000,
    ] {
        let input = sample(len);
        let cpu = cpu_exclusive(&input);
        let gpu_out = gpu_scan(&gpu, &input);
        assert_eq!(gpu_out.len(), cpu.len(), "length mismatch at len {len}");
        assert_eq!(gpu_out, cpu, "exclusive scan mismatch at len {len}");
    }
}

/// **The scan past the dispatch-dimension limit** (the tetos slice, §0.0): a
/// bucket scan at 8 M elements is `2²⁴ + 1` entries = 65 537 blocks — past the
/// 65 535 workgroups-per-dimension cap that was the sim's first ceiling. The
/// 2-D dispatch folds the blocks into a rectangle and the kernels linearise
/// the workgroup id; this drives the EXACT product shape and reconciles it
/// bit-for-bit against the CPU oracle (integer scan — no ε).
///
/// The small-size sweep above cannot catch a broken fold: every length it
/// tries fits one dimension, so `dispatch_2d` degenerates to the old shape.
#[test]
#[ignore = "needs a GPU adapter; ~130 MB of buffers"]
fn the_scan_survives_past_the_dispatch_dimension_limit() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping gpu_scan");
        return;
    };
    // 2²⁴ + 1: the `num_buckets + 1` of an 8 M-element neighbourhood grid.
    let len = (1usize << 24) + 1;
    let input = sample(len);
    let cpu = cpu_exclusive(&input);
    let gpu_out = gpu_scan(&gpu, &input);
    assert_eq!(gpu_out, cpu, "exclusive scan mismatch at len {len}");
    eprintln!("scan of {len} entries (65 537 blocks, 2-D dispatch) bit-exact");
}
