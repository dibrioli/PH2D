//! G1 of B19.6: the root stack cut at its spatial items (per-pixel runs in bands, spatial items
//! alone) is the pre-B19.6 walk to the bit — 22 modes, mask, clipping (hidden/opacity-0 bases),
//! groups, per-pixel and spatial adjustments, glass — at 1/2/3/7/16/32 bands.

use super::barreira::{BARREIRA_ANTIGA, FIOS_DA_REGIAO};
use super::*;
use ph2d_painter_effects::MAX_BLEND_MODES;
use ph2d_painter_effects::adjustments::{
    AdjustmentKind, AdjustmentParams, BrightnessContrastParams, GaussianBlurParams,
    MotionBlurParams, NoiseParams,
};
use std::sync::atomic::Ordering::Relaxed;

/// ORACLE — `compor_regiao` before B19.6, verbatim but for the name.
fn compor_regiao_antigo(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    region: Region,
    vidros: Option<&super::vidro::Vidros>,
) -> (Vec<[f32; 4]>, Option<Vec<[f32; 3]>>) {
    let (rx, ry, rw, rh) = (region.x, region.y, region.w, region.h);
    let mut acc = vec![[0.0f32; 4]; (rw as usize) * (rh as usize)];
    // A transparência do papel nasce 1 (nada por cima): o enchimento em paralelo (`plane_copy`).
    let mut t = vidros.map(|_| {
        let mut t = Vec::new();
        crate::plane_copy::size_to(&mut t, acc.len(), [1.0f32; 3]);
        t
    });
    // Per-pixel compositing is spatially independent → split the region's rows into disjoint bands
    // across the cores (bit-identical to the serial walk; each band runs the same bottom→top layer
    // walk over its own rows). This was the painter's biggest per-frame cost on a non-trivial doc
    // stack: `take_preview_arc` recomposites the shape methods' whole-shape dirty bbox EVERY move
    // (~31 ms serial @2048², Enio's "FPS 60→10"). SPATIAL adjustments (blur/bloom/…) read
    // neighbouring pixels — banding would seam at band edges — so those stacks stay serial.
    let threads = parallel_threads(
        (rw as usize) * (rh as usize),
        if has_spatial_adjustment(stack, stack.root()) {
            1
        } else {
            rh as usize
        },
    );
    if threads <= 1 {
        composite_into(
            &mut acc,
            stack.root(),
            stack,
            src,
            (width, height),
            rx,
            ry,
            rw,
            rh,
            0,
            None,
            vidros.zip(t.as_deref_mut()),
        );
        return (acc, t);
    }
    let rpb = (rh as usize).div_ceil(threads);
    let band_px = rpb * (rw as usize);
    let mut t_bands: Vec<Option<&mut [[f32; 3]]>> = match t.as_deref_mut() {
        Some(tt) => tt.chunks_mut(band_px).map(Some).collect(),
        None => (0..acc.len().div_ceil(band_px)).map(|_| None).collect(),
    };
    std::thread::scope(|s| {
        let handles: Vec<_> = acc
            .chunks_mut(band_px)
            .zip(t_bands.iter_mut())
            .enumerate()
            .map(|(bi, (band, tb))| {
                let band_ry = ry + (bi * rpb) as u32;
                let band_rh = (band.len() / (rw as usize)) as u32;
                let tb = tb.take();
                s.spawn(move || {
                    composite_into(
                        band,
                        stack.root(),
                        stack,
                        src,
                        (width, height),
                        rx,
                        band_ry,
                        rw,
                        band_rh,
                        0,
                        None,
                        vidros.zip(tb),
                    );
                })
            })
            .collect();
        for h in handles {
            h.join().expect("composite band panicked");
        }
    });
    drop(t_bands);
    (acc, t)
}

fn tinta(w: u32, h: u32, semente: u32) -> LayerImage {
    let mut img = LayerImage::transparent(w, h);
    let mut x = semente.wrapping_mul(2_654_435_761).wrapping_add(1);
    for p in img.rgba8.as_chunks_mut::<4>().0 {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        let a = [0u8, 0, 255, 128, 37, 255, 1, 254][(x >> 29) as usize];
        *p = [(x >> 3) as u8, (x >> 11) as u8, (x >> 19) as u8, a];
    }
    img
}

fn raster(s: &mut LayerStack, src: &mut MapPixelSource, w: u32, h: u32, k: u32) -> LayerId {
    let id = s.add_raster(format!("L{k}"), w, h).unwrap();
    src.insert(id, tinta(w, h, k));
    id
}

fn ajuste(s: &mut LayerStack, p: AdjustmentParams) -> LayerId {
    let id = s.add_adjustment(p.kind()).unwrap();
    s.adjustment_mut(id).unwrap().params = p;
    id
}

/// Bottom → top: base (glass) · modes 0..8 (mask, clip chains on a hidden and an opacity-0 base) ·
/// Gaussian · Noise (absolute row) · modes 8..15 · group{raster, Motion} · clipped raster ·
/// modes 15..22 · Brightness/Contrast. With `espacial = false` the two spatial items are hidden.
fn pilha(
    w: u32,
    h: u32,
    espacial: bool,
) -> (LayerStack, MapPixelSource, super::super::vidro::Vidros) {
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    let base = raster(&mut s, &mut src, w, h, 100);
    let mut vidros = super::super::vidro::Vidros::new();
    let plano: Vec<_> = src.images[&base]
        .rgba8
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| super::super::vidro::sela(*p, [p[0] / 2, p[1], p[3]]))
        .collect();
    vidros.insert(base, std::sync::Arc::new(plano));
    let mut modo = 0u8;
    let mut faixa = |s: &mut LayerStack, src: &mut MapPixelSource, ate: u8| {
        while modo < ate {
            let id = raster(s, src, w, h, u32::from(modo) + 1);
            s.set_blend_mode(id, BlendMode::from_u8(modo));
            s.set_opacity(id, 0.35 + 0.03 * f32::from(modo));
            match modo % 7 {
                1 => {
                    let m = s.add_mask(id).unwrap();
                    src.insert(m, tinta(w, h, 200 + u32::from(modo)));
                }
                3 | 4 => s.set_clipping(id, true),
                5 => s.set_visible(id, false),
                6 => s.set_opacity(id, 0.0),
                _ => {}
            }
            modo += 1;
        }
    };
    faixa(&mut s, &mut src, 8);
    let blur = ajuste(
        &mut s,
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 2.5 }),
    );
    s.adjustment_mut(blur).unwrap().opacity = 0.8;
    ajuste(
        &mut s,
        AdjustmentParams::Noise(NoiseParams {
            amount: 0.25,
            ..NoiseParams::default()
        }),
    );
    faixa(&mut s, &mut src, 15);
    let grupo = s.add_group("grupo").unwrap();
    s.set_blend_mode(grupo, BlendMode::Overlay);
    let filho = raster(&mut s, &mut src, w, h, 300);
    s.move_into_group(filho, grupo);
    let motion = ajuste(
        &mut s,
        AdjustmentParams::MotionBlur(MotionBlurParams {
            distance: 5.0,
            angle: 0.7,
        }),
    );
    s.move_into_group(motion, grupo);
    let recortado = raster(&mut s, &mut src, w, h, 301);
    s.set_clipping(recortado, true);
    faixa(&mut s, &mut src, MAX_BLEND_MODES);
    ajuste(
        &mut s,
        AdjustmentParams::BrightnessContrast(BrightnessContrastParams {
            brightness: 0.1,
            contrast: 0.2,
            legacy: false,
        }),
    );
    if !espacial {
        s.set_visible(blur, false);
        s.set_visible(grupo, false);
    }
    (s, src, vidros)
}

fn bits(v: &(Vec<[f32; 4]>, Option<Vec<[f32; 3]>>)) -> (Vec<[u32; 4]>, Option<Vec<[u32; 3]>>) {
    (
        v.0.iter().map(|p| p.map(f32::to_bits)).collect(),
        v.1.as_ref()
            .map(|t| t.iter().map(|p| p.map(f32::to_bits)).collect()),
    )
}

/// G1 (B19.6): `compor_regiao` with the barrier is the old single walk to the bit, whole canvas and
/// an inner window, with and without glass, at 1/2/3/7/16/32 bands; and above the threshold the
/// default rule really bands (CONTROL).
#[test]
fn a_composicao_em_faixas_e_a_da_rota_antiga_ao_bit() {
    for (w, h) in [(1, 1), (7, 3), (513, 300), (1025, 257), (3840, 64)] {
        for espacial in [true, false] {
            let (s, src, vidros) = pilha(w, h, espacial);
            assert_eq!(has_spatial_adjustment(&s, s.root()), espacial);
            let janela = Region {
                x: w / 3,
                y: h / 4,
                w: (w / 2).max(1),
                h: (h / 2).max(1),
            };
            let inteira = Region { x: 0, y: 0, w, h };
            for r in [inteira, janela] {
                for v in [None, Some(&vidros)] {
                    let antigo = bits(&compor_regiao_antigo(&s, &src, w, h, r, v));
                    for fios in [1usize, 2, 3, 7, 16, 32] {
                        FIOS_DA_REGIAO.store(fios, Relaxed);
                        let novo = bits(&compor_regiao(&s, &src, w, h, r, v));
                        FIOS_DA_REGIAO.store(0, Relaxed);
                        assert!(
                            novo == antigo,
                            "{w}x{h} espacial={espacial} {r:?} vidro={} {fios} fios",
                            v.is_some()
                        );
                    }
                }
            }
        }
    }
    assert!(
        parallel_threads(3840 * 64, 64) > 1
            || std::thread::available_parallelism().map_or(1, usize::from) == 1
    );
}

/// Sonda B19.6 (`--ignored`, `--profile smoke`): `composite` a 3840×2160 com 16 camadas PINTADAS e um
/// Gaussian r20 a meio — a regra antiga (tudo num núcleo, A) e a barreira (B), ABBA, o mínimo de 6.
#[test]
#[ignore]
fn mede_a_barreira_espacial_4k() {
    let (w, h) = (3840u32, 2160u32);
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    for k in 0..16u32 {
        if k == 8 {
            ajuste(
                &mut s,
                AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 20.0 }),
            );
        }
        let id = raster(&mut s, &mut src, w, h, k + 1);
        s.set_blend_mode(id, BlendMode::from_u8((k % 4) as u8));
    }
    let mut best = [f64::MAX; 2];
    for i in 0..12usize {
        let v = [0usize, 1, 1, 0][i % 4];
        BARREIRA_ANTIGA.store(v == 0, Relaxed);
        let t0 = std::time::Instant::now();
        let out = composite(&s, &src, w, h);
        best[v] = best[v].min(t0.elapsed().as_secs_f64() * 1e3);
        std::hint::black_box(out);
    }
    BARREIRA_ANTIGA.store(false, Relaxed);
    println!(
        "[B19.6] 3840x2160, 16 camadas pintadas + Gaussian r20 a meio: antiga {:.1} ms | barreira {:.1} ms",
        best[0], best[1]
    );
}
