//! Os gates dos **OSSOS INTELIGENTES**.
//!
//! A LEI do mapeamento (ângulo → tempo de acção) está gateada em `ph2d-skeleton`. Aqui mede-se o que
//! só existe com um mundo ECS **e** um documento de timeline: que girar o controlo move o que a
//! acção anima, que o que ele escreve é **pré-visualização**, e que um controlo sem acção é inerte.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, RootOrder};
use ph2d_timeline::PropKind;

/// A faixa do controlo na cena de teste: de `0` a um quarto de volta.
pub(super) const ATE: f64 = std::f64::consts::FRAC_PI_2;

/// Um mundo com um osso de CONTROLO e um objecto que a acção desloca em X, de `0` a `10`.
///
/// ⚠️ O clip é montado pela porta do PRODUTO (`add_clip` + `insert_key`), que é quem cria a binding
/// — uma fixtura que montasse as tracks à mão mediria outro programa.
/// ⚠️ `pub(super)` porque o irmão [`super::list_tests`] corre sobre a MESMA cena — duas fixturas
/// para o mesmo módulo divergiriam.
pub(super) fn cena() -> (SimWorld, TimelineDoc, Entity, Entity) {
    let mut sim = SimWorld::default();
    let controlo = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Control"),
            RootOrder(0),
            ph2d_skeleton_ecs::Bone {
                length: 10.0,
                strength: 1.0,
            },
        ))
        .id();
    let movido = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Driven"), RootOrder(1)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    let mut doc = TimelineDoc::default();
    let i = doc.add_clip("Correction".into());
    doc.set_active(i);
    for (t, v) in [(0.0, 0.0_f32), (2.0, 10.0)] {
        doc.insert_key(
            movido.to_bits(),
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    // A binding nasce sem `entity` resolvida (ela é runtime, reconstruída do `wire_id` no load).
    for b in doc.bindings_mut() {
        b.entity = movido.to_bits();
        b.missing = false;
    }
    // ⚠️⚠️ **A acção fica FECHADA, e é load-bearing:** um controlo não percorre o clip que está
    // ABERTO (ali o artista está a gravá-lo). O `insert_key` escreve no clip activo, então a
    // fixtura tem de o abrir para gravar e **fechá-lo** para medir — sem esta linha os gates
    // abaixo medem um controlo que a lei manda estar inerte.
    doc.set_active(0);
    (sim, doc, controlo, movido)
}

pub(super) fn liga(sim: &mut SimWorld, e: Entity, clip: &str) {
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_skeleton_ecs::SmartBone {
            clip: clip.into(),
            target: String::new(),
            from: 0.0,
            to: ATE,
        });
}

fn x(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .expect("tem pose")
        .translation
        .x
}

/// ⭐⭐⭐ **GIRAR O CONTROLO PERCORRE A ACÇÃO** — a razão de existir da coisa.
#[test]
fn turning_the_control_bone_runs_the_action() {
    let (mut sim, doc, controlo, movido) = cena();
    liga(&mut sim, controlo, "Correction");
    let mut pv = PreviewDrive::default();

    // No princípio da faixa, a acção está no princípio.
    drive(&mut sim, &doc, &mut pv);
    assert!(
        (x(&sim, movido) - 0.0).abs() < 1e-4,
        "no princípio: {}",
        x(&sim, movido)
    );

    // A meio da faixa, a meio da acção.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
        #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
        {
            t.rotation = (ATE / 2.0) as f32;
        }
    }
    drive(&mut sim, &doc, &mut pv);
    assert!(
        (x(&sim, movido) - 5.0).abs() < 1e-3,
        "a meio: {}",
        x(&sim, movido)
    );

    // E no fim, no fim.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
        #[expect(clippy::cast_possible_truncation, reason = "idem")]
        {
            t.rotation = ATE as f32;
        }
    }
    drive(&mut sim, &doc, &mut pv);
    assert!(
        (x(&sim, movido) - 10.0).abs() < 1e-3,
        "no fim: {}",
        x(&sim, movido)
    );
}

/// ⭐⭐⭐ **O QUE A ACÇÃO ESCREVE É PRÉ-VISUALIZAÇÃO** — vê-se, não se guarda nem se desfaz.
///
/// ⚠️ Sem isto, cada clique com um controlo fora do repouso empilharia um passo de undo cujo
/// conteúdo é *«a acção correu»*. É a mesma lei da âncora de IK, e o ledger é o MESMO.
#[test]
fn what_the_action_writes_is_preview_not_document() {
    let (mut sim, doc, controlo, movido) = cena();
    liga(&mut sim, controlo, "Correction");
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
        #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
        {
            t.rotation = ATE as f32;
        }
    }
    let mut pv = PreviewDrive::default();
    // ⚠️ **Cinco quadros com a `settle` de cada um**, e não um só: um controlo em repouso escreve o
    // MESMO valor todo quadro, e é a partir do 2.º que a promoção a documento acontecia.
    for _ in 0..5 {
        drive(&mut sim, &doc, &mut pv);
        pv.settle();
    }
    assert!((x(&sim, movido) - 10.0).abs() < 1e-3, "a acção correu");
    // ⭐ O ledger tem de saber repor o AUTORADO (a pose de repouso), que é o que a fotografia usa.
    assert!(
        !pv.is_empty(),
        "o ledger ficou vazio — o que a acção escreveu viraria documento"
    );
}

/// ⛔ **UM CONTROLO SEM ACÇÃO É INERTE, e um que nomeia uma acção que não existe também.**
///
/// ⚠️ A segunda metade é o caso real: o artista renomeia ou apaga o clip. Inventar um índice poria
/// o osso a percorrer a animação do vizinho — **em silêncio**, que é o pior modo de falha.
#[test]
fn a_control_without_an_action_does_nothing_at_all() {
    for nome in ["", "Nao Existe"] {
        let (mut sim, doc, controlo, movido) = cena();
        liga(&mut sim, controlo, nome);
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
            #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
            {
                t.rotation = ATE as f32;
            }
        }
        let mut pv = PreviewDrive::default();
        let feitas = drive(&mut sim, &doc, &mut pv);
        assert_eq!(feitas, 0, "com clip {nome:?} escreveu-se alguma coisa");
        assert!(
            (x(&sim, movido) - 0.0).abs() < 1e-9,
            "com clip {nome:?} o objecto mexeu-se para {}",
            x(&sim, movido)
        );
    }
}

/// ⭐ **SEM CONTROLO NENHUM, O PASSE SAI CEDO** — a guarda que impede toda cena de pagar uma
/// varredura do mundo por quadro. É a mesma lei que o passe da âncora já segue.
#[test]
fn a_scene_without_smart_bones_pays_nothing() {
    let (mut sim, doc, _, movido) = cena();
    let mut pv = PreviewDrive::default();
    assert_eq!(drive(&mut sim, &doc, &mut pv), 0);
    assert!(
        pv.is_empty(),
        "sem controlo nenhum o ledger tem de ficar vazio"
    );
    assert!((x(&sim, movido) - 0.0).abs() < 1e-9);
}

/// ⭐⭐⭐ **UM CONTROLO NÃO PERCORRE A ACÇÃO QUE ESTÁ ABERTA NA TIMELINE.**
///
/// ⚠️⚠️ **Não é uma nicety — é o que torna a feature usável.** O artista grava a acção na timeline; se o controlo a percorresse
/// ao mesmo tempo, os dois escreveriam o mesmo objecto no mesmo quadro e o controlo corre DEPOIS do
/// passe da timeline ⇒ arrastar o playhead não moveria nada e a pose acabada de pôr seria reposta
/// antes de ser vista. ⚠️ E o autokey corre **ainda mais tarde**: com o objecto seleccionado ele
/// leria a saída do controlo como *«o artista mexeu»*. É a lei do Moho — dentro de uma acção o
/// relógio é o do editor.
#[test]
fn a_control_stands_down_on_the_action_that_is_open_for_editing() {
    let (mut sim, mut doc, controlo, movido) = cena();
    liga(&mut sim, controlo, "Correction");
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
        #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
        {
            t.rotation = ATE as f32;
        }
    }
    let mut pv = PreviewDrive::default();

    // FECHADA: ela corre.
    assert!(
        drive(&mut sim, &doc, &mut pv) > 0,
        "com a acção FECHADA o controlo tem de a percorrer"
    );
    let a_correr = x(&sim, movido);
    assert!((a_correr - 10.0).abs() < 1e-3, "o controlo: {a_correr}");

    // ABERTA para edição: ele fica quieto, e a pose que o artista pôs sobrevive ao quadro.
    let i = doc
        .clips()
        .iter()
        .position(|c| c.name == "Correction")
        .expect("a acção da fixtura");
    doc.set_active(i);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(movido) {
        t.translation.x = 3.0;
    }
    let mut pv = PreviewDrive::default();
    assert_eq!(
        drive(&mut sim, &doc, &mut pv),
        0,
        "o controlo percorreu a acção que está ABERTA — o artista não conseguiria gravá-la"
    );
    assert!(
        (x(&sim, movido) - 3.0).abs() < 1e-9,
        "a pose que o artista pôs foi reposta pelo ângulo do osso ({}) enquanto a acção estava em \
         edição",
        x(&sim, movido)
    );
}

/// ⭐⭐⭐ **A CENA DE SMOKE TRAZ UMA ACÇÃO COM CONTEÚDO, E ELA NÃO FICA ABERTA.**
///
/// ⛔⛔ **As duas metades falham em silêncio e de maneiras opostas.** Sem CONTEÚDO, girar o osso não
/// move nada e o dono lê *«não funciona»* sobre um motor correcto. ABERTA, o controlo fica inerte
/// **por lei** — e a cena passa a ensinar o contrário do que o app faz, que o `CLAUDE.md` §5.0
/// nomeia como pior que uma cena ausente.
///
/// ⚠️ Ele corre a MESMA porta que a cena chama (`seed_demo_action`), nunca uma cópia dos números:
/// uma sonda que semeia o documento à mão mede outro programa.
#[test]
fn the_smoke_scene_ships_an_action_with_something_in_it() {
    let mut doc = TimelineDoc::default();
    let aberta_antes = doc.active_index();
    let folha = 4_242_u64;
    crate::vec_bone_smoke::seed_demo_action(&mut doc, folha);

    let i = doc
        .clips()
        .iter()
        .position(|c| c.name == crate::vec_bone_smoke::DEMO_ACTION)
        .expect("a cena tem de trazer a acção que o selector `Action` oferece");
    assert_ne!(
        i,
        doc.active_index(),
        "a acção da cena ficou ABERTA na timeline — o controlo que a percorre nasce inerte, e a \
         cena passa a ensinar o contrário do que o app faz"
    );
    assert_eq!(
        doc.active_index(),
        aberta_antes,
        "semear a acção mudou a acção aberta — quem abriu a timeline noutra coisa perde-a"
    );
    assert!(
        doc.clip_end_seconds(i) > 0.0,
        "a acção da cena tem extensão ZERO — o `action_time` multiplica por ela, então girar o \
         osso prenderia a acção no instante 0 para todo ângulo"
    );
    assert!(
        doc.bindings().iter().any(|b| b.entity == folha),
        "a acção da cena não anima objecto nenhum — girar o osso não moveria nada, e o dono leria \
         isso como o motor partido"
    );
}

/// ⛔ **O GESTO NÃO CRIA NADA** — ordem do dono (2026-09-08: *«porque criar Bone Action no inspector
/// e na timeline? Melhor não criar nada»*).
///
/// ⚠️ O *Add Smart Bone* insere exactamente um `SmartBone::default()`, e é este gate que diz o que
/// esse default tem de ser: **vazio nos dois nomes**. Um default com clip ou alvo pré-preenchidos
/// voltaria a casar o controlo com alguma coisa que ninguém escolheu — que é o defeito que os dois
/// desenhos anteriores tinham, cada um à sua maneira.
#[test]
fn the_gesture_attaches_an_empty_control_and_creates_nothing() {
    let d = ph2d_skeleton_ecs::SmartBone::default();
    assert!(
        d.clip.is_empty(),
        "o controlo nasceu casado com {:?}",
        d.clip
    );
    assert!(
        d.target.is_empty(),
        "o controlo nasceu com alvo {:?}",
        d.target
    );
    // E vazio ele é INERTE — a outra metade, que o gate irmão mede sobre o mundo inteiro.
    let (mut sim, doc, controlo, movido) = cena();
    sim.world_mut().entity_mut(controlo).insert(d);
    let mut pv = PreviewDrive::default();
    assert_eq!(drive(&mut sim, &doc, &mut pv), 0);
    assert!((x(&sim, movido) - 0.0).abs() < 1e-9);
}

/// ⭐⭐⭐ **A CADEIA INTEIRA DA CENA DE SMOKE, ponta a ponta** — o que o dono configura, e o que ele
/// espera ver.
///
/// ⛔⛔ **Report do dono (2026-09-08):** *«Painel funcionou OK. Mas tudo configurado e a animação não
/// rodou ao rotacionar o bone»*. Os gates que existiam mediam o passe sobre uma fixtura MINHA; este
/// mede-o sobre a cadeia que a **cena** monta — a mesma acção (`seed_demo_action`), o mesmo default
/// do componente (`SmartBone::default()`, `0..90°`), e o osso girado como o artista o gira.
///
/// ⚠️ *Uma fixtura que não é a que o artista corre mede outro programa* — foi a 4.ª vez nesta linha.
#[test]
fn the_smoke_chain_moves_the_leaf_end_to_end() {
    let mut sim = SimWorld::default();
    let folha = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Leaf"), RootOrder(0)))
        .id();
    let controlo = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Bone 3"),
            RootOrder(1),
            ph2d_skeleton_ecs::Bone {
                length: 10.0,
                strength: 1.0,
            },
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    let mut doc = TimelineDoc::default();
    crate::vec_bone_smoke::seed_demo_action(&mut doc, folha.to_bits());

    // Exactamente o que o painel escreve: o default do componente + a acção escolhida na lista + o
    // alvo escolhido no picker.
    sim.world_mut()
        .entity_mut(controlo)
        .insert(ph2d_skeleton_ecs::SmartBone {
            clip: crate::vec_bone_smoke::DEMO_ACTION.to_string(),
            target: "Leaf".to_string(),
            ..ph2d_skeleton_ecs::SmartBone::default()
        });

    let mut pv = PreviewDrive::default();
    // No princípio da faixa a folha está em baixo.
    assert_eq!(
        drive(&mut sim, &doc, &mut pv),
        1,
        "o passe não escreveu nada"
    );
    let em_baixo = sim
        .world()
        .get::<Transform>(folha)
        .expect("a folha")
        .translation
        .y;

    // ...e no fim da faixa (o osso girado um quarto de volta), no fim.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
        #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
        {
            t.rotation = (ph2d_skeleton::FULL_TURN / 4.0) as f32;
        }
    }
    assert_eq!(
        drive(&mut sim, &doc, &mut pv),
        1,
        "o passe não escreveu nada"
    );
    let em_cima = sim
        .world()
        .get::<Transform>(folha)
        .expect("a folha")
        .translation
        .y;
    assert!(
        em_cima - em_baixo > 1.0,
        "girar o osso um quarto de volta moveu a folha de {em_baixo} para {em_cima} — a cadeia que \
         a cena monta não percorre a acção"
    );
}

/// ⭐⭐⭐ **TIRAR O CONTROLO DEVOLVE A POSE QUE O ARTISTA AUTOROU.**
///
/// ⛔⛔ **Achado da auditoria de 2026-09-08:** o *Remove Smart Bone* só tirava o componente, e o
/// objecto ficava **assado no instante da acção** em que o controlo o tinha deixado. É letra por
/// letra o report de 2026-09-07 sobre o *Remove IK* — *«não funciona plenamente»* — noutro verbo.
#[test]
fn removing_the_control_gives_the_authored_pose_back() {
    let (mut sim, doc, controlo, movido) = cena();
    liga(&mut sim, controlo, "Correction");
    let autorado = x(&sim, movido);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
        #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
        {
            t.rotation = ATE as f32;
        }
    }
    let mut pv = PreviewDrive::default();
    // ⚠️⚠️ **QUADROS DE VERDADE, com a `settle` de cada um** — e é aqui que o report de 2026-09-09
    // vive. A 1.ª redacção deste gate corria `drive` UMA vez e nunca chamava a `settle`, que é o
    // passe que a shell corre em todo quadro: medido, o ledger largava o objecto no **quadro 1**
    // (`conduz=true` no 0, `false` no 1..4) e o `remove` já não tinha o autorado para devolver.
    // *Uma fixtura que não corre o quadro do artista mede outro programa.*
    for _ in 0..5 {
        drive(&mut sim, &doc, &mut pv);
        pv.settle();
    }
    assert!(
        (x(&sim, movido) - 10.0).abs() < 1e-3,
        "a fixtura nao produz o fenomeno: a accao nao correu ({})",
        x(&sim, movido)
    );
    assert!(
        pv.drives(movido.to_bits()),
        "o ledger largou o objecto ANTES do remove -- um controlo parado escreve o mesmo valor \
         todo quadro, e a `settle` leu a constancia como «o motor largou»"
    );

    assert!(
        remove(&mut sim, &doc, controlo, &mut pv),
        "havia um controlo"
    );
    assert!(
        sim.world()
            .get::<ph2d_skeleton_ecs::SmartBone>(controlo)
            .is_none(),
        "o componente tinha de sair"
    );
    assert!(
        (x(&sim, movido) - autorado).abs() < 1e-4,
        "a pose da accao ficou ASSADA no documento: {} contra os {autorado} que o artista autorou",
        x(&sim, movido)
    );
    assert!(
        !pv.drives(movido.to_bits()),
        "o ledger continua a conduzir um objecto cujo motor foi DESLIGADO"
    );
}

/// ⛔ **E ele larga SÓ o que a acção DELE anima.**
///
/// ⚠️ Largar «tudo o que o ledger tem» apagaria a reprodução da própria timeline, que partilha
/// aqueles motores — a lista sai do **clip deste controlo**, e de mais nada.
#[test]
fn removing_one_control_does_not_release_what_another_engine_drives() {
    let (mut sim, doc, controlo, movido) = cena();
    liga(&mut sim, controlo, "Correction");
    // Um segundo objecto conduzido por OUTRO motor, que este verbo não pode tocar.
    let alheio = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Alheio"), RootOrder(2)))
        .id();
    let mut pv = PreviewDrive::default();
    let repouso = Transform::IDENTITY;
    let mut movida = repouso;
    movida.translation.x = 99.0;
    pv.driven(
        alheio,
        ph2d_preview_drive::Driven::SolverPose(repouso),
        ph2d_preview_drive::Driven::SolverPose(movida),
    );
    drive(&mut sim, &doc, &mut pv);

    assert!(remove(&mut sim, &doc, controlo, &mut pv));
    assert!(
        pv.drives(alheio.to_bits()),
        "o verbo largou um objecto que a acção dele nem anima — a reprodução da timeline morreria \
         com um clique em Remove Smart Bone"
    );
    let _ = movido;
}

/// ⭐⭐⭐ **UM CONTROLO GOVERNADO POR UMA ÂNCORA É ACHADO EM QUALQUER ORDEM.**
///
/// ⛔⛔ **Achado da auditoria de 2026-09-08:** o aviso vivia dentro do *Add Smart Bone*, logo só
/// disparava na ordem **IK → Smart**. Nas outras duas — a âncora **depois** do controlo, e o
/// `Chain` **alargado** até ele — o app ficava calado sobre o mesmo facto: o solver reescreve a
/// rotação **depois** do passe do controlo, e girar o osso não move a acção.
///
/// ⚠️ **A régua é o FACTO, não o verbo** — é por isso que as três ordens têm o mesmo veredito sem o
/// gate as enumerar do lado do produto.
#[test]
fn a_control_governed_by_an_anchor_is_found_in_any_order() {
    use ph2d_skeleton_ecs::{Bone, IkGoal};
    // Uma corrente de dois ossos: a raiz e a ponta (que leva a âncora).
    let mut sim = SimWorld::default();
    let raiz = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Raiz"),
            RootOrder(0),
            Bone {
                length: 10.0,
                strength: 1.0,
            },
        ))
        .id();
    let ponta = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Ponta"),
            RootOrder(1),
            Bone {
                length: 10.0,
                strength: 1.0,
            },
            ph2d_ecs::ChildOf(raiz),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert!(
        governed_controls(&sim).is_empty(),
        "a fixtura ja' nasce com um controlo mudo"
    );

    // ORDEM A — o controlo primeiro, a âncora depois. É a que o aviso antigo NÃO via.
    sim.world_mut()
        .entity_mut(raiz)
        .insert(ph2d_skeleton_ecs::SmartBone::default());
    assert!(
        governed_controls(&sim).is_empty(),
        "sem ancora nenhuma, o controlo e' livre"
    );
    sim.world_mut().entity_mut(ponta).insert(IkGoal {
        chain: 2,
        ..IkGoal::default()
    });
    assert_eq!(
        governed_controls(&sim),
        vec![raiz],
        "a ancora passou a governar o controlo e ninguem deu por isso -- e' a ordem Smart -> IK, \
         que o aviso pendurado no verbo nunca via"
    );

    // ORDEM B — o `Chain` encolhe e o controlo volta a ser livre; alargar torna-o mudo outra vez.
    if let Some(mut g) = sim.world_mut().get_mut::<IkGoal>(ponta) {
        g.chain = 1;
    }
    assert!(
        governed_controls(&sim).is_empty(),
        "com a corrente a parar na ponta, a raiz volta a ser autorada"
    );
    if let Some(mut g) = sim.world_mut().get_mut::<IkGoal>(ponta) {
        g.chain = 2;
    }
    assert_eq!(
        governed_controls(&sim),
        vec![raiz],
        "alargar o Chain ate' ao controlo torna-o mudo, e essa e' a TERCEIRA ordem -- nenhum verbo \
         de criacao correu aqui"
    );
}
