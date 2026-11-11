use super::po_da_tela;
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
        let novo = po_da_tela(&rgba, w, h);
        let velho = po_da_tela_antigo(&rgba, w, h);
        assert_eq!(novo.len(), velho.len(), "{w}x{h}");
        let dif = novo.iter().zip(&velho).filter(|(a, b)| a != b).count();
        assert_eq!(dif, 0, "{w}x{h}: {dif} texels diferem do oráculo");
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
    let (mut velho, mut novo) = (Vec::new(), Vec::new());
    for _ in 0..9 {
        let t0 = std::time::Instant::now();
        let p = po_da_tela_antigo(&rgba, w, h);
        velho.push(t0.elapsed().as_secs_f64() * 1e3);
        let t0 = std::time::Instant::now();
        let q = po_da_tela(&rgba, w, h);
        novo.push(t0.elapsed().as_secs_f64() * 1e3);
        assert_eq!(p, q);
    }
    velho.sort_by(f64::total_cmp);
    novo.sort_by(f64::total_cmp);
    eprintln!(
        "[po 4K] transposto: mín {:.1} ms · mediana {:.1} ms | faixas: mín {:.1} ms · mediana {:.1} ms",
        velho[0], velho[4], novo[0], novo[4]
    );
}
