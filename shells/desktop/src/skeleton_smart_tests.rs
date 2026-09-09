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
const ATE: f64 = std::f64::consts::FRAC_PI_2;

/// Um mundo com um osso de CONTROLO e um objecto que a acção desloca em X, de `0` a `10`.
///
/// ⚠️ O clip é montado pela porta do PRODUTO (`add_clip` + `insert_key`), que é quem cria a binding
/// — uma fixtura que montasse as tracks à mão mediria outro programa.
fn cena() -> (SimWorld, TimelineDoc, Entity, Entity) {
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

fn liga(sim: &mut SimWorld, e: Entity, clip: &str) {
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
    drive(&mut sim, &doc, &mut pv);
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

/// ⭐⭐⭐ **O SELECTOR ALCANÇA TODA ACÇÃO QUE O DOCUMENTO PODE TER.**
///
/// ⚠️ O chrome **não cunha um id em tempo de execução**: a lista de opções é um pool fixo. Um pool
/// menor que o tecto do documento esconderia acções que EXISTEM — o artista veria uma lista que
/// mente, e a que ele quer seria inalcançável por gesto nenhum.
///
/// ⚠️ **Este gate vive na shell** porque só ela vê as duas crates: o `ph2d-editor-core` não depende
/// do `ph2d-timeline`, então o pool tem de declarar o número e alguém tem de os confrontar.
#[test]
fn the_action_picker_reaches_every_clip_the_document_can_hold() {
    assert_eq!(
        ph2d_editor::ids::MAX_SMART_CLIPS,
        ph2d_timeline::MAX_CLIPS,
        "o pool de ids do selector e o tecto de clips do documento discordam — ou há acção sem \
         opção (inalcançável) ou opção sem acção (morta)"
    );
    // ⛔ A 2.ª asserção desta função era `IDS.len() == MAX_SMART_CLIPS` — **o compilador já o
    // garante** (`[NodeId; MAX_SMART_CLIPS]`), logo ela não podia falhar e a mensagem descrevia um
    // estado inexprimível. *Uma asserção que não pode falhar não é uma régua, é ruído com cara de
    // rigor* (auditoria de 2026-09-08).
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

/// ⭐⭐⭐ **A LISTA DE ACÇÕES ESTREITA-SE PARA O OBJECTO ESCOLHIDO.**
///
/// ⚠️⚠️ **Report do dono (2026-09-08):** *«só deve aparecer as animações relacionadas ao objeto
/// selecionado»*. ⚠️ **E a medição corrige metade da premissa dele:** a lista não cresce com os
/// OBJECTOS — ela lista clips, e o documento recusa mais que `MAX_CLIPS` (`16`). O que o filtro
/// compra não é tamanho, é **relevância**.
#[test]
fn the_action_list_narrows_to_the_chosen_object() {
    let (mut sim, mut doc, _, _movido) = cena();
    // Uma segunda acção que NÃO toca no objecto — é ela que o filtro tem de tirar.
    let outra = doc.add_clip("Somebody Else".into());
    doc.set_active(outra);
    let estranho = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Stranger"), RootOrder(9)))
        .id();
    doc.insert_key(
        estranho.to_bits(),
        PropKind::TranslationX,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    doc.set_active(0);

    let sem_alvo = ph2d_skeleton_ecs::SmartBone::default();
    let todas = crate::skeleton_smart::actions_for(sim.world(), &doc, &sem_alvo);
    assert_eq!(
        todas.len(),
        doc.clips().len(),
        "sem alvo, a lista tem de ser a do documento inteiro — filtrar por nada esconderia tudo"
    );

    let com_alvo = ph2d_skeleton_ecs::SmartBone {
        target: "Driven".to_string(),
        ..ph2d_skeleton_ecs::SmartBone::default()
    };
    let so_dele = crate::skeleton_smart::actions_for(sim.world(), &doc, &com_alvo);
    assert!(
        so_dele.iter().any(|n| n == "Correction"),
        "a acção que anima o alvo saiu da lista: {so_dele:?}"
    );
    assert!(
        !so_dele.iter().any(|n| n == "Somebody Else"),
        "uma acção que NÃO toca o alvo ficou na lista: {so_dele:?}"
    );
}

/// ⭐⭐⭐ **UM FILTRO QUE ESVAZIARIA A LISTA NÃO SE APLICA** — e é lei, não conforto.
///
/// ⚠️ Os três casos que a disparam são reais: o alvo **apagado**, o alvo **renomeado**, e um objecto
/// que **nunca foi animado**. Nos três, filtrar daria um selector com ZERO opções — *um controlo que
/// só sabe recusar é pior que um controlo ausente*, e ali o artista não teria gesto nenhum que o
/// curasse: o alvo escolhe-se no canvas, não na lista.
#[test]
fn a_filter_that_would_empty_the_list_does_not_apply() {
    let (mut sim, doc, _, _) = cena();
    let n = doc.clips().len();
    for alvo in ["Ninguem", "Stranger"] {
        // O 2.º existe no mundo e não é animado — o 1.º nem existe.
        if alvo == "Stranger" {
            sim.world_mut()
                .spawn((Transform::IDENTITY, Name::new("Stranger"), RootOrder(9)));
        }
        let sb = ph2d_skeleton_ecs::SmartBone {
            target: alvo.to_string(),
            ..ph2d_skeleton_ecs::SmartBone::default()
        };
        assert_eq!(
            crate::skeleton_smart::actions_for(sim.world(), &doc, &sb).len(),
            n,
            "com o alvo {alvo:?} o filtro esvaziou a lista em vez de se desligar"
        );
    }
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

/// ⭐⭐⭐ **A POSIÇÃO ESCOLHIDA RESOLVE-SE CONTRA A LISTA QUE O PAINEL PINTOU.**
///
/// ⛔⛔ **Report do dono (2026-09-08):** *«ao selecionar na lista de actions … não consegue
/// selecionar o clip desejado»* — e o defeito era meu, da mesma jornada. O clique devolve uma
/// **POSIÇÃO** no pool de ids, e a shell resolvia-a contra `doc.clips()`, a lista **INTEIRA**. As
/// duas coincidiam enquanto o painel mostrava todas, e deixaram de coincidir no instante em que o
/// alvo passou a **FILTRAR**: com o filtro activo, carregar na 1.ª linha escrevia o 1.º clip do
/// DOCUMENTO.
///
/// ⚠️ **É o pior modo de falha que há:** a leitura errada compila, devolve um nome **válido**, e o
/// osso passa a percorrer uma animação que o artista nunca escolheu.
///
/// ⇒ *uma posição só significa alguma coisa ao lado da lista que a produziu.*
#[test]
fn the_chosen_position_resolves_against_the_list_the_panel_painted() {
    let (mut sim, mut doc, _, _) = cena();
    // Um 2.º clip que NÃO toca o alvo, e que fica ANTES dele na lista do documento.
    let outra = doc.add_clip("Somebody Else".into());
    doc.set_active(outra);
    let estranho = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Stranger"), RootOrder(9)))
        .id();
    doc.insert_key(
        estranho.to_bits(),
        PropKind::TranslationX,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    doc.set_active(0);

    let sb = ph2d_skeleton_ecs::SmartBone {
        target: "Driven".to_string(),
        ..ph2d_skeleton_ecs::SmartBone::default()
    };
    let lista = crate::skeleton_smart::actions_for(sim.world(), &doc, &sb);
    assert_eq!(
        lista,
        vec!["Correction".to_string()],
        "a premissa deste gate é uma lista FILTRADA de um item — se isto mudar, ele deixou de medir \
         o defeito"
    );
    assert_ne!(
        doc.clips()[0].name,
        lista[0],
        "a fixtura tem de pôr o documento e a lista pintada em DESACORDO na posição 0, senão o \
         defeito não se manifesta"
    );
    assert_eq!(
        crate::skeleton_smart::action_at(sim.world(), &doc, &sb, 0).as_deref(),
        Some("Correction"),
        "a posição 0 devolveu o 1.º clip do DOCUMENTO em vez da 1.ª linha que o artista viu"
    );
    assert_eq!(
        crate::skeleton_smart::action_at(sim.world(), &doc, &sb, 1),
        None,
        "uma posição fora da lista pintada devolveu um nome — o pool de ids é fixo e maior que a \
         lista, então esta é a posição que um clique perdido produz"
    );
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
