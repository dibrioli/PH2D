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

/// ⭐⭐⭐ **UM OSSO INTELIGENTE NASCE COM ACÇÃO PRÓPRIA — ⛔ ele nunca adopta a que está aberta.**
///
/// ⚠️⚠️ **É o report do dono de 2026-09-08** (*«não há meios de selecionar nem o objeto alvo nem a
/// animação»*), e o mecanismo é este: um `TimelineDoc` novo nasce com **uma** acção chamada
/// `"Main"` ⇒ *adoptar a aberta* amarrava **todo** osso inteligente à animação principal da cena, em
/// silêncio, e nada na tela dizia a qual. É a lei do Moho (*Create Smart Bone Action* nasce com o
/// nome do osso) e o botão **New** do *Action Constraint* do Blender.
#[test]
fn a_smart_bone_is_born_with_its_own_action_never_the_open_one() {
    let doc = TimelineDoc::default();
    assert_eq!(
        doc.clips().len(),
        1,
        "a premissa deste gate é que um documento novo tem UMA acção — se isto mudar, o mecanismo \
         do report mudou com ele"
    );
    let nome = crate::skeleton_smart::fresh_action_name(&doc, "Bone 7");
    assert_ne!(
        nome,
        doc.clips()[0].name,
        "o osso inteligente adoptou a acção ABERTA — ele voltaria a casar com a animação principal \
         da cena, calado"
    );
    assert!(
        nome.contains("Bone 7"),
        "a acção não leva o nome do osso ({nome:?}) — o artista não a acha na lista"
    );
}

/// ⭐ **Dois pedidos para o mesmo osso dão nomes DIFERENTES.**
///
/// ⚠️ O nome é a referência durável desta casa (`SmartBone::clip` guarda o nome, e o passe procura
/// o clip por ele): duas acções homónimas seriam **o mesmo sujeito** para o `drive`, e o segundo
/// osso percorreria a acção do primeiro.
#[test]
fn two_actions_for_the_same_bone_never_share_a_name() {
    let mut doc = TimelineDoc::default();
    let a = crate::skeleton_smart::fresh_action_name(&doc, "Bone 7");
    doc.add_clip(a.clone());
    let b = crate::skeleton_smart::fresh_action_name(&doc, "Bone 7");
    assert_ne!(
        a, b,
        "o segundo pedido devolveu o nome que já estava tomado"
    );
    assert!(
        doc.clips().iter().all(|c| c.name != b),
        "o nome escolhido ({b:?}) já existe no documento"
    );
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
    assert_eq!(
        ph2d_editor::ids::VECTOR_BONE_SMART_CLIP_IDS.len(),
        ph2d_editor::ids::MAX_SMART_CLIPS,
        "a tabela de ids não tem o tamanho que ela própria declara"
    );
}

/// ⭐⭐⭐ **UM CONTROLO NÃO PERCORRE A ACÇÃO QUE ESTÁ ABERTA NA TIMELINE.**
///
/// ⚠️⚠️ **Não é uma nicety — é o que torna a feature usável.** Desde 2026-09-08 *Add Smart Bone*
/// **cria** a acção e **abre** a timeline nela, para o artista a gravar; se o controlo a percorresse
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
