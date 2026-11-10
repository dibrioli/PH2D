//! A orla do enterro: as células perto do que a tela mudou e a caixa dos planos — filho de [`super`] por LOC (B52.R11).

/// O alcance do enterro da orla, em células: a tinta invisível de um traço desfeito fica até 7 células do que a tela
/// mudou (`sonda_alcance_da_orla`: Wet Flat de 20 a 160 px, até 3000 quadros de água). 32 é a folga sobre o medido.
pub(super) const ALCANCE_DA_ORLA: usize = 32;

/// Uma caixa de células: `(x0, y0, x1, y1)`, 1-based e inclusiva.
pub(super) type CaixaDeCelulas = (usize, usize, usize, usize);

/// As células a até `r` (Chebyshev) de uma marcada, e a caixa delas.
pub(super) fn perto_de(
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
    let (y0, y1) = ((a / gw).saturating_sub(r), (b / gw + r).min(gh - 1));
    // Por linha, depois por coluna: cada passada é linear.
    let mut por_linha = vec![false; gw * gh];
    for y in a / gw..=b / gw {
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
