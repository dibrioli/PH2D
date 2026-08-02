//! Gates das primitivas de **blocagem** — cilindro e toro.
//!
//! O oráculo é ESTRUTURAL (contagens, fechamento, valência, a caixa), nunca um
//! literal de ponto flutuante: estas funções chamam `sin`/`cos` da `std`, que
//! não são pinadas bit a bit entre sistemas operacionais — o cabeçalho do
//! [`super`] já diz isso, e este arquivo é quem obedece.

use super::*;
use crate::Aabb;

/// Uma malha é FECHADA quando toda aresta é compartilhada por exatamente duas
/// faces. É a pergunta que separa um sólido de uma casca — e um sólido é o que
/// uma primitiva de blocagem tem de ser, senão o remesh (que precisa de dentro e
/// fora) a recusa em silêncio.
fn is_closed(mesh: &Mesh) -> bool {
    let adj = crate::Adjacency::build(mesh.vert_count(), mesh.faces());
    let edges = crate::Edges::build(mesh.faces(), &adj);
    (0..edges.len()).all(|e| edges.valence(e as u32) == 2)
}

#[test]
fn a_cylinder_is_a_closed_solid_with_quads_around_and_fans_on_the_caps() {
    let seg = 12;
    let m = cylinder(seg, 1.0, 2.0);
    assert_eq!(m.vert_count(), 2 + 2 * seg, "dois polos e dois anéis");
    assert_eq!(m.face_count(), 3 * seg, "por fatia: duas tampas e um quad");
    let quads = (0..m.face_count())
        .filter(|&i| !m.faces()[i].is_tri())
        .count();
    assert_eq!(quads, seg, "o CORPO é quad puro — é onde se esculpe");
    assert!(is_closed(&m), "uma primitiva de blocagem é um SÓLIDO");
}

/// ⚠️ **O toro não tem polo nenhum**, e é a propriedade que o torna a fixture de
/// topologia regular: todo vértice tem valência 4, então nada no alisamento
/// precisa contornar uma estrela.
#[test]
fn a_torus_is_all_quads_and_every_vertex_has_valence_four() {
    let (mj, mn) = (16, 8);
    let m = torus(mj, mn, 1.0, 0.35);
    assert_eq!(m.vert_count(), mj * mn);
    assert_eq!(m.face_count(), mj * mn);
    assert!(
        (0..m.face_count()).all(|i| !m.faces()[i].is_tri()),
        "quad puro"
    );
    assert!(is_closed(&m), "e fechado");

    let adj = crate::Adjacency::build(m.vert_count(), m.faces());
    for v in 0..m.vert_count() {
        assert_eq!(adj.valence(v), 4, "o vértice {v} tem de ter valência 4");
    }
}

/// As duas cabem na caixa que os parâmetros prometem. ⚠️ Com folga para BAIXO
/// de propósito: um polígono inscrito não alcança o círculo (a razão é
/// `cos(π/n)`), então exigir a igualdade seria pedir que a malha fosse a forma
/// ideal — e reprovaria a primitiva certa.
#[test]
fn the_primitives_fit_the_box_their_parameters_promise() {
    let cyl = cylinder(24, 1.0, 3.0);
    let b = Aabb::from_points(cyl.positions());
    assert!(
        (b.max[1] - 1.5).abs() < 1e-5 && (b.min[1] + 1.5).abs() < 1e-5,
        "a altura é exata"
    );
    assert!(
        b.max[0] <= 1.0 + 1e-5 && b.max[0] > 0.99,
        "o raio, inscrito"
    );

    let t = torus(24, 12, 2.0, 0.5);
    let b = Aabb::from_points(t.positions());
    assert!(b.max[0] <= 2.5 + 1e-5 && b.max[0] > 2.45, "maior + menor");
    assert!(
        (b.max[1] - 0.5).abs() < 1e-5,
        "a espessura é o raio MENOR, exata: {}",
        b.max[1]
    );
}

/// Um `segments` degenerado é CLAMPADO, não recusado. ⚠️ Recusar devolveria um
/// `Result` a um chamador que é um GESTO do artista, e o gesto não tem o que
/// fazer com um erro; três é o menor número de fatias que ainda fecha um sólido.
#[test]
fn a_degenerate_segment_count_is_floored_into_a_solid() {
    for m in [cylinder(0, 1.0, 1.0), cylinder(2, 1.0, 1.0)] {
        assert_eq!(m.vert_count(), 2 + 2 * 3);
        assert!(is_closed(&m));
    }
    let t = torus(1, 1, 1.0, 0.3);
    assert_eq!(t.vert_count(), 9);
    assert!(is_closed(&t));
}
