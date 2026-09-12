//! Gates do documento de escultura, **sem janela**.
//!
//! Uma [`Sculpt3dScene`] não nasce sem um `wgpu::Device`, então o que roda aqui
//! é o par [`encode`]/[`decode`] — a metade que decide *o que um arquivo guarda
//! e o que ele recusa*. O que sobra do lado da CENA (instalar as peças, matar a
//! fila de desfazer, re-cunhar os ids) é afirmado pelos arch-gates de
//! `tests/the_sculpt_document_is_wired.rs`, que leem a fonte.

use super::*;
use ph2d_mesh::{Multires, shapes};

/// Uma peça com trabalho FINO no nível de cima — a fixture que distingue
/// *guardar a pilha* de *guardar a malha viva*.
fn piece(z: f32) -> (Multires, Pose) {
    let mut stack = Multires::new(shapes::octahedron(1.0));
    assert!(stack.add_level(), "a fixture precisa do 2º nível");
    stack.mesh_mut().positions_mut()[0][1] += 0.25;
    (stack, Pose::new([z, 0.0, 0.0], 2.0))
}

fn doc_bytes(pieces: &[(Multires, Pose)], active: usize) -> Vec<u8> {
    let data: Vec<(StackData, PoseData)> = pieces
        .iter()
        .map(|(s, p)| (s.to_data(), p.to_data()))
        .collect();
    encode(&data, active)
}

/// **O que a cena escreve é o que o load devolve** — as peças, a ordem, quem
/// estava em mãos, e o trabalho de cada nível.
#[test]
fn what_the_scene_writes_is_what_the_load_reads_back() {
    let src = [piece(0.0), piece(3.0), piece(-3.0)];
    let (back, active) = decode(&doc_bytes(&src, 2)).expect("documento válido");

    assert_eq!(back.len(), 3, "as três peças");
    assert_eq!(active, 2, "quem estava em mãos");
    for (i, ((s0, p0), (s1, p1))) in src.iter().zip(back.iter()).enumerate() {
        assert_eq!(s1.level_count(), s0.level_count(), "níveis da peça {i}");
        assert_eq!(s1.level(), s0.level(), "o nível em mãos da peça {i}");
        assert!(
            (s1.mesh().positions()[0][1] - s0.mesh().positions()[0][1]).abs() < 1e-6,
            "o trabalho fino da peça {i}"
        );
        assert_eq!(p1.translation, p0.translation, "posição da peça {i}");
        assert!((p1.scale() - p0.scale()).abs() < 1e-6, "escala da peça {i}");
    }
}

/// **Um documento de OUTRA versão é recusado**, e a recusa nomeia as duas.
///
/// ⚠️ É este gate que dá sentido ao `SCULPT_DOC_VERSION` morar DENTRO do blob: o
/// `PROJECT_SCHEMA` bumpa uma vez, e daqui em diante o módulo evolui contra este
/// número. Sem a recusa, um blob de outra forma seria lido como lixo bem-formado
/// — o postcard é posicional e não avisa.
#[test]
fn a_document_from_another_version_is_refused() {
    let bytes = doc_bytes(&[piece(0.0)], 0);
    let mut doc: SculptDoc = postcard::from_bytes(&bytes).expect("re-lê");
    doc.version = SCULPT_DOC_VERSION + 1;
    let forged = postcard::to_allocvec(&doc).expect("serializa");

    match decode(&forged) {
        Err(SculptDocError::Version { found, expected }) => {
            assert_eq!(
                (found, expected),
                (SCULPT_DOC_VERSION + 1, SCULPT_DOC_VERSION)
            );
        }
        other => panic!("um documento de outra versão tem de ser recusado, e veio {other:?}"),
    }
}

/// **Bytes que não são um documento são recusados** — truncado, vazio ou de
/// outro formato.
#[test]
fn bytes_that_are_not_a_document_are_refused() {
    let bytes = doc_bytes(&[piece(0.0)], 0);
    for (label, corrupt) in [
        ("truncado", bytes[..bytes.len() / 2].to_vec()),
        ("vazio", Vec::new()),
        ("lixo", vec![0xff; 32]),
    ] {
        assert!(
            matches!(decode(&corrupt), Err(SculptDocError::Bytes(_))),
            "{label} tem de ser recusado"
        );
    }
}

/// **Geometria que não valida é recusada, e a razão atravessa** — o `DocError`
/// da `ph2d-mesh` chega inteiro ao toast, em vez de virar um "não abriu".
#[test]
fn geometry_that_does_not_validate_is_refused_with_its_reason() {
    let bytes = doc_bytes(&[piece(0.0)], 0);
    let mut doc: SculptDoc = postcard::from_bytes(&bytes).expect("re-lê");
    // Um plano por-vértice que não mede a malha: exatamente o que um arquivo
    // truncado-e-remendado produziria.
    doc.objects[0].stack.levels[0].masks = Some(vec![0.0; 2]);
    let forged = postcard::to_allocvec(&doc).expect("serializa");

    match decode(&forged) {
        Err(SculptDocError::Content(e)) => {
            assert!(
                e.to_string().contains("máscara"),
                "a razão tem de nomear o plano, e veio `{e}`"
            );
        }
        other => panic!("geometria inválida tem de ser recusada, e veio {other:?}"),
    }
}

/// **Um índice de peça ativa fora de alcance é CLAMPADO, não recusado.**
///
/// ⚠️ A assimetria é deliberada: geometria inválida é obra perdida e tem de
/// parar o load; *quem estava em mãos* é conforto de sessão, e recusar o arquivo
/// inteiro por causa dele jogaria fora a escultura para melhorar um foco.
#[test]
fn an_out_of_range_active_index_is_clamped_not_refused() {
    let bytes = doc_bytes(&[piece(0.0), piece(3.0)], 0);
    let mut doc: SculptDoc = postcard::from_bytes(&bytes).expect("re-lê");
    doc.active = 99;
    let forged = postcard::to_allocvec(&doc).expect("serializa");

    let (pieces, active) = decode(&forged).expect("o arquivo continua bom");
    assert_eq!(pieces.len(), 2);
    assert_eq!(active, 1, "clampado à última peça");
}

/// **Um documento vazio é lido como cena vazia** — é o projeto que nunca
/// esculpiu, e ele não pode falhar em abrir.
///
/// ⚠️ E o `active` não pode estourar no `saturating_sub`: sem peça nenhuma não
/// há índice válido, e o zero é o único número que não mente.
#[test]
fn an_empty_document_reads_as_an_empty_scene() {
    let (pieces, active) = decode(&encode(&[], 0)).expect("documento válido");
    assert!(pieces.is_empty());
    assert_eq!(active, 0);
}
