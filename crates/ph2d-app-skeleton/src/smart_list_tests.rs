//! ⭐⭐⭐ **A LISTA DE ACÇÕES que o painel pinta** — irmão de [`super::tests`] pelo teto de 600 LOC,
//! e o corte é por RESPONSABILIDADE: ali mede-se *o controlo PERCORRE a acção*; aqui, **quais**
//! acções ele oferece e o que uma POSIÇÃO nessa lista significa.
//!
//! ⚠️ **A fixtura é a MESMA** (`super::tests::cena`) de propósito: duas cenas para o mesmo módulo
//! divergiriam, e o gate que aqui mede o filtro tem de correr sobre o documento que ali corre o
//! motor.

use super::tests::cena;
use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, RootOrder};
use ph2d_timeline::PropKind;

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
        ph2d_editor_core::ids::MAX_SMART_CLIPS,
        ph2d_timeline::MAX_CLIPS,
        "o pool de ids do selector e o tecto de clips do documento discordam — ou há acção sem \
         opção (inalcançável) ou opção sem acção (morta)"
    );
    // ⛔ A 2.ª asserção desta função era `IDS.len() == MAX_SMART_CLIPS` — **o compilador já o
    // garante** (`[NodeId; MAX_SMART_CLIPS]`), logo ela não podia falhar e a mensagem descrevia um
    // estado inexprimível. *Uma asserção que não pode falhar não é uma régua, é ruído com cara de
    // rigor* (auditoria de 2026-09-08).
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
    let todas = crate::smart::actions_for(sim.world(), &doc, &sem_alvo);
    assert_eq!(
        todas.len(),
        doc.clips().len(),
        "sem alvo, a lista tem de ser a do documento inteiro — filtrar por nada esconderia tudo"
    );

    let com_alvo = ph2d_skeleton_ecs::SmartBone {
        target: "Driven".to_string(),
        ..ph2d_skeleton_ecs::SmartBone::default()
    };
    let so_dele = crate::smart::actions_for(sim.world(), &doc, &com_alvo);
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
            crate::smart::actions_for(sim.world(), &doc, &sb).len(),
            n,
            "com o alvo {alvo:?} o filtro esvaziou a lista em vez de se desligar"
        );
    }
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
    let lista = crate::smart::actions_for(sim.world(), &doc, &sb);
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
        crate::smart::action_at(sim.world(), &doc, &sb, 0).as_deref(),
        Some("Correction"),
        "a posição 0 devolveu o 1.º clip do DOCUMENTO em vez da 1.ª linha que o artista viu"
    );
    assert_eq!(
        crate::smart::action_at(sim.world(), &doc, &sb, 1),
        None,
        "uma posição fora da lista pintada devolveu um nome — o pool de ids é fixo e maior que a \
         lista, então esta é a posição que um clique perdido produz"
    );
}
