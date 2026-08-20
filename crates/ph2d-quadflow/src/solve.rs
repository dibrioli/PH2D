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

/// Quantas varreduras por nível — **MEDIDO, não escolhido** (`CLAUDE.md` §0.0).
///
/// Sobre a malha que o módulo abre (`sculpt_sphere`, **98 306 vértices**), no
/// piso do slider (`3,0×` a aresta de entrada = quad de `0,0344`), pela sonda
/// `measure_the_sweeps_per_level`:
///
/// | varreduras | 1 | **2** | 4 | 8 | 16 |
/// |---|---|---|---|---|---|
/// | quads | 97,1 % | **97,6 %** | 97,5 % | 97,4 % | 97,5 % |
/// | maior ciclo | 7 | **5** | 5 | 5 | 6 |
/// | desvio de aresta | 0,073 | **0,067** | 0,065 | 0,064 | 0,065 |
/// | forma (Hausdorff em quads) | 0,030 | **0,020** | 0,019 | 0,021 | 0,018 |
/// | total (campos+extração+relax) | 0,50 s | **0,86 s** | 1,27 s | 2,54 s | 4,45 s |
///
/// ⚠️ **A qualidade SATURA na segunda varredura**, e a hierarquia é a razão: cada
/// nível herda do pai um campo já quase certo, e só precisa de se acomodar. A 2 a
/// fração de quads é o **máximo das cinco colunas**, e dezesseis varreduras
/// custam **5×** o relógio para não a melhorar.
///
/// ⚠️ **E a JUSTIFICATIVA ANTERIOR DESTE NÚMERO DISSOLVEU-SE, o que este doc
/// registra de propósito.** Ela dizia *"2 é o último degrau que cabe no
/// kill-criterion de 3 s"* — e era verdade enquanto a extração custava ~1,4 s
/// pela lei do cone. Com o grafo por passo de retícula ela passou a custar
/// **0,02 s**, o teto de relógio afastou-se, e o número ficou de pé por um motivo
/// que já não existia. *Quem move o número que tornava algo inalcançável tem de
/// reconferir a nota* (`CLAUDE.md` §0.0) — a re-medição manteve o 2, agora por
/// **qualidade** e não por relógio.
pub const SWEEPS_PER_LEVEL: usize = 2;

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
