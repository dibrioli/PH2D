//! **A COSTURA** — amostrar uma vez, montar, alisar, reprojetar.
//!
//! ⚠️ **A regra que governa este ficheiro inteiro:** *um ponto de fronteira
//! pertence ao ARCO, nunca ao patch.* Os dois patches que partilham um arco
//! pedem-lhe os mesmos índices — um deles ao contrário. Amostrar por patch daria
//! dois conjuntos de pontos quase iguais sobre a mesma curva, e a malha sairia
//! **rasgada** ao longo de toda fronteira de patch: um erro pequeno demais para
//! se ver num render e grande demais para uma malha ser usável.

use std::collections::BTreeMap;

use ph2d_mesh::{Face, Mesh};
use ph2d_quantize::Quantization;
use ph2d_trace::PatchLayout;

use crate::fan::{coons, resample, segment};

/// Por que a malha não pôde ser montada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FillError {
    /// ⚠️ **A lei do patch não bate com a quantização.** `L_i` tinha de ser
    /// `e_{i-1} + e_{i+1}`, e não é. Isto é **bug a montante**, não uma
    /// propriedade da malha: o F4 devolve `e` que satisfazem a lei por
    /// construção, logo ou os lados vieram fora de ordem ou o `e` é de outro
    /// patch. Recusar em vez de remendar é o que impede uma malha torcida.
    Mismatch {
        /// Qual patch.
        patch: usize,
        /// Qual lado.
        side: usize,
        /// O que a lei exigia.
        expected: u32,
        /// O que os arcos somaram.
        got: u32,
    },
    /// Um lado não emenda no seguinte — a fronteira do patch não fecha.
    Broken {
        /// Qual patch.
        patch: usize,
        /// Qual lado.
        side: usize,
    },
    /// A malha resultante não monta.
    Mesh(String),
}

/// O que a montagem mediu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FillReport {
    /// Quantos quads.
    pub quads: usize,
    /// Quantas faces que **não** são quads. ⭐ Tem de ser **zero** — é a promessa
    /// inteira desta família de algoritmos.
    pub non_quads: usize,
    /// Quantos vértices.
    pub verts: usize,
    /// ⭐ Quantos vértices **irregulares** (valência ≠ 4). É a grandeza que o
    /// artista vê, e a que o pivô existiu para derrubar: a família local entregava
    /// 21 a 49 %, o oráculo entrega 0,2 %.
    pub irregular: usize,
    /// ⚠️ Quantas arestas ficaram com **uma** face só. Tem de ser zero numa
    /// superfície fechada: é o instrumento que denuncia a malha rasgada.
    pub boundary_edges: usize,
    /// Quantas rondas de alisamento correram.
    pub smoothing: usize,
    /// ⚠️ Quantas faces tiveram de ser **invertidas** para o volume ficar
    /// positivo. É `0` ou `todas` — qualquer outro número seria orientação
    /// inconsistente, que é outro defeito.
    pub flipped: usize,
}

/// ⚠️ **Quantas rondas de alisamento por omissão.** Elas não mudam a topologia —
/// só onde os pontos ficam — e cada uma **reprojeta** sobre a superfície de
/// referência, então a forma não escorre. O número está aqui e não numa opinião:
/// ver a tabela do `PLAN.md` §4-sexies.
pub const SMOOTHING_ROUNDS: usize = 6;

/// **MONTA A MALHA DE QUADS.**
///
/// # Errors
/// [`FillError`] quando a estrutura a montante não fecha — ver as variantes.
pub fn fill(
    reference: &Mesh,
    layout: &PatchLayout,
    quant: &Quantization,
    smoothing: usize,
) -> Result<(Mesh, FillReport), FillError> {
    let src = reference.positions();
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();

    // ── 1. Os pontos de cada ARCO, amostrados UMA vez.
    let mut corner_vid: BTreeMap<u32, u32> = BTreeMap::new();
    let mut arc_points: Vec<Vec<u32>> = Vec::with_capacity(layout.arc_chain.len());
    for (a, chain) in layout.arc_chain.iter().enumerate() {
        let n = quant.arc[a].max(1) as usize;
        let curve: Vec<[f32; 3]> = chain.iter().map(|&v| src[v as usize]).collect();
        let sampled = resample(&curve, n);
        let mut ids = Vec::with_capacity(n + 1);
        for (k, p) in sampled.iter().enumerate() {
            // As pontas são cantos de malha, e um canto é de TODOS os arcos que
            // lá chegam — por isso a chave é o vértice, não o par (arco, índice).
            let id = if k == 0 || k == n {
                let v = if k == 0 {
                    chain[0]
                } else {
                    *chain.last().unwrap_or(&chain[0])
                };
                *corner_vid.entry(v).or_insert_with(|| {
                    pos.push(src[v as usize]);
                    u32::try_from(pos.len() - 1).unwrap_or(u32::MAX)
                })
            } else {
                pos.push(*p);
                u32::try_from(pos.len() - 1).unwrap_or(u32::MAX)
            };
            ids.push(id);
        }
        arc_points.push(ids);
    }

    // ── 2. Cada patch vira o seu leque.
    for (p, sides) in layout.side_arcs.iter().enumerate() {
        let n = sides.len();
        let e = &quant.corners[p];
        if n < 3 || e.len() != n {
            return Err(FillError::Mismatch {
                patch: p,
                side: 0,
                expected: u32::try_from(n).unwrap_or(0),
                got: u32::try_from(e.len()).unwrap_or(0),
            });
        }
        // Os pontos de cada lado, do canto de entrada ao de saída.
        let mut side_pts: Vec<Vec<u32>> = Vec::with_capacity(n);
        for (i, side) in sides.iter().enumerate() {
            let mut pts: Vec<u32> = Vec::new();
            for &(a, rev) in side {
                let mut ids = arc_points[a as usize].clone();
                if rev {
                    ids.reverse();
                }
                if pts.is_empty() {
                    pts = ids;
                } else {
                    if *pts.last().unwrap_or(&u32::MAX) != ids[0] {
                        return Err(FillError::Broken { patch: p, side: i });
                    }
                    pts.extend_from_slice(&ids[1..]);
                }
            }
            side_pts.push(pts);
        }
        // ⚠️ A lei, conferida: se ela não bate aqui, o resto é geometria torcida.
        for i in 0..n {
            let want = e[(i + n - 1) % n] + e[(i + 1) % n];
            let got = u32::try_from(side_pts[i].len() - 1).unwrap_or(u32::MAX);
            if want != got {
                return Err(FillError::Mismatch {
                    patch: p,
                    side: i,
                    expected: want,
                    got,
                });
            }
            if side_pts[i].last() != side_pts[(i + 1) % n].first() {
                return Err(FillError::Broken { patch: p, side: i });
            }
        }

        // O centro: a média dos pontos de fronteira, reprojetada.
        let mut c = [0.0f32; 3];
        let mut count = 0usize;
        for pts in &side_pts {
            for &v in &pts[..pts.len() - 1] {
                let q = pos[v as usize];
                for k in 0..3 {
                    c[k] += q[k];
                }
                count += 1;
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let inv = if count == 0 { 0.0 } else { 1.0 / count as f32 };
        let center = ph2d_remesh_iso::project_onto(
            reference,
            [c[0] * inv, c[1] * inv, c[2] * inv],
            bbox_seed(reference),
        );
        pos.push(center);
        let center_vid = u32::try_from(pos.len() - 1).unwrap_or(u32::MAX);

        // Os raios: do centro ao corte de cada lado.
        let mut spoke: Vec<Vec<u32>> = Vec::with_capacity(n);
        for i in 0..n {
            let cut = e[(i + n - 1) % n] as usize;
            let tip = side_pts[i][cut];
            let steps = e[i] as usize;
            let line = segment(center, pos[tip as usize], steps);
            let mut ids = Vec::with_capacity(steps + 1);
            ids.push(center_vid);
            for q in line.iter().take(steps).skip(1) {
                pos.push(*q);
                ids.push(u32::try_from(pos.len() - 1).unwrap_or(u32::MAX));
            }
            ids.push(tip);
            spoke.push(ids);
        }

        // As `n` grades.
        for i in 0..n {
            let j = (i + 1) % n;
            let (s, t) = (e[j] as usize, e[i] as usize);
            let cut_i = e[(i + n - 1) % n] as usize;
            let cut_j = e[i] as usize;
            let bottom = &side_pts[i][cut_i..];
            let right = &side_pts[j][..=cut_j];
            let top = &spoke[j];
            let left: Vec<u32> = spoke[i].iter().rev().copied().collect();
            let grid = build_grid(&mut pos, bottom, top, &left, right, s, t);
            for k in 0..s {
                for l in 0..t {
                    faces.push(Face::quad(
                        grid[k][l],
                        grid[k + 1][l],
                        grid[k + 1][l + 1],
                        grid[k][l + 1],
                    ));
                }
            }
        }
    }

    // ── 3. A malha, com a orientação conferida.
    let mut flipped = 0usize;
    if signed_volume(&pos, &faces) < 0.0 {
        // ⚠️ **Ou todas ou nenhuma.** A orientação da grade sai da orientação da
        // fronteira do patch, que é a mesma para todos os patches; se ela estiver
        // ao contrário, está ao contrário em bloco. Inverter face a face por
        // teste local é que produziria uma malha inconsistente.
        for f in &mut faces {
            let v = f.verts().to_vec();
            *f = Face::quad(v[3], v[2], v[1], v[0]);
        }
        flipped = faces.len();
    }
    let mut mesh = Mesh::from_parts(pos, faces).map_err(|e| FillError::Mesh(format!("{e:?}")))?;

    // ── 4. Alisar, reprojetando sempre.
    for _ in 0..smoothing {
        smooth_once(&mut mesh, reference);
    }

    let report = measure(&mesh, smoothing, flipped);
    Ok((mesh, report))
}

/// Os índices da grade: bordos vindos das curvas, interior por Coons.
fn build_grid(
    pos: &mut Vec<[f32; 3]>,
    bottom: &[u32],
    top: &[u32],
    left: &[u32],
    right: &[u32],
    s: usize,
    t: usize,
) -> Vec<Vec<u32>> {
    let at = |ids: &[u32], k: usize, pos: &[[f32; 3]]| pos[ids[k] as usize];
    let b: Vec<[f32; 3]> = (0..=s).map(|k| at(bottom, k, pos)).collect();
    let tp: Vec<[f32; 3]> = (0..=s).map(|k| at(top, k, pos)).collect();
    let l: Vec<[f32; 3]> = (0..=t).map(|k| at(left, k, pos)).collect();
    let r: Vec<[f32; 3]> = (0..=t).map(|k| at(right, k, pos)).collect();
    let inner = coons(&b, &tp, &l, &r);
    let mut grid = vec![vec![0u32; t + 1]; s + 1];
    for (k, row) in grid.iter_mut().enumerate() {
        for (l_i, cell) in row.iter_mut().enumerate() {
            *cell = if l_i == 0 {
                bottom[k]
            } else if l_i == t {
                top[k]
            } else if k == 0 {
                left[l_i]
            } else if k == s {
                right[l_i]
            } else {
                pos.push(inner[k][l_i]);
                u32::try_from(pos.len() - 1).unwrap_or(u32::MAX)
            };
        }
    }
    grid
}

/// Um passo de Laplaciano tangencial, seguido de reprojeção.
fn smooth_once(mesh: &mut Mesh, reference: &Mesh) {
    let n = mesh.vert_count();
    let neighbours: Vec<Vec<u32>> = {
        let adj = mesh.adjacency();
        (0..n)
            .map(|v| adj.vert_verts.neighbours(v).to_vec())
            .collect()
    };
    let normals: Vec<[f32; 3]> = mesh.normals().to_vec();
    let seed = bbox_seed(reference);
    let mut next = vec![[0.0f32; 3]; n];
    {
        let pos = mesh.positions();
        for v in 0..n {
            let ns = &neighbours[v];
            if ns.len() < 3 {
                next[v] = pos[v];
                continue;
            }
            let mut sum = [0.0f32; 3];
            for &w in ns {
                let q = pos[w as usize];
                for k in 0..3 {
                    sum[k] += q[k];
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let inv = 1.0 / ns.len() as f32;
            let p = pos[v];
            let d = [
                sum[0].mul_add(inv, -p[0]),
                sum[1].mul_add(inv, -p[1]),
                sum[2].mul_add(inv, -p[2]),
            ];
            // ⚠️ **Só a parte TANGENTE.** A componente normal encolheria a peça a
            // cada ronda, e a reprojeção a seguir esconderia o encolhimento sem o
            // desfazer.
            let nv = normals[v];
            let along = d[0].mul_add(nv[0], d[1].mul_add(nv[1], d[2] * nv[2]));
            next[v] = [
                LAMBDA.mul_add(along.mul_add(-nv[0], d[0]), p[0]),
                LAMBDA.mul_add(along.mul_add(-nv[1], d[1]), p[1]),
                LAMBDA.mul_add(along.mul_add(-nv[2], d[2]), p[2]),
            ];
        }
    }
    for q in &mut next {
        *q = ph2d_remesh_iso::project_onto(reference, *q, seed);
    }
    mesh.positions_mut().copy_from_slice(&next);
    mesh.rebuild();
}

/// Meio passo — o amortecimento que o torna monótono.
const LAMBDA: f32 = 0.5;

/// O raio inicial da busca de reprojeção: uma fração da diagonal da caixa.
fn bbox_seed(mesh: &Mesh) -> f32 {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt() * 0.02
}

/// Volume com sinal — o que diz se a orientação saiu ao contrário.
fn signed_volume(pos: &[[f32; 3]], faces: &[Face]) -> f32 {
    let mut total = 0.0f32;
    for f in faces {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            );
            total += a[0].mul_add(
                b[1].mul_add(c[2], -(b[2] * c[1])),
                a[1].mul_add(
                    b[2].mul_add(c[0], -(b[0] * c[2])),
                    a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                ),
            );
        }
    }
    total / 6.0
}

/// As grandezas que o relatório carrega.
fn measure(mesh: &Mesh, smoothing: usize, flipped: usize) -> FillReport {
    let faces = mesh.faces();
    let quads = faces.iter().filter(|f| !f.is_tri()).count();
    let mut count: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in faces {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *count.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let boundary_edges = count.values().filter(|&&c| c == 1).count();
    let adj = mesh.adjacency();
    let irregular = (0..mesh.vert_count())
        .filter(|&v| !adj.is_border(v) && adj.valence(v) != 4)
        .count();
    FillReport {
        quads,
        non_quads: faces.len() - quads,
        verts: mesh.vert_count(),
        irregular,
        boundary_edges,
        smoothing,
        flipped,
    }
}
