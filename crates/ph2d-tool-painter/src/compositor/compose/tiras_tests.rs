//! G1 of B19.10: the cache strips (every layer per strip of `linhas_da_tira` rows) and the α = 0
//! skip in every mode are the pre-B19.6 single walk WITHOUT any skip, to the bit — 22 modes,
//! opacities, mask, clipping, group, per-pixel and spatial adjustments, glass — at strip heights
//! 1/2/3/4/7/whole × 1/2/3/7/16/32 bands.

use super::barreira::{FIOS_DA_REGIAO, LINHAS_DA_TIRA};
use super::barreira_tests::{bits, compor_regiao_antigo, pilha, raster};
use super::resumo::SEM_RESUMO;
use super::*;
use std::sync::atomic::Ordering::Relaxed;

#[test]
fn as_tiras_de_cache_e_o_salto_em_todos_os_modos_sao_a_rota_antiga_ao_bit() {
    let _trava = crate::compositor::compose::resumo::trava_das_chaves();
    for (w, h) in [(1, 1), (7, 3), (513, 300), (3840, 20)] {
        for espacial in [true, false] {
            let (s, src, vidros) = pilha(w, h, espacial);
            let janela = Region {
                x: w / 3,
                y: h / 4,
                w: (w / 2).max(1),
                h: (h / 2).max(1),
            };
            for r in [Region { x: 0, y: 0, w, h }, janela] {
                for v in [None, Some(&vidros)] {
                    SEM_SALTO.store(true, Relaxed);
                    let antigo = bits(&compor_regiao_antigo(&s, &src, w, h, r, v));
                    SEM_SALTO.store(false, Relaxed);
                    for linhas in [1u32, 2, 3, 4, 7, u32::MAX, 0] {
                        LINHAS_DA_TIRA.store(linhas, Relaxed);
                        for fios in [1usize, 2, 3, 7, 16, 32] {
                            FIOS_DA_REGIAO.store(fios, Relaxed);
                            let novo = bits(&compor_regiao(&s, &src, w, h, r, v));
                            FIOS_DA_REGIAO.store(0, Relaxed);
                            assert!(
                                novo == antigo,
                                "{w}x{h} espacial={espacial} {r:?} vidro={} tira {linhas} {fios} fios",
                                v.is_some()
                            );
                        }
                    }
                    LINHAS_DA_TIRA.store(0, Relaxed);
                }
            }
        }
    }
}

/// Sonda B19.10 (`--ignored`, `--profile smoke`, SOZINHA: mexe nas chaves globais): `composite` a
/// 3840×2160, base opaca + N ∈ {1, 4, 16} camadas (modos Normal·Multiply·Screen·Overlay em roda) —
/// vazias, traços (~10 % pintado) e ruído (¼ de α = 0) — em quatro rotas: A = a de antes (banda
/// inteira, salto só no Normal) · B = tiras · C = salto em todos os modos · D = tiras + salto ·
/// E = D + o resumo por ladrilho (o produto). ABCDEEDCBA, o mínimo de 6 de cada; a mesma saída ao bit.
#[test]
#[ignore]
fn mede_as_tiras_de_cache_4k() {
    let _trava = crate::compositor::compose::resumo::trava_das_chaves();
    let (w, h) = (3840u32, 2160u32);
    for cena in ["vazias", "tracos", "ruido"] {
        for n in [1u32, 4, 16] {
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
            for k in 0..n {
                let id = raster(&mut s, &mut src, w, h, k + 1);
                s.set_blend_mode(id, BlendMode::from_u8([0u8, 1, 6, 9][(k % 4) as usize]));
                let img = src.images.get_mut(&id).unwrap();
                if cena == "vazias" {
                    *img = LayerImage::transparent(w, h);
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
            }
            let mut best = [f64::MAX; 5];
            let mut saidas: [Option<Vec<u8>>; 5] = [None, None, None, None, None];
            for i in 0..30usize {
                let v = [0usize, 1, 2, 3, 4, 4, 3, 2, 1, 0][i % 10];
                LINHAS_DA_TIRA.store(if v == 0 || v == 2 { u32::MAX } else { 0 }, Relaxed);
                SALTO_SO_NORMAL.store(v < 2, Relaxed);
                SEM_RESUMO.store(v < 4, Relaxed);
                let t0 = std::time::Instant::now();
                let out = composite(&s, &src, w, h);
                best[v] = best[v].min(t0.elapsed().as_secs_f64() * 1e3);
                saidas[v] = Some(out);
            }
            LINHAS_DA_TIRA.store(0, Relaxed);
            SALTO_SO_NORMAL.store(false, Relaxed);
            SEM_RESUMO.store(false, Relaxed);
            assert!(
                saidas.iter().all(|o| *o == saidas[0]),
                "{cena} N={n}: a saída mudou"
            );
            println!(
                "[B19.10] 3840x2160 {cena} N={n}: antes {:.1} ms | tiras {:.1} | salto todos {:.1} | tiras+salto {:.1} | +ladrilhos {:.1}",
                best[0], best[1], best[2], best[3], best[4]
            );
        }
    }
}
