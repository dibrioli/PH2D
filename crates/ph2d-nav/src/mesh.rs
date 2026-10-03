//! A MALHA ANDÁVEL como dados: polígonos CONVEXOS em sentido anti-horário, a vizinhança por aresta, os
//! cantos e as ilhas — e as duas perguntas de lugar (*onde está este ponto?* · *qual é o ponto da
//! malha mais perto deste?*).
//!
//! Quem a constrói a partir dos colisores é a `ph2d-navmesh`; ela chega aqui por
//! [`NavMesh::from_polygons`], que é a ÚNICA porta e confere tudo o que a procura assume.

#[cfg(test)]
use crate::geom::orient;
use crate::geom::{EPS, V2, closest_on_segment, dist, side_dist};

/// Um polígono da malha: os índices dos vértices em sentido ANTI-HORÁRIO e, por aresta, quem está do
/// outro lado. A aresta `i` vai de `verts[i]` a `verts[(i + 1) % n]`.
///
/// ⚠️ (W9) É uma VISTA sobre as listas contíguas da malha ([`NavMesh::poly`]): nenhum polígono tem
/// memória própria. Medido: as três listas por polígono eram `~43 000` alocações por construção numa
/// malha de `14 571` polígonos — `0,9 ms` de criar e largar, um terço de montar a malha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Poly<'a> {
    pub verts: &'a [u32],
    /// O polígono do outro lado de cada aresta, ou `None` quando ela é PAREDE (fronteira da malha).
    pub nbrs: &'a [Option<u32>],
    /// Onde esta aresta aparece no vizinho (o índice dela na lista dele). Só tem sentido com vizinho.
    pub twin: &'a [u32],
}

impl Poly<'_> {
    #[inline]
    pub fn len(&self) -> usize {
        self.verts.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.verts.is_empty()
    }
}

/// Porque [`NavMesh::from_polygons`] recusou a entrada. ⛔ A recusa é sempre em voz alta: uma malha
/// aceite com um polígono virado ao contrário daria caminhos através das paredes, sem erro.
#[derive(Clone, Debug, PartialEq)]
pub enum MeshError {
    /// Um polígono com menos de três vértices.
    Degenerate { poly: usize },
    /// Um índice de vértice fora da lista.
    BadIndex { poly: usize, index: u32 },
    /// Um polígono com área `≤ 0` (em sentido horário, ou achatado).
    NotCcw { poly: usize, area2: f64 },
    /// Um vértice onde o polígono dobra para DENTRO (mais de [`EPS`] metros). ⚠️ Um vértice a
    /// `180°` (à tolerância) é ACEITE: a construção escreve numa grelha inteira, e três pontos da
    /// grelha quase colineares têm uma distância à recta que pode descer a `1e-11 m` — recusá-los
    /// partiria malhas legítimas. A procura não precisa de mais: as raízes nunca são colineares com
    /// as arestas que expandem (ver `polyanya`).
    NotConvex { poly: usize, at: usize },
    /// A mesma aresta orientada em dois polígonos (a malha sobrepõe-se).
    NonManifold { a: u32, b: u32 },
    /// A lista das áreas não tem um valor por polígono.
    AreaCount { polys: usize, areas: usize },
}

/// A grelha de localização: cada célula lista os polígonos cuja caixa a toca, por ordem de índice.
#[derive(Clone, Debug, Default)]
struct Grid {
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

    fn cell(&self, ix: usize, iy: usize) -> &[u32] {
        let c = iy * self.nx + ix;
        &self.items[self.off[c] as usize..self.off[c + 1] as usize]
    }
}

/// A malha andável. Imutável depois de construída; a procura ([`crate::Polyanya`]) só a lê.
#[derive(Clone, Debug)]
pub struct NavMesh {
    verts: Vec<V2>,
    /// Os anéis, contíguos: o polígono `p` é `ring[ring_off[p]..ring_off[p + 1]]`, e o vizinho e o
    /// gémeo de cada aresta estão nas MESMAS posições de `nbrs` e `twin`.
    ring_off: Vec<u32>,
    ring: Vec<u32>,
    nbrs: Vec<Option<u32>>,
    twin: Vec<u32>,
    /// Um vértice que toca uma PAREDE — os únicos sítios onde um caminho mais curto pode virar.
    corner: Vec<bool>,
    /// Os polígonos que têm cada vértice, por ordem de índice — contíguos: os do vértice `v` são
    /// `vert_polys[vp_off[v]..vp_off[v + 1]]`.
    vp_off: Vec<u32>,
    vert_polys: Vec<u32>,
    /// A componente ligada de cada polígono (`0..islands`), numerada pela ordem do 1.º polígono.
    island: Vec<u32>,
    islands: u32,
    /// A ÁREA de cada polígono (`0` = o chão comum). A malha diz ONDE; quanto custa atravessar cada
    /// área é da consulta (W7) — dois agentes com tabelas diferentes partilham a mesma malha.
    area: Vec<u16>,
    /// As arestas de parede, `(de, para)` no sentido do polígono que as tem.
    walls: Vec<(u32, u32)>,
    grid: Grid,
    min: V2,
    max: V2,
}

impl NavMesh {
    /// A ÚNICA porta: confere cada polígono (≥ 3 vértices, anti-horário, convexo), liga
    /// os vizinhos pelas arestas partilhadas, marca os cantos, numera as ilhas e monta a grelha.
    pub fn from_polygons(verts: Vec<V2>, polys: Vec<Vec<u32>>) -> Result<NavMesh, MeshError> {
        let area = vec![0; polys.len()];
        Self::from_polygons_with_areas(verts, polys, area)
    }

    /// [`Self::from_polygons`] com a área de cada polígono. ⚠️ Uma aresta entre áreas diferentes
    /// continua a ser PASSAGEM (vizinhança), nunca parede: a área muda o custo, não a topologia.
    pub fn from_polygons_with_areas(
        verts: Vec<V2>,
        polys: Vec<Vec<u32>>,
        area: Vec<u16>,
    ) -> Result<NavMesh, MeshError> {
        let mut ring_off = Vec::with_capacity(polys.len() + 1);
        ring_off.push(0u32);
        let mut ring = Vec::with_capacity(polys.iter().map(Vec::len).sum());
        for p in &polys {
            ring.extend_from_slice(p);
            ring_off.push(ring.len() as u32);
        }
        Self::from_rings(verts, ring_off, ring, area)
    }

    /// ⭐ **A porta**, com os anéis já contíguos (`ring_off` com um a mais que os polígonos, a começar
    /// em `0` e a acabar em `ring.len()`): confere cada polígono (≥ 3 vértices, anti-horário,
    /// convexo), liga os vizinhos pelas arestas partilhadas, marca os cantos, numera as ilhas e monta
    /// a grelha. As outras duas chegam aqui.
    pub fn from_rings(
        verts: Vec<V2>,
        ring_off: Vec<u32>,
        ring: Vec<u32>,
        area: Vec<u16>,
    ) -> Result<NavMesh, MeshError> {
        assert!(
            ring_off.first() == Some(&0)
                && ring_off.last() == Some(&(ring.len() as u32))
                && ring_off.windows(2).all(|w| w[0] <= w[1]),
            "anéis mal formados: os deslocamentos não vão de 0 a {} por ordem",
            ring.len()
        );
        let np = ring_off.len() - 1;
        let anel = |pi: usize| &ring[ring_off[pi] as usize..ring_off[pi + 1] as usize];
        if area.len() != np {
            return Err(MeshError::AreaCount {
                polys: np,
                areas: area.len(),
            });
        }
        let nv = verts.len();
        for pi in 0..np {
            let p = anel(pi);
            if p.len() < 3 {
                return Err(MeshError::Degenerate { poly: pi });
            }
            if let Some(&bad) = p.iter().find(|&&i| i as usize >= nv) {
                return Err(MeshError::BadIndex {
                    poly: pi,
                    index: bad,
                });
            }
            let n = p.len();
            let mut area2 = 0.0;
            for i in 0..n {
                let a = verts[p[i] as usize];
                let b = verts[p[(i + 1) % n] as usize];
                area2 += a[0] * b[1] - b[0] * a[1];
            }
            if area2 <= 0.0 {
                return Err(MeshError::NotCcw { poly: pi, area2 });
            }
            for i in 0..n {
                let a = verts[p[(i + n - 1) % n] as usize];
                let b = verts[p[i] as usize];
                let c = verts[p[(i + 1) % n] as usize];
                // Convexidade, medida como DISTÂNCIA (metros) de `c` à recta `a → b`.
                if side_dist(a, b, c) < -EPS {
                    return Err(MeshError::NotConvex { poly: pi, at: i });
                }
            }
        }

        // Os polígonos de cada vértice, contíguos e por ordem de índice (contagem, depois enchimento).
        let mut vp_off = vec![0u32; nv + 1];
        for &v in &ring {
            vp_off[v as usize + 1] += 1;
        }
        for i in 0..nv {
            vp_off[i + 1] += vp_off[i];
        }
        let mut cursor = vp_off.clone();
        let mut vert_polys = vec![0u32; vp_off[nv] as usize];
        for pi in 0..np {
            for &v in anel(pi) {
                vert_polys[cursor[v as usize] as usize] = pi as u32;
                cursor[v as usize] += 1;
            }
        }
        // ⭐ As arestas que SAEM de cada vértice — `(destino, polígono, índice)`, contíguas e por ordem
        // de polígono e de aresta (contagem, depois enchimento). A pergunta «quem tem a aresta
        // orientada `(u, w)`» lê só as de `u` (medido: a vizinhança e a sobreposição pela lista dos
        // polígonos de cada vértice custavam `2,4 ms` de `3,6` numa malha de `14 571` polígonos).
        let mut eo_off = vec![0u32; nv + 1];
        for &v in &ring {
            eo_off[v as usize + 1] += 1;
        }
        for i in 0..nv {
            eo_off[i + 1] += eo_off[i];
        }
        let mut cursor = eo_off.clone();
        let mut saidas = vec![(0u32, 0u32, 0u32); eo_off[nv] as usize];
        for pi in 0..np {
            let p = anel(pi);
            let n = p.len();
            for i in 0..n {
                let u = p[i] as usize;
                saidas[cursor[u] as usize] = (p[(i + 1) % n], pi as u32, i as u32);
                cursor[u] += 1;
            }
        }
        let de = |u: u32| &saidas[eo_off[u as usize] as usize..eo_off[u as usize + 1] as usize];

        // ⚠️ A mesma aresta orientada em dois polígonos sobrepõe a malha: conferido pela ordem dos
        // polígonos e das arestas, reportando a 2.ª ocorrência (a resposta da tabela que aqui esteve).
        for pi in 0..np {
            let p = anel(pi);
            let n = p.len();
            for i in 0..n {
                let (u, w) = (p[i], p[(i + 1) % n]);
                if de(u).iter().any(|&(d, q, _)| d == w && (q as usize) < pi) {
                    return Err(MeshError::NonManifold { a: u, b: w });
                }
            }
        }
        // A vizinhança pela aresta partilhada (u, w) ⇔ (w, u): a 1.ª, pela ordem dos polígonos, entre
        // as que saem de `w`.
        let mut nbrs = vec![None; ring.len()];
        let mut twin = vec![u32::MAX; ring.len()];
        let mut corner = vec![false; nv];
        let mut walls = Vec::new();
        for (pi, fatia) in ring_off.windows(2).enumerate() {
            let (o, p) = (
                fatia[0] as usize,
                &ring[fatia[0] as usize..fatia[1] as usize],
            );
            let n = p.len();
            for i in 0..n {
                let (u, w) = (p[i], p[(i + 1) % n]);
                let par = de(w)
                    .iter()
                    .find(|&&(d, q, _)| d == u && q as usize != pi)
                    .map(|&(_, q, j)| (q, j));
                match par {
                    Some((q, j)) => {
                        nbrs[o + i] = Some(q);
                        twin[o + i] = j;
                    }
                    None => {
                        corner[u as usize] = true;
                        corner[w as usize] = true;
                        walls.push((u, w));
                    }
                }
            }
        }

        // As ilhas: componentes ligadas pela vizinhança, numeradas pela ordem do 1.º polígono.
        let mut island = vec![u32::MAX; np];
        let mut islands = 0;
        let mut stack = Vec::new();
        for start in 0..np {
            if island[start] != u32::MAX {
                continue;
            }
            island[start] = islands;
            stack.push(start as u32);
            while let Some(q) = stack.pop() {
                let (a, b) = (
                    ring_off[q as usize] as usize,
                    ring_off[q as usize + 1] as usize,
                );
                for nb in nbrs[a..b].iter().flatten() {
                    if island[*nb as usize] == u32::MAX {
                        island[*nb as usize] = islands;
                        stack.push(*nb);
                    }
                }
            }
            islands += 1;
        }

        let (min, max) = bounds(&verts, &ring);
        let grid = build_grid(&verts, &ring_off, &ring, min, max);
        Ok(NavMesh {
            verts,
            ring_off,
            ring,
            nbrs,
            twin,
            corner,
            vp_off,
            vert_polys,
            island,
            islands,
            area,
            walls,
            grid,
            min,
            max,
        })
    }

    #[inline]
    pub fn verts(&self) -> &[V2] {
        &self.verts
    }

    /// O polígono `p` (uma vista, ver [`Poly`]).
    #[inline]
    pub fn poly(&self, p: u32) -> Poly<'_> {
        let (a, b) = (
            self.ring_off[p as usize] as usize,
            self.ring_off[p as usize + 1] as usize,
        );
        Poly {
            verts: &self.ring[a..b],
            nbrs: &self.nbrs[a..b],
            twin: &self.twin[a..b],
        }
    }

    /// Quantos polígonos.
    #[inline]
    pub fn poly_count(&self) -> usize {
        self.ring_off.len() - 1
    }

    /// Os polígonos, por ordem de índice.
    pub fn polys(&self) -> impl ExactSizeIterator<Item = Poly<'_>> + '_ {
        (0..self.poly_count() as u32).map(|p| self.poly(p))
    }

    #[inline]
    pub fn vert(&self, v: u32) -> V2 {
        self.verts[v as usize]
    }

    #[inline]
    pub fn is_corner(&self, v: u32) -> bool {
        self.corner[v as usize]
    }

    #[inline]
    pub fn polys_at_vertex(&self, v: u32) -> &[u32] {
        &self.vert_polys[self.vp_off[v as usize] as usize..self.vp_off[v as usize + 1] as usize]
    }

    #[inline]
    pub fn island(&self, poly: u32) -> u32 {
        self.island[poly as usize]
    }

    #[inline]
    pub fn island_count(&self) -> u32 {
        self.islands
    }

    /// A área de custo do polígono (`0` = o chão comum). ⚠️ Não confundir com [`Self::area`], os m².
    #[inline]
    pub fn area_id(&self, poly: u32) -> u16 {
        self.area[poly as usize]
    }

    /// Há algum polígono fora do chão comum? (Sem nenhum, a procura uniforme é a resposta exacta.)
    pub fn has_areas(&self) -> bool {
        self.area.iter().any(|&a| a != 0)
    }

    #[inline]
    pub fn walls(&self) -> &[(u32, u32)] {
        &self.walls
    }

    /// A caixa da malha (`min`, `max`).
    #[inline]
    pub fn bounds(&self) -> (V2, V2) {
        (self.min, self.max)
    }

    /// A área andável (soma das áreas dos polígonos), em m².
    pub fn area(&self) -> f64 {
        self.polys()
            .map(|p| poly_area2(&self.verts, p.verts) * 0.5)
            .sum()
    }

    /// O polígono `poly` contém `p` (FECHADO, à tolerância [`EPS`])?
    pub fn poly_contains(&self, poly: u32, p: V2) -> bool {
        let pv = self.poly(poly).verts;
        let n = pv.len();
        (0..n).all(|i| {
            side_dist(
                self.verts[pv[i] as usize],
                self.verts[pv[(i + 1) % n] as usize],
                p,
            ) >= -EPS
        })
    }

    /// TODOS os polígonos que contêm `p` (fechado), por ordem de índice. Um ponto numa aresta está em
    /// dois; num vértice, em todos os do leque. Vazio ⇒ `p` está fora da malha.
    pub fn locate_all(&self, p: V2, out: &mut Vec<u32>) {
        out.clear();
        if p[0] < self.min[0] - EPS
            || p[1] < self.min[1] - EPS
            || p[0] > self.max[0] + EPS
            || p[1] > self.max[1] + EPS
        {
            return;
        }
        // Um ponto em cima da fronteira de duas células pode pertencer a polígonos registados só na
        // vizinha: consulta-se a célula do ponto e as que a tolerância alcança.
        let mut cells = Vec::with_capacity(4);
        for dx in [-EPS, EPS] {
            for dy in [-EPS, EPS] {
                if let Some(c) = self.grid.cell_of([p[0] + dx, p[1] + dy])
                    && !cells.contains(&c)
                {
                    cells.push(c);
                }
            }
        }
        for (ix, iy) in cells {
            for &q in self.grid.cell(ix, iy) {
                if !out.contains(&q) && self.poly_contains(q, p) {
                    out.push(q);
                }
            }
        }
        out.sort_unstable();
    }

    /// O polígono de menor índice que contém `p`, ou `None` fora da malha.
    pub fn locate(&self, p: V2) -> Option<u32> {
        let mut v = Vec::new();
        self.locate_all(p, &mut v);
        v.first().copied()
    }

    /// O ponto da malha mais perto de `p` — o próprio `p` se ele está dentro. Com `island`, só
    /// polígonos dessa ilha contam (o *«alvo inalcançável ⇒ o ponto alcançável mais perto»*).
    /// `None` só numa malha sem polígonos (ou numa ilha que não existe).
    pub fn nearest_point(&self, p: V2, island: Option<u32>) -> Option<(V2, u32)> {
        let mut here = Vec::new();
        self.locate_all(p, &mut here);
        if let Some(&q) = here
            .iter()
            .find(|&&q| island.is_none_or(|i| self.island[q as usize] == i))
        {
            return Some((p, q));
        }
        // Fora (ou noutra ilha): o mais perto está na fronteira de um polígono da ilha pedida.
        // ⚠️ Varre TODAS as arestas dos polígonos aceites — não só as paredes: com `island`, o
        // mais perto pode estar numa aresta interior que só é fronteira para quem vem de outra ilha
        // (nunca — uma ilha é fechada por paredes; mas sem `island` a regra fica igual e simples).
        let mut best: Option<(f64, V2, u32)> = None;
        for &(u, w) in &self.walls {
            let a = self.verts[u as usize];
            let b = self.verts[w as usize];
            let (_, c) = closest_on_segment(a, b, p);
            let d = dist(c, p);
            if best.is_some_and(|(bd, _, _)| d >= bd) {
                continue;
            }
            // De que polígono é esta parede? Do que a tem como aresta (u → w).
            let owner = self.polys_at_vertex(u).iter().copied().find(|&q| {
                let pv = self.poly(q).verts;
                let n = pv.len();
                (0..n).any(|i| pv[i] == u && pv[(i + 1) % n] == w)
            });
            let Some(owner) = owner else { continue };
            if island.is_some_and(|i| self.island[owner as usize] != i) {
                continue;
            }
            best = Some((d, c, owner));
        }
        best.map(|(_, c, q)| (c, q))
    }
}

/// A área com sinal ×2 de um anel de índices.
pub(crate) fn poly_area2(verts: &[V2], ring: &[u32]) -> f64 {
    let n = ring.len();
    let mut a = 0.0;
    for i in 0..n {
        let p = verts[ring[i] as usize];
        let q = verts[ring[(i + 1) % n] as usize];
        a += p[0] * q[1] - q[0] * p[1];
    }
    a
}

fn bounds(verts: &[V2], ring: &[u32]) -> (V2, V2) {
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for &v in ring {
        let q = verts[v as usize];
        min = [min[0].min(q[0]), min[1].min(q[1])];
        max = [max[0].max(q[0]), max[1].max(q[1])];
    }
    if ring.is_empty() {
        return ([0.0; 2], [0.0; 2]);
    }
    (min, max)
}

/// A grelha de localização. ⚠️ O lado da célula não é um número escolhido: sai da CONTAGEM, para
/// que haja `~1` polígono por célula (`√n` células por eixo sobre a maior dimensão) — com menos
/// células a pergunta paga a lista, com mais paga a memória vazia.
fn build_grid(verts: &[V2], ring_off: &[u32], ring: &[u32], min: V2, max: V2) -> Grid {
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

#[cfg(test)]
pub(crate) fn orient_ok(verts: &[V2], ring: &[u32]) -> bool {
    let n = ring.len();
    (0..n).all(|i| {
        orient(
            verts[ring[i] as usize],
            verts[ring[(i + 1) % n] as usize],
            verts[ring[(i + 2) % n] as usize],
        ) > 0.0
    })
}
