//! O ESPELHO das grades (J13b P2, `docs/Painter/Arquitetura/J13b_plano_otica_fora.md` §2): uma [`Grid`] por camada
//! só com os planos que a ÓPTICA lê (`render::pigment_row`, `oleo_real::CampoAltura`, `render/tinta.rs`, o véu);
//! os da dinâmica ficam vazios. O render existente lê-o sem mudar ([`Espelho::camadas_render`]). O acumulador
//! [`Engine::take_espelho_sujo`] junta as portas que mudam as grades ([`Engine::merge_dirty`],
//! [`Engine::mark_dirty_full`]), à parte do `take_dirty` da tela. Cópia em série: rayon nesta crate exige ADR.

use super::{Dirty, Engine};
use crate::grid::Grid;
use crate::render::RenderLayer;

/// Uma camada do espelho: a grade da óptica e o que o compositor lê da [`super::Layer`].
pub struct CamadaEspelho {
    pub grid: Grid,
    pub opacity: f64,
    pub visible: bool,
}

/// As camadas do motor, só com a óptica (ver o módulo).
#[derive(Default)]
pub struct Espelho {
    pub camadas: Vec<CamadaEspelho>,
}

impl Espelho {
    /// As camadas como o render as lê.
    #[must_use]
    pub fn camadas_render(&self) -> Vec<RenderLayer<'_>> {
        self.camadas
            .iter()
            .map(|c| RenderLayer {
                grid: &c.grid,
                opacity: c.opacity,
                visible: c.visible,
            })
            .collect()
    }

    /// Os bytes dos planos guardados.
    #[must_use]
    pub fn bytes(&self) -> usize {
        fn b<T>(v: &[T]) -> usize {
            std::mem::size_of_val(v)
        }
        self.camadas
            .iter()
            .map(|c| {
                let g = &c.grid;
                b(&g.film)
                    + b(&g.susp)
                    + b(&g.susp_rgb)
                    + b(&g.sett)
                    + b(&g.sett_rgb)
                    + b(&g.relevo)
                    + b(&g.altura)
                    + b(&g.espessura)
                    + b(&g.crista)
                    + b(&g.nivelado)
                    + b(&g.vela)
                    + b(&g.material)
                    + g.aparencia.bytes_vivos()
                    + g.oleo.bytes_vivos()
                    + b(&g.tinta)
                    + b(&g.pigmento)
                    + b(&g.wet)
                    + b(&g.paper)
            })
            .sum()
    }
}

impl Dirty {
    /// Junta a caixa `[x0..=x1]×[y0..=y1]` alargada por uma célula e presa a `[1..=w]×[1..=h]`.
    pub(super) fn une(&mut self, (x0, y0, x1, y1): (i32, i32, i32, i32), w: usize, h: usize) {
        let (nx0, ny0) = ((x0 - 1).max(1), (y0 - 1).max(1));
        let (nx1, ny1) = ((x1 + 1).min(w as i32), (y1 + 1).min(h as i32));
        match self {
            Dirty::Full => {}
            Dirty::Clean => {
                *self = Dirty::Rect {
                    x0: nx0,
                    y0: ny0,
                    x1: nx1,
                    y1: ny1,
                };
            }
            Dirty::Rect { x0, y0, x1, y1 } => {
                *x0 = (*x0).min(nx0);
                *y0 = (*y0).min(ny0);
                *x1 = (*x1).max(nx1);
                *y1 = (*y1).max(ny1);
            }
        }
    }
}

/// A orla que o rasto veste FORA da caixa que declara (`trail/transfer.rs`: `*::veste_virgens`, o maior dos três
/// alcances): o campo de altura do Real Oil lê-a, logo o espelho tem de a levar (B11.2: sem as linhas inteiras do
/// sujo, a imagem do espelho divergia da da grade).
const ORLA_DO_ESPELHO: i32 = {
    let (a, b) = (
        crate::grid::material::ORLA_VIRGEM,
        crate::grid::aparencia::ORLA_VIRGEM,
    );
    if a > b { a } else { b }
};

impl Engine {
    /// Junta a caixa de uma porta ao sujo do espelho, com a [`ORLA_DO_ESPELHO`].
    pub(super) fn suja_o_espelho(&mut self, (x0, y0, x1, y1): (i32, i32, i32, i32)) {
        let r = ORLA_DO_ESPELHO;
        self.espelho_sujo
            .une((x0 - r, y0 - r, x1 + r, y1 + r), self.width, self.height);
    }

    /// Tira e limpa o sujo do espelho (independente de [`Engine::take_dirty`]).
    pub fn take_espelho_sujo(&mut self) -> Dirty {
        std::mem::replace(&mut self.espelho_sujo, Dirty::Clean)
    }

    /// Põe em `e` a óptica de hoje: com `Rect`, só a CAIXA do [`Engine::take_espelho_sujo`] (uma célula de folga) e
    /// o anel, e só na camada activa (as outras mudam por `Full` — B11.2); `Full` copia tudo no lugar; outro nº de
    /// camadas ou outro tamanho reconstrói. Um plano que mudou de tamanho copia-se inteiro, em qualquer camada.
    pub fn copia_para(&mut self, e: &mut Espelho) {
        let sujo = self.take_espelho_sujo();
        let mesma_forma = e.camadas.len() == self.layers.len()
            && e.camadas.iter().zip(&self.layers).all(|(c, l)| {
                (c.grid.w, c.grid.h, c.grid.s, c.grid.rows)
                    == (l.grid.w, l.grid.h, l.grid.s, l.grid.rows)
            });
        if !mesma_forma {
            e.camadas = self
                .layers
                .iter()
                .map(|l| CamadaEspelho {
                    grid: so_a_otica(&l.grid),
                    opacity: l.opacity,
                    visible: l.visible,
                })
                .collect();
            return;
        }
        for (k, (c, l)) in e.camadas.iter_mut().zip(&self.layers).enumerate() {
            c.opacity = l.opacity;
            c.visible = l.visible;
            escalares(&mut c.grid, &l.grid);
            let g = &l.grid;
            let faixas = match sujo {
                Dirty::Rect { .. } if k != self.active_layer => Vec::new(),
                Dirty::Clean => Vec::new(),
                Dirty::Full => std::iter::once(0..usize::MAX).collect(),
                Dirty::Rect { x0, y0, x1, y1 } => faixas_da_caixa(g, (x0, y0, x1, y1)),
            };
            copia_faixas(&mut c.grid, g, &faixas);
        }
    }
}

/// Os índices de um `Rect` do sujo: a caixa com uma célula de folga de cada lado (o menisco lê `film[i ± 1]`,
/// `film[i ± s]`), recortada em X, e o ANEL que `solver::apply_boundaries` reescreve a cada passo fora do sujo (as
/// linhas `0, 1, h, h+1` inteiras; as colunas `w, w+1` de uma linha e `0, 1` da seguinte são contíguas).
fn faixas_da_caixa(
    g: &Grid,
    (x0, y0, x1, y1): (i32, i32, i32, i32),
) -> Vec<std::ops::Range<usize>> {
    let s = g.s;
    let (a, b) = (
        (y0.max(1) as usize) - 1,
        (y1.max(0) as usize + 1).min(g.rows - 1),
    );
    let (xa, xb) = (
        (x0.max(1) as usize) - 1,
        (x1.max(0) as usize + 1).min(s - 1),
    );
    let mut f: Vec<_> = (a..=b).map(|y| y * s + xa..y * s + xb + 1).collect();
    for y in [0, 1, g.h, g.h + 1].into_iter().filter(|&y| y < g.rows) {
        f.push(y * s..(y + 1) * s);
    }
    if s == g.w + 2 {
        f.push(0..2);
        f.extend((0..g.rows - 1).map(|y| y * s + g.w..(y + 1) * s + 2));
        f.push((g.rows - 1) * s + g.w..g.rows * s);
    } else {
        for y in 0..g.rows {
            f.extend([y * s..y * s + 2, y * s + g.w..y * s + g.w + 2]);
        }
    }
    f
}

/// Os escalares da grade (todos `Copy`; os da dinâmica vão também — custam nada e o padrão sem `..` obriga).
fn escalares(d: &mut Grid, g: &Grid) {
    (d.w, d.h, d.s, d.rows, d.cells) = (g.w, g.h, g.s, g.rows, g.cells);
    (d.oleo_real, d.flow, d.has_fluid, d.spans_enabled) =
        (g.oleo_real, g.flow, g.has_fluid, g.spans_enabled);
    d.suave_teto = g.suave_teto;
    d.borda_fundo = g.borda_fundo;
    (d.bx0, d.by0, d.bx1, d.by1) = (g.bx0, g.by0, g.bx1, g.by1);
    (d.paper_preset, d.paper_sheet) = (g.paper_preset, g.paper_sheet);
}

/// Copia as `faixas` de índices de cada plano da óptica; o plano de outro tamanho copia-se inteiro.
fn copia_faixas(d: &mut Grid, g: &Grid, faixas: &[std::ops::Range<usize>]) {
    fn um<T: Clone>(d: &mut Vec<T>, g: &[T], faixas: &[std::ops::Range<usize>]) {
        if d.len() != g.len() {
            d.clear();
            d.extend_from_slice(g);
            return;
        }
        for f in faixas {
            let (i0, i1) = (f.start.min(g.len()), f.end.min(g.len()));
            d[i0..i1].clone_from_slice(&g[i0..i1]);
        }
    }
    um(&mut d.film, &g.film, faixas);
    um(&mut d.susp, &g.susp, faixas);
    um(&mut d.susp_rgb, &g.susp_rgb, faixas);
    um(&mut d.sett, &g.sett, faixas);
    um(&mut d.sett_rgb, &g.sett_rgb, faixas);
    um(&mut d.relevo, &g.relevo, faixas);
    um(&mut d.altura, &g.altura, faixas);
    um(&mut d.espessura, &g.espessura, faixas);
    um(&mut d.crista, &g.crista, faixas);
    um(&mut d.nivelado, &g.nivelado, faixas);
    um(&mut d.vela, &g.vela, faixas);
    um(&mut d.material, &g.material, faixas);
    if d.aparencia.len() == g.aparencia.len() {
        for f in faixas {
            d.aparencia.copia_faixa_de(&g.aparencia, f.clone());
        }
    } else {
        d.aparencia.clone_from(&g.aparencia);
    }
    if d.oleo.len() == g.oleo.len() {
        for f in faixas {
            d.oleo.copia_faixa_de(&g.oleo, f.clone());
        }
    } else {
        d.oleo.clone_from(&g.oleo);
    }
    um(&mut d.tinta, &g.tinta, faixas);
    um(&mut d.pigmento, &g.pigmento, faixas);
    um(&mut d.wet, &g.wet, faixas);
    um(&mut d.paper, &g.paper, faixas);
}

/// A grade só com a óptica. O padrão sem `..` é a lei: um campo novo da [`Grid`] não compila aqui até alguém
/// decidir se a óptica o lê.
fn so_a_otica(g: &Grid) -> Grid {
    let Grid {
        w,
        h,
        s,
        rows,
        cells,
        film,
        susp,
        susp_rgb,
        sett,
        sett_rgb,
        relevo,
        altura,
        espessura,
        crista,
        nivelado,
        vela,
        material,
        aparencia,
        suave_teto,
        borda_fundo,
        oleo,
        oleo_real,
        oleo_antes: _,
        oleo_antes_assente: _,
        chao: _,
        sett_do_traco: _,
        tinta,
        pigmento,
        flow,
        vel_x: _,
        vel_y: _,
        flow_x: _,
        flow_y: _,
        frow_lo: _,
        frow_hi: _,
        wet,
        paper,
        active: _,
        bloom: _,
        bx0,
        by0,
        bx1,
        by1,
        has_fluid,
        row_lo: _,
        row_hi: _,
        live_lo: _,
        live_hi: _,
        spans_enabled,
        scratch: _,
        paper_preset,
        paper_sheet,
    } = g;
    Grid {
        w: *w,
        h: *h,
        s: *s,
        rows: *rows,
        cells: *cells,
        film: film.clone(),
        susp: susp.clone(),
        susp_rgb: susp_rgb.clone(),
        sett: sett.clone(),
        sett_rgb: sett_rgb.clone(),
        relevo: relevo.clone(),
        altura: altura.clone(),
        espessura: espessura.clone(),
        crista: crista.clone(),
        nivelado: nivelado.clone(),
        vela: vela.clone(),
        material: material.clone(),
        aparencia: aparencia.clone(),
        suave_teto: *suave_teto,
        borda_fundo: *borda_fundo,
        oleo: oleo.clone(),
        oleo_real: *oleo_real,
        oleo_antes: Vec::new(),
        oleo_antes_assente: Vec::new(),
        chao: Vec::new(),
        sett_do_traco: Vec::new(),
        tinta: tinta.clone(),
        pigmento: pigmento.clone(),
        flow: *flow,
        vel_x: Vec::new(),
        vel_y: Vec::new(),
        flow_x: Vec::new(),
        flow_y: Vec::new(),
        frow_lo: Vec::new(),
        frow_hi: Vec::new(),
        wet: wet.clone(),
        paper: paper.clone(),
        active: Vec::new(),
        bloom: Vec::new(),
        bx0: *bx0,
        by0: *by0,
        bx1: *bx1,
        by1: *by1,
        has_fluid: *has_fluid,
        row_lo: Vec::new(),
        row_hi: Vec::new(),
        live_lo: Vec::new(),
        live_hi: Vec::new(),
        spans_enabled: *spans_enabled,
        scratch: Default::default(),
        paper_preset: *paper_preset,
        paper_sheet: *paper_sheet,
    }
}

#[cfg(test)]
#[path = "espelho_tests.rs"]
mod tests;
