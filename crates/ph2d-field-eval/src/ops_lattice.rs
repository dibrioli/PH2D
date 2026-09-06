//! ⭐⭐⭐ **O GYROID** (W124) — uma linha de fórmula que dá um enchimento infinito.
//!
//! # ⭐ Por que ela cabe aqui, e não cabia antes da W123
//!
//! A superfície mínima tripla-periódica de Schoen é `sin x·cos y + sin y·cos z + sin z·cos x = 0`.
//! ⛔ A distância a ela **não tem forma fechada** — e desde a W123 esta casa sabe que o módulo
//! nunca precisou dela: precisa de um **minorante**, e um implícito dá-o por
//! `|g| / max‖∇g‖ ≤ dist`.
//!
//! # ⚠️ O divisor foi MEDIDO, e é METADE do limite grosso
//!
//! A soma dos três termos tem `‖∇g‖ ≤ 2√3 = 3,4641` por desigualdade triangular ingénua. Varrida a
//! célula inteira (`140³`), a medição dá **`1,7315`** — isto é, `√3`, exactamente. E **sobre a
//! superfície** o gradiente vive entre `1,4144` e `1,7315`, logo o campo fica no pior sítio apenas
//! **`1,22×`** mais fraco que uma distância verdadeira. *Escrever `2√3` teria custado o dobro dos
//! passos de marcha por uma desigualdade que ninguém mediu.*
//!
//! # ⚠️ E ele é uma PEÇA, não um espaço
//!
//! O gyroid é infinito por construção; o que entra no documento é ele **cortado por uma caixa**.
//! É a caixa que traz as arestas — e é nelas que o filete e o chanfro agem.

use fidget::context::Tree;

use crate::ops_joint::Edge;

/// ⭐ **GYROID** — a parede de meia-espessura `thickness` em volta da superfície de Schoen, com
/// célula `cell`, recortada pela caixa de meias-extensões `half`.
pub fn sd_gyroid(half: [f64; 3], cell: f64, thickness: f64, round: f64, chamfer: f64) -> Tree {
    let e = Edge::square(round, chamfer);
    let k = std::f64::consts::TAU / cell.max(f64::MIN_POSITIVE);
    let q = |t: Tree| t * Tree::constant(k);
    let (sx, cx) = (q(Tree::x()).sin(), q(Tree::x()).cos());
    let (sy, cy) = (q(Tree::y()).sin(), q(Tree::y()).cos());
    let (sz, cz) = (q(Tree::z()).sin(), q(Tree::z()).cos());
    let g = sx * cy + sy * cz + sz * cx;
    // ⭐ `√3` MEDIDO — ver o cabeçalho.
    let parede = g.abs() / Tree::constant(k * 3.0f64.sqrt()) - Tree::constant(thickness);
    // ⛔⛔⛔ **A CAIXA ENTRA EM PEÇAS, e não composta** — ver [`crate::ops_box::box_pieces`], que
    // existe exactamente para isto. Com a caixa já composta, a sonda de arestas achou `1058` pontos
    // por cortar e **todos nos OITO CANTOS** (`±0,400` nos três eixos, a `70,9°`): as quinas dela
    // são um `max` dentro da fórmula da caixa, e nenhuma junta por fora lhes chega.
    let (mut corpo, mut arestas) =
        crate::ops_box::box_pieces(&Tree::x(), &Tree::y(), &Tree::z(), half, round, chamfer);
    // A parede é a primeira peça, e forma aresta com cada face da caixa.
    for face in &corpo {
        arestas.push((parede.clone(), face.clone()));
    }
    corpo.insert(0, parede);
    if chamfer <= 0.0 {
        // ⚠️ **Sem chanfro, a mistura CRUA** — a lei que a nuvem pagou: os planos de corte de uma
        // junta n-ária com recuo zero passam pela própria aresta, não cortam nada e ainda contam no
        // tecto `√(activas)`.
        return crate::ops::intersection_round_n(&corpo, round);
    }
    crate::ops_joint::intersection_joint_n(&corpo, &arestas, e)
}
