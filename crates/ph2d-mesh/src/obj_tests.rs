//! Gates do import de OBJ.

use super::*;

#[test]
fn a_quad_in_the_file_stays_a_quad() {
    let m = import_obj("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n").unwrap();
    assert_eq!(m.vert_count(), 4);
    assert_eq!(m.face_count(), 1);
    assert!(
        !m.faces()[0].is_tri(),
        "triangular na porta jogaria fora a topologia que a multires precisa"
    );
    assert_eq!(m.triangle_count(), 2);
}

/// As três formas de referência do OBJ (`a`, `a/b`, `a//c`, `a/b/c`) apontam
/// para o mesmo vértice — só o primeiro campo é posição.
#[test]
fn the_slash_forms_all_resolve_to_the_position_index() {
    let base = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvn 0 0 1\n";
    for f in [
        "f 1 2 3",
        "f 1/1 2/1 3/1",
        "f 1//1 2//1 3//1",
        "f 1/1/1 2/1/1 3/1/1",
    ] {
        let m = import_obj(&format!("{base}{f}\n")).unwrap();
        assert_eq!(m.face_count(), 1, "forma {f:?}");
        assert_eq!(m.faces()[0].verts(), &[0, 1, 2], "forma {f:?}");
    }
}

/// Índice negativo conta de trás para a frente a partir do que já foi
/// declarado — parte do formato, e um arquivo exportado assim é comum.
#[test]
fn negative_indices_count_back_from_what_was_declared() {
    let m = import_obj("v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n").unwrap();
    assert_eq!(m.faces()[0].verts(), &[0, 1, 2]);
}

#[test]
fn an_index_that_names_no_vertex_is_an_error_not_a_silent_hole() {
    for bad in ["f 1 2 9", "f 0 1 2", "f 1 2 -9", "f a b c"] {
        let src = format!("v 0 0 0\nv 1 0 0\nv 0 1 0\n{bad}\n");
        assert!(
            matches!(import_obj(&src), Err(ObjError::BadFaceIndex { .. })),
            "{bad:?} devia ser recusado"
        );
    }
}

/// Cor por vértice (a extensão `v x y z r g b`) materializa o plano; um arquivo
/// sem cor deixa o plano NULO, que é o que a preguiça significa.
#[test]
fn vertex_colour_is_read_when_present_and_the_plane_stays_null_when_not() {
    let plain = import_obj("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
    assert!(plain.colors().is_none());

    let tinted = import_obj("v 0 0 0 1 0 0\nv 1 0 0 0 1 0\nv 0 1 0 0 0 1\nf 1 2 3\n").unwrap();
    let c = tinted.colors().expect("o arquivo trouxe cor");
    assert_eq!(c[0], [1.0, 0.0, 0.0]);
    assert_eq!(c[2], [0.0, 0.0, 1.0]);
}

/// Um n-gon acima de 4 vira leque de triângulos — não há representação para ele,
/// e perdê-lo em silêncio seria pior.
#[test]
fn an_ngon_becomes_a_triangle_fan() {
    let m = import_obj("v 0 0 0\nv 1 0 0\nv 2 1 0\nv 1 2 0\nv 0 2 0\nf 1 2 3 4 5\n").unwrap();
    assert_eq!(m.face_count(), 3, "um pentágono são 3 triângulos");
    assert!(m.faces().iter().all(super::Face::is_tri));
    assert_eq!(m.triangle_count(), 3);
}

/// Comentários, linhas em branco e diretivas que não entendemos são ignoradas
/// sem derrubar o arquivo.
#[test]
fn comments_and_unknown_directives_are_skipped() {
    let m = import_obj(
        "# um cubo qualquer\n\nmtllib x.mtl\ng grupo\nusemtl azul\ns 1\n\
         v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
    )
    .unwrap();
    assert_eq!(m.vert_count(), 3);
    assert_eq!(m.face_count(), 1);
}

/// O import entrega a malha **construída**: normais, adjacência e octree
/// prontos. Um import que devolvesse buffers crus deixaria o primeiro
/// consumidor descobrir a dívida.
#[test]
fn the_import_hands_back_a_built_mesh() {
    let m = import_obj("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
    assert_eq!(m.normals().len(), 3);
    assert_eq!(m.face_normals().len(), 1);
    assert!(!m.octree().is_empty());
    assert_eq!(m.adjacency().vert_faces.neighbours(0), &[0]);
    assert!(!m.bounds().is_empty());
}
