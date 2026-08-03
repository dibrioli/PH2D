//! Gates do leitor de STL.

use super::*;
use crate::export::ExportPiece;
use crate::pose::Pose;
use crate::{shapes, write_stl};

fn one(mesh: &crate::mesh::Mesh) -> Vec<u8> {
    write_stl(&[ExportPiece {
        name: None,
        mesh,
        pose: Pose::IDENTITY,
    }])
}

/// **A SOLDA é a razão de este leitor existir**, e o oráculo é a ADJACÊNCIA.
///
/// ⚠️ Um STL de um cubo traz 12 triângulos × 3 = 36 posições, das quais 8 são
/// distintas. Sem soldar, a malha desenha certo e **toda ferramenta de escultura
/// é inerte nela** — o suavizar, o subdividir, o espelho e o fechar-buraco leem
/// adjacência, e cada triângulo seria uma ilha. Contar vértices é o teste
/// barato; o que importa de verdade é a malha não ter BORDA.
#[test]
fn loose_triangles_are_welded_into_a_closed_mesh() {
    let cube = shapes::cube(1.0);
    let mesh = import_stl(&one(&cube)).expect("stl");

    assert_eq!(
        mesh.positions().len(),
        cube.positions().len(),
        "36 posições soltas tinham de soldar nos 8 cantos do cubo"
    );
    let edges = mesh.edges();
    let border = (0..edges.len() as u32)
        .filter(|&e| edges.valence(e) == 1)
        .count();
    assert_eq!(
        border, 0,
        "um cubo soldado é FECHADO; com ilhas, TODA aresta seria borda"
    );
}

/// **O ramo binário é escolhido pelo TAMANHO, não pela palavra `solid`.**
///
/// ⚠️ É a armadilha clássica do formato: o cabeçalho binário tem 80 bytes livres
/// e muitos escritores põem `solid <nome>` neles. Um leitor que decida pelo
/// prefixo lê o arquivo binário como texto e devolve **uma malha vazia** — sem
/// erro, porque um texto sem a palavra `vertex` simplesmente não tem
/// triângulos.
#[test]
fn a_binary_file_that_starts_with_solid_is_still_read_as_binary() {
    let mut bytes = one(&shapes::cube(1.0));
    bytes[..6].copy_from_slice(b"solid ");
    let mesh = import_stl(&bytes).expect("o tamanho é quem decide");
    assert_eq!(mesh.positions().len(), 8);
}

/// **ASCII também entra** — é o que scanners antigos e exportadores de CAD
/// produzem.
#[test]
fn the_ascii_flavour_loads_too() {
    let text = "solid t\n\
                facet normal 0 0 1\n outer loop\n\
                  vertex 0 0 0\n vertex 1 0 0\n vertex 0 1 0\n\
                 endloop\n endfacet\n\
                endsolid t\n";
    let mesh = import_stl(text.as_bytes()).expect("ascii");
    assert_eq!(mesh.positions().len(), 3);
    assert_eq!(mesh.faces().len(), 1);
}

/// **`-0.0` e `+0.0` são o MESMO ponto**, e sem normalizar o zero a costura abre
/// exatamente sobre o plano de simetria — onde o espelho trabalha.
#[test]
fn negative_zero_welds_with_positive_zero() {
    // Dois triângulos que partilham a aresta em x = 0, um deles escrito com -0.
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0u8; 80]);
    bytes.extend_from_slice(&2u32.to_le_bytes());
    let tri = |b: &mut Vec<u8>, v: [[f32; 3]; 3]| {
        b.extend_from_slice(&[0u8; 12]);
        for p in v {
            for c in p {
                b.extend_from_slice(&c.to_le_bytes());
            }
        }
        b.extend_from_slice(&0u16.to_le_bytes());
    };
    tri(
        &mut bytes,
        [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
    );
    tri(
        &mut bytes,
        [[-0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]],
    );

    let mesh = import_stl(&bytes).expect("stl");
    assert_eq!(
        mesh.positions().len(),
        4,
        "os dois zeros são o mesmo canto: 4 posições, não 5"
    );
}

/// **Um triângulo degenerado é DESCARTADO**, não guardado como face.
///
/// ⚠️ Os índices de uma face colapsada são válidos, então o `from_parts` a
/// aceitaria — e ela envenenaria a normal (comprimento zero) e a adjacência.
#[test]
fn a_collapsed_triangle_is_dropped_not_stored() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0u8; 80]);
    bytes.extend_from_slice(&2u32.to_le_bytes());
    let tri = |b: &mut Vec<u8>, v: [[f32; 3]; 3]| {
        b.extend_from_slice(&[0u8; 12]);
        for p in v {
            for c in p {
                b.extend_from_slice(&c.to_le_bytes());
            }
        }
        b.extend_from_slice(&0u16.to_le_bytes());
    };
    tri(
        &mut bytes,
        [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
    );
    tri(
        &mut bytes,
        [[5.0, 5.0, 5.0], [5.0, 5.0, 5.0], [5.0, 5.0, 5.0]],
    );

    let mesh = import_stl(&bytes).expect("stl");
    assert_eq!(mesh.faces().len(), 1, "a face colapsada não vira face");
}

/// **Um binário truncado é RECUSADO com o número**, em vez de cair no ramo ASCII
/// e devolver uma malha vazia.
#[test]
fn a_truncated_binary_is_refused_by_name() {
    let full = one(&shapes::cube(1.0));
    let cut = &full[..full.len() - 20];
    match import_stl(cut) {
        Err(StlError::Truncated { expected, got }) => {
            assert_eq!(expected, full.len());
            assert_eq!(got, cut.len());
        }
        other => panic!("um binário truncado tem de se nomear, e deu {other:?}"),
    }
}
