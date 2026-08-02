//! Os gates de fechar buraco — ver o cabeçalho do `holes.rs`.

use super::*;
use crate::{shapes, shapes_open};

/// **A malha é FECHADA e ORIENTÁVEL?** — as duas metades, e a segunda é a que
/// pega o defeito que este módulo pode cometer.
///
/// ⚠️ Fechada é contagem (toda aresta tem duas faces) e **orientável é
/// direção**: duas faces vizinhas percorrem a aresta partilhada em sentidos
/// OPOSTOS. Um remendo com o winding trocado fecha a malha e passa na primeira
/// metade — a tampa só fica com a normal para dentro, o que nenhuma contagem vê.
fn assert_closed_and_orientable(mesh: &Mesh) {
    let edges = mesh.edges();
    for e in 0..edges.len() {
        assert_eq!(
            edges.valence(u32::try_from(e).expect("cabe")),
            2,
            "a aresta {e} não tem duas faces: a malha não fechou"
        );
    }
    // Cada aresta não-dirigida tem de aparecer uma vez em cada sentido.
    let mut seen: Vec<(bool, bool)> = vec![(false, false); edges.len()];
    for (f, face) in mesh.faces().iter().enumerate() {
        let v = face.verts();
        for k in 0..v.len() {
            let e = edges.face_edge(f, k).expect("todo canto tem aresta") as usize;
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let slot = if a < b {
                &mut seen[e].0
            } else {
                &mut seen[e].1
            };
            assert!(
                !*slot,
                "a aresta {e} é percorrida DUAS vezes no mesmo sentido"
            );
            *slot = true;
        }
    }
    assert!(
        seen.iter().all(|&(lo, hi)| lo && hi),
        "há aresta percorrida nos dois lados no mesmo sentido: o remendo está invertido"
    );
}

#[test]
fn an_open_tube_gets_a_cap_on_each_end() {
    let mut mesh = shapes_open::open_tube3();
    let before_v = mesh.vert_count();
    let before_f = mesh.face_count();
    let report = fill_holes(&mut mesh);
    assert_eq!(report.filled(), 2, "o tubo tem duas bocas");
    assert_eq!(report.left_open(), 0);
    // Um vértice por tampa, seis triângulos por tampa.
    assert_eq!(mesh.vert_count(), before_v + 2);
    assert_eq!(mesh.face_count(), before_f + 12);
    assert_closed_and_orientable(&mesh);
}

/// ⚠️ **O gate que separa *fechou* de *fechou com a tampa virada*.** Ele é o
/// único que a inversão do winding derruba — a contagem de faces, a de vértices
/// e a valência das arestas ficam todas certas com o remendo de cabeça para
/// baixo.
#[test]
fn the_patch_faces_the_same_way_as_the_surface_it_closes() {
    let mut mesh = shapes_open::open_tube3();
    fill_holes(&mut mesh);
    assert_closed_and_orientable(&mesh);
    // E a normal da tampa aponta para FORA do tubo: o eixo é o Y, e as duas
    // tampas novas são os dois últimos vértices.
    let n = mesh.vert_count();
    let top = mesh.positions()[n - 1][1].max(mesh.positions()[n - 2][1]);
    let bottom = mesh.positions()[n - 1][1].min(mesh.positions()[n - 2][1]);
    assert!(
        top > 0.9 && bottom < -0.9,
        "as tampas ficam nas duas pontas"
    );
    for i in [n - 1, n - 2] {
        let p = mesh.positions()[i];
        let nn = mesh.normals()[i];
        assert!(
            p[1] * nn[1] > 0.0,
            "a normal da tampa em y={} aponta para fora (n={nn:?})",
            p[1]
        );
    }
}

#[test]
fn a_mesh_that_is_already_closed_is_left_alone() {
    let mut mesh = shapes::cube(1.0);
    let before = mesh.clone();
    let report = fill_holes(&mut mesh);
    assert!(report.is_noop());
    assert_eq!(report.filled(), 0);
    assert_eq!(mesh.vert_count(), before.vert_count());
    assert_eq!(mesh.faces(), before.faces());
}

/// ⚠️ **Um contorno de TRÊS arestas vira UM triângulo, sem vértice novo** — a
/// divergência declarada. O leque ali poria um vértice de valência 3 no meio do
/// que já era um triângulo.
#[test]
fn a_three_edge_hole_becomes_one_triangle_instead_of_a_fan() {
    // Um tetraedro sem uma face: quatro vértices, três faces, um furo triangular.
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let faces = vec![Face::tri(0, 2, 1), Face::tri(0, 1, 3), Face::tri(0, 3, 2)];
    let mut mesh = Mesh::from_parts(positions, faces).expect("o tetraedro aberto é válido");
    let report = fill_holes(&mut mesh);
    assert_eq!(report.filled(), 1);
    assert_eq!(
        mesh.vert_count(),
        4,
        "nenhum vértice novo num furo triangular"
    );
    assert_eq!(mesh.face_count(), 4);
    assert_closed_and_orientable(&mesh);
}

#[test]
fn the_new_vertex_carries_the_average_of_the_channels_around_the_hole() {
    let mut mesh = shapes_open::open_tube3();
    for (i, c) in mesh.colors_mut().iter_mut().enumerate() {
        *c = [if i < 6 { 1.0 } else { 0.0 }, 0.0, 0.0];
    }
    for m in mesh.masks_mut() {
        *m = 0.25;
    }
    fill_holes(&mut mesh);
    let n = mesh.vert_count();
    // A boca de baixo é feita dos seis primeiros vértices, todos vermelhos.
    let reds: Vec<f32> = [n - 1, n - 2]
        .iter()
        .map(|&i| mesh.colors().expect("cor")[i][0])
        .collect();
    assert!(
        reds.contains(&1.0) && reds.contains(&0.0),
        "cada tampa herda a cor da SUA boca (leu {reds:?})"
    );
    for i in [n - 1, n - 2] {
        assert!((mesh.masks().expect("máscara")[i] - 0.25).abs() < 1e-6);
    }
}

/// ⚠️ **A malha sem canal NÃO ganha um** — a mesma lei do `colors_mut`, e a
/// razão pela qual a cópia é condicional.
#[test]
fn a_mesh_without_channels_does_not_grow_them_when_it_is_closed() {
    let mut mesh = shapes_open::open_tube3();
    assert!(mesh.colors().is_none() && mesh.masks().is_none());
    fill_holes(&mut mesh);
    assert!(mesh.colors().is_none());
    assert!(mesh.masks().is_none());
}

/// ⚠️ **Fechar só ACRESCENTA**, e é isso que faz o desfazer ser um truncar em
/// vez de uma cópia do documento. O gate afirma a propriedade, não o atalho: se
/// um dia o remendo mexer num vértice que já existia, é esta linha que sangra —
/// e o desfazer por truncar deixa de ser correto no mesmo instante.
#[test]
fn filling_only_appends_so_undoing_it_is_a_truncation() {
    let mut mesh = shapes_open::open_tube3();
    let before_pos = mesh.positions().to_vec();
    let before_faces = mesh.faces().to_vec();
    let report = fill_holes(&mut mesh);
    assert_eq!(mesh.positions()[..before_pos.len()], before_pos[..]);
    assert_eq!(mesh.faces()[..before_faces.len()], before_faces[..]);

    mesh.truncate(report.verts_before(), report.faces_before())
        .expect("truncar de volta");
    assert_eq!(mesh.positions(), &before_pos[..]);
    assert_eq!(mesh.faces(), &before_faces[..]);
}

#[test]
fn truncating_below_what_the_surviving_faces_reference_is_refused() {
    let mut mesh = shapes::cube(1.0);
    let before = mesh.vert_count();
    assert!(
        mesh.truncate(4, mesh.face_count()).is_err(),
        "faces que citam o vértice 7 não sobrevivem a um corte em 4"
    );
    assert_eq!(mesh.vert_count(), before, "e a recusa não mexeu em nada");
    assert!(mesh.truncate(before + 1, 0).is_err(), "nem cresce");
}

/// Fechar é IDEMPOTENTE: a segunda chamada não acha buraco nenhum.
#[test]
fn filling_a_second_time_finds_nothing_to_fill() {
    let mut mesh = shapes_open::open_tube3();
    assert_eq!(fill_holes(&mut mesh).filled(), 2);
    let after = mesh.face_count();
    let second = fill_holes(&mut mesh);
    assert!(second.is_noop());
    assert_eq!(mesh.face_count(), after);
}
