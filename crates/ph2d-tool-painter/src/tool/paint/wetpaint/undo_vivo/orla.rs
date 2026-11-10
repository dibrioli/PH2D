//! A orla do enterro: as células perto do que a tela mudou e a caixa dos planos — filho de [`super`] por LOC (B52.R11).

use super::*;

/// O alcance do enterro da orla, em células: a tinta invisível de um traço desfeito fica até 7 células do que a tela
/// mudou (`sonda_alcance_da_orla`: Wet Flat de 20 a 160 px, até 3000 quadros de água). 32 é a folga sobre o medido.
pub(super) const ALCANCE_DA_ORLA: usize = 32;

/// Uma caixa de células: `(x0, y0, x1, y1)`, 1-based e inclusiva.
pub(super) type CaixaDeCelulas = (usize, usize, usize, usize);

/// A caixa a recompor nos planos: a grade inteira só em teste, para comparar.
pub(super) fn caixa_dos_planos(
    c: (usize, usize, usize, usize),
    gw: usize,
    gh: usize,
) -> (usize, usize, usize, usize) {
    #[cfg(test)]
    if super::super::undo_auditoria_tests::grade_inteira::get() {
        return (1, 1, gw, gh);
    }
    let _ = (gw, gh);
    c
}

#[cfg(test)]
#[path = "orla_antigo.rs"]
pub(super) mod antigo;
#[cfg(test)]
#[path = "orla_tests.rs"]
mod tests;

#[cfg(test)]
thread_local! {
    /// As células que o enterro da orla andou: o gate `para_la_do_tecto_o_enterro_da_orla_anda_so_a_caixa`.
    pub(super) static VISITAS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[inline]
fn visita(_n: usize) {
    #[cfg(test)]
    VISITAS.with(|v| v.set(v.get() + _n as u64));
}

/// As células a até `r` (Chebyshev) de uma marcada, numa caixa local à volta do toque.
pub(super) struct Perto {
    /// A caixa das células perto (1-based, inclusiva).
    pub(super) caixa: CaixaDeCelulas,
    x0: usize,
    y0: usize,
    lw: usize,
    m: Vec<bool>,
}

impl Perto {
    /// As linhas da caixa (0-based), cada uma com a coluna da 1.ª célula e a máscara local.
    fn linhas(&self) -> impl Iterator<Item = (usize, usize, &[bool])> {
        self.m
            .chunks(self.lw)
            .enumerate()
            .map(|(j, l)| (self.y0 + j, self.x0, l))
    }
}

/// [`antigo::perto_de`] só na caixa `toque` (0-based, inclusiva) das `marcadas`, alargada por `r` (B12.2): fora dela
/// nenhuma célula está a até `r` de uma marcada. Duas passadas lineares, por linha e por coluna.
pub(super) fn perto_na_caixa(
    marcadas: &[bool],
    toque: Option<(usize, usize, usize, usize)>,
    gw: usize,
    gh: usize,
    r: usize,
) -> Option<Perto> {
    let (tx0, ty0, tx1, ty1) = toque?;
    let (x0, x1) = (tx0.saturating_sub(r), (tx1 + r).min(gw - 1));
    let (y0, y1) = (ty0.saturating_sub(r), (ty1 + r).min(gh - 1));
    let (lw, lh) = (x1 + 1 - x0, y1 + 1 - y0);
    let mut por_linha = vec![false; lw * lh];
    for y in ty0..=ty1 {
        visita(2 * lw);
        let linha = &marcadas[y * gw + x0..y * gw + x1 + 1];
        let saida = &mut por_linha[(y - y0) * lw..(y - y0 + 1) * lw];
        let mut d = usize::MAX;
        for x in 0..lw {
            d = if linha[x] { 0 } else { d.saturating_add(1) };
            saida[x] = d <= r;
        }
        d = usize::MAX;
        for x in (0..lw).rev() {
            d = if linha[x] { 0 } else { d.saturating_add(1) };
            saida[x] |= d <= r;
        }
    }
    let mut m = vec![false; lw * lh];
    let mut d = vec![usize::MAX; lw];
    for j in 0..lh {
        visita(lw);
        for x in 0..lw {
            d[x] = if por_linha[j * lw + x] {
                0
            } else {
                d[x].saturating_add(1)
            };
            m[j * lw + x] = d[x] <= r;
        }
    }
    d.fill(usize::MAX);
    for j in (0..lh).rev() {
        visita(lw);
        for x in 0..lw {
            d[x] = if por_linha[j * lw + x] {
                0
            } else {
                d[x].saturating_add(1)
            };
            m[j * lw + x] |= d[x] <= r;
        }
    }
    let (mut cx0, mut cx1) = (gw, 0);
    for l in m.chunks(lw) {
        if let Some(x) = l.iter().position(|v| *v) {
            cx0 = cx0.min(x0 + x);
        }
        if let Some(x) = l.iter().rposition(|v| *v) {
            cx1 = cx1.max(x0 + x);
        }
    }
    (cx0 <= cx1).then_some(Perto {
        caixa: (cx0 + 1, y0 + 1, cx1 + 1, y1 + 1),
        x0,
        y0,
        lw,
        m,
    })
}

/// As células perto do toque cuja tinta não muda nenhum píxel (alfa 0) mas tem água ou pigmento: só a caixa.
pub(super) fn invisiveis(sess: &WetSession, perto: Option<&Perto>) -> Vec<bool> {
    let (gw, gh) = sess.grid;
    let mut v = vec![false; gw * gh];
    let g = sess.engine.active_grid();
    for (y, x0, l) in perto.into_iter().flat_map(Perto::linhas) {
        visita(l.len());
        for (i, _) in l.iter().enumerate().filter(|(_, p)| **p) {
            let k = y * gw + x0 + i;
            let c = (y + 1) * g.s + x0 + i + 1;
            v[k] = sess.planos.pigment[k * 4 + 3] == 0 && g.sett[c] + g.susp[c] > 0.0;
        }
    }
    v
}
