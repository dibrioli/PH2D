//! CPU/GPU parity tests for the sharpen pipelines — split out of
//! `gpu/sharpen`. Each test no-ops when no headless adapter is available
//! (CI runners without Metal/Vulkan).

use super::{sharpen_laplacian_gpu, sharpen_unsharp_gpu};
use crate::algorithm::{sharpen_laplacian, sharpen_unsharp};
use crate::gpu::try_headless_gpu;

fn ramp(w: u32, h: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 13 + y * 5) % 256) as u8;
            let g = ((x * 7 + y * 19) % 256) as u8;
            let b = ((x * 17 + y * 3) % 256) as u8;
            v.extend_from_slice(&[r, g, b, 255]);
        }
    }
    v
}

/// Sharpening shines on edges — manufacture a deterministic image
/// with a checkerboard of `lo` / `hi` value bands so the kernel
/// has work to do at every interior pixel.
fn checker(w: u32, h: u32, block: u32, lo: u8, hi: u8) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let on = ((x / block) + (y / block)).is_multiple_of(2);
            let val = if on { hi } else { lo };
            v.extend_from_slice(&[val, val, val, 255]);
        }
    }
    v
}

fn assert_within_lsb(cpu: &[u8], gpu: &[u8], max_lsb: i32, ctx: &str) {
    assert_eq!(cpu.len(), gpu.len());
    let mut worst = 0_i32;
    let mut worst_idx = 0;
    for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
        let d = (*a as i32 - *b as i32).abs();
        if d > worst {
            worst = d;
            worst_idx = i;
        }
    }
    assert!(
        worst <= max_lsb,
        "{ctx}: CPU/GPU diverged by {worst} LSB at idx {worst_idx} \
         (cpu {} vs gpu {})",
        cpu[worst_idx],
        gpu[worst_idx],
    );
}

// ── Laplacian ───────────────────────────────────────────────

#[test]
fn laplacian_gpu_zero_amount_is_noop() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(16, 16);
    let mut buf = src.clone();
    sharpen_laplacian_gpu(&mut buf, 16, 16, 0.0, &gpu);
    assert_eq!(buf, src);
}

#[test]
fn laplacian_gpu_matches_cpu_amount_one() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = checker(24, 24, 4, 60, 200);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    sharpen_laplacian(&mut cpu, 24, 24, 1.0);
    sharpen_laplacian_gpu(&mut gpu_buf, 24, 24, 1.0, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 2, "laplacian amount=1 checker");
}

#[test]
fn laplacian_gpu_matches_cpu_amount_half() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    sharpen_laplacian(&mut cpu, 24, 24, 0.5);
    sharpen_laplacian_gpu(&mut gpu_buf, 24, 24, 0.5, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 2, "laplacian amount=0.5 ramp");
}

#[test]
fn laplacian_gpu_skips_transparent() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut buf = vec![100u8, 150, 200, 0, 100, 150, 200, 255];
    sharpen_laplacian_gpu(&mut buf, 2, 1, 1.0, &gpu);
    assert_eq!(&buf[0..4], &[100, 150, 200, 0]);
}

#[test]
fn laplacian_gpu_handles_non_workgroup_aligned_dimensions() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = checker(13, 19, 3, 40, 220);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    sharpen_laplacian(&mut cpu, 13, 19, 0.8);
    sharpen_laplacian_gpu(&mut gpu_buf, 13, 19, 0.8, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 2, "laplacian 13×19");
}

// ── Unsharp Mask ────────────────────────────────────────────

#[test]
fn unsharp_gpu_zero_amount_is_noop() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(16, 16);
    let mut buf = src.clone();
    sharpen_unsharp_gpu(&mut buf, 16, 16, 0.0, 2.0, &gpu);
    assert_eq!(buf, src);
}

#[test]
fn unsharp_gpu_matches_cpu_radius_2() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    // Use checker so the unsharp combine has visible structure.
    let src = checker(24, 24, 4, 50, 210);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    sharpen_unsharp(&mut cpu, 24, 24, 1.0, 2.0);
    sharpen_unsharp_gpu(&mut gpu_buf, 24, 24, 1.0, 2.0, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 4, "unsharp r=2 amount=1");
}

#[test]
fn unsharp_gpu_matches_cpu_radius_3() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(32, 32);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    sharpen_unsharp(&mut cpu, 32, 32, 0.7, 3.0);
    sharpen_unsharp_gpu(&mut gpu_buf, 32, 32, 0.7, 3.0, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 4, "unsharp r=3 amount=0.7");
}

#[test]
fn unsharp_gpu_matches_cpu_small_radius() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    // radius=1.5 → kernel size 7 (smallest non-Laplacian path).
    let src = checker(20, 20, 4, 30, 240);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    sharpen_unsharp(&mut cpu, 20, 20, 1.2, 1.5);
    sharpen_unsharp_gpu(&mut gpu_buf, 20, 20, 1.2, 1.5, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 4, "unsharp r=1.5 amount=1.2");
}

#[test]
fn unsharp_gpu_skips_transparent() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut buf = vec![100u8, 150, 200, 0, 100, 150, 200, 255];
    sharpen_unsharp_gpu(&mut buf, 2, 1, 1.0, 2.0, &gpu);
    assert_eq!(&buf[0..4], &[100, 150, 200, 0]);
}

#[test]
fn unsharp_gpu_handles_non_workgroup_aligned_dimensions() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = checker(13, 19, 3, 40, 220);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    sharpen_unsharp(&mut cpu, 13, 19, 0.9, 2.0);
    sharpen_unsharp_gpu(&mut gpu_buf, 13, 19, 0.9, 2.0, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 4, "unsharp 13×19");
}
