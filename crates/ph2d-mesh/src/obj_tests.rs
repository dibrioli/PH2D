//! Gates do import de OBJ.

use super::*;

/// A ÚNICA peça de um arquivo de um objeto só.
///
/// ⚠️ Ela **declara a premissa** destas fixtures, e não é açúcar: desde a W8.4 a
/// porta devolve uma peça por `o`, e um arquivo sem `o` tem de dar exatamente
/// uma. Um `[0]` mudo faria um arquivo que passasse a render duas peças ler
/// como se nada tivesse mudado.
fn one(text: &str) -> Mesh {
    let mut v = import_obj(text).expect("o arquivo carrega");
    assert_eq!(v.len(), 1, "arquivo sem `o` é UMA peça");
    v.remove(0).mesh
}

#[test]
fn a_quad_in_the_file_stays_a_quad() {
    let m = one("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n");
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
        let m = one(&format!("{base}{f}\n"));
        assert_eq!(m.face_count(), 1, "forma {f:?}");
        assert_eq!(m.faces()[0].verts(), &[0, 1, 2], "forma {f:?}");
    }
}

/// Índice negativo conta de trás para a frente a partir do que já foi
/// declarado — parte do formato, e um arquivo exportado assim é comum.
#[test]
fn negative_indices_count_back_from_what_was_declared() {
    let m = one("v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n");
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
    let plain = one("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n");
    assert!(plain.colors().is_none());

    let tinted = one("v 0 0 0 1 0 0\nv 1 0 0 0 1 0\nv 0 1 0 0 0 1\nf 1 2 3\n");
    let c = tinted.colors().expect("o arquivo trouxe cor");
    assert_eq!(c[0], [1.0, 0.0, 0.0]);
    assert_eq!(c[2], [0.0, 0.0, 1.0]);
}

/// Um n-gon acima de 4 vira leque de triângulos — não há representação para ele,
/// e perdê-lo em silêncio seria pior.
#[test]
fn an_ngon_becomes_a_triangle_fan() {
    let m = one("v 0 0 0\nv 1 0 0\nv 2 1 0\nv 1 2 0\nv 0 2 0\nf 1 2 3 4 5\n");
    assert_eq!(m.face_count(), 3, "um pentágono são 3 triângulos");
    assert!(m.faces().iter().all(super::Face::is_tri));
    assert_eq!(m.triangle_count(), 3);
}

/// Comentários, linhas em branco e diretivas que não entendemos são ignoradas
/// sem derrubar o arquivo.
#[test]
fn comments_and_unknown_directives_are_skipped() {
    let m = one(
        "# um cubo qualquer\n\nmtllib x.mtl\ng grupo\nusemtl azul\ns 1\n\
         v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
    );
    assert_eq!(m.vert_count(), 3);
    assert_eq!(m.face_count(), 1);
}

/// O import entrega a malha **construída**: normais, adjacência e octree
/// prontos. Um import que devolvesse buffers crus deixaria o primeiro
/// consumidor descobrir a dívida.
#[test]
fn the_import_hands_back_a_built_mesh() {
    let m = one("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n");
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
            got.map(|v| v
                .first()
                .and_then(|p| p.mesh.faces().first().map(|f| f.verts().to_vec()))
                .unwrap_or_default())
        );
    }
}

/// O BOM não é geometria malformada: é um marcador de codificação que todo
/// editor de Windows escreve. Ele é COMIDO na porta, não recusado — recusar um
/// arquivo legal por causa de três bytes invisíveis seria o oposto do conserto.
#[test]
fn a_byte_order_mark_does_not_swallow_the_first_vertex() {
    let m = one("\u{feff}v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n");
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
    let w = one("v 0 0 0 1.0\nv 1 0 0 1.0\nv 0 1 0 1.0\nf 1 2 3\n");
    assert_eq!(w.positions()[0], [0.0, 0.0, 0.0]);
    assert!(w.colors().is_none(), "quatro números não são cor");

    // Comentário DEPOIS de uma cor.
    let c = one("v 0 0 0 1 0 0 # vermelho\nv 1 0 0 0 1 0 # verde\nv 0 1 0 0 0 1 # azul\nf 1 2 3\n");
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
        let m = one(&format!("{base}{bad}\n"));
        assert_eq!(m.face_count(), 0, "{bad:?} não é superfície");
    }
    // E o controle: a mesma forma SEM repetição continua entrando.
    let good = one(&format!("{base}f 1 2 3 4\n"));
    assert_eq!(good.face_count(), 1);
}

/// **Um `o` por peça** — a dívida que a W8.1 nomeou, e a tradução honesta de um
/// arquivo multi-objeto agora que a cena é uma LISTA.
///
/// ⚠️ O oráculo é a **geometria de cada peça**, não a contagem: um split que
/// devolvesse duas peças com a malha inteira nas duas passaria por `len() == 2`
/// e seria exatamente o defeito que a compactação existe para não ter.
#[test]
fn each_named_object_becomes_its_own_piece() {
    // Dois triângulos disjuntos, e o segundo referencia vértices declarados
    // DEPOIS do seu `o` — o caso normal.
    let src = "o cabeca\n\
               v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n\
               o corpo\n\
               v 5 0 0\nv 6 0 0\nv 5 1 0\nf 4 5 6\n";
    let pieces = import_obj(src).expect("carrega");

    assert_eq!(pieces.len(), 2, "duas peças");
    assert_eq!(pieces[0].name.as_deref(), Some("cabeca"));
    assert_eq!(pieces[1].name.as_deref(), Some("corpo"));
    for p in &pieces {
        assert_eq!(
            p.mesh.vert_count(),
            3,
            "cada peça carrega SÓ os vértices que ela usa — {:?} trouxe {}",
            p.name,
            p.mesh.vert_count()
        );
    }
    // E a compactação preservou a POSIÇÃO, não só a contagem.
    assert_eq!(pieces[1].mesh.positions()[0], [5.0, 0.0, 0.0]);
}

/// **Um `f` pode apontar para trás do próprio `o`** — o pool de vértices é do
/// ARQUIVO, e ler os índices como se fossem locais embaralha a geometria em vez
/// de falhar.
#[test]
fn a_face_may_reference_vertices_declared_before_its_object() {
    let src = "v 0 0 0\nv 1 0 0\nv 0 1 0\n\
               o tardia\n\
               f 1 2 3\n";
    let pieces = import_obj(src).expect("carrega");
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].name.as_deref(), Some("tardia"));
    assert_eq!(pieces[0].mesh.positions()[1], [1.0, 0.0, 0.0]);
}

/// **Um `o` sem faces não produz peça** — e o gate existe porque o caso é o
/// NORMAL: quase todo OBJ abre com `o <nome>` antes do primeiro `v`.
///
/// ⚠️ Uma peça vazia não seria um item a mais na lista: o `from_parts` recusa
/// malha sem face, então o arquivo INTEIRO morreria por causa de um cabeçalho.
#[test]
fn a_header_object_with_no_faces_does_not_make_an_empty_piece() {
    let src = "o so_o_cabecalho\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
    let pieces = import_obj(src).expect("carrega");
    assert_eq!(pieces.len(), 1, "o cabeçalho nomeia a peça, não cria uma");
    assert_eq!(pieces[0].name.as_deref(), Some("so_o_cabecalho"));
}

/// **Um quad atravessa a compactação como quad** — a sentinela `TRI` do 4º slot
/// não é um índice, e remapeá-la o transformaria num vértice que a peça não tem.
#[test]
fn compacting_keeps_a_quad_a_quad_and_a_triangle_a_triangle() {
    let src = "o misto\n\
               v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nv 2 0 0\n\
               f 1 2 3 4\nf 2 5 3\n";
    let m = &import_obj(src).expect("carrega")[0].mesh;
    assert_eq!(m.face_count(), 2);
    assert_eq!(m.faces()[0].verts().len(), 4, "o quad continua quad");
    assert_eq!(m.faces()[1].verts().len(), 3, "e o triângulo, triângulo");
    assert_eq!(m.vert_count(), 5);
}
