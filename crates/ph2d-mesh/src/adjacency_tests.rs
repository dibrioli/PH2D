//! Gates da adjacência CSR.

use super::*;
use crate::shapes;

/// O invariante estrutural de um CSR **recém-construído**: as linhas ladrilham
/// `values` na ordem, sem folga e sem buraco. Se ele mentir, `neighbours` lê a
/// faixa de outro vértice — silenciosamente, com índices válidos.
///
/// ⚠️ **Ele vale no BUILD e deixa de valer depois de uma edição**, e é essa a
/// diferença que faz uma mudança de topologia caber num dab: uma linha que
/// cresce se muda para o fim, deixando um rastro. O que continua valendo sempre
/// é o gate irmão — *toda linha está dentro de `values` e nenhuma se sobrepõe a
/// outra* —, e é ele que protege a leitura.
fn assert_is_packed(c: &Csr, items: usize) {
    assert_eq!(c.starts.len(), items, "uma entrada de começo por item");
    assert_eq!(c.lens.len(), items, "e uma de comprimento");
    assert_eq!(c.dead, 0, "um CSR recém-construído não tem rastro");
    let mut at = 0u32;
    for i in 0..items {
        assert_eq!(c.starts[i], at, "a linha {i} não continua a anterior");
        at += c.lens[i];
    }
    assert_eq!(
        at as usize,
        c.values.len(),
        "a última fronteira tem de ser o fim de values"
    );
}

#[test]
fn the_freshly_built_csr_is_packed() {
    let mesh = shapes::uv_sphere(8, 12, 1.0);
    let adj = mesh.adjacency();
    assert_is_packed(&adj.vert_faces, mesh.vert_count());
    assert_is_packed(&adj.vert_verts, mesh.vert_count());
}

/// **O invariante que sobrevive à edição:** toda linha cabe em `values` e
/// nenhuma pisa na outra. É ele que torna a leitura segura depois de a
/// representação deixar de ser uma soma de prefixos.
fn assert_rows_are_disjoint(c: &Csr, items: usize) {
    let mut spans: Vec<(u32, u32)> = (0..items)
        .map(|i| (c.starts[i], c.starts[i] + c.lens[i]))
        .filter(|(s, e)| s != e)
        .collect();
    for &(_, e) in &spans {
        assert!(e as usize <= c.values.len(), "uma linha sai de values");
    }
    spans.sort_unstable();
    for w in spans.windows(2) {
        assert!(
            w[0].1 <= w[1].0,
            "duas linhas se sobrepõem: {:?} e {:?}",
            w[0],
            w[1]
        );
    }
}

#[test]
fn an_edited_ring_is_exactly_what_a_rebuild_would_have_produced() {
    let mut mesh = shapes::uv_sphere(8, 12, 1.0);
    mesh.triangulate();
    // Uma troca de diagonal num par vizinho qualquer — o que o flip faz.
    let (i0, i1) = (0usize, {
        let f0 = mesh.faces()[0].verts().to_vec();
        let adj = mesh.adjacency();
        *adj.vert_faces
            .neighbours(f0[0] as usize)
            .iter()
            .find(|&&j| j != 0 && mesh.faces()[j as usize].verts().contains(&f0[1]))
            .expect("a esfera tem vizinhas")
    } as usize);
    let (t0, t1) = (
        mesh.faces()[i0].verts().to_vec(),
        mesh.faces()[i1].verts().to_vec(),
    );
    let k = (0..3)
        .find(|&k| !t1.contains(&t0[k]))
        .expect("um canto de fora");
    let c = t0[k];
    let (a, b) = (t0[(k + 1) % 3], t0[(k + 2) % 3]);
    let d = *t1
        .iter()
        .find(|v| **v != a && **v != b)
        .expect("o quarto canto");

    let mut scratch = crate::RegionScratch::default();
    mesh.relink_faces(
        &[
            (i0 as u32, Face::tri(a, d, c)),
            (i1 as u32, Face::tri(d, b, c)),
        ],
        &mut scratch,
    );

    let rebuilt = Adjacency::build(mesh.vert_count(), mesh.faces());
    let edited = mesh.adjacency();
    for v in 0..mesh.vert_count() {
        assert_eq!(
            edited.vert_faces.neighbours(v),
            rebuilt.vert_faces.neighbours(v),
            "o anel de faces de {v} divergiu do que um rebuild daria"
        );
        assert_eq!(
            edited.vert_verts.neighbours(v),
            rebuilt.vert_verts.neighbours(v),
            "o anel de vértices de {v} divergiu do que um rebuild daria"
        );
    }
    assert_rows_are_disjoint(&edited.vert_faces, mesh.vert_count());
    assert_rows_are_disjoint(&edited.vert_verts, mesh.vert_count());
}

#[test]
fn every_face_is_in_the_ring_of_each_of_its_own_vertices() {
    let mesh = shapes::uv_sphere(6, 9, 1.0);
    let adj = mesh.adjacency();
    for (fi, f) in mesh.faces().iter().enumerate() {
        for &v in f.verts() {
            assert!(
                adj.vert_faces.neighbours(v as usize).contains(&(fi as u32)),
                "a face {fi} contém o vértice {v} mas não está no anel dele"
            );
        }
    }
    // E o inverso: nada aparece num anel sem estar na face.
    for v in 0..mesh.vert_count() {
        for &fi in adj.vert_faces.neighbours(v) {
            assert!(
                mesh.faces()[fi as usize].verts().contains(&(v as u32)),
                "o anel do vértice {v} lista a face {fi}, que não o contém"
            );
        }
    }
}

/// ⚠️ **O gate que separa a implementação certa da tentadora.** Num quad, os
/// vizinhos de um vértice são os dois do CICLO da face — nunca o da diagonal.
/// A versão ingênua ("acrescente todos os outros vértices da face") passa em
/// qualquer fixture de triângulos, porque num triângulo todo par é uma aresta,
/// e só falha aqui.
#[test]
fn a_quads_ring_is_the_cycle_never_the_diagonal() {
    let adj = Adjacency::build(4, &[Face::quad(0, 1, 2, 3)]);
    for (v, expected) in [(0, [1u32, 3]), (1, [0, 2]), (2, [1, 3]), (3, [0, 2])] {
        let mut got = adj.vert_verts.neighbours(v).to_vec();
        got.sort_unstable();
        let mut want = expected.to_vec();
        want.sort_unstable();
        assert_eq!(got, want, "vizinhos errados do vértice {v} do quad");
    }
}

#[test]
fn the_cube_has_three_faces_and_three_neighbours_at_every_corner() {
    let mesh = shapes::cube(2.0);
    let adj = mesh.adjacency();
    assert_eq!(mesh.vert_count(), 8);
    for v in 0..8 {
        assert_eq!(
            adj.vert_faces.neighbours(v).len(),
            3,
            "toda quina de um cubo pertence a 3 faces (vértice {v})"
        );
        assert_eq!(
            adj.vert_verts.neighbours(v).len(),
            3,
            "toda quina de um cubo tem 3 arestas (vértice {v})"
        );
    }
}

/// A relação é simétrica: se `a` vê `b`, `b` vê `a`. Uma quebra disto faz o
/// Smooth puxar um vértice para um vizinho que não o puxa de volta.
#[test]
fn the_vertex_ring_is_symmetric() {
    let mesh = shapes::uv_sphere(7, 11, 1.0);
    let adj = mesh.adjacency();
    for v in 0..mesh.vert_count() {
        for &n in adj.vert_verts.neighbours(v) {
            assert!(
                adj.vert_verts.neighbours(n as usize).contains(&(v as u32)),
                "{v} vê {n}, mas {n} não vê {v}"
            );
        }
    }
}

/// Ninguém é vizinho de si mesmo — um Smooth que lê o próprio vértice como
/// vizinho fica mais fraco conforme a valência, o que é invisível e errado.
#[test]
fn a_vertex_is_never_its_own_neighbour() {
    let mesh = shapes::uv_sphere(5, 8, 1.0);
    let adj = mesh.adjacency();
    for v in 0..mesh.vert_count() {
        assert!(!adj.vert_verts.neighbours(v).contains(&(v as u32)));
    }
}

/// Um vértice sem face EXISTE e tem anel vazio. Se ele sumisse, todo índice
/// depois dele deslocaria — e um OBJ com um vértice solto é comum.
#[test]
fn a_loose_vertex_survives_with_an_empty_ring() {
    let adj = Adjacency::build(5, &[Face::tri(0, 1, 2)]);
    assert_eq!(adj.vert_faces.len(), 5);
    assert!(adj.vert_faces.neighbours(3).is_empty());
    assert!(adj.vert_verts.neighbours(4).is_empty());
    // E fora de alcance é vazio, não pânico.
    assert!(adj.vert_faces.neighbours(99).is_empty());
}

/// Sem duplicatas: um vértice que aparece em N faces com o mesmo vizinho lista
/// esse vizinho UMA vez. É o carimbo que garante isso.
#[test]
fn the_ring_has_no_duplicates() {
    let mesh = shapes::uv_sphere(6, 10, 1.0);
    let adj = mesh.adjacency();
    for v in 0..mesh.vert_count() {
        let mut seen = adj.vert_verts.neighbours(v).to_vec();
        let before = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(before, seen.len(), "o anel do vértice {v} tem repetido");
    }
}
