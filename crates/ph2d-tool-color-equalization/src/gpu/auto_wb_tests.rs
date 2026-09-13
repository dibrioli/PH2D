//! Testes de `auto_wb.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

use super::*;
use crate::algorithm::auto_white_balance;
use crate::gpu::try_headless_gpu;

fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..(w * h) {
        v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
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

#[test]
fn auto_wb_gpu_all_transparent_is_noop() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut src = vec![100_u8, 150, 200, 0]; // alpha 0
    src.extend_from_slice(&[200, 80, 40, 0]);
    let mut buf = src.clone();
    auto_white_balance_gpu(&mut buf, 2, 1, &gpu);
    assert_eq!(buf, src);
}

#[test]
fn auto_wb_gpu_pure_grey_is_near_noop() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = solid(16, 16, [128, 128, 128]);
    let mut buf = src.clone();
    auto_white_balance_gpu(&mut buf, 16, 16, &gpu);
    // gains all equal 1 → identity (within rounding).
    for (a, b) in buf.iter().zip(src.iter()) {
        assert!(a.abs_diff(*b) <= 1);
    }
}

#[test]
fn auto_wb_gpu_matches_cpu_solid_red_cast() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = solid(16, 16, [200, 100, 100]);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    auto_white_balance(&mut cpu);
    auto_white_balance_gpu(&mut gpu_buf, 16, 16, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 2, "solid red cast 200/100/100");
}

#[test]
fn auto_wb_gpu_matches_cpu_solid_blue_cast() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = solid(16, 16, [80, 100, 220]);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    auto_white_balance(&mut cpu);
    auto_white_balance_gpu(&mut gpu_buf, 16, 16, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 2, "solid blue cast 80/100/220");
}

#[test]
fn auto_wb_gpu_matches_cpu_diverse_input() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    // Varied per-pixel input — exercises the reduce over many bins.
    let mut src = Vec::with_capacity(32 * 32 * 4);
    for y in 0..32_u32 {
        for x in 0..32_u32 {
            let r = ((x * 11 + y * 5) % 256) as u8;
            let g = ((x * 7 + y * 13) % 256) as u8;
            let b = ((x * 3 + y * 17) % 256) as u8;
            src.extend_from_slice(&[r, g, b, 255]);
        }
    }
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    auto_white_balance(&mut cpu);
    auto_white_balance_gpu(&mut gpu_buf, 32, 32, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 2, "diverse 32×32 ramp");
}

#[test]
fn auto_wb_gpu_matches_cpu_with_transparent_border() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    // 16×16 with a transparent 2-px border and a red-tinted core —
    // the CPU reduce skips alpha=0 pixels; GPU must do the same so
    // the gains land on the opaque-pixel mean only.
    let mut src = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16_u32 {
        for x in 0..16_u32 {
            let opaque = (2..14).contains(&x) && (2..14).contains(&y);
            let alpha: u8 = if opaque { 255 } else { 0 };
            src.extend_from_slice(&[200, 100, 100, alpha]);
        }
    }
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    auto_white_balance(&mut cpu);
    auto_white_balance_gpu(&mut gpu_buf, 16, 16, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 2, "transparent border + red core");
}

#[test]
fn auto_wb_gpu_skips_when_channel_mean_is_zero() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    // Pure red channel — green + blue means are zero → both CPU
    // and GPU short-circuit (no rescale).
    let src = solid(8, 8, [200, 0, 0]);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    auto_white_balance(&mut cpu);
    auto_white_balance_gpu(&mut gpu_buf, 8, 8, &gpu);
    // Both should pass through unchanged.
    assert_eq!(cpu, src);
    assert_eq!(gpu_buf, src);
}

#[test]
fn auto_wb_gpu_handles_non_workgroup_aligned_dimensions() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = solid(13, 19, [180, 110, 90]);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    auto_white_balance(&mut cpu);
    auto_white_balance_gpu(&mut gpu_buf, 13, 19, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 2, "13×19 red cast");
}
