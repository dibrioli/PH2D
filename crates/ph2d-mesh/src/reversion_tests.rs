//! Os gates da reversão — ver o cabeçalho do `reversion.rs`.

use super::*;
use crate::shapes;
use crate::subdivide::subdivide;

#[test]
fn reversing_a_subdivided_quad_mesh_gives_the_original_topology_back() {
    let cube = shapes::cube(1.0);
    let fine = subdivide(&cube);
    let rev = reverse_subdivision(&fine).expect("uma subdivisão reverte");
    assert_eq!(
        rev.coarse().vert_count(),
        cube.vert_count(),
        "os pares são os vértices grossos"
    );
    assert_eq!(rev.coarse().face_count(), cube.face_count());
    assert!(rev.coarse().faces().iter().all(|f| !f.is_tri()));
}

#[test]
fn reversing_a_subdivided_triangle_mesh_gives_the_original_topology_back() {
    let octa = shapes::octahedron(1.0);
    let fine = subdivide(&octa);
    let rev = reverse_subdivision(&fine).expect("uma subdivisão de triângulos reverte");
    assert_eq!(rev.coarse().vert_count(), octa.vert_count());
    assert_eq!(rev.coarse().face_count(), octa.face_count());
    assert!(rev.coarse().faces().iter().all(Face::is_tri));
}

#[test]
fn the_renumbering_is_a_bijection_of_the_fine_vertices() {
    let fine = subdivide(&shapes::cube(1.0));
    let rev = reverse_subdivision(&fine).expect("reverte");
    let map = rev.renumber();
    assert_eq!(map.len(), fine.vert_count());
    let mut seen = vec![false; fine.vert_count()];
    for &o in map {
        assert!(!seen[o as usize], "índice repetido na renumeração");
        seen[o as usize] = true;
    }
    assert!(seen.iter().all(|&s| s));
}

/// ⚠️ O gate que diz *o que a renumeração SIGNIFICA*: o bloco dos pares vem
/// primeiro, e cada um deles é o vértice grosso de mesmo índice. Sem isto a
/// permutação seria só uma bijeção qualquer, que é o que a irmã acima afirma.
#[test]
fn the_first_block_of_the_renumbering_is_the_coarse_vertices_in_order() {
    let fine = subdivide(&shapes::cube(1.0));
    let rev = reverse_subdivision(&fine).expect("reverte");
    let vc = rev.coarse().vert_count();
    for i in 0..vc {
        let old = rev.renumber()[i] as usize;
        assert_eq!(
            rev.coarse().positions()[i],
            fine.positions()[old],
            "o vértice grosso {i} é a cópia do fino {old}"
        );
    }
}

#[test]
fn a_mesh_that_is_not_a_subdivision_is_refused() {
    // O octaedro tem oito faces — divisível por quatro, então a recusa barata
    // não o pega e quem decide é a estrutura. É a fixture que separa *"a
    // contagem não fecha"* de *"a etiquetagem não fecha"*.
    let octa = shapes::octahedron(1.0);
    assert_eq!(octa.face_count() % 4, 0, "a fixture contém o fenômeno");
    assert!(reverse_subdivision(&octa).is_none());
}

#[test]
fn a_face_count_that_is_not_a_multiple_of_four_is_refused() {
    let cube = shapes::cube(1.0);
    assert_eq!(cube.face_count(), 6);
    assert!(reverse_subdivision(&cube).is_none());
}

#[test]
fn an_empty_mesh_is_refused() {
    assert!(reverse_subdivision(&Mesh::default()).is_none());
}

#[test]
fn the_coarse_level_carries_the_colour_and_the_mask_of_the_vertices_it_keeps() {
    let mut fine = subdivide(&shapes::cube(1.0));
    for (i, c) in fine.colors_mut().iter_mut().enumerate() {
        *c = [i as f32, 0.0, 0.0];
    }
    for (i, m) in fine.masks_mut().iter_mut().enumerate() {
        *m = i as f32 * 0.01;
    }
    let rev = reverse_subdivision(&fine).expect("reverte");
    let (coarse, map) = rev.into_parts();
    for (i, &old) in map.iter().take(coarse.vert_count()).enumerate() {
        let old = old as usize;
        assert_eq!(
            coarse.colors().expect("cor")[i],
            fine.colors().expect("")[old]
        );
        assert_eq!(
            coarse.masks().expect("máscara")[i],
            fine.masks().expect("")[old]
        );
    }
}

/// ⚠️ **A malha sem canal NÃO ganha um.** Materializar cor e máscara na malha
/// grossa custaria 16 B/vértice por planos que ninguém pediu — a mesma lei do
/// `colors_mut`, e a razão pela qual a cópia é condicional.
#[test]
fn a_mesh_without_channels_does_not_grow_them_on_the_way_down() {
    let fine = subdivide(&shapes::cube(1.0));
    assert!(fine.colors().is_none() && fine.masks().is_none());
    let rev = reverse_subdivision(&fine).expect("reverte");
    assert!(rev.coarse().colors().is_none());
    assert!(rev.coarse().masks().is_none());
}

#[test]
fn reversing_twice_walks_two_levels_down() {
    let cube = shapes::cube(1.0);
    let once = subdivide(&cube);
    let twice = subdivide(&once);
    let a = reverse_subdivision(&twice).expect("primeira");
    assert_eq!(a.coarse().face_count(), once.face_count());
    let b = reverse_subdivision(a.coarse()).expect("segunda");
    assert_eq!(b.coarse().face_count(), cube.face_count());
    assert_eq!(b.coarse().vert_count(), cube.vert_count());
}

/// A esfera UV mistura quads no corpo com triângulos nos polos, e é a única
/// fixture em que o ramo *misto* do `is_regular` decide alguma coisa.
#[test]
fn a_mixed_quad_and_triangle_mesh_reverses() {
    let sphere = shapes::uv_sphere(4, 6, 1.0);
    let fine = subdivide(&sphere);
    let rev = reverse_subdivision(&fine).expect("a esfera UV reverte");
    assert_eq!(rev.coarse().vert_count(), sphere.vert_count());
    assert_eq!(rev.coarse().face_count(), sphere.face_count());
}

/// ⚠️ **A malha da cena `=3` do smoke é EXATAMENTE esta** — uma esfera UV duas
/// vezes subdividida (`sculpt3d::smoke_mesh`). O arch-gate do shell pina que a
/// cena a constrói assim; este pina que ela de fato REVERTE duas vezes, que é a
/// metade geométrica e a que nenhuma leitura de fonte enxerga. Uma cena cuja
/// malha não reverte é uma cena que demonstra o log de recusa.
#[test]
fn the_mesh_of_the_reversion_smoke_scene_reverses_twice() {
    let coarse = shapes::uv_sphere(12, 18, 1.0);
    let once = subdivide(&coarse);
    let dense = subdivide(&once);
    let a = reverse_subdivision(&dense).expect("a cena reverte uma vez");
    assert_eq!(a.coarse().face_count(), once.face_count());
    let b = reverse_subdivision(a.coarse()).expect("e duas");
    assert_eq!(b.coarse().face_count(), coarse.face_count());
    assert_eq!(b.coarse().vert_count(), coarse.vert_count());
}

/// ⚠️ **Malha malformada não é o mesmo que malha que não é subdivisão**, e o
/// gate afirma as duas metades: as formas quebradas do [`shapes_open`] são
/// RECUSADAS enquanto ninguém as subdividiu, e as que SÃO uma subdivisão
/// revertem — mesmo carregando a degeneração que as põe naquele módulo. Uma
/// face colapsada continua sendo uma face, e recusar por causa dela seria
/// recusar uma reversão correta.
#[test]
fn a_malformed_shape_is_refused_but_its_subdivision_is_not() {
    use crate::shapes_open;
    for (name, mesh) in [
        ("open_tube3", shapes_open::open_tube3()),
        ("collapsed_tetra", shapes_open::collapsed_tetra()),
        ("sliver_bipyramid", shapes_open::sliver_bipyramid()),
    ] {
        assert!(
            reverse_subdivision(&mesh).is_none(),
            "{name} não é subdivisão de nada"
        );
        let fine = subdivide(&mesh);
        let rev =
            reverse_subdivision(&fine).unwrap_or_else(|| panic!("a subdivisão de {name} reverte"));
        assert_eq!(rev.coarse().vert_count(), mesh.vert_count(), "{name}");
        assert_eq!(rev.coarse().face_count(), mesh.face_count(), "{name}");
    }
    // ⚠️ E o `pillow` NÃO entra na lista: ele é um triângulo de dois lados (duas
    // faces sobre os MESMOS três vértices), então nem ele nem a subdivisão dele
    // revertem — a etiquetagem acha dois pares vizinhos, que é a recusa certa.
    assert!(reverse_subdivision(&subdivide(&shapes_open::pillow())).is_none());
}
