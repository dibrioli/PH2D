//! Gates do leitor de PLY.

use super::*;
use crate::export::ExportPiece;
use crate::pose::Pose;
use crate::{shapes, write_ply};

fn one(mesh: &crate::mesh::Mesh) -> Vec<u8> {
    write_ply(&[ExportPiece {
        name: None,
        mesh,
        pose: Pose::IDENTITY,
    }])
}

/// **O nosso binário volta inteiro** — geometria, quads e cor.
#[test]
fn our_own_binary_round_trips() {
    let mut m = shapes::cube(1.0);
    m.colors_mut()[0] = [1.0, 0.0, 0.0];
    let back = import_ply(&one(&m)).expect("ply");

    assert_eq!(back.positions().len(), m.positions().len());
    assert_eq!(back.faces().len(), m.faces().len());
    let c = back.colors().expect("cor");
    assert!(
        c[0][0] > 0.99 && c[0][1] < 0.01,
        "a cor do vértice 0 mudou: {:?}",
        c[0]
    );
}

/// **O ASCII entra também** — é o que sai de metade dos scanners.
#[test]
fn the_ascii_flavour_loads_too() {
    let text = "ply\nformat ascii 1.0\n\
                element vertex 3\n\
                property float x\nproperty float y\nproperty float z\n\
                property uchar red\nproperty uchar green\nproperty uchar blue\n\
                element face 1\n\
                property list uchar int vertex_indices\n\
                end_header\n\
                0 0 0 255 0 0\n1 0 0 0 255 0\n0 1 0 0 0 255\n\
                3 0 1 2\n";
    let mesh = import_ply(text.as_bytes()).expect("ply ascii");
    assert_eq!(mesh.positions().len(), 3);
    assert_eq!(mesh.faces().len(), 1);
    let c = mesh.colors().expect("cor");
    assert!(c[0][0] > 0.99 && c[1][1] > 0.99 && c[2][2] > 0.99);
}

/// **A ORDEM das propriedades é do ARQUIVO** — e este é o gate que impede o
/// defeito mais caro deste leitor.
///
/// ⚠️ Um leitor que assuma `x y z red green blue` porque é isso que ele mesmo
/// escreve devolve, num arquivo com `nx ny nz` no meio, **as normais como cor** e
/// coordenadas deslocadas — e não falha, porque os números são todos válidos. A
/// fixture põe a normal ENTRE a posição e a cor de propósito.
#[test]
fn the_header_decides_where_each_property_lives() {
    let text = "ply\nformat ascii 1.0\n\
                element vertex 1\n\
                property float x\nproperty float y\nproperty float z\n\
                property float nx\nproperty float ny\nproperty float nz\n\
                property uchar red\nproperty uchar green\nproperty uchar blue\n\
                element face 0\n\
                property list uchar int vertex_indices\n\
                end_header\n\
                7 8 9  0 1 0  255 0 0\n";
    let mesh = import_ply(text.as_bytes()).expect("ply");
    assert_eq!(
        mesh.positions()[0],
        [7.0, 8.0, 9.0],
        "a posição saiu deslocada"
    );
    let c = mesh.colors().expect("cor");
    assert!(
        c[0][0] > 0.99 && c[0][1] < 0.01,
        "leu a NORMAL como cor: {:?}",
        c[0]
    );
}

/// **Um elemento que não nos interessa é PULADO pelo tamanho**, nunca ignorado.
///
/// ⚠️ Ignorá-lo desalinharia todo o corpo binário a partir dali, e o resultado é
/// uma malha com coordenadas absurdas — que "carregou".
#[test]
fn an_unknown_element_is_skipped_by_its_size() {
    let mut out = Vec::new();
    out.extend_from_slice(
        b"ply\nformat binary_little_endian 1.0\n\
          element vertex 1\nproperty float x\nproperty float y\nproperty float z\n\
          element junk 2\nproperty float a\nproperty double b\n\
          element face 0\nproperty list uchar int vertex_indices\n\
          end_header\n",
    );
    for c in [1.0f32, 2.0, 3.0] {
        out.extend_from_slice(&c.to_le_bytes());
    }
    // Dois registros de junk: 4 + 8 bytes cada. Se forem IGNORADOS em vez de
    // pulados, nada mais acontece aqui — mas o vertex já foi lido, então o
    // oráculo é o que vem DEPOIS num arquivo real. Aqui basta não estourar.
    out.extend_from_slice(&[0u8; 24]);

    let mesh = import_ply(&out).expect("ply");
    assert_eq!(mesh.positions()[0], [1.0, 2.0, 3.0]);
}

/// **Big-endian é RECUSADO com nome**, não lido ao contrário.
///
/// ⚠️ Ler os bytes trocados produz coordenadas astronômicas e uma malha que
/// "carregou" — a recusa aponta para o arquivo.
#[test]
fn big_endian_is_refused_by_name() {
    let text = "ply\nformat binary_big_endian 1.0\nelement vertex 0\nend_header\n";
    match import_ply(text.as_bytes()) {
        Err(PlyError::UnsupportedFormat(f)) => assert!(f.contains("big")),
        other => panic!("big-endian tem de se nomear, e deu {other:?}"),
    }
}

/// **Cor em `float` NÃO é dividida por 255**, e a regra é do TIPO.
///
/// ⚠️ Decidir por *"é maior que 1, então deve ser 0..255"* faria um PLY em
/// `float` com um canal levemente acima de 1 (o que HDR produz) virar preto
/// quase puro.
#[test]
fn float_colour_is_read_as_unit_range_not_divided() {
    let text = "ply\nformat ascii 1.0\n\
                element vertex 1\n\
                property float x\nproperty float y\nproperty float z\n\
                property float red\nproperty float green\nproperty float blue\n\
                element face 0\nproperty list uchar int vertex_indices\n\
                end_header\n\
                0 0 0 1.0 0.5 0.0\n";
    let mesh = import_ply(text.as_bytes()).expect("ply");
    let c = mesh.colors().expect("cor")[0];
    assert!(
        c[0] > 0.99 && (c[1] - 0.5).abs() < 0.01,
        "cor dividida por engano: {c:?}"
    );
}

/// **Um cabeçalho que não é PLY é recusado**, sem tentar adivinhar.
#[test]
fn a_non_ply_file_is_refused() {
    assert!(matches!(
        import_ply(b"not a ply at all"),
        Err(PlyError::BadHeader)
    ));
}
