//! **OS CAMPOS, RESOLVIDOS DE CIMA PARA BAIXO** — a porta que o produto chama
//! (ADR-0160, Q3.5).
//!
//! Resolver na malha fina direto é o que a Q1/Q2 fizeram, e a Q3 mediu o preço:
//! **60,9 %** de quads, porque o campo de posição não forma platôs e a extração
//! não tem retícula a que agarrar. Aqui os campos são resolvidos no nível mais
//! **GROSSO** — poucos vértices, célula grande em relação ao espaçamento, o
//! arredondamento MORDE — e prolongados para baixo.
//!
//! ⚠️ **A prolongação NÃO é uma cópia:**
//!
//! - a **direção** do pai é projetada no plano tangente do filho (ela era
//!   tangente a outro plano);
//! - a **origem** do pai é arredondada à retícula do filho, ancorada no vértice
//!   dele. É este arredondamento que faz o platô descer intacto: todos os filhos
//!   de um pai recebem o MESMO nó de retícula, e é isso que a Q3 procurava sem
//!   encontrar.

use ph2d_mesh::Mesh;

use crate::hierarchy::Hierarchy;
use crate::orientation::{self, OrientationField};
use crate::position::{self, PositionField};
use crate::scale::ScaleField;

/// Quantas varreduras por nível.
///
/// ⚠️ **É pequeno de propósito, e a hierarquia é a razão:** cada nível herda um
/// campo já quase certo do pai, então ele só precisa de se acomodar. As dezenas
/// de varreduras da Q1 eram o preço de partir da semente **em cada** ponto do
/// modelo. O número tem tabela ao lado no gate `the_hierarchy_beats_the_flat_solve`.
pub const SWEEPS_PER_LEVEL: usize = 8;

/// **RESOLVE os dois campos pela hierarquia.**
///
/// É a porta que a extração e o produto chamam; a
/// [`crate::orientation::solve_orientation`] e a
/// [`crate::position::solve_position`] ficam como o caminho **plano**, que os
/// gates usam para medir o que a hierarquia compra.
#[must_use]
pub fn solve_fields(mesh: &Mesh, scale: &ScaleField) -> (OrientationField, PositionField) {
    solve_fields_with(mesh, scale, SWEEPS_PER_LEVEL, crate::hierarchy::COARSEST)
}

/// A mesma lei, com os dois números abertos — a porta da sonda que os escolheu.
#[must_use]
pub fn solve_fields_with(
    mesh: &Mesh,
    scale: &ScaleField,
    sweeps: usize,
    coarsest: usize,
) -> (OrientationField, PositionField) {
    let h = Hierarchy::build_to(mesh, coarsest);

    // A escala de cada nível: a média dos filhos. ⚠️ Ela sobe com o campo porque
    // a retícula do nível grosso tem de ser a MESMA que a do fino — é a mesma
    // grade vista de longe, não uma grade mais grossa.
    let mut scales: Vec<Vec<f32>> = Vec::with_capacity(h.depth());
    scales.push((0..mesh.vert_count()).map(|v| scale.at(v)).collect());
    for l in 0..h.depth() - 1 {
        let parent = &h.level(l).parent;
        let up = h.level(l + 1).len();
        let (mut sum, mut count) = (vec![0.0f32; up], vec![0.0f32; up]);
        for (v, &p) in parent.iter().enumerate() {
            sum[p as usize] += scales[l][v];
            count[p as usize] += 1.0;
        }
        scales.push((0..up).map(|i| sum[i] / count[i].max(1.0)).collect());
    }

    let mut dirs: Vec<[f32; 3]> = Vec::new();
    let mut pos: Vec<[f32; 3]> = Vec::new();

    for l in h.coarse_to_fine() {
        let lv = h.level(l);
        let is_coarsest = l + 1 == h.depth();

        if is_coarsest {
            dirs = orientation::seed_dirs(&lv.normals);
            pos = lv.positions.clone();
        } else {
            // PROLONGAR do nível de cima.
            let parent = &lv.parent;
            let (up_dirs, up_pos) = (dirs, pos);
            dirs = (0..lv.len())
                .map(|v| orientation::project_tangent(up_dirs[parent[v] as usize], lv.normals[v]))
                .collect();
            pos = (0..lv.len())
                .map(|v| {
                    position::position_round_4(
                        up_pos[parent[v] as usize],
                        dirs[v],
                        lv.normals[v],
                        lv.positions[v],
                        scales[l][v],
                    )
                })
                .collect();
        }

        orientation::smooth_on(&mut dirs, &lv.normals, &lv.adjacency, sweeps);
        position::smooth_on(
            &mut pos,
            &lv.positions,
            &lv.normals,
            &dirs,
            &scales[l],
            &lv.adjacency,
            sweeps,
        );
    }

    (orientation::field_from(dirs), position::field_from(pos))
}

#[cfg(test)]
#[path = "solve_tests.rs"]
mod tests;
