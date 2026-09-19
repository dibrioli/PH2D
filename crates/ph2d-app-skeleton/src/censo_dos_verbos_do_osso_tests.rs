//! ⭐⭐⭐ **O CENSO DOS VERBOS DO OSSO: o clique CHEGA a um efeito?**
//!
//! ⛔⛔⛔ **É a segunda metade da pergunta que o `§5.0` nomeia sobre este repo inteiro** — *«nenhum
//! instrumento pergunta se o VALOR chega a um consumidor»*. O censo irmão
//! ([`crate::knobs::censo_dos_knobs_do_osso_tests`]) fechou-a para os **números** em 2026-09-18 e
//! deixou os **verbos** abertos por escrito: eles tinham censo de *chegam ao barramento* e nenhum de
//! *chegam a um efeito*.
//!
//! ⚠️ **A família é a de metade dos reports do dono nesta linha:** o botão pinta, acende sob o rato,
//! o clique atravessa o painel — e o mundo não se mexe. Do lado de fora isso é indistinguível de um
//! verbo que recusou, e o artista conclui que *a ferramenta* não funciona.
//!
//! # ⚠️ A régua é o PRODUTO, e a fotografia é a do UNDO
//!
//! Cada verbo é corrido pela **porta que a shell chama** sobre um palco montado para ele, e o que se
//! mede é a captura [`ph2d_ecs::scene::world_to_snapshot`] — a mesma que a fila do undo fotografa.
//! ⛔ *Uma régua que re-escrevesse a lei mediria outro programa*, e foi por isso que o
//! [`crate::smart::add`] teve de nascer: das catorze rotas ele era a única cujo efeito estava
//! escrito **dentro da fase do quadro**.
//!
//! # ⭐⭐ O que torna o censo honesto não é a corrida de hoje
//!
//! É o `match` **exaustivo** de [`VerboDoOsso`] em [`corre`]: um verbo novo **não compila** até
//! alguém dizer como se corre. *É a diferença entre uma lista que alguém tem de se lembrar de
//! estender e uma que não fica verde sem a extensão.*

use super::{Consumidor, VerboDoOsso, of_id};
use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot};
use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};
use ph2d_editor_core::ids;
use ph2d_preview_drive::PreviewDrive;
use ph2d_skeleton_ecs::Bone;
use ph2d_skeleton_live::skin_live::Keep;
use ph2d_timeline::TimelineDoc;
use ph2d_vec_scene::{ShapeKind, VecPathId, VecScene, cook};

/// O palco do censo: um braço de dois ossos com uma forma presa por cima, mais tudo o que as portas
/// dos catorze verbos pedem.
struct Palco {
    sim: SimWorld,
    cena: VecScene,
    mapa: ph2d_vec_entities::entities::VecEntityMap,
    caminho: VecPathId,
    reg: ComponentRegistry,
    doc: TimelineDoc,
    preview: PreviewDrive,
    /// `[raiz, ponta]` — os verbos de ramo agem na **raiz**, os de junta na **ponta**.
    ossos: [Entity; 2],
}

/// O registo com os componentes do esqueleto — a cópia profunda do espelho não copia o que ele não
/// conhece, e a captura não fotografa o que ele não descreve.
fn registo() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_skeleton_ecs::register_skeleton_components(&mut reg);
    reg
}

/// Um osso em `pos` (local do pai), comprimento `len`, alcance 1.
fn osso(sim: &mut SimWorld, nome: &str, pos: [f32; 2], len: f64, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(pos[0], pos[1]),
                ..Transform::IDENTITY
            },
            Name::new(nome),
            RootOrder(0),
            Bone {
                length: len,
                strength: 1.0,
                ..Bone::default()
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

/// ⚠️ **O braço DOBRA de propósito** (a ponta sobe): uma cadeia recta é simétrica, e o espelho sobre
/// ela devolveria a mesma geometria — *uma fixtura que não contém o fenómeno não prova que ele
/// aconteceu*, e foi essa a armadilha que o censo dos números pagou com o `Length` num osso recto.
fn palco() -> Palco {
    let mut sim = SimWorld::default();
    let mut cena = VecScene::new();
    let mut mapa = ph2d_vec_entities::entities::VecEntityMap::new();
    let caminho = cena.push_path(cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]));
    ph2d_vec_entities::entities::sync(&mut sim, &mut cena, &mut mapa);
    let raiz = osso(&mut sim, "Arm", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Forearm", [20.0, 0.0], 20.0, Some(raiz));
    sim.world_mut()
        .get_mut::<Transform>(ponta)
        .expect("Transform")
        .rotation = 0.4;
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    Palco {
        sim,
        cena,
        mapa,
        caminho,
        reg: registo(),
        doc: TimelineDoc::default(),
        preview: PreviewDrive::default(),
        ossos: [raiz, ponta],
    }
}

/// Prende a forma do palco aos ossos — a pré-condição de *Expand* e *Release*.
fn prende(p: &mut Palco) {
    let n = ph2d_skeleton_live::skin_live::bind(
        &mut p.sim,
        &p.cena,
        &p.mapa,
        &[p.caminho],
        Some(p.ossos[0]),
    );
    assert_eq!(
        n, 1,
        "a pre-condicao do palco falhou: a forma nao ficou presa"
    );
}

/// ⭐ **A FOTOGRAFIA** — a captura do mundo, que é a mesma que a fila do undo tira.
///
/// ⚠️ **A cena vectorial entra ao lado dela**, e não por gosto: o *Expand* escreve a geometria
/// deformada no **documento do vector**, que não é uma entidade. Sem esta metade, uma mutação que
/// fizesse o *Expand* chamar o *Release* ficava invisível — os dois tiram o `SkinBind`.
fn foto(p: &mut Palco) -> (WorldSnapshot, Vec<[f64; 2]>) {
    let mut prop = ph2d_ecs::TransformPropagationState::new(p.sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    ph2d_ecs::scene::world_to_snapshot(p.sim.world_mut(), &mut prop, &mut work, &p.reg, &mut snap)
        .expect("a captura do mundo");
    let geo = p
        .cena
        .path(p.caminho)
        .map(|c| c.verts_all().map(|v| v.anchor).collect())
        .unwrap_or_default();
    (snap, geo)
}

/// ⭐⭐⭐ **CORRE O VERBO pela porta que a shell chama** — e monta a pré-condição dele.
///
/// ⚠️ **A pré-condição é montada ANTES da fotografia** (ver [`mede`]): um verbo que tira alguma coisa
/// precisa que ela lá esteja, e montá-la depois da foto faria o censo medir o arranjo em vez do
/// verbo.
///
/// ⛔ **O `match` não tem `_ =>`, e é ele o gate:** um verbo novo não compila até alguém dizer como
/// se corre.
fn arma(p: &mut Palco, v: VerboDoOsso) {
    let [raiz, ponta] = p.ossos;
    match v {
        // ⚠️⚠️ **PRENDER, DOBRAR e RE-COZINHAR — nesta ordem, e a razão é medida.** O `bind`
        // guarda a pose de AGORA como repouso, logo prender e assar no mesmo instante devolve a
        // fonte: *o Expand não tem nada para assar num corpo que não saiu do repouso*, e a 1.ª
        // redacção desta fixtura leu os dois verbos como idênticos. E o `recook` é quem escreve a
        // geometria deformada no documento — ele corre **uma vez por quadro** no app, e sem ele o
        // desenho ainda é o autorado.
        VerboDoOsso::Assar | VerboDoOsso::Soltar => {
            prende(p);
            p.sim
                .world_mut()
                .get_mut::<Transform>(ponta)
                .expect("Transform")
                .rotation = 1.1;
            ph2d_skeleton_live::skin_live::recook(&p.sim, &mut p.cena);
        }
        // ⚠️ Repor um repouso que nunca foi guardado é um **no-op declarado** (a recusa
        // `SemPoseDeRepouso`), e a pose tem de estar FORA dele, senão repor não move nada.
        VerboDoOsso::ReporRepouso => {
            assert!(ph2d_skeleton_live::pose_de_repouso::guardar(&mut p.sim, raiz) > 0);
            p.sim
                .world_mut()
                .get_mut::<Transform>(ponta)
                .expect("Transform")
                .rotation = 1.1;
        }
        VerboDoOsso::AncoraTirar => {
            assert!(
                crate::goal::add(&mut p.sim, ponta).is_some(),
                "a ancora nasce"
            );
        }
        VerboDoOsso::LimiteTirar => {
            assert!(
                crate::bone_limit::add_limit(&mut p.sim, ponta),
                "o limite nasce"
            );
        }
        VerboDoOsso::InteligenteTirar => {
            assert!(crate::smart::add(&mut p.sim, ponta), "o controlo nasce");
        }
        VerboDoOsso::Espelhar
        | VerboDoOsso::Apontar
        | VerboDoOsso::Prender
        | VerboDoOsso::GuardarRepouso
        | VerboDoOsso::AncoraPor
        | VerboDoOsso::LimitePor
        | VerboDoOsso::InteligentePor
        | VerboDoOsso::InteligenteEscolherAlvo => {}
    }
}

/// ⭐⭐⭐ **O verbo, pela porta do produto.** Ver [`arma`] para as pré-condições.
fn corre(p: &mut Palco, v: VerboDoOsso) {
    let [raiz, ponta] = p.ossos;
    match v {
        VerboDoOsso::Espelhar => {
            ph2d_skeleton_live::espelho::espelha(&mut p.sim, &p.reg, raiz);
        }
        VerboDoOsso::Apontar => {
            ph2d_skeleton_live::goal::add_look_at(&mut p.sim, ponta);
        }
        VerboDoOsso::Prender => {
            ph2d_skeleton_live::skin_live::bind(
                &mut p.sim,
                &p.cena,
                &p.mapa,
                &[p.caminho],
                Some(raiz),
            );
        }
        VerboDoOsso::Assar | VerboDoOsso::Soltar => {
            let keep = if v == VerboDoOsso::Assar {
                Keep::Deformed
            } else {
                Keep::Source
            };
            ph2d_skeleton_live::skin_live::release(
                &mut p.sim,
                &mut p.cena,
                &p.mapa,
                &[p.caminho],
                keep,
            );
        }
        VerboDoOsso::ReporRepouso => {
            ph2d_skeleton_live::pose_de_repouso::repor(&mut p.sim, raiz);
        }
        VerboDoOsso::GuardarRepouso => {
            ph2d_skeleton_live::pose_de_repouso::guardar(&mut p.sim, raiz);
        }
        VerboDoOsso::AncoraPor => {
            crate::goal::add(&mut p.sim, ponta);
        }
        VerboDoOsso::AncoraTirar => {
            crate::goal::remove(&mut p.sim, ponta, &mut p.preview);
        }
        VerboDoOsso::LimitePor => {
            crate::bone_limit::add_limit(&mut p.sim, ponta);
        }
        VerboDoOsso::LimiteTirar => {
            crate::bone_limit::remove_limit(&mut p.sim, ponta);
        }
        VerboDoOsso::InteligentePor => {
            crate::smart::add(&mut p.sim, ponta);
        }
        VerboDoOsso::InteligenteTirar => {
            let doc = std::mem::take(&mut p.doc);
            crate::smart::remove(&mut p.sim, &doc, ponta, &mut p.preview);
            p.doc = doc;
        }
        // ⛔ O consumidor dele é o CLIQUE SEGUINTE, e ele não toca no mundo — ver [`Consumidor::Modo`].
        VerboDoOsso::InteligenteEscolherAlvo => {}
    }
}

/// Monta, fotografa, corre, fotografa — e devolve se o mundo (ou o desenho) se mexeu.
fn mede(v: VerboDoOsso) -> bool {
    let mut p = palco();
    arma(&mut p, v);
    let antes = foto(&mut p);
    corre(&mut p, v);
    foto(&mut p) != antes
}

/// ⭐⭐⭐ **TODO VERBO DO OSSO CHEGA A UM EFEITO — menos UM, que declara o consumidor dele.**
#[test]
fn todo_verbo_do_osso_chega_a_um_efeito() {
    let mut mortos = Vec::new();
    let mut medidos = 0;
    for v in VerboDoOsso::TODOS {
        if v.consumidor() == Consumidor::Modo {
            continue;
        }
        medidos += 1;
        if !mede(v) {
            mortos.push(v);
        }
    }
    assert_eq!(
        medidos, 13,
        "o censo mediu {medidos} verbos de 13 — um verbo novo entrou sem corrida, ou alguem \
         declarou `Consumidor::Modo` para escapar a esta regua"
    );
    assert!(
        mortos.is_empty(),
        "estes botoes da seccao do osso nao mexem UM BYTE do mundo: {mortos:?} — o painel promete \
         um verbo que a cena nao sente, e do lado de fora isso le'-se como «a ferramenta nao \
         funciona»"
    );
}

/// ⭐⭐ **O CONTROLO NEGATIVO** — sem correr verbo nenhum, a fotografia é a MESMA.
///
/// ⛔ Sem ele a régua seria vácuo: se a captura variasse sozinha (uma ordem instável, um relógio, um
/// id de alocação), os treze liam-se **vivos** e o censo nunca mais acusaria nada.
#[test]
fn sem_verbo_nenhum_a_fotografia_nao_se_mexe() {
    for v in VerboDoOsso::TODOS {
        let mut p = palco();
        arma(&mut p, v);
        let antes = foto(&mut p);
        assert!(
            foto(&mut p) == antes,
            "a fotografia do palco de {v:?} MUDOU sem ninguem correr o verbo — a regua deste censo \
             nao mede o verbo, mede ruido"
        );
    }
}

/// ⭐⭐⭐ **ASSAR E SOLTAR NÃO SÃO O MESMO VERBO** — o discriminador que a fotografia do mundo não dá.
///
/// Os dois tiram o `SkinBind`, logo os dois mexem no mundo e o censo acima ficaria verde se um
/// chamasse o outro. ⚠️ **O que os separa vive no DOCUMENTO DO VECTOR:** o *Expand* escreve a
/// geometria deformada no desenho; o *Release* devolve a fonte autorada.
#[test]
fn assar_e_soltar_nao_sao_o_mesmo_verbo() {
    let geo = |v| {
        let mut p = palco();
        arma(&mut p, v);
        corre(&mut p, v);
        foto(&mut p).1
    };
    let assado = geo(VerboDoOsso::Assar);
    let solto = geo(VerboDoOsso::Soltar);
    assert_ne!(
        assado, solto,
        "o Expand e o Release devolveram o MESMO desenho — ou o palco nao esta' deformado (e entao \
         nao ha' nada para assar), ou um dos dois verbos esta' a chamar o outro"
    );
}

/// ⭐⭐⭐ **CADA ID É O VERBO QUE O NOME DIZ** — o controlo que paga a derivação por POSIÇÃO.
///
/// ⛔⛔ O [`of_id`] resolve pela posição na [`ids::VECTOR_BONE_VERBS`], que é o que impede uma segunda
/// lista de catorze braços. O preço é que **trocar dois itens da tabela** faria o botão que diz
/// *Bind* mandar *Release*, e nada acusaria — *um botão que faz o contrário do que diz é pior do que
/// um morto*. Esta tabela, escrita por NOME, é o que o torna observável.
#[test]
fn cada_id_da_tabela_e_o_verbo_que_o_nome_diz() {
    let pares = [
        (ids::VECTOR_BONE_MIRROR, VerboDoOsso::Espelhar),
        (ids::VECTOR_BONE_LOOK_AT, VerboDoOsso::Apontar),
        (ids::VECTOR_BONE_BIND, VerboDoOsso::Prender),
        (ids::VECTOR_BONE_EXPAND, VerboDoOsso::Assar),
        (ids::VECTOR_BONE_RELEASE, VerboDoOsso::Soltar),
        (ids::VECTOR_BONE_REST_APPLY, VerboDoOsso::ReporRepouso),
        (ids::VECTOR_BONE_REST_SET, VerboDoOsso::GuardarRepouso),
        (ids::VECTOR_BONE_IK_ADD, VerboDoOsso::AncoraPor),
        (ids::VECTOR_BONE_IK_REMOVE, VerboDoOsso::AncoraTirar),
        (ids::VECTOR_BONE_LIMIT_ADD, VerboDoOsso::LimitePor),
        (ids::VECTOR_BONE_LIMIT_REMOVE, VerboDoOsso::LimiteTirar),
        (ids::VECTOR_BONE_SMART_ADD, VerboDoOsso::InteligentePor),
        (ids::VECTOR_BONE_SMART_REMOVE, VerboDoOsso::InteligenteTirar),
        (
            ids::VECTOR_BONE_SMART_PICK,
            VerboDoOsso::InteligenteEscolherAlvo,
        ),
    ];
    assert_eq!(
        pares.len(),
        ids::VECTOR_BONE_VERBS.len(),
        "a tabela de ids cresceu e este controlo nao — o verbo novo fica sem quem pine o nome dele \
         ao significado"
    );
    for (id, esperado) in pares {
        assert_eq!(
            of_id(id),
            Some(esperado),
            "o id do {esperado:?} resolve para outro verbo — a ordem da VECTOR_BONE_VERBS e a da \
             VerboDoOsso::TODOS divergiram, e o botao passa a fazer o que o vizinho dele diz"
        );
    }
    assert_eq!(
        of_id(ids::VECTOR_BONE_LENGTH),
        None,
        "um id que NAO e' verbo desta seccao tem de cair fora — um `_ =>` mandaria todo clique \
         desconhecido espelhar um ramo"
    );
}

/// ⭐ **A lista `TODOS` cobre o enum** — o molde do censo dos números, com as duas metades que uma
/// mutação já exigiu lá: **distintos**, e não só a contagem.
#[test]
fn a_lista_todos_cobre_o_enum() {
    for v in VerboDoOsso::TODOS {
        // ⚠️ O `match` sem `_ =>` é o gate: uma variante nova **não compila** aqui.
        match v {
            VerboDoOsso::Espelhar
            | VerboDoOsso::Apontar
            | VerboDoOsso::Prender
            | VerboDoOsso::Assar
            | VerboDoOsso::Soltar
            | VerboDoOsso::ReporRepouso
            | VerboDoOsso::GuardarRepouso
            | VerboDoOsso::AncoraPor
            | VerboDoOsso::AncoraTirar
            | VerboDoOsso::LimitePor
            | VerboDoOsso::LimiteTirar
            | VerboDoOsso::InteligentePor
            | VerboDoOsso::InteligenteTirar
            | VerboDoOsso::InteligenteEscolherAlvo => {}
        }
    }
    let distintos: std::collections::BTreeSet<String> = VerboDoOsso::TODOS
        .into_iter()
        .map(|v| format!("{v:?}"))
        .collect();
    assert_eq!(
        distintos.len(),
        VerboDoOsso::TODOS.len(),
        "a lista TODOS tem uma variante REPETIDA: outra ficou de fora, e o censo mede-a zero vezes \
         sem reprovar"
    );
    assert_eq!(
        VerboDoOsso::TODOS.len(),
        ids::VECTOR_BONE_VERBS.len(),
        "a populacao do enum e a da tabela de ids divergiram — o `of_id` passa a devolver `None` \
         para o ultimo botao, e ele morre em silencio"
    );
    assert_eq!(
        VerboDoOsso::TODOS
            .into_iter()
            .filter(|v| v.consumidor() == Consumidor::Modo)
            .count(),
        1,
        "o numero de verbos ISENTOS da regua do mundo mudou — uma isencao nova e' uma decisao, e \
         ela escreve-se com a medicao ao lado (ver `Consumidor::Modo`)"
    );
}
