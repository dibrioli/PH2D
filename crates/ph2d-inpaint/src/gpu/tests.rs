//! GPU⇄CPU reconciliation + GPU-standalone known-answer tests. These need a real
//! adapter, so each no-ops when none is available (CI without a GPU) and is
//! `#[ignore]` by default — run headless on Metal with
//! `cargo test -p ph2d-inpaint --features gpu -- --ignored`.
//!
//! Parity target (B38.3): each level is integer arithmetic on premultiplied RGBA8
//! (SSD, ties to the lower source index, Q16 vote), so the GPU gives the CPU's
//! bytes — on every fixture, opaque or with intermediate alpha and chroma. The GPU
//! also satisfies the same known-answer properties the CPU does.

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

/// The fixtures of the byte gate: opaque stripes, the edge of yellow paint on an empty layer, a
/// chroma × alpha ramp, and one above the 128² size where the Heal routes to the GPU.
fn fixturas_do_portao() -> Vec<(usize, usize, Vec<u8>, Vec<u8>)> {
    let listras = rgba_from(96, 96, |x, y| {
        let s = if (x / 3) % 2 == 0 { 210 } else { 40 };
        [s, ((y * 3) % 256) as u8, ((x + y) % 256) as u8]
    });
    let mut borda = vec![0u8; 64 * 48 * 4];
    for y in 0..48 {
        for x in 32..64 {
            let o = (y * 64 + x) * 4;
            borda[o..o + 4].copy_from_slice(&[250, 225, 20, 255]);
        }
    }
    let mut rampa = vec![0u8; 80 * 64 * 4];
    for y in 0..64 {
        for x in 0..80 {
            let o = (y * 80 + x) * 4;
            let a = ((x * 255) / 79) as u8;
            rampa[o..o + 4].copy_from_slice(&[
                (x * 3) as u8,
                200 - y as u8,
                ((x ^ y) * 4) as u8,
                a,
            ]);
        }
    }
    let grande = rgba_from(160, 160, |x, y| {
        [
            (x * 7 % 256) as u8,
            (y * 5 % 256) as u8,
            ((x / 8 + y / 8) % 2 * 200) as u8,
        ]
    });
    vec![
        (96, 96, listras, rect_mask(96, 96, 36, 36, 20, 20)),
        (64, 48, borda, rect_mask(64, 48, 20, 16, 24, 16)),
        (80, 64, rampa, rect_mask(80, 64, 30, 20, 22, 18)),
        (160, 160, grande, rect_mask(160, 160, 60, 50, 40, 44)),
    ]
}

/// **B38.3 — the Heal is the same byte on the GPU and on the CPU** (the route by size/adapter no longer
/// changes the patch). Red before: f32 SSD/vote, the 96² stripes differed (mean ≤ 2, max ≤ 48 was the gate).
#[test]
#[ignore = "needs a GPU adapter; run with --features gpu -- --ignored"]
fn heal_na_placa_e_na_cpu_e_o_mesmo_byte() {
    let Some(gpu) = try_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    eprintln!("adapter: {:?}", gpu.adapter.get_info().name);
    let mut mudaram = 0;
    for (k, (w, h, img, mask)) in fixturas_do_portao().into_iter().enumerate() {
        let req = InpaintRequest {
            width: w as u32,
            height: h as u32,
            rgba: &img,
            mask: &mask,
            params: InpaintParams::default(),
        };
        let cpu = inpaint_cpu(&req).rgba;
        let gpu_out = inpaint_gpu(&gpu, &req).expect("gpu level").rgba;
        let (mean, max) = diff(&cpu, &gpu_out);
        mudaram += usize::from(cpu != img);
        assert!(
            cpu == gpu_out,
            "fixtura {k}: placa × CPU diferem (média {mean:.3}, pior {max})"
        );
    }
    assert!(
        mudaram >= 3,
        "CONTROLO: só {mudaram} fixturas mudaram o buraco"
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
    .expect("gpu level")
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
    .expect("gpu level")
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

thread_local! {
    /// Quantos níveis da placa correm antes de o readback FALHAR (`None` = nenhum falha) — a injecção do B41.1.
    static FALHA_DEPOIS_DE: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
}

/// O readback de um nível, com a falha injectada no ponto em que a placa a devolveria (`map_async`).
pub(super) fn injecta_falha(
    r: Result<(), wgpu::BufferAsyncError>,
) -> Result<(), wgpu::BufferAsyncError> {
    match FALHA_DEPOIS_DE.with(std::cell::Cell::get) {
        Some(0) => {
            FALHA_DEPOIS_DE.with(|c| c.set(None));
            Err(wgpu::BufferAsyncError)
        }
        Some(k) => {
            FALHA_DEPOIS_DE.with(|c| c.set(Some(k - 1)));
            r
        }
        None => r,
    }
}

/// **B41.1/B41.A1 — a placa que recusa um nível não fecha o programa:** o `inpaint` cai no `inpaint_cpu`, e a
/// resposta é a referência ao byte (nunca um híbrido de níveis da placa com níveis da CPU).
#[test]
#[ignore = "needs a GPU adapter; run with --features gpu -- --ignored"]
fn inpaint_gpu_devolve_erro_e_o_chamador_cai_na_cpu() {
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
    let req = InpaintRequest {
        width: w as u32,
        height: h as u32,
        rgba: &img,
        mask: &mask,
        params: InpaintParams::default(),
    };
    FALHA_DEPOIS_DE.with(|c| c.set(Some(1)));
    let out = crate::inpaint(Some(&gpu), &req).rgba;
    let consumida = FALHA_DEPOIS_DE.with(std::cell::Cell::take).is_none();
    assert!(
        consumida,
        "o controlo: a falha injectada não chegou a correr"
    );
    assert!(
        out == inpaint_cpu(&req).rgba,
        "a queda não é a referência da CPU ao byte"
    );
}
