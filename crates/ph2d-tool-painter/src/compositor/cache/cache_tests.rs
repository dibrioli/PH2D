//! B19.9: the cut-point cache without the per-frame canvas copies, against the copying path
//! (`COPIAS_ANTIGAS`, the code as it was before B19.9) in the same process, and against `composite`.

use super::*;
use ph2d_painter_effects::adjustments::{
    AdjustmentKind, AdjustmentParams, BrightnessContrastParams, GaussianBlurParams,
};
use std::sync::atomic::Ordering::Relaxed;

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

fn bc(brightness: f32, contrast: f32) -> AdjustmentParams {
    AdjustmentParams::BrightnessContrast(BrightnessContrastParams {
        brightness,
        contrast,
        legacy: false,
    })
}

/// Root (top-first): topo · blur · grupo(filho, Overlay) · alto · meio (Multiply, masked) · baixo · base.
fn pilha(w: u32, h: u32) -> (LayerStack, MapPixelSource, [LayerId; 3]) {
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    let base = s.add_raster("base", w, h).unwrap();
    src.insert(base, tinta(w, h, 1));
    let baixo = s
        .add_adjustment(AdjustmentKind::BrightnessContrast)
        .unwrap();
    s.adjustment_mut(baixo).unwrap().params = bc(0.15, 0.10);
    let meio = s.add_raster("meio", w, h).unwrap();
    s.set_blend_mode(meio, BlendMode::Multiply);
    src.insert(meio, tinta(w, h, 2));
    let mascara = s.add_mask(meio).unwrap();
    src.insert(mascara, tinta(w, h, 3));
    let alto = s
        .add_adjustment(AdjustmentKind::BrightnessContrast)
        .unwrap();
    s.adjustment_mut(alto).unwrap().params = bc(-0.20, 0.25);
    let grupo = s.add_group("grupo").unwrap();
    s.set_blend_mode(grupo, BlendMode::Overlay);
    let filho = s.add_raster("filho", w, h).unwrap();
    src.insert(filho, tinta(w, h, 4));
    s.move_into_group(filho, grupo);
    let blur = s.add_adjustment(AdjustmentKind::GaussianBlur).unwrap();
    s.adjustment_mut(blur).unwrap().params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 3.0 });
    s.adjustment_mut(blur).unwrap().opacity = 0.7;
    let topo = s.add_raster("topo", w, h).unwrap();
    s.set_opacity(topo, 0.6);
    s.set_blend_mode(topo, BlendMode::Screen);
    src.insert(topo, tinta(w, h, 5));
    (s, src, [baixo, alto, blur])
}

/// One drag session: cold, then param-only frames on each adjustment in turn (incl. the same one
/// twice in a row and a jump back down), each frame compared to `composite`.
fn sessao(w: u32, h: u32) -> Vec<Vec<u8>> {
    let (mut s, src, [baixo, alto, blur]) = pilha(w, h);
    let mut cache = CompositorCache::new();
    let mut saidas = vec![composite_with_cache(&s, &src, w, h, &mut cache)];
    assert_eq!(saidas[0], composite(&s, &src, w, h), "frio {w}x{h}");
    let passos = [
        (alto, 0.3),
        (alto, -0.1),
        (baixo, 0.4),
        (blur, 0.0),
        (alto, 0.05),
        (baixo, -0.2),
    ];
    for (k, &(id, v)) in passos.iter().enumerate() {
        let p = &mut s.adjustment_mut(id).unwrap().params;
        *p = match p {
            AdjustmentParams::GaussianBlur(_) => {
                AdjustmentParams::GaussianBlur(GaussianBlurParams {
                    radius: 1.5 + k as f32,
                })
            }
            _ => bc(v, 0.1 * k as f32),
        };
        cache.invalidate_above(id, &s);
        let out = composite_with_cache(&s, &src, w, h, &mut cache);
        assert_eq!(out, composite(&s, &src, w, h), "passo {k} {w}x{h}");
        saidas.push(out);
    }
    saidas
}

#[test]
fn o_cache_sem_copias_e_o_das_tres_copias_ao_bit() {
    for (w, h) in [(1, 1), (7, 3), (61, 47)] {
        COPIAS_ANTIGAS.store(true, Relaxed);
        let antigas = sessao(w, h);
        COPIAS_ANTIGAS.store(false, Relaxed);
        let novas = sessao(w, h);
        assert!(
            antigas == novas,
            "{w}x{h}: o cache sem cópias mudou a saída"
        );
    }
}

/// Probe (`--ignored`, `--profile smoke`, ALONE: flips the global toggle): 3840×2160, the drag of
/// the higher adjustment (restart from its cut), copies (A) × no copies (B), ABBA, min of 8.
#[test]
#[ignore]
fn mede_o_arrasto_do_ajuste_4k() {
    let (w, h) = (3840u32, 2160u32);
    let (mut s, src, [_, alto, blur]) = pilha(w, h);
    // Only the per-pixel adjustment: the drag being measured is the cache's, not the blur's.
    s.adjustment_mut(blur).unwrap().visible = false;
    let mut caches = [CompositorCache::new(), CompositorCache::new()];
    let mut best = [f64::MAX; 2];
    let mut saidas: [Option<Vec<u8>>; 2] = [None, None];
    for i in 0..16usize {
        let v = [0usize, 1, 1, 0][i % 4];
        COPIAS_ANTIGAS.store(v == 0, Relaxed);
        s.adjustment_mut(alto).unwrap().params = bc(0.01 * (i / 4) as f32, 0.2);
        caches[v].invalidate_above(alto, &s);
        let t0 = std::time::Instant::now();
        let out = composite_with_cache(&s, &src, w, h, &mut caches[v]);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        if i >= 2 {
            best[v] = best[v].min(ms);
        }
        saidas[v] = Some(out);
    }
    COPIAS_ANTIGAS.store(false, Relaxed);
    assert!(saidas[0] == saidas[1], "as duas vias divergiram");
    println!(
        "[B19.9] 3840x2160, arrasto do ajuste alto: 3 cópias {:.2} ms | sem cópias {:.2} ms",
        best[0], best[1]
    );
}
