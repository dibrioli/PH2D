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

/// O arquivo em que o deslize é **silencioso**: a linha malformada fica no meio
/// e as faces só citam índices que ainda existem depois do encolhimento, então
/// nada estoura — a face apenas passa a apontar para o vértice ERRADO.
///
/// ⚠️ Sem esta forma o defeito se esconde: se a linha malformada for a última,
/// ou se as faces citarem o fim da lista, o índice sai de alcance e o import já
/// devolve `BadFaceIndex` **por acidente** — um gate ali ficaria verde
/// descrevendo outro mecanismo.
fn sliding_file(bad_v: &str) -> String {
    format!("v 0 0 0\nv 1 0 0\n{bad_v}\nv 0 1 0\nv 2 2 2\nf 1 2 4\n")
}

#[test]
fn a_malformed_vertex_line_is_refused_instead_of_sliding_every_index_after_it() {
    // O título da nota antiga — "a linha `v` malformada" — descrevia só o
    // primeiro destes.
    //
    // ⚠️ **Os canais são de DUAS classes, e uma fixture só da primeira deixa
    // metade da cura sem gate.** Onde sobram menos de três números a linha é
    // DESCARTADA e todo índice seguinte desliza; onde sobram três ou mais o
    // vértice FICA com as coordenadas erradas. Uma mutação que volte a pular o
    // token inconversível em silêncio passa por toda a primeira classe — foi
    // ela que expôs este buraco.
    let channels = [
        // descarta ⇒ desliza
        ("truncamento", "v 9 9"),
        ("continuação de linha", "v 9 9 \\\n9"),
        ("locale/compactação", "v 1,0 0 0"),
        // fica ⇒ corrompe: sobram três números (`0 0 1.0`) e o vértice entra
        // como (0, 0, 1) onde o arquivo diz (1, 0, 0).
        ("locale com o campo w", "v 1,0 0 0 1.0"),
        ("não-finito", "v inf 0 0"),
    ];
    for (name, bad) in channels {
        let src = sliding_file(bad);
        let got = import_obj(&src);
        assert!(
            matches!(got, Err(ObjError::BadVertex { .. })),
            "{name}: {bad:?} devia RECUSAR o arquivo, e devolveu {:?}",
            got.map(|m| m
                .faces()
                .first()
                .map(|f| f.verts().to_vec())
                .unwrap_or_default())
        );
    }
}

/// O BOM não é geometria malformada: é um marcador de codificação que todo
/// editor de Windows escreve. Ele é COMIDO na porta, não recusado — recusar um
/// arquivo legal por causa de três bytes invisíveis seria o oposto do conserto.
#[test]
fn a_byte_order_mark_does_not_swallow_the_first_vertex() {
    let m = import_obj("\u{feff}v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n")
        .expect("um OBJ com BOM é um OBJ legal");
    assert_eq!(m.vert_count(), 3, "o BOM comeu o primeiro vértice");
    assert_eq!(m.positions()[0], [0.0, 0.0, 0.0]);
}

/// ⚠️ **O CONTROLE, e ele é a metade que impede a cura de virar regressão.**
/// Estas duas linhas são OBJ **legal** e carregam certo hoje; uma cura ingênua
/// — *"existe um 4º token ⇒ é cor"*, ou *"todo token tem de ser número"* — as
/// quebraria, trocando um defeito por outro em arquivos que funcionam.
#[test]
fn the_two_legal_forms_that_a_naive_cure_would_break_still_load() {
    // O campo `w`, parte do formato (`v x y z w`).
    let w = import_obj("v 0 0 0 1.0\nv 1 0 0 1.0\nv 0 1 0 1.0\nf 1 2 3\n")
        .expect("o campo w é parte do formato");
    assert_eq!(w.positions()[0], [0.0, 0.0, 0.0]);
    assert!(w.colors().is_none(), "quatro números não são cor");

    // Comentário DEPOIS de uma cor.
    let c = import_obj(
        "v 0 0 0 1 0 0 # vermelho\nv 1 0 0 0 1 0 # verde\nv 0 1 0 0 0 1 # azul\nf 1 2 3\n",
    )
    .expect("comentário no fim de linha é legal");
    assert_eq!(c.colors().expect("trouxe cor")[0], [1.0, 0.0, 0.0]);
}

/// Uma face com índice repetido é descartada — como no `ImportOBJ.js:88-92`.
///
/// ⚠️ Ela tem área zero **por construção**, então aceitá-la era fabricar, na
/// porta de entrada, exatamente a face degenerada cujo voto na normal o
/// `normals.rs` teve de aprender a recusar. As duas metades do mesmo defeito.
#[test]
fn a_face_that_names_the_same_vertex_twice_is_dropped() {
    let base = "v 0 0 0\nv 1 0 0\nv 0 1 0\nv 1 1 0\n";
    for bad in ["f 1 1 2", "f 1 2 2", "f 1 2 3 1", "f 1 2 3 2"] {
        let m = import_obj(&format!("{base}{bad}\n")).expect("o arquivo em si é legal");
        assert_eq!(m.face_count(), 0, "{bad:?} não é superfície");
    }
    // E o controle: a mesma forma SEM repetição continua entrando.
    let good = import_obj(&format!("{base}f 1 2 3 4\n")).unwrap();
    assert_eq!(good.face_count(), 1);
}
