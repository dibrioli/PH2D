//! ⭐⭐⭐ **A RONDA DE RELAXAÇÃO SOBRE BUFFERS** — a mesma lei do [`crate::relax`], sem tocar
//! na [`Mesh`] e em paralelo.
//!
//! # ⛔⛔⛔ Por que ela existe: três desperdícios MEDIDOS, todos por ronda
//!
//! Medido em 2026-08-28, `sculpt_eared` na densidade do botão: o acabamento era **`11,5 s`
//! de `17,7 s`** (`65 %`) numa passagem, e o botão corre a cadeia **duas** vezes.
//!
//! 1. ⛔ **`Mesh::rebuild()` a cada ronda.** Ele reconstrói as normais de face, a
//!    **adjacência**, as normais de vértice, a **curvatura** e a **octree** da saída. Uma
//!    relaxação **não muda a topologia** e não lê nenhuma das três últimas — a projecção
//!    consulta a octree da *superfície*, não a da malha que está a ser relaxada.
//!    ⚠️ **A porta única do `rebuild` continua a valer** (a `Mesh` nunca fica publicada
//!    meio-derivada): ela é chamada **uma vez, no fim**.
//! 2. ⛔ **A incidência vértice→face reconstruída a cada ronda**, pela mesma razão.
//! 3. ⛔ **Tudo sequencial** num laço embaraçosamente paralelo, numa máquina de 32 núcleos.
//!
//! # ⚠️ O paralelismo é DETERMINÍSTICO, e isso decidiu a forma
//!
//! ⛔ A forma óbvia — percorrer faces em paralelo e **acumular** no vértice — muda a **ordem
//! da soma em `f32`** entre corridas, e duas corridas do mesmo binário deixariam de dar a
//! mesma malha. ⭐ A forma que fica é **duas passagens**:
//!
//! 1. **por FACE**, a escrever no índice da própria face (sem contenção, sem soma);
//! 2. **por VÉRTICE**, a ler as faces incidentes na ordem do `vert_faces` (que é ordenada e
//!    invariante) — *a soma acontece sempre na mesma ordem*.
//!
//! *Um resultado que depende de quantos núcleos a máquina tem não é um resultado.*

use ph2d_mesh::{Face, Mesh};
use rayon::prelude::*;

use crate::quality::Hint;
use crate::relax::{square_from, square_harmonic, steer};

/// O amortecimento — meio passo, o mesmo do irmão sequencial.
const LAMBDA: f32 = 0.5;

/// O que uma face pede aos seus quatro cantos, em espaço de mundo.
#[derive(Clone, Copy)]
struct Ask {
    corner: [[f32; 3]; 4],
    used: bool,
}

impl Default for Ask {
    fn default() -> Self {
        Self {
            corner: [[0.0; 3]; 4],
            used: false,
        }
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

/// ⭐ **A incidência, construída UMA vez** — a topologia não muda numa relaxação.
///
/// ⚠️ Guarda a [`ph2d_mesh::Adjacency`] inteira e não uma cópia própria: é dela que a lei
/// das normais **da casa** precisa, e reescrever a incidência aqui seria uma segunda
/// resposta à mesma pergunta.
pub(crate) struct Topology {
    pub adj: ph2d_mesh::Adjacency,
}

impl Topology {
    pub fn of(mesh: &Mesh) -> Self {
        Self {
            adj: ph2d_mesh::Adjacency::build(mesh.vert_count(), mesh.faces()),
        }
    }
}

/// As normais de vértice — **pela lei da casa**, não por uma minha.
///
/// ⛔⛔ **A 1.ª redacção somava normais de Newell não normalizadas e o resultado MUDOU**
/// (medido 2026-08-28: `726` rondas passaram a `724` e o `p99` mexeu na 2.ª casa). Uma
/// relaxação que muda de resultado ao ser optimizada não foi optimizada — foi substituída.
/// ⭐ [`ph2d_mesh::recompute_face_normals`] e [`ph2d_mesh::recompute_vertex_normals`] são a
/// mesma lei que o `Mesh::rebuild` corre, **e já são paralelas**.
fn vertex_normals(
    pos: &[[f32; 3]],
    faces: &[Face],
    topo: &Topology,
    fnorm: &mut Vec<[f32; 3]>,
    vnorm: &mut Vec<[f32; 3]>,
) {
    ph2d_mesh::recompute_face_normals(pos, faces, fnorm);
    // ⚠️ **Ela recebe uma FATIA e não redimensiona** — quem chama é que a dimensiona. Sem
    // isto ela escreve em zero vértices e o laço lê fora dos limites.
    vnorm.resize(pos.len(), [0.0, 1.0, 0.0]);
    ph2d_mesh::recompute_vertex_normals(fnorm, &topo.adj.vert_faces, vnorm, None);
}

/// ⭐⭐⭐ **UMA RONDA**, sobre buffers. Devolve o maior movimento de um vértice.
///
/// `seed` é o piso do raio de reprojecção — ver [`crate::relax::square_relax_capped`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn round(
    pos: &mut Vec<[f32; 3]>,
    faces: &[Face],
    topo: &Topology,
    surface: &Mesh,
    hint: &[Hint],
    pull: f32,
    origin: &[[f32; 3]],
    max_travel: f32,
    seed: f32,
    fnorm: &mut Vec<[f32; 3]>,
    vnorm: &mut Vec<[f32; 3]>,
) -> f32 {
    vertex_normals(pos, faces, topo, fnorm, vnorm);
    let normals: &[[f32; 3]] = vnorm;

    // ── 1. Por FACE, a escrever no índice da própria face. Sem contenção, sem soma.
    let asks: Vec<Ask> = faces
        .par_iter()
        .enumerate()
        .map(|(fi, f)| {
            let v = f.verts();
            if v.len() != 4 {
                return Ask::default();
            }
            let p = [
                pos[v[0] as usize],
                pos[v[1] as usize],
                pos[v[2] as usize],
                pos[v[3] as usize],
            ];
            let c3 = [
                0.25 * (p[0][0] + p[1][0] + p[2][0] + p[3][0]),
                0.25 * (p[0][1] + p[1][1] + p[2][1] + p[3][1]),
                0.25 * (p[0][2] + p[1][2] + p[2][2] + p[3][2]),
            ];
            // ⚠️ Newell, não o produto de duas arestas: um quad alabeado não tem normal
            // única, e o ajuste tem de correr no plano de mínimos quadrados.
            let mut nrm = [0.0f32; 3];
            for k in 0..4 {
                let (a, b) = (p[k], p[(k + 1) % 4]);
                nrm[0] += (a[1] - b[1]) * (a[2] + b[2]);
                nrm[1] += (a[2] - b[2]) * (a[0] + b[0]);
                nrm[2] += (a[0] - b[0]) * (a[1] + b[1]);
            }
            let nl = norm(nrm);
            if nl < 1.0e-12 {
                return Ask::default();
            }
            let nu = [nrm[0] / nl, nrm[1] / nl, nrm[2] / nl];
            let r = sub(p[0], c3);
            let along = dot(r, nu);
            let e1r = [
                along.mul_add(-nu[0], r[0]),
                along.mul_add(-nu[1], r[1]),
                along.mul_add(-nu[2], r[2]),
            ];
            let e1l = norm(e1r);
            if e1l < 1.0e-12 {
                return Ask::default();
            }
            let e1 = [e1r[0] / e1l, e1r[1] / e1l, e1r[2] / e1l];
            let e2 = cross(nu, e1);
            let mut z = [[0.0f32; 2]; 4];
            for k in 0..4 {
                let d = sub(p[k], c3);
                z[k] = [dot(d, e1), dot(d, e2)];
            }
            let (mut hz, ccw) = square_harmonic(z);
            // ⭐ O relevo entra aqui — ver [`crate::relax::steer`]. A direcção vem em espaço
            // de MUNDO e é lida no plano do próprio quad.
            if let Some(h) = hint.get(fi).filter(|h| h.weight > 0.0) {
                let f2 = [dot(h.dir, e1), dot(h.dir, e2)];
                hz = steer(hz, f2, (h.weight * pull).clamp(0.0, 1.0));
            }
            let w = square_from(hz, ccw);
            let mut corner = [[0.0f32; 3]; 4];
            for k in 0..4 {
                for t in 0..3 {
                    corner[k][t] = w[k][1].mul_add(e2[t], w[k][0].mul_add(e1[t], c3[t]));
                }
            }
            Ask { corner, used: true }
        })
        .collect();

    // ── 2. Por VÉRTICE, a ler as faces incidentes na ordem do `vert_faces`.
    let seed_floor = seed;
    let next: Vec<[f32; 3]> = (0..pos.len())
        .into_par_iter()
        .map(|v| {
            let p = pos[v];
            let inc = topo.adj.vert_faces.neighbours(v);
            let mut acc = [0.0f32; 3];
            let mut cnt = 0u32;
            for &f in inc {
                let a = &asks[f as usize];
                let verts = faces[f as usize].verts();
                let Some(k) = verts.iter().position(|&i| i as usize == v) else {
                    continue;
                };
                let want = if a.used { a.corner[k] } else { p };
                for (o, w) in acc.iter_mut().zip(want) {
                    *o += w;
                }
                cnt += 1;
            }
            if cnt == 0 {
                return p;
            }
            #[allow(clippy::cast_precision_loss)]
            let inv = 1.0 / cnt as f32;
            let d = [
                acc[0].mul_add(inv, -p[0]),
                acc[1].mul_add(inv, -p[1]),
                acc[2].mul_add(inv, -p[2]),
            ];
            let nv = normals[v];
            let along = dot(d, nv);
            let mut q = [
                LAMBDA.mul_add(along.mul_add(-nv[0], d[0]), p[0]),
                LAMBDA.mul_add(along.mul_add(-nv[1], d[1]), p[1]),
                LAMBDA.mul_add(along.mul_add(-nv[2], d[2]), p[2]),
            ];
            if let Some(o) = origin.get(v).filter(|_| max_travel.is_finite()) {
                let t = sub(q, *o);
                let l = norm(t);
                if l > max_travel {
                    let s = max_travel / l;
                    q = [
                        t[0].mul_add(s, o[0]),
                        t[1].mul_add(s, o[1]),
                        t[2].mul_add(s, o[2]),
                    ];
                }
            }
            q
        })
        .collect();

    // ── 3. O raio da reprojecção sai do maior movimento — ver o irmão sequencial.
    let moved = next
        .par_iter()
        .zip(pos.par_iter())
        .map(|(q, p)| norm(sub(*q, *p)))
        .reduce(|| 0.0f32, f32::max);
    let radius = (2.0 * moved).max(seed_floor);
    let landed: Vec<[f32; 3]> = next
        .par_iter()
        .map(|q| ph2d_remesh_iso::project_onto(surface, *q, radius))
        .collect();
    let real = landed
        .par_iter()
        .zip(pos.par_iter())
        .map(|(q, p)| norm(sub(*q, *p)))
        .reduce(|| 0.0f32, f32::max);
    pos.copy_from_slice(&landed);
    real
}
