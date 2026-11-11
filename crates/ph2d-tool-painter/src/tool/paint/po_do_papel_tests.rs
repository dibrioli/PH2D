use super::{LADO, PoDoTraco, po_da_tela, po_na_janela};
use crate::compositor::Region;
use ph2d_painter_brush::texture::{RAIO_DA_CAIXA, caixa};
use rayon::prelude::*;

/// ORÁCULO (verbatim antes do B49.1): o passe vertical sobre a tela TRANSPOSTA, com `vec!` por linha.
fn po_da_tela_antigo(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
    if !rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 0) {
        return Vec::new();
    }
    let tres = |v: &mut Vec<f32>, n: usize| {
        v.par_chunks_mut(n).for_each(|linha| {
            let mut tmp = vec![0.0f32; n];
            for _ in 0..3 {
                caixa(linha, &mut tmp, RAIO_DA_CAIXA);
                linha.copy_from_slice(&tmp);
            }
        });
    };
    let mut a: Vec<f32> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| f32::from(p[3]) / 255.0)
        .collect();
    tres(&mut a, w);
    let mut t = vec![0.0f32; w * h];
    t.par_chunks_mut(h).enumerate().for_each(|(x, col)| {
        for (y, c) in col.iter_mut().enumerate() {
            *c = a[y * w + x];
        }
    });
    tres(&mut t, h);
    let mut out = vec![0u8; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, linha)| {
        for (x, o) in linha.iter_mut().enumerate() {
            *o = (t[x * h + y].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    });
    out
}

/// Verbatim do passo 1 (cfa301d907), só para a sonda intercalada: a conversão `f32` em série e as cópias entre caixas.
fn po_da_tela_passo1(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
    const FAIXA: usize = 32;
    if !rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 0) {
        return Vec::new();
    }
    let mut a: Vec<f32> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| f32::from(p[3]) / 255.0)
        .collect();
    a.par_chunks_mut(w).for_each_init(
        || vec![0.0f32; w],
        |tmp, linha| {
            for _ in 0..3 {
                caixa(linha, tmp, RAIO_DA_CAIXA);
                linha.copy_from_slice(tmp);
            }
        },
    );
    let mut faixas = vec![0u8; w * h];
    faixas.par_chunks_mut(h * FAIXA).enumerate().for_each_init(
        || (vec![0.0f32; h * FAIXA], vec![0.0f32; h * FAIXA]),
        |(f, g), (s, saida)| {
            let x0 = s * FAIXA;
            let l = saida.len() / h;
            let (f, g) = (&mut f[..h * l], &mut g[..h * l]);
            for (y, dst) in f.chunks_exact_mut(l).enumerate() {
                dst.copy_from_slice(&a[y * w + x0..y * w + x0 + l]);
            }
            for _ in 0..3 {
                super::caixa_em_faixa(f, g, l, RAIO_DA_CAIXA);
                f.copy_from_slice(g);
            }
            for (o, v) in saida.iter_mut().zip(f.iter()) {
                *o = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
        },
    );
    let mut out = vec![0u8; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, linha)| {
        for (s, faixa) in faixas.chunks(h * FAIXA).enumerate() {
            let l = faixa.len() / h;
            linha[s * FAIXA..s * FAIXA + l].copy_from_slice(&faixa[y * l..(y + 1) * l]);
        }
    });
    out
}

/// Uma camada `w`×`h` com alfa de todos os níveis (ruído determinístico, manchas e furos), metade vazia à direita.
fn tela(w: usize, h: usize, semente: u32) -> Vec<u8> {
    let mut s = semente.wrapping_mul(2_654_435_761).max(1);
    (0..w * h)
        .flat_map(|i| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let (x, y) = (i % w, i / w);
            let a = if x > w / 2 && y % 3 != 0 {
                0
            } else if (x / 9 + y / 5) % 4 == 0 {
                255
            } else {
                (s >> 24) as u8
            };
            [0, 0, 0, a]
        })
        .collect()
}

/// As faixas de 32 colunas dão o pó do oráculo transposto ao bit: larguras múltiplas de 32, ímpares, menores que a
/// faixa e que a caixa, e a 4K.
#[test]
fn as_faixas_do_po_sao_a_transposicao_ao_bit() {
    for (i, &(w, h)) in [
        (1, 1),
        (1, 40),
        (40, 1),
        (3, 4),
        (31, 31),
        (32, 11),
        (33, 64),
        (65, 23),
        (127, 97),
        (3840, 2160),
    ]
    .iter()
    .enumerate()
    {
        let rgba = tela(w, h, i as u32 + 1);
        // A mesma tela com faixas de linhas SEM tinta (o salto das linhas vazias do passe horizontal).
        let mut furada = rgba.clone();
        for (y, linha) in furada.chunks_mut(w * 4).enumerate() {
            if y % 7 < 3 || y < h / 4 {
                linha.fill(0);
            }
        }
        for (nome, rgba) in [("cheia", &rgba), ("furada", &furada)] {
            let novo = po_da_tela(rgba, w, h);
            let velho = po_da_tela_antigo(rgba, w, h);
            assert_eq!(novo.len(), velho.len(), "{w}x{h} {nome}");
            let dif = novo.iter().zip(&velho).filter(|(a, b)| a != b).count();
            assert_eq!(dif, 0, "{w}x{h} {nome}: {dif} texels diferem do oráculo");
        }
    }
    assert!(po_da_tela(&[0u8; 4 * 12], 4, 3).is_empty());
}

/// O custo do pó no pen-down a 4K (3840×2160, metade com tinta), o oráculo transposto e as faixas INTERCALADOS no
/// mesmo processo (o mínimo vale): `--profile smoke`.
#[test]
#[ignore = "medição: cargo test --profile smoke -- --ignored mede_o_po_a_4k --nocapture"]
fn mede_o_po_a_4k() {
    let (w, h) = (3840usize, 2160usize);
    let rgba: Vec<u8> = (0..w * h)
        .flat_map(|i| [0, 0, 0, if (i / 7) % 2 == 0 { 255 } else { 0 }])
        .collect();
    let vazia = vec![0u8; w * h * 4];
    // Só um quarto das linhas com tinta (um desenho na parte de baixo da tela).
    let mut quarto = rgba.clone();
    quarto[..w * 4 * (h * 3 / 4)].fill(0);
    let (mut q1, mut q) = (Vec::new(), Vec::new());
    for _ in 0..9 {
        let t0 = std::time::Instant::now();
        let a = po_da_tela_passo1(&quarto, w, h);
        q1.push(t0.elapsed().as_secs_f64() * 1e3);
        let t0 = std::time::Instant::now();
        let b = po_da_tela(&quarto, w, h);
        q.push(t0.elapsed().as_secs_f64() * 1e3);
        assert_eq!(a, b);
    }
    q1.sort_by(f64::total_cmp);
    q.sort_by(f64::total_cmp);
    eprintln!(
        "[po 4K, 1/4 das linhas com tinta] passo 1: mín {:.2} · med {:.2} | agora: mín {:.2} · med {:.2} ms",
        q1[0], q1[4], q[0], q[4]
    );
    let (mut velho, mut passo1, mut novo, mut nada1, mut nada) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let ms = |t0: std::time::Instant| t0.elapsed().as_secs_f64() * 1e3;
    for _ in 0..9 {
        let t0 = std::time::Instant::now();
        let p = po_da_tela_antigo(&rgba, w, h);
        velho.push(ms(t0));
        let t0 = std::time::Instant::now();
        let q = po_da_tela_passo1(&rgba, w, h);
        passo1.push(ms(t0));
        let t0 = std::time::Instant::now();
        let r = po_da_tela(&rgba, w, h);
        novo.push(ms(t0));
        assert_eq!(p, q);
        assert_eq!(p, r);
        let t0 = std::time::Instant::now();
        assert!(po_da_tela_passo1(&vazia, w, h).is_empty());
        nada1.push(ms(t0));
        let t0 = std::time::Instant::now();
        assert!(po_da_tela(&vazia, w, h).is_empty());
        nada.push(ms(t0));
    }
    for v in [&mut velho, &mut passo1, &mut novo, &mut nada1, &mut nada] {
        v.sort_by(f64::total_cmp);
    }
    eprintln!(
        "[po 4K] transposto: mín {:.2} · med {:.2} | passo 1: mín {:.2} · med {:.2} | agora: mín {:.2} · med {:.2} ms",
        velho[0], velho[4], passo1[0], passo1[4], novo[0], novo[4]
    );
    eprintln!(
        "[po 4K, camada vazia] passo 1: mín {:.2} · med {:.2} | agora: mín {:.2} · med {:.2} ms",
        nada1[0], nada1[4], nada[0], nada[4]
    );
}

/// O pó de uma janela montado de ladrilhos (256², ímpares, 1 texel, o apron a cortar a borda) é o da janela da tela
/// INTEIRA ao bit: a lei por janela não tem história. E fica a menos de 1 nível `u8` da soma corrida de [`po_da_tela`].
#[test]
fn os_ladrilhos_do_po_sao_a_tela_inteira_ao_bit() {
    for (i, &(w, h, t)) in [
        (1, 1, 1),
        (1, 40, 7),
        (40, 1, 3),
        (31, 31, 4),
        (65, 23, 16),
        (127, 97, 37),
        (600, 300, 256),
        (3840, 2160, 256),
    ]
    .iter()
    .enumerate()
    {
        let rgba = tela(w, h, i as u32 + 11);
        let inteira = po_na_janela(&rgba, w, h, (0, 0, w, h));
        let mut montada = vec![0u8; w * h];
        for ty in (0..h).step_by(t) {
            for tx in (0..w).step_by(t) {
                let (x1, y1) = ((tx + t).min(w), (ty + t).min(h));
                let l = po_na_janela(&rgba, w, h, (tx, ty, x1, y1));
                for (y, linha) in l.chunks(x1 - tx).enumerate() {
                    montada[(ty + y) * w + tx..(ty + y) * w + x1].copy_from_slice(linha);
                }
            }
        }
        let dif = inteira.iter().zip(&montada).filter(|(a, b)| a != b).count();
        assert_eq!(
            dif, 0,
            "{w}x{h} ladrilho {t}: {dif} texels diferem da tela inteira"
        );
        let corrida = po_da_tela(&rgba, w, h);
        let (n, max) = corrida.iter().zip(&inteira).fold((0, 0), |(n, m), (a, b)| {
            let d = a.abs_diff(*b);
            (n + usize::from(d != 0), m.max(d))
        });
        assert!(
            max <= 1,
            "{w}x{h}: a lei por janela afasta-se {max} níveis da soma corrida"
        );
        eprintln!(
            "[po {w}x{h}] lei por janela × soma corrida: {n} de {} texels a 1 nível",
            w * h
        );
    }
}

/// O custo de um ladrilho 256² (o 1.º lote de um traço pequeno) × a tela inteira, intercalados: `--profile smoke`.
#[test]
#[ignore = "medição: cargo test --profile smoke -- --ignored mede_o_ladrilho_do_po_a_4k --nocapture"]
fn mede_o_ladrilho_do_po_a_4k() {
    let (w, h) = (3840usize, 2160usize);
    let rgba = tela(w, h, 5);
    let (mut tela_ms, mut lad, mut quatro) = (Vec::new(), Vec::new(), Vec::new());
    let ms = |t0: std::time::Instant| t0.elapsed().as_secs_f64() * 1e3;
    for _ in 0..9 {
        let t0 = std::time::Instant::now();
        let a = po_da_tela(&rgba, w, h);
        tela_ms.push(ms(t0));
        let t0 = std::time::Instant::now();
        let b = po_na_janela(&rgba, w, h, (1024, 768, 1280, 1024));
        lad.push(ms(t0));
        let t0 = std::time::Instant::now();
        let c = po_na_janela(&rgba, w, h, (1024, 768, 1536, 1280));
        quatro.push(ms(t0));
        assert!(a.len() == w * h && b.len() == 256 * 256 && c.len() == 512 * 512);
    }
    for v in [&mut tela_ms, &mut lad, &mut quatro] {
        v.sort_by(f64::total_cmp);
    }
    eprintln!(
        "[po 4K] tela inteira: mín {:.2} · med {:.2} | 1 ladrilho 256²: mín {:.3} · med {:.3} | 2×2 ladrilhos: mín {:.3} · med {:.3} ms",
        tela_ms[0], tela_ms[4], lad[0], lad[4], quatro[0], quatro[4]
    );
}

fn alfa_de(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect()
}

/// O pó preguiçoso só desfoca os ladrilhos que a caixa do lote toca, e o que desfoca é o da tela inteira ao bit; a
/// tela sem tinta não tem pó; a tela toda garantida é a lei por janela inteira.
#[test]
fn o_po_so_toca_a_janela_do_traco() {
    for (i, &(w, h, caixas)) in [
        (
            300usize,
            200usize,
            &[(10u32, 10u32, 20u32, 20u32), (250, 150, 50, 50)][..],
        ),
        (
            1000,
            700,
            &[(500, 300, 60, 60), (240, 250, 40, 40), (999, 699, 1, 1)][..],
        ),
        (3840, 2160, &[(1900, 1000, 120, 90), (0, 0, 3, 3)][..]),
    ]
    .iter()
    .enumerate()
    {
        let rgba = tela(w, h, i as u32 + 31);
        let inteira = po_na_janela(&rgba, w, h, (0, 0, w, h));
        let mut po = PoDoTraco::do_alfa(alfa_de(&rgba), w, h);
        assert!(!po.vazio());
        let mut tocados = vec![false; w * h];
        for &(x, y, cw, ch) in caixas {
            po.garante(Region { x, y, w: cw, h: ch });
            let (x0, y0) = (x as usize, y as usize);
            let (x1, y1) = ((x0 + cw as usize).min(w), (y0 + ch as usize).min(h));
            for ty in y0 / LADO..=(y1 - 1) / LADO {
                for tx in x0 / LADO..=(x1 - 1) / LADO {
                    for yy in ty * LADO..((ty + 1) * LADO).min(h) {
                        for xx in tx * LADO..((tx + 1) * LADO).min(w) {
                            tocados[yy * w + xx] = true;
                        }
                    }
                }
            }
        }
        let n = tocados.iter().filter(|&&t| t).count();
        assert_eq!(
            po.texels_prontos(),
            n,
            "{w}x{h}: desfocou fora dos ladrilhos da caixa"
        );
        assert!(
            n < w * h || w * h <= LADO * LADO * 2,
            "{w}x{h}: desfocou a tela inteira"
        );
        let mut dif = 0;
        for (k, (&a, &b)) in po.plano().iter().zip(&inteira).enumerate() {
            let esperado = if tocados[k] { b } else { 0 };
            dif += usize::from(a != esperado);
        }
        assert_eq!(
            dif, 0,
            "{w}x{h}: {dif} texels do pó preguiçoso diferem da tela inteira"
        );
        po.garante(Region {
            x: 0,
            y: 0,
            w: w as u32,
            h: h as u32,
        });
        assert_eq!(po.texels_prontos(), w * h);
        assert!(
            po.plano() == inteira.as_slice(),
            "{w}x{h}: a tela garantida não é a inteira"
        );
    }
    assert!(PoDoTraco::do_alfa(vec![0; 12], 4, 3).vazio());
}

/// O pen-down 4K de um Pastel: a tela inteira (`po_da_tela`, o de antes) × o preguiçoso (o alfa + 1 caixa de dab de
/// 120 px), intercalados: `--profile smoke`.
#[test]
#[ignore = "medição: cargo test --profile smoke -- --ignored mede_o_po_preguicoso_a_4k --nocapture"]
fn mede_o_po_preguicoso_a_4k() {
    use rayon::prelude::*;
    let (w, h) = (3840usize, 2160usize);
    let rgba = tela(w, h, 5);
    let (mut antes, mut depois) = (Vec::new(), Vec::new());
    let ms = |t0: std::time::Instant| t0.elapsed().as_secs_f64() * 1e3;
    for _ in 0..9 {
        let t0 = std::time::Instant::now();
        let a = po_da_tela(&rgba, w, h);
        antes.push(ms(t0));
        let t0 = std::time::Instant::now();
        let alfa = rgba.as_chunks::<4>().0.par_iter().map(|p| p[3]).collect();
        let mut po = PoDoTraco::do_alfa(alfa, w, h);
        po.garante(Region {
            x: 1900,
            y: 1000,
            w: 122,
            h: 122,
        });
        depois.push(ms(t0));
        assert!(a.len() == w * h && po.plano().len() == w * h);
    }
    for v in [&mut antes, &mut depois] {
        v.sort_by(f64::total_cmp);
    }
    eprintln!(
        "[po 4K pen-down] tela inteira: mín {:.2} · med {:.2} | preguiçoso: mín {:.2} · med {:.2} ms",
        antes[0], antes[4], depois[0], depois[4]
    );
}
