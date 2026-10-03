//! ⭐ **O CUSTO de um caminho sobre uma malha com ÁREAS** (W7, plano 30 §2.5): `∑ custo da área ×
//! comprimento` de cada pedaço. A régua de toda a medição da W7 e a conta do refinamento do caminho.
//!
//! # A tabela
//!
//! `costs[a]` é o multiplicador da área `a` (o chão comum é `costs[0]`, por norma `1`). Uma área
//! além da tabela custa `1`. ⛔ Uma área PROIBIDA não é um custo infinito aqui: é um FURO na malha
//! (a construção tira-a), porque as ilhas, os cantos, o ponto alcançável mais perto e as paredes do
//! desvio leem a malha — e com um custo infinito todos eles mentiriam.
//!
//! # A fronteira
//!
//! Um pedaço que corre EM CIMA de uma aresta entre duas áreas está no fecho das duas: cobra-se a mais
//! barata (o caminho pode encostar-se à área barata tanto quanto quiser).

use crate::geom::{EPS, V2, dist, lerp};
use crate::mesh::NavMesh;
use crate::polyanya::clip;

/// O multiplicador da área `a` (`1` para uma área fora da tabela).
#[inline]
pub fn cost_of(costs: &[f64], a: u16) -> f64 {
    costs.get(a as usize).copied().unwrap_or(1.0)
}

/// O pedaço `[t0, t1]` do segmento `a → b` que está dentro do polígono `poly` (fechado, à tolerância
/// [`EPS`] em metros), ou `None`.
fn inside_range(mesh: &NavMesh, poly: u32, a: V2, b: V2) -> Option<(f64, f64)> {
    let pv = &mesh.polys()[poly as usize].verts;
    let n = pv.len();
    let (mut lo, mut hi) = (0.0, 1.0);
    for i in 0..n {
        let u = mesh.vert(pv[i]);
        let w = mesh.vert(pv[(i + 1) % n]);
        clip(
            &mut lo,
            &mut hi,
            crate::geom::side_dist(u, w, a),
            crate::geom::side_dist(u, w, b),
        );
        if hi < lo {
            return None;
        }
    }
    Some((lo, hi))
}

/// O custo do segmento `a → b` andado DENTRO da malha, ou `None` se ele sai dela (cruza uma parede).
/// Os dois extremos têm de estar na malha.
pub fn segment_cost(mesh: &NavMesh, costs: &[f64], a: V2, b: V2) -> Option<f64> {
    let l = dist(a, b);
    let mut here = Vec::new();
    if l <= EPS {
        mesh.locate_all(a, &mut here);
        return (!here.is_empty()).then_some(0.0);
    }
    let tol = EPS / l;
    let mut t = 0.0;
    let mut total = 0.0;
    // O tecto do laço: cada passo avança para outro polígono (uma malha partida não prende aqui).
    for _ in 0..=2 * mesh.polys().len() {
        if t >= 1.0 - tol {
            return Some(total);
        }
        mesh.locate_all(lerp(a, b, t), &mut here);
        // Entre os polígonos que contêm o ponto de agora, os que levam o segmento para a frente: o
        // mais BARATO (na fronteira entre duas áreas, o pedaço está no fecho das duas).
        let mut best: Option<(f64, f64)> = None;
        for &q in &here {
            let Some((t0, t1)) = inside_range(mesh, q, a, b) else {
                continue;
            };
            if t0 > t + tol || t1 <= t + tol {
                continue;
            }
            let c = cost_of(costs, mesh.area_id(q));
            if best.is_none_or(|(bc, bt)| c < bc || (c == bc && t1 > bt)) {
                best = Some((c, t1));
            }
        }
        let (c, t1) = best?;
        total += c * (t1.min(1.0) - t) * l;
        t = t1;
    }
    None
}

/// O custo de uma linha poligonal (a soma dos troços), ou `None` se algum troço sai da malha.
pub fn path_cost(mesh: &NavMesh, costs: &[f64], pts: &[V2]) -> Option<f64> {
    pts.windows(2)
        .map(|w| segment_cost(mesh, costs, w[0], w[1]))
        .sum()
}
