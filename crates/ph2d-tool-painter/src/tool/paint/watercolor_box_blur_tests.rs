//! O [`box_blur`] em FAIXAS de colunas dá o byte da versão de antes (a coluna varrida para um
//! buffer transposto e transposta de volta) — ADR-0173.

use super::*;

/// O `box_blur` de ANTES da passagem vertical em faixas, copiado À LETRA (em série: a ordem das
/// somas por linha e por coluna é a mesma em paralelo, que é o que o fazia byte-idêntico).
fn box_blur_de_antes(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    if radius == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let mut tmp = vec![0.0f32; w * h];
    for (trow, srow) in tmp.chunks_mut(w).zip(src.chunks(w)) {
        let mut pref = vec![0.0f32; w + 1];
        for x in 0..w {
            pref[x + 1] = pref[x] + srow[x];
        }
        for (x, t) in trow.iter_mut().enumerate() {
            let lo = x.saturating_sub(radius);
            let hi = (x + radius).min(w - 1);
            *t = (pref[hi + 1] - pref[lo]) / (hi - lo + 1) as f32;
        }
    }
    let mut out_t = vec![0.0f32; w * h];
    for (x, ocol) in out_t.chunks_mut(h).enumerate() {
        let mut pref = vec![0.0f32; h + 1];
        for y in 0..h {
            pref[y + 1] = pref[y] + tmp[y * w + x];
        }
        for (y, o) in ocol.iter_mut().enumerate() {
            let lo = y.saturating_sub(radius);
            let hi = (y + radius).min(h - 1);
            *o = (pref[hi + 1] - pref[lo]) / (hi - lo + 1) as f32;
        }
    }
    let mut out = vec![0.0f32; w * h];
    for (y, orow) in out.chunks_mut(w).enumerate() {
        for (x, o) in orow.iter_mut().enumerate() {
            *o = out_t[x * h + y];
        }
    }
    out
}

/// Um campo com MAGNITUDES misturadas (a soma em `f32` só é sensível à ordem quando os termos têm
/// escalas diferentes — um campo liso não apanharia uma ordem trocada) e reprodutível.
fn campo(w: usize, h: usize, semente: u32) -> Vec<f32> {
    let mut s = semente.wrapping_mul(2_654_435_761).max(1);
    (0..w * h)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let u = (s >> 8) as f32 / (1u32 << 24) as f32;
            match s % 4 {
                0 => u * 1e-3,
                1 => u * 255.0,
                2 => 0.0,
                _ => u,
            }
        })
        .collect()
}

#[test]
fn o_borrao_em_faixas_da_o_byte_da_versao_de_antes() {
    // Larguras abaixo, igual e acima da faixa, não múltiplas dela; uma linha só; uma coluna só;
    // raios maiores que a imagem.
    let casos = [
        (1usize, 1usize, 1usize),
        (1, 37, 3),
        (37, 1, 3),
        (63, 17, 2),
        (64, 64, 5),
        (65, 40, 7),
        (130, 90, 12),
        (200, 150, 33),
        (31, 20, 100),
    ];
    let mut mudou = 0usize;
    for (i, &(w, h, r)) in casos.iter().enumerate() {
        let src = campo(w, h, i as u32 + 7);
        let novo = box_blur(&src, w, h, r);
        let antes = box_blur_de_antes(&src, w, h, r);
        assert_eq!(novo.len(), antes.len());
        for (k, (a, b)) in novo.iter().zip(&antes).enumerate() {
            assert_eq!(a.to_bits(), b.to_bits(), "{w}×{h} r{r}, texel {k}: {a} contra {b}");
        }
        mudou += novo.iter().zip(&src).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    }
    // CONTROLO: o borrão de facto mexe no campo (senão compararíamos duas cópias da entrada).
    assert!(mudou > 10_000, "o borrão tem de mudar o campo: mudou {mudou}");
}
