//! Gates do grafo de arestas.
//!
//! ⚠️ **As fixtures ABERTAS são as que importam.** Numa esfera fechada toda
//! aresta tem valência 2, então um bug que confunda borda com interior fica
//! verde — foi exatamente assim que a classe inteira da borda ficou invisível
//! até a W6.0 (`open_tube3`, `pillow`).

use super::*;
use crate::{Mesh, shapes, shapes_open};

/// A contagem de arestas conferida contra **Euler**, que não conhece este
/// código: numa malha fechada de triângulos `V − E + F = 2`.
#[test]
fn a_closed_triangle_mesh_obeys_eulers_formula() {
    let mesh = shapes::uv_sphere(16, 24, 1.0);
    let e = mesh.edges();
    let (v, f) = (mesh.vert_count() as i64, mesh.face_count() as i64);
    // A esfera UV tem polos em LEQUE de triângulos e o resto em quads, então a
    // fórmula vale sobre a contagem de FACES, não de triângulos.
    assert_eq!(v - e.len() as i64 + f, 2, "V={v} E={} F={f}", e.len());
}

/// **Numa malha FECHADA toda aresta tem valência 2** — e é o controle que dá
/// sentido ao gate seguinte.
#[test]
fn every_edge_of_a_closed_mesh_is_shared_by_exactly_two_faces() {
    let mesh = shapes::uv_sphere(12, 18, 1.0);
    let e = mesh.edges();
    for i in 0..e.len() as u32 {
        assert_eq!(e.valence(i), 2, "aresta {i}");
    }
}

/// **Numa malha ABERTA a beira tem valência 1**, e o número de arestas de borda
/// é o número de vértices de borda: a borda é uma curva FECHADA.
#[test]
fn the_lip_of_an_open_mesh_has_edges_of_valence_one() {
    let mesh = shapes_open::open_tube3();
    let e = mesh.edges();
    let border_edges = (0..e.len() as u32).filter(|&i| e.valence(i) == 1).count();
    let border_verts = (0..mesh.vert_count())
        .filter(|&v| mesh.adjacency().is_border(v))
        .count();
    assert!(border_verts > 0, "a fixture tem de ser ABERTA");
    assert_eq!(
        border_edges, border_verts,
        "a borda é um par de laços fechados: tantas arestas quantos vértices"
    );
    // E nenhuma aresta com mais de duas faces: o tubo é manifold.
    assert!((0..e.len() as u32).all(|i| e.valence(i) <= 2));
}

/// **As duas faces de uma aresta interior nomeiam o MESMO índice**, que é a
/// razão inteira de este grafo existir: sem isso, subdividir poria dois
/// vértices onde deve haver um, e a malha rasgaria ao longo de toda emenda.
#[test]
fn the_two_faces_of_an_edge_name_the_same_index() {
    let mesh = shapes::uv_sphere(10, 14, 1.0);
    let e = mesh.edges();
    // Para cada aresta, quantas vezes ela aparece nos cantos das faces.
    let mut seen = vec![0u32; e.len()];
    for (f, face) in mesh.faces().iter().enumerate() {
        for k in 0..face.verts().len() {
            let id = e.face_edge(f, k).expect("todo canto tem aresta");
            seen[id as usize] += 1;
        }
    }
    for (i, &count) in seen.iter().enumerate() {
        assert_eq!(
            count,
            e.valence(i as u32),
            "a aresta {i} foi nomeada {count} vezes e tem valência {}",
            e.valence(i as u32)
        );
    }
}

/// O slot 3 de um triângulo devolve `None` — quem itera pelos cantos não pode
/// ler a sentinela como se fosse um índice.
#[test]
fn a_triangle_has_no_fourth_edge() {
    let mesh = shapes::uv_sphere(8, 10, 1.0);
    let e = mesh.edges();
    for (f, face) in mesh.faces().iter().enumerate() {
        assert_eq!(e.face_edge(f, 3).is_none(), face.is_tri(), "face {f}");
    }
}

/// **A numeração é função da MALHA, não da corrida** — construir duas vezes dá
/// exatamente os mesmos ids. É o que torna seguro guardá-los entre chamadas.
#[test]
fn building_twice_gives_the_same_numbering() {
    let mesh = shapes::uv_sphere(9, 13, 1.0);
    let a = Edges::build(mesh.faces(), mesh.adjacency());
    let b = Edges::build(mesh.faces(), mesh.adjacency());
    assert_eq!(a, b);
}

/// Um par que não é aresta não recebe um índice inventado.
#[test]
fn a_pair_that_is_not_an_edge_has_no_index() {
    let mesh: Mesh = shapes::uv_sphere(8, 12, 1.0);
    let e = mesh.edges();
    // O polo norte e o polo sul não compartilham aresta em esfera nenhuma.
    let north = 0u32;
    let south = (mesh.vert_count() - 1) as u32;
    assert!(e.id_of(mesh.adjacency(), north, south).is_none());
}

/// **Um quad nomeia quatro arestas DISTINTAS.** Sem isto, subdividir um quad
/// poria dois de seus quatro vértices novos no mesmo índice e a face dobraria
/// sobre si mesma.
///
/// ⚠️ A fixture é a esfera UV (polos em leque de triângulos, miolo em quads), e
/// **a primeira que escolhi foi o `pillow` — que são dois TRIÂNGULOS**. Quem
/// pegou foi a própria asserção de premissa (`quads > 0`): sem ela o laço teria
/// zero iterações e o gate passaria dizendo nada.
#[test]
fn a_quad_names_four_distinct_edges() {
    let mesh = shapes::uv_sphere(10, 16, 1.0);
    let e = mesh.edges();
    let mut quads = 0;
    for (f, face) in mesh.faces().iter().enumerate() {
        if face.is_tri() {
            continue;
        }
        quads += 1;
        let ids: Vec<u32> = (0..4).map(|k| e.face_edge(f, k).expect("quad")).collect();
        for i in 0..4 {
            for j in i + 1..4 {
                assert_ne!(ids[i], ids[j], "face {f}: cantos {i} e {j}");
            }
        }
    }
    assert!(quads > 0, "a fixture tem de ter quads");
}
