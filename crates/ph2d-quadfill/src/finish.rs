//! ⭐⭐ **O ACABAMENTO** — alisar, orientar e MEDIR o que saiu.
//!
//! ⚠️ **Irmão da [`crate::stitch`] pelo teto de LOC (HR-18, 700) e por ASSUNTO:** lá a
//! malha **nasce** (a amostragem partilhada dos arcos, o achatamento, a grade de cada
//! patch); aqui ela é *arrumada e julgada*. ⭐ Os dois passos deste ficheiro são os
//! únicos que tocam a malha **depois** de ela existir, e o terceiro é o que diz o que
//! ela vale.

use std::collections::BTreeMap;

use ph2d_mesh::{Face, Mesh};

use crate::report::{FillReport, Provenance};

/// ⭐⭐⭐ **O ACABAMENTO, para quem NÃO passou pelo [`crate::fill`]** — `rounds` passos de
/// Laplaciano tangencial com reprojeção.
///
/// ⛔⛔ **Ele existe porque a cadeia da EXTRACÇÃO não o tinha.** Medido 2026-08-26: o caminho
/// do `ph2d_quadextract` monta a malha e entrega-a **crua**, enquanto o irmão dela — o
/// `fill` — corre [`crate::SMOOTHING_ROUNDS`] passos disto desde sempre. *Dois caminhos para
/// o mesmo produto, e só um com acabamento.*
///
/// ⚠️ **`surface` é a malha ORIGINAL, nunca a remalhada** — a mesma lei que o doc do
/// [`crate::fill`] escreve com o defeito de 2026-08-21 ao lado.
pub fn smooth(mesh: &mut Mesh, surface: &Mesh, rounds: usize) {
    for _ in 0..rounds {
        smooth_once(mesh, surface);
    }
}

/// Um passo de Laplaciano tangencial, seguido de reprojeção.
pub(crate) fn smooth_once(mesh: &mut Mesh, reference: &Mesh) {
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
    // ⛔⛔ **AQUI a reprojeção é SEM direção, e a alternativa foi medida e
    // rejeitada.** Parece a irmã da colocação — que passou a levar a normal para
    // não atravessar um vinco côncavo ([`ph2d_remesh_iso::project_facing`]) — mas
    // não é: lá a normal é um **facto** (o ponto nasceu sobre uma face concreta do
    // patch achatado); aqui seria a normal de vértice da malha **que o alisamento
    // está a mexer**, ou seja uma estimativa que a própria ronda invalida.
    //
    // Medido em 2026-08-22, esfera 24×36: com a direção no alisamento as dobras
    // foram de **1 para 10** e a aresta máxima de `2,58×` para `5,85×`. *Uma
    // estimativa que se realimenta é pior que nenhuma.*
    for q in next.iter_mut() {
        *q = ph2d_remesh_iso::project_onto(reference, *q, seed);
    }
    mesh.positions_mut().copy_from_slice(&next);
    mesh.rebuild();
}

/// Meio passo — o amortecimento que o torna monótono.
const LAMBDA: f32 = 0.5;

/// O raio inicial da busca de reprojeção: uma fração da diagonal da caixa.
pub(crate) fn bbox_seed(mesh: &Mesh) -> f32 {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt() * 0.02
}

/// Volume com sinal — o que diz se a orientação saiu ao contrário.
pub(crate) fn signed_volume(pos: &[[f32; 3]], faces: &[Face]) -> f32 {
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
pub(crate) fn measure(
    mesh: &Mesh,
    surface: &Mesh,
    prov: &[Provenance],
    smoothing: usize,
    flipped: usize,
) -> FillReport {
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
    // ⭐ **As duas grandezas geométricas**, medidas sobre as arestas da saída.
    let mut lens: Vec<f32> = Vec::with_capacity(count.len());
    let pos = mesh.positions();
    for (a, b) in count.keys() {
        let (p, q) = (pos[*a as usize], pos[*b as usize]);
        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        lens.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
    }
    lens.sort_by(f32::total_cmp);
    let edge_max = lens.last().copied().unwrap_or(0.0);
    let edge_median = lens.get(lens.len() / 2).copied().unwrap_or(0.0);
    // ⭐⭐ **De que FASE são as pontas das arestas longas** — ver
    // [`FillReport::edge_long_prov`]. A barra é relativa à MEDIANA e não ao alvo:
    // esta função não conhece o alvo do chamador, e a mediana é o alvo realizado.
    let mut edge_long_prov = [0usize; Provenance::COUNT];
    for ((a, b), _) in count.iter() {
        let (p, q) = (pos[*a as usize], pos[*b as usize]);
        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        let len = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
        if len > edge_median * 3.0 {
            for v in [*a, *b] {
                if let Some(pr) = prov.get(v as usize) {
                    edge_long_prov[*pr as usize] += 1;
                }
            }
        }
    }

    let mut by_provenance = [0usize; Provenance::COUNT];
    let irregular = (0..mesh.vert_count())
        .filter(|&v| !adj.is_border(v) && adj.valence(v) != 4)
        .inspect(|&v| {
            if let Some(p) = prov.get(v) {
                by_provenance[*p as usize] += 1;
            }
        })
        .count();
    FillReport {
        by_provenance,
        edge_max,
        edge_median,
        quads,
        non_quads: faces.len() - quads,
        verts: mesh.vert_count(),
        irregular,
        boundary_edges,
        smoothing,
        flipped,
        // ⚠️ Preenchidos pelo `fill`, que é quem sabe quantos patches achataram.
        flattened: 0,
        patches: 0,
        sampled: 0,
        sample_misses: 0,
        flatten_residual: 0.0,
        flatten_rounds: 0,
        rough: 0.0,
        dirty_patches: 0,
        combed_patches: 0,
        fell_back: 0,
        slid: 0,
        quad_patches: 0,
        slid_refused: [0; 5],
        conformal: 0.0,
        regraduated: 0,
        domain_cells: (0, 0),
        edge_long_prov,
        shape: crate::shape::quad_shape(mesh),
        skew_prov: crate::shape::skew_by_provenance(mesh, prov),
        skew_by_fan: (0.0, 0.0),
        domain_skew: (0.0, 0.0),
        // ⭐⭐ **A CONTAGEM DE DOBRAS entra no relatório da fase**, e não numa
        // sonda. Ela é o defeito que o artista fotografa e o único campo, com os
        // dois de aresta, que uma malha de posições embaralhadas não reproduz.
        folded: crate::quality::folded_against(surface, mesh),
        // ⭐ **A SEGUNDA régua, e ela não consulta a referência.** Ver
        // [`crate::quality::folded_by_neighbours`] — a primeira tem piso de ruído
        // numa peça com bico fino, e uma sozinha não decide.
        folded_local: crate::quality::folded_by_neighbours(mesh),
        // ⭐⭐ **A PROVENIÊNCIA das faces dobradas — quem nomeia a FASE.** Ver
        // [`FillReport::folded_prov`].
        folded_prov: {
            let mut tally = [0usize; Provenance::COUNT];
            for f in crate::quality::folded_faces_by_neighbours(mesh) {
                for &v in mesh.faces()[f as usize].verts() {
                    if let Some(p) = prov.get(v as usize) {
                        tally[*p as usize] += 1;
                    }
                }
            }
            tally
        },
    }
}
