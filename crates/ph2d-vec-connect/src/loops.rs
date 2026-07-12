//! **O laço** (`source == target`) — a forma que aponta para si mesma.
//!
//! Ele **não passa pelo roteador**: não há o que buscar. Sai por um lado, dá a volta, e entra
//! de volta pelo mesmo lado. Seis pontos, determinístico, sem A\*. É o que o mxGraph faz (o
//! `isLoopStyleEnabled` intercepta antes de qualquer edge style) — e por um bom motivo: um
//! roteador cuja origem e destino são o mesmo ponto, com a mesma direção de saída, não tem
//! problema para resolver.

use crate::{Aabb, Dir};

/// O quanto o laço se afasta da forma, em múltiplos do jetty.
const LOOP_OUT: f64 = 2.0;
/// A largura do laço ao longo da face, em múltiplos do jetty.
const LOOP_SPAN: f64 = 3.0;

/// O laço: sai da face `dir`, avança, corre paralelo a ela, e volta.
///
/// `spread` abre laços múltiplos em leque — dois conectores que voltam para a mesma forma não
/// podem se sobrepor.
#[must_use]
pub(crate) fn self_loop(bbox: Aabb, dir: Dir, jetty: f64, spread: f64) -> Vec<[f64; 2]> {
    let j = jetty.max(1e-9);
    let out = j * LOOP_OUT + spread.abs();
    let span = j * LOOP_SPAN;
    let c = bbox.center();

    // A face por onde sai, e os dois pontos de ancoragem nela (separados por `span`).
    let (a, b, n) = match dir {
        Dir::East => (
            [bbox.max[0], c[1] + span * 0.5],
            [bbox.max[0], c[1] - span * 0.5],
            [1.0, 0.0],
        ),
        Dir::West => (
            [bbox.min[0], c[1] + span * 0.5],
            [bbox.min[0], c[1] - span * 0.5],
            [-1.0, 0.0],
        ),
        Dir::North => (
            [c[0] - span * 0.5, bbox.max[1]],
            [c[0] + span * 0.5, bbox.max[1]],
            [0.0, 1.0],
        ),
        Dir::South => (
            [c[0] - span * 0.5, bbox.min[1]],
            [c[0] + span * 0.5, bbox.min[1]],
            [0.0, -1.0],
        ),
    };
    let push = |p: [f64; 2]| [p[0] + n[0] * out, p[1] + n[1] * out];
    vec![a, push(a), push(b), b]
}
