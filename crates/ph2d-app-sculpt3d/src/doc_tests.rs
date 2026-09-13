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

/// **HR-14 — a FORMA do documento salvo é PINADA** — o gate que o doc do
/// [`SCULPT_DOC_VERSION`] prometia.
///
/// ⛔ **Ele era NOMEADO e NÃO EXISTIA** (medido 2026-09-13: nenhum commit no
/// histórico do git o escreveu). O doc dizia que ele *«transforma "lembre-se" em
/// vermelho»*, e sem ele o «lembre-se» era tudo o que havia entre um campo novo
/// na `ph2d-mesh` e o arquivo do artista lido como lixo bem-formado — o postcard
/// é posicional e não avisa.
///
/// ⚠️ **A fixtura instancia TODO campo opcional do blob** (cor e máscara no nível
/// e no detalhe, dois níveis, pose fora da origem) e AFIRMA-o antes de medir: um
/// `Option` a `None` custa um byte de discriminante e deixa o golden cego a tudo
/// o que mora dentro do `Some` — a lição que a `ph2d-field` pagou por três
/// degraus (`the_shape_of_a_saved_modifier_stack_is_pinned`).
///
/// Quebrou? **Suba o `SCULPT_DOC_VERSION`** e só então re-pine, com a conta do
/// degrau escrita ao lado. Re-pinar para seguir é apagar a prova.
#[test]
fn the_shape_of_a_saved_scene_is_pinned() {
    let mut stack = Multires::new(shapes::octahedron(1.0));
    stack.mesh_mut().colors_mut()[0] = [0.25, 0.5, 0.75];
    stack.mesh_mut().masks_mut()[1] = 0.5;
    assert!(stack.add_level(), "a fixtura precisa do 2º nível");
    stack.mesh_mut().positions_mut()[0][1] += 0.25;
    stack.mesh_mut().colors_mut()[2] = [0.75, 0.5, 0.25];
    stack.mesh_mut().masks_mut()[3] = 0.25;
    // ⚠️ O detalhe só guarda cor e máscara quando é RE-ENCODADO — um nível acabado de nascer
    // tem `colors: None` por construção (`Multires::add_level`). Descer e voltar a subir é o
    // gesto do artista que o escreve, e é o estado que um arquivo real carrega.
    assert!(stack.lower().is_some(), "a fixtura desce ao nível 0");
    assert!(stack.higher(), "e volta ao nível 1");
    let pose = Pose::new([1.5, -2.0, 0.5], 2.0);
    let data = stack.to_data();

    // ⛔ Controlo: a fixtura instancia o que o golden diz defender.
    assert!(data.levels.len() >= 2, "a fixtura perdeu o 2º nível");
    for (i, nivel) in data.levels.iter().enumerate() {
        assert!(
            nivel.colors.is_some() && nivel.masks.is_some(),
            "o nível {i} da fixtura não traz cor e máscara — o golden ficaria cego a elas"
        );
    }
    assert!(
        data.details
            .iter()
            .skip(1)
            .all(|d| d.colors.is_some() && d.masks.is_some()),
        "o detalhe da fixtura não traz cor e máscara — o golden ficaria cego a elas"
    );

    let bytes = encode(&[(data, pose.to_data())], 0);
    assert_eq!(
        bytes.len(),
        // ⚠️ MEDIDO na criação do gate (2026-09-13), e a conta FECHA à mão — que é o que separa
        // um golden de um número copiado da saída:
        //
        // | pedaço | bytes |
        // |---|---:|
        // | versão + nº de peças + peça activa (varints) | 3 |
        // | nível 0: 6 posições (`f32` fixo) · 8 faces · cor e máscara `Some` | 73 + 65 + 74 + 26 = 238 |
        // | nível 1: 18 posições · 32 faces · cor e máscara `Some` | 217 + 257 + 218 + 74 = 766 |
        // | detalhe 0 (vazio: comprimento + dois `None`) · detalhe 1 (18 `xyz` + cor + máscara) | 3 + 509 |
        // | nº de níveis + nº de detalhes + nível em mãos | 3 |
        // | pose: translação + escala (`f32`) | 16 |
        // | **total** | **1538** |
        //
        // ⚠️ Cada face custa **8** bytes e não 4: a `Face` é `[u32; 4]`, e o quarto índice de um
        // triângulo é o sentinela `TRI`, que em varint custa 5.
        1538,
        "a forma serializada da cena mudou — suba SCULPT_DOC_VERSION, não re-pine este número"
    );
    let (back, _) = decode(&bytes).expect("ida e volta");
    assert_eq!(back.len(), 1, "a peça voltou");
}
