//! **GPU whole-stream reduction parity** — the reusable primitive the deformer
//! family (`bend`/`twist`/`spherize`) needs, gated ALONE before any node is
//! wired to it, exactly as the scan was gated before any grid.
//!
//! **`Max`/`Min` are asserted BIT-EXACT, and that is not optimism:** they are
//! associative *and exact* over floats, so the tree order the device uses and
//! the sequential order the CPU uses provably agree (there is a CPU-only unit
//! test in `reduce.rs` pinning that property on its own). `Sum` is asserted to a
//! documented relative ε, because float addition is not associative — claiming
//! bit-parity there would be a gate that fails for being right.
//!
//! The lengths straddle every structural boundary, because an off-by-one in the
//! block seam or the recursion is invisible on a single block and fatal on many:
//! a partial final block, exactly one full block (no recursion at all), the
//! two-level boundary at `256²`, and past it.
//!
//! `#[ignore]`: needs a real adapter. Run on the GPU lane:
//!   cargo test -p ph2d-gpu-cook --test gpu_reduce --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::reduce::{Reduce, ReduceOp, ReduceScratch};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// A deterministic input, shifted by `bias`. No duplicate extremes, so a wrong
/// answer cannot coincide with the right one.
///
/// ⚠️ **`bias` is load-bearing, and a mutation proved it.** With the
/// zero-straddling sample alone (`bias = 0`) this gate stayed GREEN under a
/// broken recursion: a short second level reads zeros past its end, and `max`
/// **absorbs** spurious zeros whenever the true answer is positive. The gate was
/// green over a reduction that only worked for one block. Biasing the whole
/// column below zero makes those same zeros *win*, and above zero makes them win
/// for `min` — so [`BIASES`] is what turns "the fixture contains the phenomenon"
/// from a hope into a fact.
fn sample(len: usize, bias: f32) -> Vec<f32> {
    (0..len)
        .map(|i| {
            let h = (i as u64).wrapping_mul(2_654_435_761) >> 11;
            (h % 100_000) as f32 * 0.001 - 50.0 + bias
        })
        .collect()
}

/// Straddling zero (the realistic case), entirely BELOW it (where a stray `0`
/// beats every real element under `max`), and entirely ABOVE it (same, for
/// `min`). Every operator has a bias here that cannot forgive a stray identity.
const BIASES: [f32; 3] = [0.0, -1_000.0, 1_000.0];

/// Run the GPU reduction and read the single result back.
fn gpu_reduce(gpu: &GpuContext, op: ReduceOp, input: &[f32]) -> f32 {
    let n = input.len() as u32;
    let data = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("reduce data"),
        size: (u64::from(n) * 4).max(4),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    gpu.queue
        .write_buffer(&data, 0, bytemuck::cast_slice(input));

    let out = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("reduce out"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let reduce = Reduce::new(gpu);
    let mut scratch = ReduceScratch::default();
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    reduce.reduce_into(gpu, &mut encoder, op, &data, n, &out, &mut scratch);

    let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("reduce readback"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
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

/// The structural lengths: a partial final block, exactly one block (the
/// recursion's base case and the ONLY case for a small stream), just past one
/// block (the first time a second level runs at all), the two-level boundary,
/// and past it.
const LENGTHS: [usize; 7] = [1, 7, 255, 256, 257, 65_536, 70_001];

#[test]
#[ignore = "needs a GPU adapter"]
fn gpu_max_and_min_match_the_cpu_bit_for_bit() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    for op in [ReduceOp::Max, ReduceOp::Min] {
        for bias in BIASES {
            for len in LENGTHS {
                let input = sample(len, bias);
                let want = op.cpu(&input);
                let got = gpu_reduce(&gpu, op, &input);
                assert_eq!(
                    want.to_bits(),
                    got.to_bits(),
                    "{op:?} over {len} elements (bias {bias}): \
                     cpu {want} != gpu {got} (BIT-exact is the claim)"
                );
            }
        }
        println!("{op:?}: bit-exact over {LENGTHS:?} × biases {BIASES:?}");
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn gpu_sum_matches_the_cpu_within_the_documented_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    // Float addition is not associative, so this is a RELATIVE bound against the
    // magnitude actually summed (the naive absolute-ε gate would be vacuous at
    // 70 000 elements and impossible at 1).
    for len in LENGTHS {
        let input = sample(len, 0.0);
        let want = ReduceOp::Sum.cpu(&input);
        let got = gpu_reduce(&gpu, ReduceOp::Sum, &input);
        let scale: f32 = input.iter().map(|v| v.abs()).sum::<f32>().max(1.0);
        let rel = (want - got).abs() / scale;
        assert!(
            rel < 1e-6,
            "Sum over {len}: cpu {want} vs gpu {got} — relative {rel:e} exceeds 1e-6"
        );
        println!("Sum over {len:>6}: cpu {want:>14.4} gpu {got:>14.4} rel {rel:e}");
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn an_all_negative_column_reports_its_own_maximum_not_zero() {
    // The identity bug this primitive's docs name, driven end to end: seeded with
    // `0.0` instead of the operator's identity, a `Max` over an all-negative
    // column answers 0 — plausible, wrong, and agreed with by every fixture that
    // happens to contain a positive number. `sample()` straddles zero on purpose;
    // this one does not straddle at all, which is what makes it the control.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    // 300 elements => two blocks => the recursion runs too, so the identity has
    // to be right at BOTH levels.
    let input: Vec<f32> = (0..300).map(|i| -1.0 - (i as f32) * 0.25).collect();
    let got = gpu_reduce(&gpu, ReduceOp::Max, &input);
    assert_eq!(
        got, -1.0,
        "max of an all-negative column is its largest element"
    );
    let got_min = gpu_reduce(&gpu, ReduceOp::Min, &input);
    assert_eq!(got_min, -1.0 - 299.0 * 0.25);
}

#[test]
#[ignore = "needs a GPU adapter"]
fn an_empty_stream_leaves_the_callers_slot_untouched() {
    // `n == 0` writes nothing, by contract: there is no identity to publish that
    // the caller could not have supplied, and writing one would let an empty
    // stream masquerade as a measured extent. The deformers rely on this to seed
    // their own degenerate-case value.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let sentinel: f32 = 12.5;
    let data = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("empty data"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let out = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("empty out"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    gpu.queue.write_buffer(&out, 0, &sentinel.to_le_bytes());

    let reduce = Reduce::new(&gpu);
    let mut scratch = ReduceScratch::default();
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    reduce.reduce_into(
        &gpu,
        &mut encoder,
        ReduceOp::Max,
        &data,
        0,
        &out,
        &mut scratch,
    );

    let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("empty readback"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&out, 0, &readback, 0, 4);
    gpu.queue.submit(Some(encoder.finish()));
    drop(scratch);

    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let v = bytemuck::cast_slice::<u8, f32>(&slice.get_mapped_range())[0];
    readback.unmap();
    assert_eq!(v, sentinel, "an empty reduction must not write");
}
