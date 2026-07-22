//! **The number that justifies the reduction** — GPU whole-stream reduce vs the
//! CPU fold, measured, at the sizes the deformers will actually see.
//!
//! This is the *smoke* of a primitive with no pixels: `ph2d-gpu-cook::reduce` has
//! no artist-visible surface of its own (the deformers that consume it are the
//! next slice), so the thing to look at is its throughput. Run it and read the
//! table:
//!
//! ```text
//! cargo test -p ph2d-gpu-cook --test gpu_reduce_perf --release -- --ignored --nocapture
//! ```
//!
//! ⚠️ **`--release` is not a preference.** The CPU column is a `fold` over an
//! `f32` slice; in a debug build it is ~an order of magnitude slower than the
//! code that ships, which would flatter the GPU for the wrong reason. A
//! measurement taken in the wrong profile is not a measurement.
//!
//! ⚠️ **What this deliberately does NOT claim.** The GPU column is *encode +
//! submit + wait*, i.e. it PAYS the full round trip for a single reduction. That
//! is the honest number for a standalone call and the pessimistic one for the
//! real client: in a cook, the reduction is one pass among many inside a single
//! already-submitted encoder, so its marginal cost is the dispatch alone. Read
//! the crossover below as "past here the device wins even paying for the whole
//! trip", never as the deformers' budget — that gets measured on the deformers.
//!
//! There is **no assertion on wall-clock**: this is a probe, and a timing bar
//! here would flake on machine load while telling nobody anything they could act
//! on. The correctness bar lives in `gpu_reduce.rs`, where it is bit-exact.

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::reduce::{Reduce, ReduceOp, ReduceScratch};
use std::time::Instant;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn sample(len: usize) -> Vec<f32> {
    (0..len)
        .map(|i| {
            let h = (i as u64).wrapping_mul(2_654_435_761) >> 11;
            (h % 100_000) as f32 * 0.001 - 50.0
        })
        .collect()
}

/// One full standalone reduction: upload is EXCLUDED (the real client's column
/// is already resident on the device — uploading it here would measure the PCIe
/// bus, not the reduction), encode + submit + wait are included.
fn gpu_once(gpu: &GpuContext, reduce: &Reduce, data: &wgpu::Buffer, n: u32) -> f32 {
    let out = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("perf out"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("perf readback"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut scratch = ReduceScratch::default();
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    reduce.reduce_into(
        gpu,
        &mut encoder,
        ReduceOp::Max,
        data,
        n,
        &out,
        &mut scratch,
    );
    encoder.copy_buffer_to_buffer(&out, 0, &readback, 0, 4);
    gpu.queue.submit(Some(encoder.finish()));
    drop(scratch);
    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let v = bytemuck::cast_slice::<u8, f32>(&slice.get_mapped_range())[0];
    readback.unmap();
    v
}

#[test]
#[ignore = "needs a GPU adapter; measurement, not a bar"]
fn reduce_throughput_gpu_vs_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reduce = Reduce::new(&gpu);

    println!(
        "\n=== whole-stream Max: GPU (encode+submit+wait) vs CPU fold ===\n\
         {:>12}  {:>12}  {:>12}  {:>9}",
        "elements", "cpu (ms)", "gpu (ms)", "speedup"
    );

    for &n in &[1_024usize, 16_384, 262_144, 1_048_576, 4_194_304] {
        let input = sample(n);
        let data = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("perf data"),
            size: (input.len() * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue
            .write_buffer(&data, 0, bytemuck::cast_slice(&input));

        // Warm the pipeline/allocator so the first sample is not the outlier.
        let warm = gpu_once(&gpu, &reduce, &data, n as u32);
        let want = ReduceOp::Max.cpu(&input);
        assert_eq!(
            warm.to_bits(),
            want.to_bits(),
            "the probe must still be measuring the RIGHT answer at {n}"
        );

        const REPS: u32 = 20;
        let t0 = Instant::now();
        for _ in 0..REPS {
            std::hint::black_box(gpu_once(&gpu, &reduce, &data, n as u32));
        }
        let gpu_ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);

        let t1 = Instant::now();
        for _ in 0..REPS {
            std::hint::black_box(ReduceOp::Max.cpu(std::hint::black_box(&input)));
        }
        let cpu_ms = t1.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);

        println!(
            "{n:>12}  {cpu_ms:>12.4}  {gpu_ms:>12.4}  {:>8.2}×",
            cpu_ms / gpu_ms
        );
    }
    println!(
        "\nRead: the GPU column pays a full round trip per call. Inside a cook the\n\
         reduction is one pass in an encoder that is submitted anyway, so its\n\
         marginal cost is the dispatch — this table is the pessimistic bound.\n"
    );
}
