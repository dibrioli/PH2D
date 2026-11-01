//! O ecrã da célula pela lei ÚNICA da tinta (F4-2 E5, ADR-0197; `docs/Painter/Arquitetura/E5_ecra.md`): com
//! [`crate::sim::LeiDaTinta::Pigmento`] a cor e a cobertura vêm das somas de [`crate::grid::tinta`], nunca da massa.
//! Também a altura do Óleo Real e a orla dele pela mesma porta.

use crate::grid::Grid;
use crate::grid::tinta::{self, CelulaTinta};
use ph2d_paint_tinta::{Bandas, Optica, decompoe};

/// A cor sRGB `0..255` e a cobertura `α` da célula `i` pela lei da tinta («exacto sobre branco», F4-2 §3.2);
/// `None` sem tinta (`α == 0`). Seca, a cor é a escolhida; molhada (`film > 0`) escurece (`s(m, molhado)`).
#[must_use]
pub(crate) fn ecra_da_celula(lg: &Grid, i: usize) -> Option<([f64; 3], f64)> {
    let c = tinta::em(lg, i);
    let molhado = lg.film.get(i).is_some_and(|&w| w > 0.0);
    let o =
        tinta::pigmento::optica_celula(&c.filme, &c.aberto, lg.pigmento.get(i), c.meio(), molhado);
    let (cor, a) = ecra_sobre_o_fundo(&o, &c, lg.pigmento.get(i));
    (a > 0.0).then(|| (cor.0.map(|v| f64::from(v) * 255.0), f64::from(a)))
}

/// `ecra_sobre_branco` com a cobertura da PILHA (o aberto/filme sobre o fundo curado): `C = 1 − (R|branco − R|preto)`
/// com o fundo `r_g` e `r_g0`, decomposta por [`decompoe`] (cor em sRGB). Sem fundo (`r_g = 1`, `r_g0 = 0`) é a
/// mesma conta. A pilha é a do matiz preso ([`matiz::pilha_presa`], E10d1): a camada fina tem o matiz da cor escolhida.
fn ecra_sobre_o_fundo(
    o: &Optica<3>,
    c: &CelulaTinta,
    lado: Option<&tinta::pigmento::LadoEspectral>,
) -> (Bandas<3>, f32) {
    let (rw, rb) = matiz::pilha_presa(o, c, lado);
    let cob: [f32; 3] = core::array::from_fn(|b| 1.0 - (rw[b] - rb[b]));
    decompoe(&Bandas(rw), ((cob[0] + cob[1]) + cob[2]) / 3.0)
}

/// A altura que já curou, em unidades de MASSA (as do campo de altura do Óleo Real): o fecho tira a massa de `sett`
/// e põe o volume em `altura_fechada` (E8), logo a soma das duas não muda.
#[must_use]
pub(crate) fn altura_fechada_em_massa(c: &CelulaTinta) -> f32 {
    c.altura_fechada * tinta::M_UM
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

#[path = "tinta_matiz.rs"]
pub(crate) mod matiz; // E10d1: a camada fina prende o matiz

#[cfg(test)]
#[path = "tinta_tests.rs"]
mod tests;
