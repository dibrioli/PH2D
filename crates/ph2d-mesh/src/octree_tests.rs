//! Gates do octree frouxo.

use super::*;
use crate::mesh::Mesh;
use crate::shapes;

/// Toda face está em **exatamente uma** folha. O build particiona no lugar, e
/// uma partição que perde ou duplica um elemento é a falha que faz um dab
/// esquecer geometria — sem erro, sem aviso.
#[test]
fn the_leaves_are_a_partition_of_the_faces() {
    let mesh = shapes::uv_sphere(40, 40, 1.0);
    let tree = mesh.octree();
    let mut ids = tree.face_indices.clone();
    ids.sort_unstable();
    let expected: Vec<u32> = (0..mesh.face_count() as u32).collect();
    assert_eq!(ids, expected, "o octree perdeu ou duplicou faces");

    // E as faixas das folhas cobrem `face_indices` sem buraco nem sobreposição.
    let mut covered = vec![false; mesh.face_count()];
    for n in &tree.nodes {
        if n.first_child != u32::MAX {
            continue;
        }
        for k in n.start..n.start + n.len {
            assert!(!covered[k as usize], "a posição {k} está em duas folhas");
            covered[k as usize] = true;
        }
    }
    assert!(covered.iter().all(|c| *c), "sobrou posição sem folha");
}

/// A caixa frouxa de um nó contém de fato tudo que está sob ele. Este é o
/// invariante do qual a consulta depende: se a caixa mentir para menos, a
/// travessia poda uma sub-árvore que tinha resposta.
#[test]
fn every_loose_box_contains_the_geometry_beneath_it() {
    let mesh = shapes::uv_sphere(30, 30, 1.0);
    let tree = mesh.octree();
    for (ni, n) in tree.nodes.iter().enumerate() {
        if n.first_child != u32::MAX {
            continue;
        }
        for k in n.start..n.start + n.len {
            let fi = tree.face_indices[k as usize];
            for &v in mesh.faces()[fi as usize].verts() {
                let p = mesh.positions()[v as usize];
                for ((c, lo), hi) in p.iter().zip(&n.loose.min).zip(&n.loose.max) {
                    assert!(
                        *c >= lo - 1e-5 && *c <= hi + 1e-5,
                        "o nó {ni} guarda a face {fi}, cujo vértice sai da caixa frouxa dele"
                    );
                }
            }
        }
    }
}

/// Uma esfera que engole a malha devolve TODAS as faces; uma longe devolve
/// nenhuma. As duas pontas do mesmo invariante.
#[test]
fn a_query_that_swallows_the_mesh_returns_everything_and_a_far_one_returns_nothing() {
    let mesh = shapes::uv_sphere(20, 20, 1.0);
    let mut out = Vec::new();

    mesh.octree()
        .faces_in_sphere([0.0, 0.0, 0.0], 100.0, &mut out);
    out.sort_unstable();
    assert_eq!(out.len(), mesh.face_count());

    mesh.octree()
        .faces_in_sphere([50.0, 0.0, 0.0], 1.0, &mut out);
    assert!(out.is_empty(), "consulta longe achou {} faces", out.len());
}

/// ⚠️ **O gate que justifica a caixa FROUXA existir.** Uma face grande é
/// arquivada pelo CENTRO dela, então ela mora num nó cuja caixa de partição não
/// a contém inteira. Consultando perto da PONTA dela — longe do centro — só a
/// caixa frouxa acha; a de partição poda e a face some.
#[test]
fn a_big_face_is_found_from_its_far_corner_not_only_from_its_centre() {
    // Muitos triângulos miúdos forçam a subdivisão, e um triângulo enorme
    // atravessa vários nós.
    let mut positions = vec![[-10.0, 0.0, -10.0], [10.0, 0.0, -10.0], [0.0, 0.0, 10.0]];
    let mut faces = vec![Face::tri(0, 1, 2)];
    for i in 0..400 {
        let x = (i % 20) as f32 * 0.05;
        let z = (i / 20) as f32 * 0.05;
        let b = positions.len() as u32;
        positions.push([x, 1.0, z]);
        positions.push([x + 0.04, 1.0, z]);
        positions.push([x, 1.0, z + 0.04]);
        faces.push(Face::tri(b, b + 1, b + 2));
    }
    let mesh = Mesh::from_parts(positions, faces).unwrap();
    assert!(
        mesh.octree().node_count() > 1,
        "a fixture não subdividiu, então não testa nada"
    );

    let mut out = Vec::new();
    // A ponta do triângulo grande, a 10 unidades do centro dele.
    mesh.octree()
        .faces_in_sphere([9.5, 0.0, -9.5], 1.0, &mut out);
    assert!(
        out.contains(&0),
        "a face grande sumiu da consulta feita na ponta dela"
    );
}

/// O build é determinístico: a mesma malha dá a mesma árvore. Um octree que
/// depende de ordem de iteração de mapa faria a consulta — e portanto o dab —
/// mudar entre execuções.
#[test]
fn the_build_is_deterministic() {
    let mesh = shapes::uv_sphere(15, 17, 1.0);
    let a = Octree::build(mesh.positions(), mesh.faces());
    let b = Octree::build(mesh.positions(), mesh.faces());
    assert_eq!(a.node_count(), b.node_count());
    assert_eq!(a.face_indices, b.face_indices);
}

/// Malha vazia: nenhum nó, consulta vazia, sem pânico.
#[test]
fn an_empty_mesh_has_an_empty_tree() {
    let tree = Octree::build(&[], &[]);
    assert!(tree.is_empty());
    let mut out = vec![7, 8, 9];
    tree.faces_in_sphere([0.0; 3], 1.0, &mut out);
    assert!(out.is_empty(), "a consulta tem de LIMPAR a saída");
}
