//! GPU⇄CPU reconciliation + GPU-standalone known-answer tests. These need a real
//! adapter, so each no-ops when none is available (CI without a GPU) and is
//! `#[ignore]` by default — run headless on Metal with
//! `cargo test -p ph2d-inpaint --features gpu -- --ignored`.
//!
//! Parity target: because PatchMatch has arg-min branches (unlike a branchless
//! filter), the GPU is not bit-exact with the CPU — float-summation order and
//! rare arg-min ties differ. We assert the reconstruction agrees within a small
//! PERCEPTUAL ε (mean ≤ ~2/255, bounded max), and — the stronger check — that the
//! GPU independently satisfies the same known-answer properties the CPU does.

use crate::{InpaintParams, InpaintRequest, inpaint_cpu, inpaint_gpu};
use ph2d_gpu::GpuContext;
use std::sync::OnceLock;

fn try_gpu() -> Option<GpuContext> {
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn rgba_from<F: Fn(usize, usize) -> [u8; 3]>(w: usize, h: usize, f: F) -> Vec<u8> {
    let mut v = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let c = f(x, y);
            let o = (y * w + x) * 4;
            v[o..o + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    v
}

fn rect_mask(w: usize, h: usize, hx: usize, hy: usize, hw: usize, hh: usize) -> Vec<u8> {
    let mut m = vec![0u8; w * h];
    for y in hy..hy + hh {
        for x in hx..hx + hw {
            m[y * w + x] = 255;
        }
    }
    m
}

/// Mean + max absolute per-channel difference over the whole image.
fn diff(a: &[u8], b: &[u8]) -> (f64, i32) {
    let mut sum = 0i64;
    let mut max = 0i32;
    for (x, y) in a.iter().zip(b.iter()) {
        let d = (i32::from(*x) - i32::from(*y)).abs();
        sum += i64::from(d);
        max = max.max(d);
    }
    (sum as f64 / a.len() as f64, max)
}

#[test]
#[ignore = "needs a GPU adapter; run with --features gpu -- --ignored"]
fn gpu_reconciles_with_cpu_within_epsilon() {
    let Some(gpu) = try_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let (w, h) = (96, 96);
    let img = rgba_from(w, h, |x, y| {
        let s = if (x / 3) % 2 == 0 { 210 } else { 40 };
        [s, ((y * 3) % 256) as u8, ((x + y) % 256) as u8]
    });
    let mask = rect_mask(w, h, 36, 36, 20, 20);
    let params = InpaintParams::default();
    let req = InpaintRequest {
        width: w as u32,
        height: h as u32,
        rgba: &img,
        mask: &mask,
        params,
    };
    let cpu = inpaint_cpu(&req).rgba;
    let gpu_out = inpaint_gpu(&gpu, &req).rgba;
    let (mean, max) = diff(&cpu, &gpu_out);
    assert!(
        mean <= 2.0,
        "GPU⇄CPU mean diff {mean:.3} too high (max {max})"
    );
    assert!(
        max <= 48,
        "GPU⇄CPU max diff {max} too high (mean {mean:.3})"
    );
}

#[test]
#[ignore = "needs a GPU adapter; run with --features gpu -- --ignored"]
fn gpu_reconstructs_periodic_stripes() {
    let Some(gpu) = try_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let (w, h) = (64, 64);
    let stripe = |x: usize| {
        if (x / 2).is_multiple_of(2) {
            235u8
        } else {
            25u8
        }
    };
    let img = rgba_from(w, h, |x, _| [stripe(x); 3]);
    let mask = rect_mask(w, h, 26, 26, 12, 12);
    let out = inpaint_gpu(
        &gpu,
        &InpaintRequest {
            width: w as u32,
            height: h as u32,
            rgba: &img,
            mask: &mask,
            params: InpaintParams::default(),
        },
    )
    .rgba;
    let mut worst = 0i32;
    for y in 26..38 {
        for x in 26..38 {
            let o = (y * w + x) * 4;
            worst = worst.max((i32::from(out[o]) - i32::from(stripe(x))).abs());
        }
    }
    assert!(worst <= 40, "GPU stripe reconstruction worst error {worst}");
}

#[test]
#[ignore = "needs a GPU adapter; run with --features gpu -- --ignored"]
fn gpu_flat_colour_stays_flat_and_known_pixels_untouched() {
    let Some(gpu) = try_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let (w, h) = (48, 48);
    let col = [70u8, 140, 200];
    let img = rgba_from(w, h, |_, _| col);
    let mask = rect_mask(w, h, 18, 18, 10, 10);
    let out = inpaint_gpu(
        &gpu,
        &InpaintRequest {
            width: w as u32,
            height: h as u32,
            rgba: &img,
            mask: &mask,
            params: InpaintParams::default(),
        },
    )
    .rgba;
    for y in 0..h {
        for x in 0..w {
            let o = (y * w + x) * 4;
            if mask[y * w + x] < 128 {
                assert_eq!(&out[o..o + 4], &img[o..o + 4], "known pixel changed");
            } else {
                for k in 0..3 {
                    let d = (i32::from(out[o + k]) - i32::from(col[k])).abs();
                    assert!(d <= 3, "flat fill off at ({x},{y}) ch {k}: {d}");
                }
            }
        }
    }
}
