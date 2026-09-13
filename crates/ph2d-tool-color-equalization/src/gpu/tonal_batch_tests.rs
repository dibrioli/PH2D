//! Testes de `tonal_batch.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

use super::*;
use crate::algorithm::adjust_tonal;
use crate::gpu::try_headless_gpu;

/// 24×24 diverse-colour ramp (varies every component independently
/// so each stage exercises a fresh distribution).
fn ramp(w: u32, h: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 17 + y * 5) % 256) as u8;
            let g = ((x * 7 + y * 23) % 256) as u8;
            let b = ((x * 13 + y * 41) % 256) as u8;
            v.extend_from_slice(&[r, g, b, 255]);
        }
    }
    v
}

fn assert_within_lsb(cpu: &[u8], gpu: &[u8], max_lsb: i32, ctx: &str) {
    assert_eq!(cpu.len(), gpu.len());
    let mut worst = 0_i32;
    let mut worst_idx = 0;
    let mut diff_count = 0_u64;
    let mut sum_diff = 0_u64;
    for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
        let d = (*a as i32 - *b as i32).abs();
        if d > worst {
            worst = d;
            worst_idx = i;
        }
        if d > 0 {
            diff_count += 1;
            sum_diff += d as u64;
        }
    }
    assert!(
        worst <= max_lsb,
        "{ctx}: CPU/GPU diverged by {worst} LSB at idx {worst_idx} \
             (cpu {} vs gpu {}); mean delta {:.2} over {} pixels",
        cpu[worst_idx],
        gpu[worst_idx],
        sum_diff as f64 / diff_count.max(1) as f64,
        diff_count,
    );
}

#[test]
fn tonal_gpu_identity_is_noop() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(16, 16);
    let mut buf = src.clone();
    let params = ColorEqualizationParams::default();
    adjust_tonal_gpu(&mut buf, 16, 16, &params, &gpu);
    // tonal_is_identity short-circuits before dispatch.
    assert_eq!(buf, src);
}

#[test]
fn tonal_gpu_matches_cpu_brightness_only() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        brightness: 0.4,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 3, "brightness=0.4");
}

#[test]
fn tonal_gpu_matches_cpu_contrast_above_one() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        contrast: 1.5,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 3, "contrast=1.5");
}

#[test]
fn tonal_gpu_matches_cpu_contrast_below_one() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        contrast: 0.7,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 3, "contrast=0.7");
}

#[test]
fn tonal_gpu_matches_cpu_temperature_warm() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        temperature: 0.6,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 3, "temperature=+0.6 (warm)");
}

#[test]
fn tonal_gpu_matches_cpu_temperature_cool() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        temperature: -0.6,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 3, "temperature=-0.6 (cool)");
}

#[test]
fn tonal_gpu_matches_cpu_tint() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        tint: 0.5,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 3, "tint=+0.5 (magenta)");
}

#[test]
fn tonal_gpu_matches_cpu_exposure() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        exposure: 1.0, // +1 EV
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 3, "exposure=+1 EV");
}

#[test]
fn tonal_gpu_matches_cpu_vibrance() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        vibrance: 0.5,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
    // OKLab cbrt + cube introduces a touch more drift than the
    // linear-sRGB stages — observed worst-case is 3 LSB.
    assert_within_lsb(&cpu, &gpu_buf, 3, "vibrance=0.5");
}

#[test]
fn tonal_gpu_matches_cpu_saturation_grayscale() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(24, 24);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        saturation: -1.0,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 3, "saturation=-1 (full desat)");
}

#[test]
fn tonal_gpu_matches_cpu_full_stack() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(32, 32);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        exposure: 0.5,
        temperature: 0.3,
        tint: -0.2,
        brightness: 0.15,
        contrast: 1.2,
        vibrance: 0.3,
        saturation: 0.4,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 32, 32, &params, &gpu);
    // 7 stages compose → accumulated rounding peaks around 4 LSB
    // in observed runs.
    assert_within_lsb(&cpu, &gpu_buf, 4, "full Phase 1 stack");
}

#[test]
fn tonal_gpu_skips_transparent_pixels() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut buf = vec![100u8, 150, 200, 0, 100, 150, 200, 255];
    let params = ColorEqualizationParams {
        brightness: 0.8,
        saturation: -1.0,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal_gpu(&mut buf, 2, 1, &params, &gpu);
    // Transparent pixel: passthrough (matches CPU).
    assert_eq!(&buf[0..4], &[100, 150, 200, 0]);
    // Opaque pixel must have been adjusted.
    assert_ne!(&buf[4..7], &[100, 150, 200]);
}

#[test]
fn tonal_gpu_handles_non_workgroup_aligned_dimensions() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let src = ramp(13, 19);
    let mut cpu = src.clone();
    let mut gpu_buf = src.clone();
    let params = ColorEqualizationParams {
        brightness: 0.2,
        contrast: 1.3,
        saturation: 0.5,
        ..ColorEqualizationParams::default()
    };
    adjust_tonal(&mut cpu, &params);
    adjust_tonal_gpu(&mut gpu_buf, 13, 19, &params, &gpu);
    assert_within_lsb(&cpu, &gpu_buf, 4, "13×19 non-aligned");
}
