//! **A cadência da secagem é do TILE 64×64** (B13.4, `docs/Painter/missao_4k/medicoes/N10_agua_wet.md`): um pingo a
//! correr longe não muda o ritmo de uma poça calma. A porta `n % dry_every` continua global (3 se algum tile corre,
//! senão 6); dentro do passe o tile a correr seca com o par lento, o calmo com o par calmo só quando `n % 6 == 0` —
//! senão a célula salta. Tudo calmo = mapa vazio = o passe de antes, ao bit.

use super::DryConst;
use crate::grid::Grid;

/// O lado do tile, em potência de 2 (64 células, coordenadas da grade COM pad).
pub const TILE_LOG: u32 = 6;
/// O `vmax` a partir do qual a água «corre» (a mesma fronteira da cadência global).
pub const VMAX_CORRE: f64 = 0.5;
/// O par calmo `(evap, rewet)` e a sua cadência — os de fábrica do [`crate::sim::Sim`].
pub const EVAP_CALMO: f64 = 0.001;
pub const REWET_CALMO: f64 = 0.0001;
pub const CADENCIA_CALMA: u64 = 6;
/// A régua da prancha B13.4 (`prancha_b13_4_tests.rs`): ligada, o mapa por tile não se usa e a grade inteira
/// segue a cadência global — o de antes. Só para medir; nunca ligada no produto.
pub static CADENCIA_GLOBAL: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Os tiles da grade `g` (com pad): `(nx, ny)`.
#[must_use]
pub fn tiles_da_grade(g: &Grid) -> (usize, usize) {
    let t = 1usize << TILE_LOG;
    (g.s.div_ceil(t), (g.h + 2).div_ceil(t))
}

/// O `vmax` de cada tile (o MESMO do advect: célula ativa, o fluxo do [`crate::flow::FlowRowSampler`]) e o seu
/// máximo. Corre depois do advect, que não escreve `flow_x/y`.
#[must_use]
pub fn vmax_por_tile(g: &Grid) -> (Vec<f64>, f64) {
    let (nx, ny) = tiles_da_grade(g);
    let mut v = vec![0.0f64; nx * ny];
    let mut max = 0.0f64;
    for y in g.by0.max(0)..=g.by1 {
        let (bx0, bx1) = g.span_x(y);
        if bx0 > bx1 {
            continue;
        }
        let mut sampler = crate::flow::FlowRowSampler::new(&g.flow, y);
        let lin = (y as usize >> TILE_LOG) * nx;
        let mut i = bx0 as usize + y as usize * g.s;
        for x in bx0..=bx1 {
            if g.active[i] != 0 {
                let (ux, uy) = sampler.at(&g.flow_x, &g.flow_y, x, i, g.flow.s);
                let a = ux.abs().max(uy.abs());
                let t = &mut v[lin + (x as usize >> TILE_LOG)];
                if a > *t {
                    *t = a;
                }
                if a > max {
                    max = a;
                }
            }
            i += 1;
        }
    }
    (v, max)
}

/// O mapa «corre?» por tile a partir do `vmax` do advect; vazio quando nada corre (o caminho de hoje).
#[must_use]
pub fn corre_por_tile(g: &Grid, vmax: f64) -> Vec<bool> {
    if vmax < VMAX_CORRE {
        return Vec::new();
    }
    let (v, max) = vmax_por_tile(g);
    debug_assert_eq!(
        max.to_bits(),
        vmax.to_bits(),
        "o vmax por tile não é o do advect"
    );
    v.into_iter().map(|t| t >= VMAX_CORRE).collect()
}

/// O mapa de um passe de secagem: o tile a correr usa o [`DryConst`] do chamador, o calmo o seu.
pub struct SecaPorTile<'a> {
    pub corre: &'a [bool],
    pub nx: usize,
    /// `n % 6 == 0`: o tile calmo seca neste passe (senão salta).
    pub calmo_seca: bool,
    pub evap_calmo: f64,
    pub rewet_calmo: f64,
}

/// O par de constantes do passe: as do tile a correr e as do calmo.
pub(crate) struct Escolha<'a> {
    pub corre: &'a [bool],
    pub nx: usize,
    pub calmo: Option<DryConst>,
}

impl<'a> Escolha<'a> {
    pub fn new(
        t: Option<&SecaPorTile<'a>>,
        g: &Grid,
        p: &crate::sim::Params,
        ext_bypass: bool,
    ) -> Option<Self> {
        let t = t.filter(|t| !t.corre.is_empty())?;
        Some(Escolha {
            corre: t.corre,
            nx: t.nx,
            calmo: t
                .calmo_seca
                .then(|| DryConst::new(g, p, t.evap_calmo, t.rewet_calmo, ext_bypass)),
        })
    }
}

/// As constantes da célula `(x, y)` (com pad); `None` = salta.
#[inline]
pub(crate) fn escolhe<'c>(
    c: &'c DryConst,
    e: Option<&'c Escolha<'_>>,
    x: usize,
    y: usize,
) -> Option<&'c DryConst> {
    let Some(e) = e else { return Some(c) };
    if e.corre[(y >> TILE_LOG) * e.nx + (x >> TILE_LOG)] {
        Some(c)
    } else {
        e.calmo.as_ref()
    }
}
