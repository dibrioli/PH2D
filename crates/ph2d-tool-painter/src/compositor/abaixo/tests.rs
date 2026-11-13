//! B19.11: the banded backdrop against the one walk it replaced (kept verbatim below as the oracle).

use super::*;
use ph2d_painter_effects::adjustments::AdjustmentKind;

/// ORACLE — `composite_below` as it was before B19.11 (`585e486e76`), verbatim but for the name.
fn composite_below_antigo(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    anchor: LayerId,
    ground: [u8; 3],
) -> Vec<u8> {
    // The below-slices along the root→anchor path, outermost first.
    let mut slices: Vec<&[LayerId]> = Vec::new();
    fn descend<'a>(
        stack: &'a LayerStack,
        ids: &'a [LayerId],
        anchor: LayerId,
        slices: &mut Vec<&'a [LayerId]>,
        depth: usize,
    ) -> bool {
        if depth > MAX_GROUP_DEPTH {
            return false;
        }
        for (i, &id) in ids.iter().enumerate() {
            if id == anchor {
                slices.push(&ids[i + 1..]);
                return true;
            }
            if let Some(LayerKind::Group(gr)) = stack.get(id).map(|l| &l.kind) {
                slices.push(&ids[i + 1..]); // provisional: below the group
                if descend(stack, &gr.children, anchor, slices, depth + 1) {
                    return true;
                }
                slices.pop();
            }
        }
        false
    }
    let found = descend(stack, stack.root(), anchor, &mut slices, 0);
    // ⚠️ **Nothing below ⇒ the ground IS the answer, and the accumulator never exists.** Compositing
    // an empty set over the ground leaves the ground, so the long path below would spend a
    // `[f32;4]`-per-pixel buffer plus an encode to reproduce a flat colour: at 4096² that is **268 MB
    // written and read back, plus 67 MB encoded**, and it is what a ONE-LAYER document paid on every
    // watercolor pen-down (measured: 75-112 ms, near-flat in brush radius because none of it looks at
    // the dab). The allocation used to sit above `descend`, so the early-out was unreachable by
    // construction — the guard existed and the work happened anyway.
    //
    // Byte-identical, and the premise is a gate rather than an argument: the flat fill is the same
    // bytes as `encode(decode_byte(ground))` only because the byte round-trip is the identity on all
    // 256 values (`the_byte_round_trip_is_the_identity`). Conservative on purpose — a
    // non-empty slice of layers that all happen to be invisible still takes the long path, because
    // *"empty"* is a fact about the list and *"invisible"* is a fact about each layer.
    if !found || slices.iter().all(|s| s.is_empty()) {
        return flat_fill(width, height, [ground[0], ground[1], ground[2], 255]);
    }
    // Seed the accumulator with the opaque ground.
    let g = [
        super::decode_byte(ground[0]),
        super::decode_byte(ground[1]),
        super::decode_byte(ground[2]),
        1.0f32,
    ];
    let mut acc = vec![g; (width as usize) * (height as usize)];
    for ids in slices {
        composite_into(
            &mut acc,
            ids,
            stack,
            src,
            (width, height),
            0,
            0,
            width,
            height,
            0,
            None,
            None,
        );
    }
    encode(&acc)
}

/// Deterministic straight RGBA with every alpha regime (0, partial, opaque).
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

/// Below the anchor: a base, a per-pixel adjustment, a masked Multiply, a clipping Screen, Noise
/// (absolute coordinates), `extra`, then a group (Overlay) holding a child under the anchor; a layer
/// above the anchor must not show. Returns the stack, the pixels and the anchor.
fn pilha(w: u32, h: u32, extra: Option<AdjustmentKind>) -> (LayerStack, MapPixelSource, LayerId) {
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    let base = s.add_raster("base", w, h).unwrap();
    src.insert(base, tinta(w, h, 1));
    let bc = s
        .add_adjustment(AdjustmentKind::BrightnessContrast)
        .unwrap();
    s.adjustment_mut(bc).unwrap().params = {
        use ph2d_painter_effects::adjustments::{AdjustmentParams, BrightnessContrastParams};
        AdjustmentParams::BrightnessContrast(BrightnessContrastParams {
            brightness: 0.15,
            contrast: 0.2,
            legacy: false,
        })
    };
    let mult = s.add_raster("mult", w, h).unwrap();
    s.set_blend_mode(mult, BlendMode::Multiply);
    s.set_opacity(mult, 0.7);
    src.insert(mult, tinta(w, h, 2));
    let mascara = s.add_mask(mult).unwrap();
    src.insert(mascara, tinta(w, h, 3));
    let clip = s.add_raster("clip", w, h).unwrap();
    s.get_mut(clip).unwrap().clipping = true;
    s.set_blend_mode(clip, BlendMode::Screen);
    src.insert(clip, tinta(w, h, 4));
    // Noise reads the ABSOLUTE coordinate and is band-safe (Halftone would force the one walk).
    let ruido = s.add_adjustment(AdjustmentKind::Noise).unwrap();
    s.adjustment_mut(ruido).unwrap().params = {
        use ph2d_painter_effects::adjustments::{AdjustmentParams, NoiseParams};
        AdjustmentParams::Noise(NoiseParams {
            amount: 0.3,
            ..Default::default()
        })
    };
    if let Some(k) = extra {
        let _ = s.add_adjustment(k).unwrap();
    }
    let grupo = s.add_group("grupo").unwrap();
    s.set_blend_mode(grupo, BlendMode::Overlay);
    s.set_opacity(grupo, 0.8);
    let filho = s.add_raster("filho", w, h).unwrap();
    src.insert(filho, tinta(w, h, 5));
    s.move_into_group(filho, grupo);
    let ancora = s.add_raster("ancora", w, h).unwrap();
    src.insert(ancora, tinta(w, h, 6));
    s.move_into_group(ancora, grupo);
    let topo = s.add_raster("topo", w, h).unwrap();
    src.insert(topo, tinta(w, h, 7));
    (s, src, ancora)
}

/// ⭐ **B19.11: the banded backdrop IS the one walk, to the bit** — at 1×1, 7×3, 513×300 and 3840×64,
/// over 1..32 bands and strips of 1 px, 1 000 px and the default, with modes, a mask, clipping,
/// per-pixel and coordinate adjustments, a group around the anchor; plus every adjustment kind
/// (the spatial ones take the one walk), and the anchor with nothing below.
#[test]
fn o_chao_em_faixas_e_o_da_rota_antiga_ao_bit() {
    let chao = [231u8, 222, 200];
    for (w, h) in [(1u32, 1u32), (7, 3), (513, 300), (3840, 64)] {
        let (s, src, ancora) = pilha(w, h, None);
        // CONTROLO: no spatial adjustment, or every split below would silently be the one walk.
        assert!(!super::super::compose::has_spatial_adjustment(&s, s.root()));
        let antigo = composite_below_antigo(&s, &src, w, h, ancora, chao);
        assert_ne!(
            antigo,
            flat_fill(w, h, [chao[0], chao[1], chao[2], 255]),
            "{w}×{h}: vazia"
        );
        assert_eq!(
            composite_below(&s, &src, w, h, ancora, chao),
            antigo,
            "{w}×{h} por omissão"
        );
        for fios in [1usize, 2, 3, 7, 16, 32] {
            for faixa in [1usize, 1000, FAIXA_PX, usize::MAX] {
                let novo = composite_below_com(&s, &src, w, h, ancora, chao, Some(fios), faixa);
                assert!(novo == antigo, "{w}×{h}, {fios} fios, faixa {faixa}");
            }
        }
        let fundo = s.root()[s.root().len() - 1];
        assert_eq!(
            composite_below(&s, &src, w, h, fundo, chao),
            composite_below_antigo(&s, &src, w, h, fundo, chao),
            "{w}×{h}: nada por baixo"
        );
    }
    let (w, h) = (513u32, 300u32);
    for k in AdjustmentKind::ALL {
        let (s, src, ancora) = pilha(w, h, Some(k));
        let antigo = composite_below_antigo(&s, &src, w, h, ancora, chao);
        for fios in [1usize, 7] {
            let novo = composite_below_com(&s, &src, w, h, ancora, chao, Some(fios), 1000);
            assert!(novo == antigo, "{k:?}, {fios} fios");
        }
    }
    // CONTROLO: the default really takes the banded branch at these sizes.
    assert!(
        parallel_threads(3840 * 64, 64) > 1
            || std::thread::available_parallelism().map_or(1, usize::from) == 1
    );
}

/// Sonda B19.11 (`--ignored`, `--profile smoke`): `composite_below` a 3840×2160 com 3 camadas por
/// baixo da âncora — a rota antiga (A), as faixas por omissão (B) e a banda inteira sem tiras (C),
/// intercaladas ABCCBA, o mínimo de 6 de cada.
#[test]
#[ignore]
fn mede_o_composite_below_4k() {
    let (w, h) = (3840u32, 2160u32);
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    let base = s.add_raster("base", w, h).unwrap();
    src.insert(
        base,
        LayerImage {
            width: w,
            height: h,
            rgba8: vec![255; (w * h * 4) as usize].into(),
        },
    );
    for k in 1..3u32 {
        let id = s.add_raster(format!("L{k}"), w, h).unwrap();
        src.insert(id, tinta(w, h, k));
    }
    let ancora = s.add_raster("ancora", w, h).unwrap();
    let chao = [231u8, 222, 200];
    let mut best = [f64::MAX; 3];
    for i in 0..18usize {
        let v = [0usize, 1, 2, 2, 1, 0][i % 6];
        let t0 = std::time::Instant::now();
        let out = match v {
            0 => composite_below_antigo(&s, &src, w, h, ancora, chao),
            1 => composite_below(&s, &src, w, h, ancora, chao),
            _ => composite_below_com(&s, &src, w, h, ancora, chao, None, usize::MAX),
        };
        best[v] = best[v].min(t0.elapsed().as_secs_f64() * 1e3);
        std::hint::black_box(out);
    }
    println!(
        "[B19.11] 3840x2160, 3 camadas por baixo: antiga {:.2} ms | faixas {:.2} ms | banda inteira {:.2} ms",
        best[0], best[1], best[2]
    );
}

/// Sonda do salto α = 0 (`--ignored`, `--profile smoke`, SOZINHA: liga e desliga o salto global):
/// 3840×2160, base opaca + 2 camadas — `tinta` (¼ de α = 0), traços (~10 % pintado) e vazias (o
/// pen-down de `mede_o_pen_down_da_aguada_4k`) —, sem salto
/// (A) × com salto (B), intercaladas ABBA, o mínimo de 8 de cada; a saída é a mesma ao bit.
#[test]
#[ignore]
fn mede_o_salto_alfa_zero_4k() {
    use super::super::compose::SEM_SALTO;
    use std::sync::atomic::Ordering::Relaxed;
    let (w, h) = (3840u32, 2160u32);
    for cena in ["tinta", "tracos", "vazias"] {
        let mut s = LayerStack::new();
        let mut src = MapPixelSource::default();
        let base = s.add_raster("base", w, h).unwrap();
        src.insert(
            base,
            LayerImage {
                width: w,
                height: h,
                rgba8: vec![255; (w * h * 4) as usize].into(),
            },
        );
        for k in 1..3u32 {
            let id = s.add_raster(format!("L{k}"), w, h).unwrap();
            let mut img = tinta(w, h, k);
            if cena == "vazias" {
                img = LayerImage::transparent(w, h);
            } else if cena == "tracos" {
                for (i, p) in img
                    .rgba8_mut()
                    .as_chunks_mut::<4>()
                    .0
                    .iter_mut()
                    .enumerate()
                {
                    let (x, y) = (i as u32 % w, i as u32 / w);
                    p[3] = if (x / 48 + y / 48 + k) % 10 == 0 {
                        255
                    } else {
                        0
                    };
                }
            }
            src.insert(id, img);
        }
        let ancora = s.add_raster("ancora", w, h).unwrap();
        let chao = [231u8, 222, 200];
        let mut best = [f64::MAX; 2];
        let mut saidas: [Option<Vec<u8>>; 2] = [None, None];
        for i in 0..16usize {
            let v = [0usize, 1, 1, 0][i % 4];
            SEM_SALTO.store(v == 0, Relaxed);
            let t0 = std::time::Instant::now();
            let out = composite_below(&s, &src, w, h, ancora, chao);
            best[v] = best[v].min(t0.elapsed().as_secs_f64() * 1e3);
            saidas[v] = Some(out);
        }
        SEM_SALTO.store(false, Relaxed);
        assert!(saidas[0] == saidas[1], "{cena}: o salto mudou a saída");
        println!(
            "[B19.11 salto] 3840x2160, {cena}: sem salto {:.2} ms | com salto {:.2} ms",
            best[0], best[1]
        );
    }
}

/// ⭐ **Um fio descartado pára entre faixas** (n.º 7 item 3f, «Continuação 6»): com `corte.para` de pé a meio, cada
/// banda acaba no máximo a faixa em curso (`depois ≤ bandas`), o plano não sai (`None`) e muito ficou por compor.
/// CONTROLO: sem corte, a mesma rota é o `composite_below` ao bit e acaba todas as faixas.
///
/// **Mutação que sangra:** tirar o `if parou() { return; }` do laço das faixas (`depois` ≈ o resto do plano).
#[test]
fn o_fundo_cortado_para_entre_faixas() {
    use std::sync::atomic::Ordering::SeqCst;
    let (w, h, bandas) = (1024u32, 512u32, 4usize);
    let chao = [231u8, 222, 200];
    let (s, src, ancora) = pilha(w, h, None);
    let faixas = h; // uma linha por faixa
    let inteiro = Corte::default();
    let todo = composite_below_corte(
        &s,
        &src,
        w,
        h,
        ancora,
        chao,
        Some(bandas),
        w as usize,
        Some(&inteiro),
    );
    assert!(
        todo.as_deref() == Some(&composite_below(&s, &src, w, h, ancora, chao)[..]),
        "CONTROLO: sem corte, o mesmo bit"
    );
    assert_eq!(
        inteiro.feitas.load(SeqCst),
        faixas,
        "CONTROLO: todas as faixas"
    );
    assert_eq!(inteiro.depois.load(SeqCst), 0);

    let corte = Corte::default();
    let (saiu, ms) = std::thread::scope(|e| {
        let fio = e.spawn(|| {
            composite_below_corte(
                &s,
                &src,
                w,
                h,
                ancora,
                chao,
                Some(bandas),
                w as usize,
                Some(&corte),
            )
        });
        while corte.feitas.load(SeqCst) == 0 {
            std::hint::spin_loop();
        }
        let t0 = std::time::Instant::now();
        corte.para.store(true, SeqCst);
        let saiu = fio.join().expect("o fio");
        (saiu, t0.elapsed().as_secs_f64() * 1e3)
    });
    let (feitas, depois) = (corte.feitas.load(SeqCst), corte.depois.load(SeqCst));
    eprintln!("cortado: {feitas}/{faixas} faixas, {depois} depois do corte, saiu em {ms:.3} ms");
    assert!(saiu.is_none(), "cortado, o plano não sai");
    assert!(
        depois as usize <= bandas,
        "≤ 1 faixa por banda depois do corte: {depois} > {bandas}"
    );
    assert!(feitas < faixas / 2, "parou cedo: {feitas}/{faixas}");
}
