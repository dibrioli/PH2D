//! B19.9: the cut-point cache without the per-frame canvas copies, against the copying path
//! (`COPIAS_ANTIGAS`, the code as it was before B19.9) in the same process, and against `composite`.

use super::*;
use ph2d_painter_effects::adjustments::{
    AdjustmentKind, AdjustmentParams, BrightnessContrastParams, GaussianBlurParams, NoiseParams,
};
use std::sync::atomic::Ordering::Relaxed;

fn tinta(w: u32, h: u32, semente: u32) -> LayerImage {
    let mut img = LayerImage::transparent(w, h);
    let mut x = semente.wrapping_mul(2_654_435_761).wrapping_add(1);
    for p in img.rgba8_mut().as_chunks_mut::<4>().0 {
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
    pilha_com(w, h, tinta)
}

fn pilha_com(
    w: u32,
    h: u32,
    tinta: impl Fn(u32, u32, u32) -> LayerImage,
) -> (LayerStack, MapPixelSource, [LayerId; 3]) {
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

/// Root (top-first): topo · cima (Gaussian) · grupo(filho, Overlay) · alto · ruído · meio (Multiply,
/// masked) · baixo · blur · base. One Gaussian BELOW the dragged adjustments (baked into their cuts),
/// one ABOVE (the B19.6 barrier on the drag); the Noise reads the absolute row, so a band at the wrong
/// `y` changes bytes.
fn pilha_faixas(w: u32, h: u32) -> (LayerStack, MapPixelSource, [LayerId; 5]) {
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    let base = s.add_raster("base", w, h).unwrap();
    src.insert(base, tinta(w, h, 1));
    let blur = s.add_adjustment(AdjustmentKind::GaussianBlur).unwrap();
    s.adjustment_mut(blur).unwrap().params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 2.0 });
    let baixo = s
        .add_adjustment(AdjustmentKind::BrightnessContrast)
        .unwrap();
    s.adjustment_mut(baixo).unwrap().params = bc(0.15, 0.10);
    let meio = s.add_raster("meio", w, h).unwrap();
    s.set_blend_mode(meio, BlendMode::Multiply);
    src.insert(meio, tinta(w, h, 2));
    let mascara = s.add_mask(meio).unwrap();
    src.insert(mascara, tinta(w, h, 3));
    let ruido = s.add_adjustment(AdjustmentKind::Noise).unwrap();
    s.adjustment_mut(ruido).unwrap().params = AdjustmentParams::Noise(NoiseParams {
        amount: 0.3,
        ..NoiseParams::default()
    });
    let alto = s
        .add_adjustment(AdjustmentKind::BrightnessContrast)
        .unwrap();
    s.adjustment_mut(alto).unwrap().params = bc(-0.20, 0.25);
    let grupo = s.add_group("grupo").unwrap();
    s.set_blend_mode(grupo, BlendMode::Overlay);
    let filho = s.add_raster("filho", w, h).unwrap();
    src.insert(filho, tinta(w, h, 4));
    s.move_into_group(filho, grupo);
    let cima = s.add_adjustment(AdjustmentKind::GaussianBlur).unwrap();
    s.adjustment_mut(cima).unwrap().params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 1.5 });
    s.adjustment_mut(cima).unwrap().opacity = 0.8;
    let topo = s.add_raster("topo", w, h).unwrap();
    s.set_opacity(topo, 0.6);
    s.set_blend_mode(topo, BlendMode::Screen);
    src.insert(topo, tinta(w, h, 5));
    (s, src, [blur, baixo, ruido, alto, cima])
}

/// A drag session through [`composite_with_cache_into`] with ONE reused output buffer (B19.12: no
/// reallocation after the cold frame); with `fios = 1` each frame is also checked against `composite`.
fn sessao_faixas(w: u32, h: u32, fios: usize) -> Vec<Vec<u8>> {
    let (mut s, src, [blur, baixo, ruido, alto, cima]) = pilha_faixas(w, h);
    FIOS_DO_CACHE.store(fios, Relaxed);
    let mut cache = CompositorCache::new();
    let mut out = Vec::new();
    composite_with_cache_into(&s, &src, w, h, &mut cache, &mut out);
    let ptr = out.as_ptr();
    let mut saidas = vec![out.clone()];
    let passos = [
        baixo, alto, cima, alto, ruido, baixo, cima, blur, alto, baixo,
    ];
    for (k, &id) in passos.iter().enumerate() {
        let p = &mut s.adjustment_mut(id).unwrap().params;
        *p = match p {
            AdjustmentParams::GaussianBlur(_) => {
                AdjustmentParams::GaussianBlur(GaussianBlurParams {
                    radius: 1.0 + k as f32,
                })
            }
            AdjustmentParams::Noise(_) => AdjustmentParams::Noise(NoiseParams {
                amount: 0.1 * k as f32,
                ..NoiseParams::default()
            }),
            _ => bc(0.05 * k as f32 - 0.2, 0.1 * k as f32),
        };
        cache.invalidate_above(id, &s);
        composite_with_cache_into(&s, &src, w, h, &mut cache, &mut out);
        assert_eq!(
            out.as_ptr(),
            ptr,
            "passo {k} {w}x{h}: o buffer de saída realocou"
        );
        if fios == 1 {
            assert_eq!(out, composite(&s, &src, w, h), "passo {k} {w}x{h}");
        }
        saidas.push(out.clone());
    }
    FIOS_DO_CACHE.store(0, Relaxed);
    saidas
}

/// B19.6/B19.10 + B19.12: the banded cache walk (cuts per band, stitched) and the reused output are
/// the serial walk to the bit, for 1..32 bands, above the `1 << 17` px threshold too.
#[test]
fn o_cache_em_faixas_e_o_serial_ao_bit() {
    for (w, h) in [(1, 1), (7, 3), (61, 47), (513, 300)] {
        let serial = sessao_faixas(w, h, 1);
        for fios in [2, 3, 7, 16, 32] {
            assert!(
                sessao_faixas(w, h, fios) == serial,
                "{w}x{h} em {fios} faixas: divergiu do serial"
            );
        }
    }
}

/// Probe (`--ignored`, `--profile smoke`, ALONE: flips the global toggles): 3840×2160, the drag of
/// the higher adjustment (restart from its cut). A = before B19.6/B19.12 (serial walk, a fresh output
/// `Vec`); B = the banded walk into the reused output. ABBA, min of 8 per variant.
#[test]
#[ignore]
fn mede_o_arrasto_do_ajuste_4k() {
    let (w, h) = (3840u32, 2160u32);
    let (mut s, src, [_, alto, blur]) = pilha(w, h);
    // Only the per-pixel adjustment: the drag being measured is the cache's, not the blur's.
    s.adjustment_mut(blur).unwrap().visible = false;
    let mut caches = [CompositorCache::new(), CompositorCache::new()];
    let mut best = [f64::MAX; 2];
    let mut saidas: [Vec<u8>; 2] = [Vec::new(), Vec::new()];
    for i in 0..18usize {
        let v = [0usize, 1, 1, 0][i % 4];
        FIOS_DO_CACHE.store(if v == 0 { 1 } else { 0 }, Relaxed);
        s.adjustment_mut(alto).unwrap().params = bc(0.01 * (i / 4) as f32, 0.2);
        caches[v].invalidate_above(alto, &s);
        let t0 = std::time::Instant::now();
        if v == 0 {
            saidas[0] = composite_with_cache(&s, &src, w, h, &mut caches[0]);
        } else {
            composite_with_cache_into(&s, &src, w, h, &mut caches[1], &mut saidas[1]);
        }
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        if i >= 2 {
            best[v] = best[v].min(ms);
        }
    }
    FIOS_DO_CACHE.store(0, Relaxed);
    assert!(saidas[0] == saidas[1], "as duas vias divergiram");
    println!(
        "[B19.6] 3840x2160, arrasto do ajuste alto: serial + Vec nova {:.2} ms | faixas + buffer reusado {:.2} ms",
        best[0], best[1]
    );
}

/// Probe (`--ignored`, `--profile smoke`, ALONE): 3840×2160, the drag of `alto` with the Gaussian
/// (r 20) ABOVE it, and the drag of the Gaussian itself. A = the pre-barrier rule (a spatial item in
/// the restart ⇒ the serial walk, `FIOS_DO_CACHE = 1`); B = the barrier. ABBA, min of 8 per variant.
#[test]
#[ignore]
fn mede_o_arrasto_com_espacial_acima_4k() {
    let (w, h) = (3840u32, 2160u32);
    let (mut s, src, [_, alto, blur]) = pilha(w, h);
    s.adjustment_mut(blur).unwrap().params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 20.0 });
    for (nome, arrastado) in [("alto", alto), ("gaussian", blur)] {
        let mut caches = [CompositorCache::new(), CompositorCache::new()];
        let mut best = [f64::MAX; 2];
        let mut saidas: [Vec<u8>; 2] = [Vec::new(), Vec::new()];
        for i in 0..18usize {
            let v = [0usize, 1, 1, 0][i % 4];
            FIOS_DO_CACHE.store(if v == 0 { 1 } else { 0 }, Relaxed);
            let p = &mut s.adjustment_mut(arrastado).unwrap().params;
            *p = match p {
                AdjustmentParams::GaussianBlur(_) => {
                    AdjustmentParams::GaussianBlur(GaussianBlurParams {
                        radius: 20.0 + (i / 4) as f32,
                    })
                }
                _ => bc(0.01 * (i / 4) as f32, 0.2),
            };
            caches[v].invalidate_above(arrastado, &s);
            let t0 = std::time::Instant::now();
            composite_with_cache_into(&s, &src, w, h, &mut caches[v], &mut saidas[v]);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            if i >= 2 {
                best[v] = best[v].min(ms);
            }
            if i % 4 == 3 {
                assert!(saidas[0] == saidas[1], "{nome}: as duas vias divergiram");
            }
        }
        FIOS_DO_CACHE.store(0, Relaxed);
        println!(
            "[B19.6] 3840x2160, arrasto de {nome} com Gaussian r20 acima: serial {:.2} ms | barreira {:.2} ms",
            best[0], best[1]
        );
    }
}

/// B19.10: o arrasto de ajustes com o resumo por ladrilho dá o byte da rota sem resumo. As camadas são
/// ESPARSAS (blocos de 64 px todos α=0), senão o resumo não salta nada e o gate não prova nada.
#[test]
fn o_arrasto_de_ajustes_com_resumo_e_a_rota_sem_resumo_ao_bit() {
    let _trava = crate::compositor::compose::resumo::trava_das_chaves();
    use crate::compositor::compose::resumo::SEM_RESUMO;
    let esparsa = |w: u32, h: u32, semente: u32| {
        let mut img = tinta(w, h, semente);
        for (i, p) in img
            .rgba8_mut()
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .enumerate()
        {
            let (x, y) = (i as u32 % w, i as u32 / w);
            if !(x / 64 + y / 3 + semente).is_multiple_of(3) {
                *p = [0; 4];
            }
        }
        img
    };
    let (w, h) = (300, 40);
    let sessao_esparsa = || {
        let (mut s, src, [baixo, alto, _]) = pilha_com(w, h, esparsa);
        let mut cache = CompositorCache::new();
        let mut v = vec![composite_with_cache(&s, &src, w, h, &mut cache)];
        for (k, id) in [alto, baixo, alto].into_iter().enumerate() {
            s.adjustment_mut(id).unwrap().params = bc(0.1 * k as f32, -0.2);
            cache.invalidate_above(id, &s);
            v.push(composite_with_cache(&s, &src, w, h, &mut cache));
        }
        v
    };
    SEM_RESUMO.store(true, Relaxed);
    let sem = sessao_esparsa();
    SEM_RESUMO.store(false, Relaxed);
    let com = sessao_esparsa();
    assert!(sem == com, "o resumo mudou o arrasto de ajustes");
}
