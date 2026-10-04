#![forbid(unsafe_code)]
//! ⭐⭐ **A CONSTRUÇÃO DA MALHA ANDÁVEL** — dos obstáculos e da região ao [`NavMesh`] da `ph2d-nav`
//! (plano [30](../../../docs/Components/30_plano_navegacao.md) §1 e §2.3, W1).
//!
//! # O caminho, e quem decide cada passo
//!
//! ```text
//! obstáculos ──recuo pelo raio (Minkowski, grelha inteira)──┐
//!                                                           ├─► união ─► região − obstáculos   (Clipper2, inteiros)
//! região ──────encolhe pelo raio (semi-planos)──────────────┘                │
//!                                                                            ▼
//!            NavMesh ◄── fusão em convexos ◄── dentro/fora por paridade ◄── Delaunay com restrições (spade)
//!                       (Hertel–Mehlhorn,        (nossa, sobre o índice
//!                        orientação exacta)       das faces)
//! ```
//!
//! ⚠️ **Toda decisão de TOPOLOGIA é exacta** (a grelha de [`lattice`], orientação em `i128`): um `f64`
//! erra o sinal de três pontos quase colineares, e é assim que uma malha sai com um polígono virado ao
//! contrário — que a `ph2d-nav` recusa, mas que aqui não chega a existir.
//!
//! ⚠️ **O determinismo foi MEDIDO, não suposto** (W0, `tests/it/determinismo.rs`): a mesma entrada
//! dá a mesma malha AO BIT em dois PROCESSOS diferentes (as sementes de `HashSet` mudam por processo,
//! e o `spade` usa um dentro do carregamento em lote — só para pertença, e a medição confirma-o).

pub mod inflate;
pub mod lattice;
pub mod tiles;
pub mod triangulate;

pub use inflate::{Corner, Shape};
use ph2d_nav::{MeshError, NavMesh, V2};
pub use tiles::{TILE_M, TileStats, TiledMesh};
pub use triangulate::TriError;

use clipper2_rust::{
    FillRule, Path64, Paths64, Point64, difference_64, intersect_64, union_64, union_subjects_64,
};

/// Os lados do polígono que circunscreve o disco, por omissão (medido na W1: `0,48 %` do raio de
/// excesso nos cantos — ver [`inflate`]).
pub const DISK_SIDES: u32 = 32;

/// Como construir.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    /// O raio do agente (metros). `0` ⇒ a malha é a região menos os obstáculos tal como são.
    pub agent_radius: f64,
    /// Como os cantos recuam ([`Corner::Round`] é o produto).
    pub corner: Corner,
    /// Os lados do disco — uma potência de dois `≥ 4` (outro valor é arredondado para cima).
    pub disk_sides: u32,
    /// Fundir os triângulos em convexos maiores (`false` ⇒ a malha é a triangulação crua).
    pub merge: bool,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            agent_radius: 0.0,
            corner: Corner::Round,
            disk_sides: DISK_SIDES,
            merge: true,
        }
    }
}

/// Porque a construção recusou.
#[derive(Clone, Debug, PartialEq)]
pub enum BuildError {
    Triangulation(TriError),
    Mesh(MeshError),
}

/// O que a construção contou (as tabelas de custo e de fidelidade da W0/W1 lêem daqui).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BuildStats {
    pub obstacles_in: usize,
    pub rings: usize,
    pub ring_verts: usize,
    pub triangles: usize,
    pub polygons: usize,
}

/// (W7) Uma ÁREA DE CUSTO: a forma (recuada pelo raio do agente, como um obstáculo — o corpo sente-a
/// quando lhe toca) e o número que a malha guarda em cada polígono dela (`0` é o chão comum).
#[derive(Clone, Debug, PartialEq)]
pub struct Area {
    pub shape: Shape,
    pub id: u16,
}

/// A malha e o que custou.
#[derive(Clone, Debug)]
pub struct Built {
    pub mesh: NavMesh,
    pub stats: BuildStats,
}

/// Constrói a malha andável: `region` é um polígono CONVEXO (qualquer sentido) onde se anda, e
/// `obstacles` o que não se atravessa. Uma região vazia (ou toda tapada) dá uma malha SEM polígonos
/// — não é erro: é a resposta, e o agente diz *«sem região»*.
pub fn build(region: &[V2], obstacles: &[Shape], params: &Params) -> Result<Built, BuildError> {
    build_with_areas(region, obstacles, &[], params)
}

/// ⭐ (W7) [`build`] com ÁREAS DE CUSTO: a fronteira de cada área vira aresta da malha e cada polígono
/// guarda o `id` da área onde está ([`NavMesh::area_id`]). Onde duas áreas se sobrepõem manda a que
/// vem PRIMEIRO na lista (o chamador ordena-as). Sem áreas, é a construção de sempre, ao bit.
pub fn build_with_areas(
    region: &[V2],
    obstacles: &[Shape],
    areas: &[Area],
    params: &Params,
) -> Result<Built, BuildError> {
    let mut stats = BuildStats {
        obstacles_in: obstacles.len(),
        ..BuildStats::default()
    };
    let Some((walk, aneis_das_areas)) = chao_e_areas(region, obstacles, areas, params) else {
        return finish(Vec::new(), Vec::new(), Vec::new(), stats);
    };
    let feito =
        poligonos(walk, &aneis_das_areas, params.merge).map_err(BuildError::Triangulation)?;
    stats.rings = feito.rings;
    stats.ring_verts = feito.ring_verts;
    stats.triangles = feito.triangles;
    finish(feito.pts, feito.polys, feito.ids, stats)
}

/// O chão andável da construção inteira e os anéis das áreas (`None` = a região recuada não sobra).
#[allow(clippy::type_complexity)]
pub(crate) fn chao_e_areas(
    region: &[V2],
    obstacles: &[Shape],
    areas: &[Area],
    params: &Params,
) -> Option<(Paths64, Vec<(Vec<lattice::P>, u16)>)> {
    let n = params.disk_sides.max(4).next_power_of_two();
    let r = params.agent_radius.max(0.0);
    let reg = inflate::inset_region(region, r);
    if reg.len() < 3 {
        return None;
    }
    let holes: Paths64 = obstacles
        .iter()
        .map(|o| inflate::inflate(o, r, params.corner, n))
        .filter(|ring| ring.len() >= 3)
        .map(|ring| to_path(&ring))
        .collect();
    let walk: Paths64 = if holes.is_empty() {
        vec![to_path(&reg)]
    } else {
        let merged = union_subjects_64(&holes, FillRule::NonZero);
        difference_64(&vec![to_path(&reg)], &merged, FillRule::NonZero)
    };
    let aneis_das_areas: Vec<(Vec<lattice::P>, u16)> = areas
        .iter()
        .map(|a| (inflate::inflate(&a.shape, r, params.corner, n), a.id))
        .collect();
    Some((walk, aneis_das_areas))
}

/// O que [`poligonos`] devolve: os vértices na grelha, os polígonos, a área de cada um e as contas.
pub(crate) struct Poligonos {
    pub(crate) pts: Vec<lattice::P>,
    pub(crate) polys: Vec<Vec<u32>>,
    pub(crate) ids: Vec<u16>,
    pub(crate) rings: usize,
    pub(crate) ring_verts: usize,
    pub(crate) triangles: usize,
}

/// ⭐ (W7) **O chão andável `walk` partido pelas áreas → polígonos**, a porta ÚNICA da construção
/// inteira e de cada mosaico. Os PEDAÇOS: o chão comum e, por área (pela ordem: manda a primeira),
/// o que dela cai no chão e nenhuma anterior reclamou; uma só triangulação; a fusão em convexos só
/// dentro do mesmo pedaço. Sem áreas é o caminho de sempre, ao bit.
pub(crate) fn poligonos(
    walk: Paths64,
    areas: &[(Vec<lattice::P>, u16)],
    merge: bool,
) -> Result<Poligonos, TriError> {
    let (ids, aneis) = pedacos(walk, areas);
    let (pts, tris, pedaco) = triangulate::triangulate_pieces(&aneis)?;
    let triangles = tris.len();
    let (polys, pedaco) = if merge {
        triangulate::merge_convex_labeled(&pts, &tris, &pedaco)
    } else {
        (tris.iter().map(|t| t.to_vec()).collect(), pedaco)
    };
    Ok(Poligonos {
        ids: pedaco.iter().map(|&k| ids[k as usize]).collect(),
        pts,
        polys,
        rings: aneis.iter().map(Vec::len).sum(),
        ring_verts: aneis.iter().flatten().map(Vec::len).sum(),
        triangles,
    })
}

/// Os PEDAÇOS do chão: o comum e um por área (a área de cada um, e os anéis de cada um).
#[allow(clippy::type_complexity)]
pub(crate) fn pedacos(
    walk: Paths64,
    areas: &[(Vec<lattice::P>, u16)],
) -> (Vec<u16>, Vec<Vec<Vec<lattice::P>>>) {
    let mut pieces: Vec<(Paths64, u16)> = Vec::new();
    let mut reclamado: Paths64 = Vec::new();
    for (ring, id) in areas {
        if ring.len() < 3 {
            continue;
        }
        let forma = vec![to_path(ring)];
        let mut piece = intersect_64(&walk, &forma, FillRule::NonZero);
        if !reclamado.is_empty() {
            piece = difference_64(&piece, &reclamado, FillRule::NonZero);
        }
        reclamado = union_64(&reclamado, &forma, FillRule::NonZero);
        pieces.push((piece, *id));
    }
    let chao = if reclamado.is_empty() {
        walk
    } else {
        difference_64(&walk, &reclamado, FillRule::NonZero)
    };
    pieces.insert(0, (chao, 0));
    let aneis: Vec<Vec<Vec<lattice::P>>> = pieces
        .iter()
        .map(|(paths, _)| {
            paths
                .iter()
                .map(|p| p.iter().map(|q| (q.x, q.y)).collect())
                .collect()
        })
        .collect();
    (pieces.iter().map(|p| p.1).collect(), aneis)
}

fn finish(
    pts: Vec<lattice::P>,
    polys: Vec<Vec<u32>>,
    ids: Vec<u16>,
    mut stats: BuildStats,
) -> Result<Built, BuildError> {
    stats.polygons = polys.len();
    // Só os vértices que algum polígono usa (a triangulação guarda os dos anéis inteiros).
    let mut remap = vec![u32::MAX; pts.len()];
    let mut verts: Vec<V2> = Vec::new();
    let polys: Vec<Vec<u32>> = polys
        .into_iter()
        .map(|ring| {
            ring.into_iter()
                .map(|v| {
                    if remap[v as usize] == u32::MAX {
                        remap[v as usize] = verts.len() as u32;
                        verts.push(lattice::to_world(pts[v as usize]));
                    }
                    remap[v as usize]
                })
                .collect()
        })
        .collect();
    let mesh = NavMesh::from_polygons_with_areas(verts, polys, ids).map_err(BuildError::Mesh)?;
    Ok(Built { mesh, stats })
}

fn to_path(ring: &[lattice::P]) -> Path64 {
    ring.iter().map(|&(x, y)| Point64::new(x, y)).collect()
}

#[cfg(test)]
mod tests;
