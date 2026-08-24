//! ⭐⭐⭐ **O CALIBRE — e por que a translação de UMA costura não quer dizer nada.**
//!
//! # ⛔⛔ A medição que não podia responder
//!
//! A extracção precisa que as translações das costuras sejam **inteiras**, senão as
//! isolinhas de `(u, v)` não casam dos dois lados. A primeira medição perguntou
//! directamente *«quão longe de inteiro estão elas?»* e respondeu `0,408` de mediana
//! com `0,498` de máximo — ou seja **uniformemente distribuídas**, com `0,5` a ser o
//! pior caso possível.
//!
//! ⚠️ **E essa medição não tinha como estar certa**, porque a grandeza é de **calibre**:
//! somar uma constante `o_p` ao `(u, v)` de um patch muda **todas** as translações que
//! lhe tocam, sem mudar coisa nenhuma na peça. *Perguntar se um número de calibre é
//! inteiro é perguntar sobre a escolha de quem o escreveu.*
//!
//! # ⭐ O que é invariante: a volta a um CICLO
//!
//! Numa aresta de costura, `z_b = R^k z_a + t`. Sob o calibre, `t ↦ t + o_b − R^k o_a`.
//! ⇒ **numa ÁRVORE de expansão do grafo de patches as translações podem ser todas
//! levadas a `0`** — escolhendo `o_b = R^k o_a − t` a partir da raiz.
//!
//! ⭐⭐⭐ O que sobra são as arestas que **fecham ciclo**, e **essas** são
//! invariantes: são a holonomia de translação da volta, e é delas — e só delas — que a
//! extracção precisa de inteiros.
//!
//! ⚠️ *É a mesma estrutura do salto de período do [`crate::comb`], um andar acima: lá o
//! que fecha é a rotação, aqui é a translação.*

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::solve::{GridMap, turn2};

/// O calibre fixado.
#[derive(Debug, Clone, Default)]
pub struct Gauge {
    /// Por patch, o deslocamento que leva as costuras de árvore a `0`.
    pub offset: Vec<[f32; 2]>,
    /// Por costura, `true` se ela está na árvore de expansão.
    pub in_tree: Vec<bool>,
    /// ⭐⭐⭐ Por costura que **fecha ciclo**, a translação já invariante.
    ///
    /// *São estas — e só estas — que a extracção precisa de ter inteiras.*
    pub cycle: Vec<(usize, [f32; 2])>,
}

/// O que o calibre mediu.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GaugeReport {
    /// Patches alcançados a partir da raiz.
    pub reached: usize,
    /// Patches ao todo.
    pub patches: usize,
    /// Costuras na árvore.
    pub tree: usize,
    /// ⭐ Costuras que fecham ciclo. ⚠️ **`E − V + componentes`**, e é o número de
    /// inteiros que a extracção tem de escolher.
    pub cycles: usize,
    /// ⛔ Costuras sem salto de período — não entram no calibre.
    pub loose: usize,
    /// ⭐⭐⭐ **A distância a inteiro das translações de CICLO**: mediana e pior.
    ///
    /// ⚠️ Ao contrário da distância medida sobre as translações cruas, **esta é
    /// invariante** — ver o doc deste módulo.
    pub frac_p50: f32,
    /// A pior distância a inteiro de uma translação de ciclo.
    pub frac_max: f32,
}

/// ⭐⭐⭐ **FIXA O CALIBRE** e devolve as translações de ciclo, já invariantes.
#[must_use]
pub fn fix(cut: &CutMesh, combed: &Combed, map: &GridMap) -> (Gauge, GaugeReport) {
    let np = cut.origin.len();
    let mut rep = GaugeReport {
        patches: np,
        ..GaugeReport::default()
    };
    // Adjacência de patches pelas costuras que têm salto.
    let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); np];
    for (s, seam) in cut.seams.iter().enumerate() {
        if combed.jump.get(s).copied().flatten().is_none() {
            rep.loose += 1;
            continue;
        }
        let (a, b) = (seam.side[0].patch as usize, seam.side[1].patch as usize);
        if a < np && b < np {
            adj[a].push((b, s));
            adj[b].push((a, s));
        }
    }

    let mut offset = vec![[0.0f32; 2]; np];
    let mut in_tree = vec![false; cut.seams.len()];
    let mut seen = vec![false; np];
    // ⚠️ **Semente por componente** — um grafo de patches partido tem uma árvore por
    // pedaço, e cada uma tem o seu calibre. *Uma só semente deixaria metade sem calibre
    // e as translações delas seriam lidas como enormes.*
    for root in 0..np {
        if seen[root] {
            continue;
        }
        seen[root] = true;
        let mut queue = std::collections::VecDeque::from([root]);
        rep.reached += 1;
        while let Some(a) = queue.pop_front() {
            for &(b, s) in &adj[a] {
                if seen[b] {
                    continue;
                }
                let Some(k) = combed.jump.get(s).copied().flatten() else {
                    continue;
                };
                let t = map.shift[s];
                let seam = &cut.seams[s];
                // `z_b = R^k z_a + t`. ⚠️ **O sentido importa:** se eu vim de `b` para
                // `a`, a relação lê-se ao contrário.
                let from_first = seam.side[0].patch as usize == a;
                offset[b] = if from_first {
                    // `o_b = R^k o_a − t`
                    let r = turn2(offset[a], k);
                    [r[0] - t[0], r[1] - t[1]]
                } else {
                    // `o_a = R^k o_b − t`  ⇒  `o_b = R^{−k}(o_a + t)`
                    turn2([offset[a][0] + t[0], offset[a][1] + t[1]], -k)
                };
                seen[b] = true;
                in_tree[s] = true;
                rep.tree += 1;
                queue.push_back(b);
            }
        }
    }

    // ── ⭐ As translações que sobram, já invariantes.
    let mut cycle: Vec<(usize, [f32; 2])> = Vec::new();
    let mut frac: Vec<f32> = Vec::new();
    for (s, seam) in cut.seams.iter().enumerate() {
        if in_tree[s] {
            continue;
        }
        let Some(k) = combed.jump.get(s).copied().flatten() else {
            continue;
        };
        let (a, b) = (seam.side[0].patch as usize, seam.side[1].patch as usize);
        if a >= np || b >= np {
            continue;
        }
        let t = map.shift[s];
        // `t' = t + o_b − R^k o_a`
        let r = turn2(offset[a], k);
        let inv = [t[0] + offset[b][0] - r[0], t[1] + offset[b][1] - r[1]];
        frac.push(
            (inv[0] - inv[0].round())
                .abs()
                .max((inv[1] - inv[1].round()).abs()),
        );
        cycle.push((s, inv));
    }
    rep.cycles = cycle.len();
    frac.sort_by(f32::total_cmp);
    if !frac.is_empty() {
        rep.frac_p50 = frac[frac.len() / 2];
        rep.frac_max = frac.last().copied().unwrap_or(0.0);
    }

    (
        Gauge {
            offset,
            in_tree,
            cycle,
        },
        rep,
    )
}

#[cfg(test)]
#[path = "gauge_tests.rs"]
mod tests;
