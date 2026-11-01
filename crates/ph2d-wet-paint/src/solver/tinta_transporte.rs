//! **O escoamento transporta as SOMAS** (F4_2 E7, ADR-0197). Com [`LeiDaTinta::Pigmento`] a cor do suspenso
//! vive em `Grid::tinta[i].filme` e a do assentado em `.aberto`; aqui a soma vai com a MESMA fracção que a massa:
//! a cor de chegada é a soma das somas (extensivas), nunca média de RGB nem K-M. Doc:
//! `docs/Painter/Arquitetura/E7_escoamento.md`.
//!
//! Duas formas de porta: (1) gather LINEAR (Jacobi, sedimentar): cada componente da soma é a mesma combinação
//! linear dos vizinhos que a massa ⇒ conserva igual; (2) mudança LOCAL in-place (backrun, levantar do tubo): a
//! fracção é lida da massa (`1 − depois/antes`) e aplicada à soma.

use crate::grid::Grid;
use crate::grid::tinta::CelulaTinta;
use crate::grid::tinta::pigmento::{self as pig, ABERTO, FILME, LadoEspectral};
use crate::sim::{LeiDaTinta, Params};
use ph2d_paint_tinta::{Soma, transfere};

/// A lei nova corre neste passo (o campo existe: [`crate::grid::tinta::garante`] no início do passo).
#[inline]
pub(crate) fn liga(p: &Params, g: &Grid) -> bool {
    p.tinta == LeiDaTinta::Pigmento && g.tinta.len() == g.cells
}

/// Move a fracção `f` do filme da célula `de` para o de `para` (`de == para` é nada).
#[inline]
pub(crate) fn move_filme(g: &mut Grid, de: usize, para: usize, f: f64) {
    if de == para || f <= 0.0 {
        return;
    }
    let t = &mut g.tinta;
    let mut origem = t[de].filme;
    transfere(&mut origem, &mut t[para].filme, f as f32);
    t[de].filme = origem;
    pig::move_em(g, (de, FILME), Some((para, FILME)), f as f32);
}

/// Junta ao filme de `i` a fracção `w[k]` do filme de cada canto (o gather sequencial do [`super::advect`]).
#[inline]
pub(crate) fn junta_cantos(g: &mut Grid, i: usize, cantos: [usize; 4], w: [f64; 4]) {
    for (c, wk) in cantos.into_iter().zip(w) {
        move_filme(g, c, i, wk);
    }
}

/// Levanta a fracção `f` do aberto da célula para o filme dela (o backrun sequencial).
#[inline]
pub(crate) fn levanta(t: &mut [CelulaTinta], l: &mut [LadoEspectral], i: usize, f: f64) {
    let c = &mut t[i];
    transfere(&mut c.aberto, &mut c.filme, f as f32);
    if let Some(l) = l.get_mut(i) {
        let [a, b] = l;
        let _ = ph2d_paint_tinta::Espectro::transfere(b, a, f as f32);
    }
}

/// O campo existe (para as portas sem [`Params`]: com a lei velha ele está vazio).
#[inline]
pub(crate) fn existe(g: &Grid) -> bool {
    g.tinta.len() == g.cells
}

/// As células da faixa viva, na ordem das linhas (as que os passes Jacobi escrevem).
pub(crate) fn celulas_da_faixa(g: &Grid) -> Vec<usize> {
    let mut v = Vec::new();
    for y in g.by0..=g.by1 {
        let (lo, hi) = g.span_x(y);
        if lo <= hi {
            let base = y as usize * g.s;
            v.extend((lo as usize..=hi as usize).map(|x| base + x));
        }
    }
    v
}

/// O gather do [`super::advect_jacobi`] sobre as somas, com o `uv`/`outflow` que o passe deixou no rascunho
/// (rota não fundida; o `susp` ainda é o do início — antes do `commit`).
pub(super) fn advect_jacobi(g: &mut Grid) {
    let celulas = celulas_da_faixa(g);
    let (wf, hf, s) = (g.w as f64, g.h as f64, g.s);
    gather_linear(g, celulas.into_iter(), |g, i, t| {
        let (uv, outflow, susp) = (&g.scratch.uv, &g.scratch.outflow, &g.susp);
        let leave = f64::from(outflow[i]).min(1.0);
        t.push((i, 1.0 - leave));
        if g.active[i] == 0 {
            return;
        }
        let (x, y) = ((i % s) as i32, (i / s) as i32);
        let (ux, uy) = (f64::from(uv[i][0]), f64::from(uv[i][1]));
        let Some((x0, fx, y0, fy)) = super::backtrace(x, y, ux, uy, wf, hf) else {
            return;
        };
        let i00 = x0 as usize + y0 as usize * s;
        let cantos = [i00, i00 + 1, i00 + s, i00 + s + 1];
        let w = [
            (1.0 - fx) * (1.0 - fy),
            fx * (1.0 - fy),
            (1.0 - fx) * fy,
            fx * fy,
        ];
        let k = cantos.map(|c| super::take_scale(f64::from(outflow[c])));
        let k = [w[0] * k[0], w[1] * k[1], w[2] * k[2], w[3] * k[3]];
        let want: f64 = (0..4).map(|n| f64::from(susp[cantos[n]]) * k[n]).sum();
        if want > 0.0 {
            t.extend((0..4).map(|n| (cantos[n], k[n])));
        }
    });
}

/// `Σ coefₖ · somaₖ`, acumulado em `f64` (a mesma combinação linear da massa).
fn combina(termos: &[(Soma, f64)]) -> Soma {
    let mut acc = [0.0f64; 6];
    for (s, c) in termos {
        acc[0] += f64::from(s.v) * c;
        acc[1] += f64::from(s.lab[0]) * c;
        acc[2] += f64::from(s.lab[1]) * c;
        acc[3] += f64::from(s.lab[2]) * c;
        acc[4] += f64::from(s.t) * c;
        acc[5] += f64::from(s.m) * c;
    }
    Soma {
        v: acc[0].max(0.0) as f32,
        lab: [acc[1] as f32, acc[2] as f32, acc[3] as f32],
        t: acc[4] as f32,
        m: acc[5].max(0.0) as f32,
    }
}

/// Gather LINEAR em Jacobi: para cada célula de `celulas`, `filme' = Σ coef · filme(fonte)` lido do estado do
/// INÍCIO (o `termos` empilha `(fonte, coef)` — a própria célula com o que fica, e cada fonte com o que dá).
/// O `tinta` sai de `g` durante a conta para o `termos` poder ler a grade.
pub(crate) fn gather_linear(
    g: &mut Grid,
    celulas: impl Iterator<Item = usize>,
    mut termos: impl FnMut(&Grid, usize, &mut Vec<(usize, f64)>),
) {
    let velha = std::mem::take(&mut g.tinta);
    let lado = std::mem::take(&mut g.pigmento);
    let (mut fontes, mut pesos, mut novas) = (Vec::new(), Vec::new(), Vec::new());
    let mut pesos_e = Vec::new();
    for i in celulas {
        fontes.clear();
        termos(g, i, &mut fontes);
        pesos.clear();
        pesos.extend(fontes.iter().map(|&(j, c)| (velha[j].filme, c)));
        pesos_e.clear();
        pesos_e.extend(
            fontes
                .iter()
                .filter_map(|&(j, c)| lado.get(j).map(|l| (l[FILME], c))),
        );
        let e = (!lado.is_empty()).then(|| pig::combina(&pesos_e));
        novas.push((i, combina(&pesos), e));
    }
    g.tinta = velha;
    g.pigmento = lado;
    for (i, s, e) in novas {
        g.tinta[i].filme = s;
        if let Some(e) = e {
            g.pigmento[i][FILME] = e;
        }
    }
}

/// O que o `sett` era antes de uma passagem in-place que LEVANTA (assentado → suspenso); `None` sem a lei nova.
pub(crate) fn fotografa_sett(p: &Params, g: &Grid) -> Option<Vec<f32>> {
    liga(p, g).then(|| g.sett.clone())
}

/// Depois da passagem: onde o `sett` desceu, a mesma fracção do aberto passa ao filme da célula.
pub(crate) fn levanta_pelo_sett(g: &mut Grid, antes: Option<Vec<f32>>) {
    let Some(antes) = antes else { return };
    for (i, &a) in antes.iter().enumerate() {
        let d = g.sett[i];
        if a > 0.0 && d < a {
            let f = 1.0 - f64::from(d) / f64::from(a);
            let c = &mut g.tinta[i];
            transfere(&mut c.aberto, &mut c.filme, f as f32);
            pig::move_em(g, (i, ABERTO), Some((i, FILME)), f as f32);
        }
    }
}

/// O que o `susp` era antes de uma passagem in-place que o TIRA da tela; `None` sem a lei nova.
pub(crate) fn fotografa_susp(p: &Params, g: &Grid) -> Option<Vec<f32>> {
    liga(p, g).then(|| g.susp.clone())
}

/// Depois da passagem: onde o `susp` desceu, o filme perde a mesma fracção (o destino — o pincel — é da E6b).
pub(crate) fn tira_pelo_susp(g: &mut Grid, antes: Option<Vec<f32>>) {
    let Some(antes) = antes else { return };
    for (i, &a) in antes.iter().enumerate() {
        let d = g.susp[i];
        if a > 0.0 && d < a {
            let f = 1.0 - f64::from(d) / f64::from(a);
            let _ = transfere(&mut g.tinta[i].filme, &mut Soma::default(), f as f32);
            pig::move_em(g, (i, FILME), None, f as f32);
        }
    }
}

/// O anel de dreno de [`super::super::apply_boundaries`] zera o `susp`: o filme vai com ele.
/// ⚠️ Ainda não chamado pelo `project.rs` (fora dos ficheiros da E7) — só pelo teste.
pub(crate) fn drena_o_anel(g: &mut Grid) {
    if !existe(g) {
        return;
    }
    let (s, w, h, rows) = (g.s, g.w, g.h, g.rows);
    let mut zera = |i: usize| {
        g.tinta[i].filme = Soma::default();
        if let Some(l) = g.pigmento.get_mut(i) {
            l[FILME] = ph2d_paint_tinta::Espectro::default();
        }
    };
    for y in [0usize, 1, h, h + 1] {
        (y * s..y * s + s).for_each(&mut zera);
    }
    for y in 0..rows {
        for x in [0usize, 1, w, w + 1] {
            zera(y * s + x);
        }
    }
}

#[cfg(test)]
#[path = "tinta_transporte_tests.rs"]
mod tests;
