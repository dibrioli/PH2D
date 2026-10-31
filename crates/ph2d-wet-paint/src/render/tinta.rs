//! O ecrã da célula pela lei ÚNICA da tinta (F4-2 E5, ADR-0197; `docs/Painter/Arquitetura/E5_ecra.md`): com
//! [`crate::sim::LeiDaTinta::Pigmento`] a cor e a cobertura vêm das somas de [`crate::grid::tinta`], nunca da massa.
//! Também a altura do Óleo Real e a orla dele pela mesma porta.

use crate::grid::Grid;
use crate::grid::tinta::{self, CelulaTinta};
use ph2d_paint_tinta::oklab::linear_para_srgb;
use ph2d_paint_tinta::{Bandas, ecra_sobre_branco, optica_celula, s_fator};

/// A cor sRGB `0..255` e a cobertura `α` da célula `i` pela lei da tinta («exacto sobre branco», F4-2 §3.2);
/// `None` sem tinta (`α == 0`). O «molhado escurece» é `s_fator(m, film > 0)`.
#[must_use]
pub(crate) fn ecra_da_celula(lg: &Grid, i: usize) -> Option<([f64; 3], f64)> {
    let c = tinta::em(lg, i);
    let molhado = lg.film.get(i).is_some_and(|&w| w > 0.0);
    let o = optica_celula(&c.filme, &c.aberto, s_fator(c.m, molhado));
    let (cor, a) = ecra_sobre_branco(&o, &Bandas(c.r_g));
    (a > 0.0).then(|| {
        (
            cor.0.map(|v| f64::from(linear_para_srgb(v)) * 255.0),
            f64::from(a),
        )
    })
}

/// A altura que já curou, em unidades de MASSA (as do campo de altura do Óleo Real).
// TODO(E8): `altura_fechada · M_UM` quando a E6a definir `M_UM` em `grid/tinta.rs`.
#[must_use]
pub(crate) fn altura_fechada_em_massa(c: &CelulaTinta) -> f32 {
    let _ = c;
    0.0
}

/// A massa que dá a ALTURA do Óleo Real na célula `i` (o ponto único de `oleo_campo.rs`): `sett + susp`, mais a
/// altura fechada quando a grade tem a tinta nova (o campo [`Grid::tinta`] só existe depois de `Pigmento` correr).
#[inline]
#[must_use]
pub(crate) fn massa_da_altura(lg: &Grid, i: usize) -> f64 {
    let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
    if lg.tinta.is_empty() {
        m
    } else {
        m + f64::from(altura_fechada_em_massa(&tinta::em(lg, i)))
    }
}

/// A Transparency da tinta da célula `i` (só da célula: o cartão não a tem): `0` sem o campo do material.
#[inline]
pub(super) fn transparencia_da_celula(g: &Grid, i: usize) -> f64 {
    g.material
        .get(i)
        .map_or(0.0, |m| crate::material::peso(m.transparencia))
}

#[cfg(test)]
#[path = "tinta_tests.rs"]
mod tests;
