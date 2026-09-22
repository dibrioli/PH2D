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
    let data: Vec<(StackData, PoseData, Option<&ph2d_mesh_colors::Tinta>)> = pieces
        .iter()
        .map(|(s, p)| (s.to_data(), p.to_data(), None))
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
    for (i, ((s0, p0), lida)) in src.iter().zip(back.iter()).enumerate() {
        let (s1, p1) = (&lida.stack, &lida.pose);
        assert_eq!(s1.level_count(), s0.level_count(), "níveis da peça {i}");
        assert_eq!(s1.level(), s0.level(), "o nível em mãos da peça {i}");
        assert!(
            (s1.mesh().positions()[0][1] - s0.mesh().positions()[0][1]).abs() < 1e-6,
            "o trabalho fino da peça {i}"
        );
        assert_eq!(p1.translation, p0.translation, "posição da peça {i}");
        assert!((p1.scale() - p0.scale()).abs() < 1e-6, "escala da peça {i}");
        assert!(
            lida.tinta.is_none(),
            "o CONTROLO: esta fixtura não tem plano, e um `Some` aqui diria que \
             o decode o INVENTA"
        );
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

    let bytes = encode(&[(data, pose.to_data(), None)], 0);
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
        // | plano de tinta fina: `None` (2026-09-21, `SCULPT_DOC_VERSION` 2) | 1 |
        // | **total** | **1539** |
        //
        // ⚠️ Cada face custa **8** bytes e não 4: a `Face` é `[u32; 4]`, e o quarto índice de um
        // triângulo é o sentinela `TRI`, que em varint custa 5.
        //
        // ⭐ **O degrau de `1538` para `1539` é UM byte e a conta fecha:** o campo `tinta` é um
        // `Option` e um `None` custa exactamente o discriminante. A VERSÃO não muda nada — `1` e
        // `2` são ambos um varint de um byte.
        1539,
        "a forma serializada da cena mudou — suba SCULPT_DOC_VERSION, não re-pine este número"
    );
    let (back, _) = decode(&bytes).expect("ida e volta");
    assert_eq!(back.len(), 1, "a peça voltou");
}

/// Uma peça com PLANO de tinta fina, pintado num sítio e não noutro.
fn peca_com_plano(nivel: u8) -> (Multires, Pose, ph2d_mesh_colors::Tinta) {
    let mut stack = Multires::new(shapes::octahedron(1.0));
    // a cor POR VÉRTICE, que viaja dentro do `stack` e de que a semente sai
    for i in 0..stack.mesh().vert_count() {
        stack.mesh_mut().colors_mut()[i] = [0.2, 0.4, 0.6];
    }
    let m = stack.mesh();
    let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
    let mut t = ph2d_mesh_colors::Tinta::semeada(m.colors().unwrap(), faces(), nivel);
    // e a MANCHA: um pedaço contíguo com outra cor, que é a forma de um traço
    let n = t.amostras().len();
    for i in (n / 4)..(n / 3) {
        t.amostras_mut()[i] = [0.9, 0.1, 0.05];
    }
    (stack, Pose::new([1.0, 0.0, 0.0], 1.5), t)
}

/// ⭐⭐⭐⭐ **GATE — O PLANO DE TINTA FINA ATRAVESSA O FICHEIRO, AO BIT.**
///
/// ⚠️ **E a metade que importa é o CONTROLO:** o plano é re-semeado da cor por
/// vértice quando ele não existe, e a cor por vértice viaja ao lado. Sem a
/// asserção de que a MANCHA voltou, um `decode` que simplesmente re-semeasse
/// passaria — *e o artista veria exactamente o que ele vê hoje: a tinta a
/// voltar à resolução da malha*.
#[test]
fn o_plano_de_tinta_fina_atravessa_o_ficheiro_ao_bit() {
    let (stack, pose, t) = peca_com_plano(2);
    let semente = {
        let m = stack.mesh();
        let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
        ph2d_mesh_colors::Tinta::semeada(m.colors().unwrap(), faces(), 2)
    };
    assert_ne!(
        t.amostras(),
        semente.amostras(),
        "o CONTROLO da fixtura: sem uma mancha, re-semear devolveria o mesmo e \
         este gate ficava VÁCUO"
    );

    let bytes = encode(&[(stack.to_data(), pose.to_data(), Some(&t))], 0);
    let (lidas, _) = decode(&bytes).expect("ida e volta");
    let volta = lidas[0].tinta.as_ref().expect("a peça tinha plano");

    assert_eq!(volta.nivel(), t.nivel(), "o degrau do plano");
    assert_eq!(
        volta.amostras().len(),
        t.amostras().len(),
        "a contagem de amostras — a topologia é DERIVADA da malha lida"
    );
    let bits = |a: &[[f32; 3]]| -> Vec<[u32; 3]> {
        a.iter()
            .map(|c| [c[0].to_bits(), c[1].to_bits(), c[2].to_bits()])
            .collect()
    };
    assert_eq!(
        bits(volta.amostras()),
        bits(t.amostras()),
        "as amostras não voltaram AO BIT"
    );
}

/// ⭐⭐⭐ **GATE — UM DOCUMENTO v1 ABRE, e as peças vêm sem plano.**
///
/// ⛔⛔ **Sem a migração, todo `.ph2dproj` que o dono já gravou deixaria de
/// abrir** — o `decode` recusaria por versão, e a recusa leva o load inteiro
/// (a lei declarada no cabeçalho deste módulo). *Subir a versão de um formato
/// sem degrau é apagar o trabalho de quem já o usou.*
///
/// ⚠️ A fixtura escreve os bytes v1 **pela forma congelada**, não por um
/// ficheiro guardado: um golden binário aqui envelheceria com o `StackData`, e
/// o que este gate afirma é a MIGRAÇÃO, não a geometria.
#[test]
fn um_documento_da_versao_anterior_abre_e_vem_sem_plano() {
    #[derive(serde::Serialize)]
    struct ObjectV1 {
        stack: StackData,
        pose: PoseData,
    }
    #[derive(serde::Serialize)]
    struct DocV1 {
        version: u32,
        objects: Vec<ObjectV1>,
        active: u32,
    }
    let (stack, pose) = piece(1.0);
    let bytes = postcard::to_allocvec(&DocV1 {
        version: 1,
        objects: vec![ObjectV1 {
            stack: stack.to_data(),
            pose: pose.to_data(),
        }],
        active: 0,
    })
    .expect("serializa");

    let (lidas, active) = decode(&bytes).expect("um v1 tem de ABRIR");
    assert_eq!(lidas.len(), 1, "a peça do v1");
    assert_eq!(active, 0);
    assert_eq!(
        lidas[0].stack.level_count(),
        stack.level_count(),
        "a pilha do v1 tem de atravessar a migração inteira"
    );
    assert!(
        lidas[0].tinta.is_none(),
        "um v1 não tinha plano, e inventar um seria escrever trabalho que ninguém fez"
    );
}

/// ⛔⛔ **GATE — um plano que não descreve a malha RECUSA o load, e diz qual
/// peça.**
///
/// A mesma lei da geometria: abrir *sem* ele mostraria a peça com a tinta na
/// resolução da MALHA — que é o que o artista vê quando perde o detalhe fino —
/// e o **próximo Ctrl+S gravaria essa perda por cima**.
///
/// ⛔⛔⛔ **E ele corre sobre AS DUAS FORMAS, porque a 1.ª redacção deixou uma
/// mutação SOBREVIVER.** A fixtura semeada escolhe sempre CORRIDAS, logo a
/// cerca da contagem da forma CRUA nunca era exercida e apagá-la não mudava um
/// bit. *Uma sobrevivente por falta de corpus constrói-se, não se nomeia* — e o
/// corpus que faltava é um plano de amostras TODAS DISTINTAS, onde o escritor
/// escolhe a outra forma.
///
/// ⚠️ **Cada célula AFIRMA a forma que tomou**, senão o dia em que a conta do
/// escritor mudar leva as duas a medir a mesma metade, em silêncio.
#[test]
fn um_plano_que_nao_descreve_a_malha_recusa_o_load() {
    use super::doc_tinta::AmostrasDoc;

    let (stack, pose, junto) = peca_com_plano(2);
    let distinto = {
        let m = stack.mesh();
        let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
        let mut t = ph2d_mesh_colors::Tinta::nova(m.vert_count(), faces(), 2);
        let n = t.amostras().len();
        for i in 0..n {
            t.amostras_mut()[i] = [i as f32, -(i as f32), 0.5];
        }
        t
    };

    for (nome, t, corridas_esperadas) in [
        ("uma mancha num plano semeado", &junto, true),
        ("amostras todas distintas", &distinto, false),
    ] {
        let bytes = encode(&[(stack.to_data(), pose.to_data(), Some(t))], 0);
        assert!(decode(&bytes).is_ok(), "{nome}: o CONTROLO, assim ele abre");

        let mut doc: SculptDoc = postcard::from_bytes(&bytes).expect("re-lê");
        let amostras = &mut doc.objects[0]
            .tinta
            .as_mut()
            .expect("a fixtura tem plano")
            .amostras;
        assert_eq!(
            matches!(amostras, AmostrasDoc::Corridas(_)),
            corridas_esperadas,
            "{nome}: a fixtura deixou de tomar a forma que este caso existe \
             para exercer — as duas células passariam a medir a MESMA metade"
        );
        match amostras {
            AmostrasDoc::Corridas(c) => c.push((1, [0.0, 0.0, 0.0])),
            AmostrasDoc::Cruas(v) => v.push([0.0, 0.0, 0.0]),
        }
        let forjado = postcard::to_allocvec(&doc).expect("serializa");

        match decode(&forjado) {
            Err(SculptDocError::Tinta { peca, esperadas }) => {
                assert_eq!(peca, 0, "{nome}: a recusa tem de NOMEAR a peça");
                assert_eq!(
                    esperadas,
                    t.amostras().len(),
                    "{nome}: e dizer quantas amostras a malha pede"
                );
            }
            other => panic!("{nome}: tinha de ser recusado, e veio {other:?}"),
        }
    }
}

/// ⭐⭐⭐⭐ **GATE — AS DUAS FORMAS, e cada metade mede uma delas.**
///
/// ⛔⛔ **A 1.ª redacção deste gate tinha a barra do desenho ERRADO.** Ela
/// exigia `< 1/50` sobre uma fixtura **semeada**, e a medição de 2026-09-21
/// diz que uma semente de cor chapada não é chapada nos BITS — a interpolação
/// baricêntrica soma `1` com erro de último bit. A tabela medida está no
/// cabeçalho do [`super::doc_tinta`]; o que ficou foram as duas metades que ela
/// de facto sustenta.
#[test]
fn as_duas_formas_das_amostras_fazem_o_que_prometem() {
    let (stack, pose, _) = peca_com_plano(3);
    let m = stack.mesh();
    let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
    let sem = encode(&[(stack.to_data(), pose.to_data(), None)], 0).len();
    let custo = |t: &ph2d_mesh_colors::Tinta| {
        encode(&[(stack.to_data(), pose.to_data(), Some(t))], 0).len() - sem
    };

    // (a) ⭐ O plano de uma peça NUNCA PINTADA cabe em nada. É o caso do
    //     artista que acabou de armar o degrau, e é o que tira `75 MB` de cima
    //     de um `Ctrl+S` na peça de fábrica.
    let nova = ph2d_mesh_colors::Tinta::nova(m.vert_count(), faces(), 3);
    let cru = nova.amostras().len() * 12;
    let a = custo(&nova);
    assert!(
        a * 50 < cru,
        "um plano por pintar custou {a} B contra {cru} B crus — as corridas \
         deixaram de juntar, e armar o degrau volta a custar o plano inteiro"
    );

    // (b) ⛔ E o plano de uma peça com cor VARIADA nunca custa MAIS que cru.
    //     É isto que a segunda forma compra: sem ela a medição lê `1,083×`,
    //     porque cada corrida de uma amostra paga o varian­te a mais.
    let n = m.vert_count();
    let variadas: Vec<[f32; 3]> = (0..n).map(|i| [i as f32 / n as f32, 0.4, 0.6]).collect();
    let semeada = ph2d_mesh_colors::Tinta::semeada(&variadas, faces(), 3);
    let cru = semeada.amostras().len() * 12;
    let b = custo(&semeada);
    assert!(
        b <= cru + 16,
        "um plano de amostras todas distintas custou {b} B contra {cru} B crus: \
         o escritor deixou de escolher a forma MENOR"
    );
    assert!(
        b * 100 > cru * 90,
        "o CONTROLO: neste caso a forma CRUA tem de ganhar, e {b} B contra \
         {cru} B diz que as corridas juntaram — a fixtura deixou de ter \
         amostras distintas e a metade (b) mede outra coisa"
    );
}
