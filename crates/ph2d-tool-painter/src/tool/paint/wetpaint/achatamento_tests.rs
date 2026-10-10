//! **O escorrimento não se corta em RECTÂNGULOS** (W1f; smoke do dono 10/10: *«o estranho achatamento do
//! escorrimento que acaba por formar áreas retas»*).
//!
//! A cena: o S do oráculo ([`super::sonda_achatamento::cena`], tela 1000×640, Wet Round a 100 px, água 1,
//! 30 s). A régua é a de `ferramentas/recto.py` (a mesma conta), calibrada no Rebelle
//! (`fixtures/rebelle_gotas/s_*.png`, cinco telas): a orla de baixo nunca fica plana mais de **22**
//! colunas seguidas e **0** fios largos têm a ponta cortada a direito. A captura do dono lê 62 colunas.
//!
//! ⚠️ **O controlo está DENTRO:** a mesma cena com a lei da gota da W1e (`drip_head = ALTURA_DA_GOTA`, o
//! que cortava em rectângulos) tem de REPROVAR — sem isso um verde não diria se a régua discrimina.
use super::sonda_achatamento::{H, Variante, W, cena};

/// A orla plana mais longa do Rebelle no S (cinco telas: 19–22).
const PLANO_DO_REBELLE: usize = 22;
/// A fracção de fios largos (≥ 8 colunas) de ponta quadrada que se tolera (Rebelle 0; o produto sem a
/// lei, no máximo, 1 de 24 = 0,04; com ela 3 de 3 a meio e 20 de 22 no máximo).
const PONTAS_QUADRADAS: f64 = 0.15;

fn tinta(px: &[u8]) -> Vec<bool> {
    px.as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            let (mx, mn) = (p[..3].iter().max(), p[..3].iter().min());
            mx.zip(mn).is_some_and(|(a, b)| a - b > 12)
        })
        .collect()
}

/// O comprimento da corrida horizontal de tinta que contém cada pixel.
fn corridas(t: &[bool]) -> Vec<usize> {
    let mut out = vec![0; t.len()];
    for y in 0..H {
        let row = &t[y * W..(y + 1) * W];
        let mut x = 0;
        while x < W {
            if !row[x] {
                x += 1;
                continue;
            }
            let x0 = x;
            while x + 1 < W && row[x + 1] {
                x += 1;
            }
            out[y * W + x0..=y * W + x].fill(x - x0 + 1);
            x += 1;
        }
    }
    out
}

/// O que a régua lê: (a orla plana mais longa, fios largos, deles de ponta quadrada).
pub(super) fn le(px: &[u8]) -> (usize, usize, usize) {
    let t = tinta(px);
    let run = corridas(&t);
    let larga = |x: usize, y: usize| t[y * W + x] && run[y * W + x] > 8;
    let cols: Vec<usize> = (0..W).filter(|&x| (0..H).any(|y| larga(x, y))).collect();
    let (a, b) = (cols[0], cols[cols.len() - 1]);
    let m = (b - a) * 8 / 100;
    let xs: Vec<usize> = (a + m..=b - m).collect();
    // A orla de baixo: desce desde a primeira tinta larga, sem buraco de mais de 3 linhas.
    let orla: Vec<Option<usize>> = xs
        .iter()
        .map(|&x| {
            let y0 = (0..H).find(|&y| larga(x, y))?;
            let (mut yb, mut buraco) = (y0, 0);
            for y in y0..H {
                if larga(x, y) {
                    (yb, buraco) = (y, 0);
                } else {
                    buraco += 1;
                    if buraco > 3 {
                        break;
                    }
                }
            }
            Some(yb)
        })
        .collect();
    let mut plano = 0;
    let mut k = 0;
    while k < orla.len() {
        let mut j = k;
        while orla[k].is_some() && j + 1 < orla.len() && orla[j + 1] == orla[k] {
            j += 1;
        }
        if orla[k].is_some() {
            plano = plano.max(j - k + 1);
        }
        k = j + 1;
    }
    // Os fios: a tinta mais baixa de cada coluna, ≥ 6 px abaixo da orla mediana (±40 colunas).
    let baixo: Vec<Option<usize>> = xs
        .iter()
        .map(|&x| (0..H).rev().find(|&y| t[y * W + x]))
        .collect();
    let base: Vec<Option<usize>> = (0..orla.len())
        .map(|k| {
            let mut j: Vec<usize> = orla[k.saturating_sub(40)..(k + 41).min(orla.len())]
                .iter()
                .flatten()
                .copied()
                .collect();
            j.sort_unstable();
            (!j.is_empty()).then(|| j[j.len() / 2])
        })
        .collect();
    let fio = |k: usize| matches!((baixo[k], base[k]), (Some(b), Some(m)) if b >= m + 6);
    let (mut fios, mut quadrados, mut k) = (0, 0, 0);
    while k < baixo.len() {
        if !fio(k) {
            k += 1;
            continue;
        }
        let mut j = k;
        while j + 1 < baixo.len() && fio(j + 1) {
            j += 1;
        }
        let w = j - k + 1;
        if w >= 8 {
            let fundo = (k..=j).filter_map(|i| baixo[i]).max().unwrap_or(0);
            let na_ultima = (k..=j)
                .filter(|&i| baixo[i].is_some_and(|b| b + 1 >= fundo))
                .count();
            fios += 1;
            if na_ultima as f64 >= 0.7 * w as f64 {
                quadrados += 1;
            }
        }
        k = j + 1;
    }
    (plano, fios, quadrados)
}

pub(super) fn barra((plano, fios, quadrados): (usize, usize, usize)) -> Result<(), String> {
    let mut f = Vec::new();
    if plano > PLANO_DO_REBELLE {
        f.push(format!(
            "orla de baixo plana em {plano} colunas seguidas (Rebelle ≤ {PLANO_DO_REBELLE})"
        ));
    }
    if fios > 0 && quadrados as f64 > PONTAS_QUADRADAS * fios as f64 {
        f.push(format!(
            "{quadrados} de {fios} fios com a ponta cortada a direito (Rebelle 0)"
        ));
    }
    if f.is_empty() {
        Ok(())
    } else {
        Err(f.join(" · "))
    }
}

#[test]
fn o_escorrimento_do_s_nao_se_corta_em_rectangulos() {
    let lei = Variante {
        gota: Some(ph2d_wet_paint::solver::ALTURA_DA_GOTA),
        ..Variante::default()
    };
    let mut falhas = Vec::new();
    for (nome, tilt) in [("meia", (3, 4)), ("máxima", (3, 8))] {
        let produto = le(&cena(&Variante::default(), Some(tilt), 30, &mut |_, _| {}).0);
        let controlo = le(&cena(&lei, Some(tilt), 30, &mut |_, _| {}).0);
        eprintln!("[recto] {nome}: produto {produto:?} · com a lei da gota {controlo:?}");
        assert!(
            barra(controlo).is_err(),
            "CONTROLO: a lei da gota passou a régua do recto a {nome} — a régua não discrimina"
        );
        if let Err(e) = barra(produto) {
            falhas.push(format!("inclinação {nome}: {e}"));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join(" · "));
}
