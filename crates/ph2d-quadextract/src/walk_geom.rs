//! ⭐ **A GEOMETRIA EXACTA DO TRIÂNGULO** que o passeio interroga — de que lado está o
//! ponto, o triângulo está dobrado, o alvo está dentro, por que aresta se sai.
//!
//! ⚠️ **Separado do [`super::walk`] por tecto de LOC, e o corte é o natural:** aqui não há
//! travessia nenhuma, só predicados sobre três pontos — e todos exactos, sem epsilon.

use crate::exact::{P, orient};
use crate::ingest::Topo;

/// O sinal da área da imagem de uma face.
/// ⭐ **Sobre QUAL lado do triângulo o ponto cai** — `None` se está no interior.
///
/// ⚠️ O índice do lado é o do laço da travessia (`uv[k] → uv[(k+1) % 3]`), e é ele que
/// indexa [`Topo::twin`] e [`Topo::xf`]. *Uma convenção de lado escrita duas vezes é duas
/// convenções.*
pub(super) fn on_edge_side(tri: [P; 3], t: P) -> Option<usize> {
    let [a, b, c] = tri;
    if crate::exact::orient(a, b, t) == 0 {
        Some(0)
    } else if crate::exact::orient(b, c, t) == 0 {
        Some(1)
    } else if crate::exact::orient(c, a, t) == 0 {
        Some(2)
    } else {
        None
    }
}

pub(crate) fn face_sign(topo: &Topo, f: usize) -> i8 {
    let [a, b, c] = topo.uv[f];
    orient(a, b, c)
}

/// O ponto está **dentro ou sobre** o triângulo-imagem?
pub(super) fn contains(topo: &Topo, f: usize, q: P) -> bool {
    let [a, b, c] = topo.uv[f];
    let s = orient(a, b, c);
    if s == 0 {
        return false;
    }
    let e = [orient(a, b, q), orient(b, c, q), orient(c, a, q)];
    e.iter().all(|&x| x == 0 || x == s)
}

/// ⭐ **A ARESTA POR ONDE O SEGMENTO SAI** — e o desempate que apaga os casos
/// especiais.
pub(super) fn exit_side(topo: &Topo, f: usize, entry: Option<usize>, o: P, t: P) -> Option<usize> {
    let mut best: Option<(usize, u8)> = None;
    for k in 0..3usize {
        if entry == Some(k) {
            continue;
        }
        let a = topo.uv[f][k];
        let b = topo.uv[f][(k + 1) % 3];
        if !crosses(o, t, a, b) {
            continue;
        }
        let n = u8::from(on_segment(o, t, a)) + u8::from(on_segment(o, t, b));
        if best.is_none_or(|(_, c)| n < c) {
            best = Some((k, n));
        }
    }
    best.map(|(k, _)| k)
}

/// Os dois segmentos tocam-se ou cruzam-se? Fechado nos extremos, de propósito: um
/// segmento que passa por um vértice **atravessa** as duas arestas que ali se
/// encontram, e é o desempate que decide qual delas serve.
pub(super) fn crosses(o: P, t: P, a: P, b: P) -> bool {
    let (d1, d2) = (orient(o, t, a), orient(o, t, b));
    let (d3, d4) = (orient(a, b, o), orient(a, b, t));
    d1 * d2 <= 0 && d3 * d4 <= 0
}

/// O ponto está no segmento `[o, t]`, extremos incluídos?
pub(super) fn on_segment(o: P, t: P, q: P) -> bool {
    orient(o, t, q) == 0
        && q[0] >= o[0].min(t[0])
        && q[0] <= o[0].max(t[0])
        && q[1] >= o[1].min(t[1])
        && q[1] <= o[1].max(t[1])
}
