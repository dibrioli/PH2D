//! As grelhas de LOCALIZAÇÃO da malha — *que polígonos podem conter este ponto?* Nunca são observáveis:
//! quem pergunta confere cada candidato e ordena a resposta ([`crate::NavMesh::locate_all`]).
//!
//! Uma malha inteira tem UMA grelha; a malha por mosaicos ([`crate::MalhaPorBlocos`], W10) tem uma POR
//! MOSAICO, refeita só quando o mosaico muda (medido: a grelha da malha inteira era `0,44 ms` de cada
//! porta, plano 30 §18.1).

use std::sync::Arc;

use crate::geom::{EPS, V2};

/// A grelha de localização: cada célula lista os polígonos cuja caixa a toca, por ordem de índice.
#[derive(Clone, Debug, Default)]
pub(crate) struct Grid {
    min: V2,
    cell: f64,
    nx: usize,
    ny: usize,
    /// As listas das células, contíguas: a célula `c` é `items[off[c]..off[c + 1]]`.
    off: Vec<u32>,
    items: Vec<u32>,
}

impl Grid {
    fn cell_of(&self, p: V2) -> Option<(usize, usize)> {
        if self.nx == 0 {
            return None;
        }
        let fx = ((p[0] - self.min[0]) / self.cell).floor();
        let fy = ((p[1] - self.min[1]) / self.cell).floor();
        if !(fx.is_finite() && fy.is_finite()) {
            return None;
        }
        // A caixa da malha é fechada: um ponto em cima do bordo de cima cai na última célula.
        let ix = fx.clamp(0.0, (self.nx - 1) as f64) as usize;
        let iy = fy.clamp(0.0, (self.ny - 1) as f64) as usize;
        Some((ix, iy))
    }

    fn cell(&self, c: usize) -> &[u32] {
        &self.items[self.off[c] as usize..self.off[c + 1] as usize]
    }

    /// ⚠️ O lado da célula não é um número escolhido: sai da CONTAGEM, para que haja `~1` polígono por
    /// célula (`√n` células por eixo sobre a maior dimensão) — com menos células a pergunta paga a
    /// lista, com mais paga a memória vazia.
    pub(crate) fn build(verts: &[V2], ring_off: &[u32], ring: &[u32], min: V2, max: V2) -> Grid {
        let np = ring_off.len() - 1;
        if np == 0 {
            return Grid::default();
        }
        let ext = (max[0] - min[0]).max(max[1] - min[1]).max(EPS);
        let per_axis = (np as f64).sqrt().ceil().max(1.0);
        let cell = ext / per_axis;
        let nx = (((max[0] - min[0]) / cell).floor() as usize + 1).max(1);
        let ny = (((max[1] - min[1]) / cell).floor() as usize + 1).max(1);
        let mut grid = Grid {
            min,
            cell,
            nx,
            ny,
            off: vec![0; nx * ny + 1],
            items: Vec::new(),
        };
        // As células de cada polígono pela caixa dele; duas passagens (contar, encher) e as listas
        // contíguas, cada uma por ordem de índice.
        let alcance: Vec<((usize, usize), (usize, usize))> = (0..np)
            .map(|p| {
                let mut lo = [f64::INFINITY; 2];
                let mut hi = [f64::NEG_INFINITY; 2];
                for &v in &ring[ring_off[p] as usize..ring_off[p + 1] as usize] {
                    let q = verts[v as usize];
                    lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                    hi = [hi[0].max(q[0]), hi[1].max(q[1])];
                }
                (
                    grid.cell_of(lo).unwrap_or((0, 0)),
                    grid.cell_of(hi).unwrap_or((nx - 1, ny - 1)),
                )
            })
            .collect();
        for &((ax, ay), (bx, by)) in &alcance {
            for iy in ay..=by {
                for ix in ax..=bx {
                    grid.off[iy * nx + ix + 1] += 1;
                }
            }
        }
        for c in 0..nx * ny {
            grid.off[c + 1] += grid.off[c];
        }
        let mut cursor = grid.off.clone();
        grid.items = vec![0; grid.off[nx * ny] as usize];
        for (pi, &((ax, ay), (bx, by))) in alcance.iter().enumerate() {
            for iy in ay..=by {
                for ix in ax..=bx {
                    let c = iy * nx + ix;
                    grid.items[cursor[c] as usize] = pi as u32;
                    cursor[c] += 1;
                }
            }
        }
        grid
    }
}

/// A grelha de UM mosaico na malha montada: os índices dela são LOCAIS, somados a `base`.
#[derive(Clone, Debug)]
pub(crate) struct GrelhaDoBloco {
    pub(crate) base: u32,
    pub(crate) grid: Arc<Grid>,
}

/// Onde procurar os polígonos de um ponto.
#[derive(Clone, Debug)]
pub(crate) enum Localizador {
    /// A malha inteira, uma grelha.
    Uma(Grid),
    /// (W10) Um mosaico por célula de uma grelha regular: `xs`/`ys` são as fronteiras das colunas e
    /// das linhas (EXACTAS, as dos rectângulos dos mosaicos), `blocos[iy * nx + ix]` o mosaico.
    Blocos {
        xs: Vec<f64>,
        ys: Vec<f64>,
        blocos: Vec<Option<GrelhaDoBloco>>,
    },
}

impl Default for Localizador {
    fn default() -> Self {
        Localizador::Uma(Grid::default())
    }
}

/// A coluna (ou a linha) de `x` entre as fronteiras `xs` (`xs.len() - 1` colunas), presa às pontas.
fn faixa(xs: &[f64], x: f64) -> usize {
    let n = xs.len() - 1;
    // O 1.º `i` com `xs[i + 1] > x`: a coluna `[xs[i], xs[i + 1])`; a última fecha em cima.
    xs[1..n].partition_point(|&f| f <= x)
}

impl Localizador {
    /// Chama `f` com cada candidato (índice GLOBAL; pode repetir-se) das células que o ponto e a
    /// tolerância alcançam — um ponto em cima da fronteira de duas células pode pertencer a polígonos
    /// registados só na vizinha.
    pub(crate) fn candidatos(&self, p: V2, mut f: impl FnMut(u32)) {
        // (bloco, célula) já visitados: no máximo quatro.
        let mut vistas: [(usize, usize); 4] = [(usize::MAX, usize::MAX); 4];
        let mut n = 0;
        for dx in [-EPS, EPS] {
            for dy in [-EPS, EPS] {
                let q = [p[0] + dx, p[1] + dy];
                let (b, grid, base) = match self {
                    Localizador::Uma(g) => (0, g, 0),
                    Localizador::Blocos { xs, ys, blocos } => {
                        if xs.len() < 2 || ys.len() < 2 {
                            continue;
                        }
                        let b = faixa(ys, q[1]) * (xs.len() - 1) + faixa(xs, q[0]);
                        let Some(g) = &blocos[b] else { continue };
                        (b, &*g.grid, g.base)
                    }
                };
                let Some((ix, iy)) = grid.cell_of(q) else {
                    continue;
                };
                let c = iy * grid.nx + ix;
                if vistas[..n].contains(&(b, c)) {
                    continue;
                }
                vistas[n] = (b, c);
                n += 1;
                grid.cell(c).iter().for_each(|&i| f(base + i));
            }
        }
    }
}
