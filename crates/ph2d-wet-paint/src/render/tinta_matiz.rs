//! A camada FINA prende o matiz à cor escolhida (E10d1, dono Q8; `docs/Painter/Arquitetura/E10d1_camada_fina.md`).
//! Sobre o papel, o resultado por banda dá a claridade e a croma; o MATIZ Oklab é o da cor escolhida. Sobre tinta
//! curada (vidrado) fica por banda, pesado pelo papel que ainda se vê através do fundo, `g = média(R_g − R_g0)`.

use crate::grid::tinta::CelulaTinta;
use crate::grid::tinta::pigmento::{ABERTO, FILME, LadoEspectral};
use ph2d_paint_tinta::oklab::{linear_para_oklab, oklab_para_linear};
use ph2d_paint_tinta::{Bandas, Optica, camada, cor_pigmento};

/// Abaixo desta croma Oklab a cor escolhida não tem matiz (cinzento): fica por banda.
const CROMA_MIN: f32 = 1e-4;

/// O Oklab da cor escolhida da célula (filme + aberto; com o lado espectral, a cor do Pigment mix).
fn cor_escolhida(c: &CelulaTinta, lado: Option<&LadoEspectral>) -> Option<[f32; 3]> {
    let s = c.filme.junta(c.aberto);
    match lado {
        Some(l) => cor_pigmento(&s, &l[FILME].junta(l[ABERTO])).map(|t| t.lab),
        None => s.cor().map(|t| t.lab),
    }
}

#[inline]
fn na_gama(lab: [f32; 3]) -> bool {
    oklab_para_linear(lab)
        .iter()
        .all(|&x| (-1e-5..=1.0 + 1e-5).contains(&x))
}

/// A maior croma `≤ c` em `(L, direcção)` dentro do sRGB (bissecção; a croma de `L` é monótona até à borda).
fn croma_na_gama(l: f32, dir: [f32; 2], c: f32) -> f32 {
    if na_gama([l, c * dir[0], c * dir[1]]) {
        return c;
    }
    let (mut lo, mut hi) = (0.0f32, c);
    for _ in 0..20 {
        let m = 0.5 * (lo + hi);
        if na_gama([l, m * dir[0], m * dir[1]]) {
            lo = m;
        } else {
            hi = m;
        }
    }
    lo
}

/// `rw` (a pilha por banda, linear) com o matiz da camada sobre o papel preso ao da cor escolhida:
/// `R = rw + g·(preso(R_p) − R_p)`, `R_p = camada(o, 1)`, `g = média(R_g − R_g0)` (`1` sem fundo curado, `→ 0`
/// sob tinta curada opaca). `preso` = `(L_p, C_p, h_escolhida)` com a croma reduzida até caber no sRGB.
#[must_use]
pub(crate) fn prende_o_matiz(
    o: &Optica<3>,
    c: &CelulaTinta,
    lado: Option<&LadoEspectral>,
    rw: Bandas<3>,
) -> Bandas<3> {
    let g = (((c.r_g[0] - c.r_g0[0]) + (c.r_g[1] - c.r_g0[1])) + (c.r_g[2] - c.r_g0[2])) / 3.0;
    let g = g.clamp(0.0, 1.0);
    if g <= 0.0 {
        return rw;
    }
    let Some(esc) = cor_escolhida(c, lado) else {
        return rw;
    };
    let ce = esc[1].hypot(esc[2]);
    if ce < CROMA_MIN {
        return rw;
    }
    let dir = [esc[1] / ce, esc[2] / ce];
    let rp = camada(o, &Bandas([1.0; 3])).0.map(|r| r.clamp(0.0, 1.0));
    let lp = linear_para_oklab(rp);
    let cp = croma_na_gama(lp[0], dir, lp[1].hypot(lp[2]));
    let preso = oklab_para_linear([lp[0], cp * dir[0], cp * dir[1]]).map(|r| r.clamp(0.0, 1.0));
    Bandas(core::array::from_fn(|b| {
        (rw.0[b] + g * (preso[b] - rp[b])).clamp(0.0, 1.0)
    }))
}

/// A pilha da célula com o matiz preso: `(sobre o branco, sobre o preto)`. Sobre o preto entra o MESMO acréscimo
/// (preso a `[0,1]`), para que a cobertura `1 − (R|branco − R|preto)` do ecrã seja a que o FECHO grava
/// (`drying/tinta.rs fecha_celula` ⇒ `(r_g, r_g0)`): fechar continua invisível. Sem matiz preso é [`crate::grid::tinta::pilha`].
#[must_use]
pub(crate) fn pilha_presa(
    o: &Optica<3>,
    c: &CelulaTinta,
    lado: Option<&LadoEspectral>,
) -> ([f32; 3], [f32; 3]) {
    let rw = camada(o, &Bandas(c.r_g));
    let rb = camada(o, &Bandas(c.r_g0)).0;
    let preso = prende_o_matiz(o, c, lado, rw).0;
    let r_g0 = core::array::from_fn(|b| (rb[b] + (preso[b] - rw.0[b])).clamp(0.0, 1.0));
    (preso, r_g0)
}

/// O mesmo para quem mede: o desvio de matiz (graus Oklab, `[0, 180]`) de `srgb` (0..1) à direcção `esc`.
#[cfg(test)]
pub(crate) fn desvio_de_matiz(srgb: [f32; 3], esc: [f32; 3]) -> f32 {
    use ph2d_paint_tinta::oklab::srgb_para_oklab;
    let q = srgb_para_oklab(srgb);
    let d = (q[2].atan2(q[1]) - esc[2].atan2(esc[1])).to_degrees();
    let d = d.rem_euclid(360.0);
    d.min(360.0 - d)
}

#[cfg(test)]
#[path = "tinta_matiz_tests.rs"]
mod tests;
