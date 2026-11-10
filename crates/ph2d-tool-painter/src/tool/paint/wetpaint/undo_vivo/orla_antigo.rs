//! O enterro da orla de antes do B12.2 (a grade inteira), VERBATIM salvo o contador: o oráculo de
//! `o_enterro_sem_passo_na_caixa_da_o_bit_do_antigo`.

use super::super::*;
use super::{CaixaDeCelulas, visita};

thread_local! { static ANTIGO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

pub(in super::super) fn get() -> bool {
    ANTIGO.with(std::cell::Cell::get)
}

pub(in super::super) fn set(v: bool) {
    ANTIGO.with(|c| c.set(v));
}

/// As células a até `r` (Chebyshev) de uma marcada, e a caixa delas.
pub(in super::super) fn perto_de(
    marcadas: &[bool],
    gw: usize,
    gh: usize,
    r: usize,
) -> (Vec<bool>, Option<CaixaDeCelulas>) {
    let mut fora = vec![false; gw * gh];
    let (Some(a), Some(b)) = (
        marcadas.iter().position(|m| *m),
        marcadas.iter().rposition(|m| *m),
    ) else {
        return (fora, None);
    };
    visita(a + marcadas.len() - b);
    let (y0, y1) = ((a / gw).saturating_sub(r), (b / gw + r).min(gh - 1));
    // Por linha, depois por coluna: cada passada é linear.
    let mut por_linha = vec![false; gw * gh];
    for y in a / gw..=b / gw {
        visita(2 * gw);
        let (linha, saida) = (
            &marcadas[y * gw..(y + 1) * gw],
            &mut por_linha[y * gw..(y + 1) * gw],
        );
        let mut d = usize::MAX;
        for x in 0..gw {
            d = if linha[x] { 0 } else { d.saturating_add(1) };
            saida[x] = d <= r;
        }
        d = usize::MAX;
        for x in (0..gw).rev() {
            d = if linha[x] { 0 } else { d.saturating_add(1) };
            saida[x] |= d <= r;
        }
    }
    let mut d = vec![usize::MAX; gw];
    for y in y0..=y1 {
        visita(gw);
        for x in 0..gw {
            d[x] = if por_linha[y * gw + x] {
                0
            } else {
                d[x].saturating_add(1)
            };
            fora[y * gw + x] = d[x] <= r;
        }
    }
    d.fill(usize::MAX);
    for y in (y0..=y1).rev() {
        visita(gw);
        for x in 0..gw {
            d[x] = if por_linha[y * gw + x] {
                0
            } else {
                d[x].saturating_add(1)
            };
            fora[y * gw + x] |= d[x] <= r;
        }
    }
    let (mut cx0, mut cx1) = (gw, 0);
    for y in y0..=y1 {
        let l = &fora[y * gw..(y + 1) * gw];
        if let Some(x) = l.iter().position(|m| *m) {
            cx0 = cx0.min(x);
        }
        if let Some(x) = l.iter().rposition(|m| *m) {
            cx1 = cx1.max(x);
        }
    }
    (fora, Some((cx0 + 1, y0 + 1, cx1 + 1, y1 + 1)))
}

/// As células invisíveis perto do toque, na grade inteira.
pub(in super::super) fn invisiveis(sess: &WetSession, perto: &[bool]) -> Vec<bool> {
    let (gw, gh) = sess.grid;
    visita(gw * gh);
    let g = sess.engine.active_grid();
    (0..gw * gh)
        .map(|k| {
            let c = (k / gw + 1) * g.s + k % gw + 1;
            perto[k] && sess.planos.pigment[k * 4 + 3] == 0 && g.sett[c] + g.susp[c] > 0.0
        })
        .collect()
}
