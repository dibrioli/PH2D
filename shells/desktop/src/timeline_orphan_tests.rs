//! **Deletar o objeto RESETA a timeline dele** (Enio, 2026-07-22 — 2ª rodada).
//!
//! Report da 1ª rodada: *"quando crio outro objeto após deletar o primeiro, a timeline não
//! funciona para o segundo objeto"*. Report da 2ª: *"Vc não limpou a timeline ao deletar o
//! objeto e ela veio totalmente bugada para o novo objeto criado. A timeline precisa ser
//! resetada ao deletar o objeto."*
//!
//! A 1ª rodada tratou o sequestro (a track dormente adotava o homônimo novo) RESERVANDO o
//! nome — e manteve a dormência, os containers, o arranjo e o loop de pé. A ordem da 2ª
//! rodada remove a causa em vez do sintoma: **objeto morto não deixa timeline para trás**.
//! O `timeline_persist::upkeep` purga as tracks/bindings de quem morreu, e quando o ÚLTIMO
//! objeto animado sai, o documento inteiro volta ao estado de fábrica — containers, lanes,
//! strips, clips e loop juntos, porque composição autorada em volta de um objeto que não
//! existe é exatamente o que chegava "totalmente bugada" ao objeto seguinte.
//!
//! O que fica de pé, por construção:
//! - **delete de UM entre VÁRIOS** purga só o morto — o trabalho dos vivos é intocável;
//! - **empate de nome** continua dormente (recusa dupla: não cura, não purga) — o gate mora
//!   em `timeline_persist`;
//! - **recuperação**: a purga é UM passo do undo da timeline; Ctrl+Z global devolve o objeto
//!   (Name literal do snapshot) e o Ctrl+Z da timeline devolve o documento, que o heal recola.

use crate::name_unique::unique_name;
use crate::timeline_persist::upkeep;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc, apply_intent};

/// **Um frame, na ordem do `timeline_bridge::run`**: apply (que marca as bindings órfãs) e
/// depois upkeep (heal, depois purga). Devolve o que o upkeep devolve: *o documento resetou?*
///
/// Os testes abaixo dirigem ISTO entre os gestos do artista, em vez de chamar as metades
/// soltas: a ordem apply→heal→purga é o produto, e um teste que a pulasse estaria medindo
/// uma sequência que o produto não executa.
fn run_frame(sim: &mut SimWorld, timeline: &mut TimelineState) -> bool {
    apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
    upkeep(timeline, sim.world_mut())
}

/// Uma key de `TranslationX` no objeto `entity`, pelo caminho real de autoria.
fn key_at(timeline: &mut TimelineState, entity: u64, t: f64, v: f32) {
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(
        timeline,
        &mut ph,
        ph2d_timeline::TimelineIntent::AddKey {
            entity,
            prop: PropKind::TranslationX,
            t: RationalTime::from_seconds(t),
            value: AnimValue::Float(v),
            interp: Interp::Linear,
        },
    );
}

/// **O gate do report original** — o segundo objeto nasce LIVRE da animação do primeiro.
///
/// Dirige a sequência exata do artista: cria, anima, deleta, cria de novo. O oráculo é a
/// POSE: o objeto novo é posto em x = 50 e tem de continuar lá depois do apply do frame. Sem
/// a purga, a track dormente adota o homônimo e o apply a reescreve para o valor keyado no
/// objeto MORTO — o sequestro que o artista descreve como "a timeline não funciona".
///
/// E o nome volta ao pote NA HORA: com a track purgada não há mais o que reservar, então o
/// segundo objeto pode se chamar "Sprite" de novo — a reserva de nomes da 1ª rodada saiu
/// junto com a dormência que a exigia.
#[test]
fn a_new_object_does_not_inherit_the_deleted_objects_animation() {
    let mut sim = SimWorld::new();
    let mut timeline = TimelineState::new();

    // 1. O artista cria um objeto e o anima.
    let first_name = unique_name(&mut sim, "Sprite");
    let first = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(first_name.clone())))
        .id();
    key_at(&mut timeline, first.to_bits(), 0.0, -7.0);
    run_frame(&mut sim, &mut timeline);

    // 2. E o deleta. O frame seguinte purga a track dele — nada fica dormente.
    sim.world_mut().despawn(first);
    run_frame(&mut sim, &mut timeline);
    assert!(
        timeline.doc.bindings().is_empty(),
        "a track do objeto morto e' PURGADA no frame seguinte — dormencia era o sequestro"
    );

    // 3. Cria OUTRO objeto — que pode se chamar "Sprite" de novo, porque nenhuma
    //    track espera por esse nome.
    let second_name = unique_name(&mut sim, "Sprite");
    assert_eq!(
        second_name, "Sprite",
        "sem track dormente nao ha' reserva: o nome volta ao pote com o objeto"
    );
    let second = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(50.0, 0.0)),
            Name::new(second_name),
        ))
        .id();

    // 4. O frame seguinte roda.
    run_frame(&mut sim, &mut timeline);

    let pose = sim
        .world()
        .get::<Transform>(second)
        .expect("o objeto novo")
        .translation;
    assert!(
        (pose.x - 50.0).abs() < 1e-6,
        "o objeto NOVO foi sequestrado pela animacao do objeto DELETADO: esta em x={}, \
         devia estar onde o artista o largou (50)",
        pose.x
    );
}

/// **O gate do report da 2ª rodada** — deletar o único objeto animado reseta a timeline
/// INTEIRA: tracks, containers, lanes, strips, clips e loop, no mesmo frame de upkeep.
///
/// A cena é a biblioteca do smoke `=3` (2 tracks, 3 containers, 1 lane com 3 strips) mais um
/// loop armado — a montagem exata que o Enio viu sobrar "totalmente bugada". O oráculo é o
/// que o painel MOSTRA (o snapshot que ele pinta), não só o doc: era na tela que o resto
/// sobrava.
#[test]
fn deleting_the_only_animated_object_resets_the_whole_timeline() {
    use ph2d_timeline::TimelineViewSnapshot;

    let mut sim = SimWorld::new();
    let obj = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("LibraryDemo")))
        .id();
    let mut timeline = TimelineState::new();
    crate::nest_smoke::build_library(&mut timeline.doc, obj.to_bits());
    timeline.doc.set_active_loop(Some((0.0, 6.0)));
    timeline.doc.fps_display = 12.0; // um ajuste de PROJETO, para provar que sobrevive

    let ph = ph2d_core::Playhead::new(1.0 / 60.0);
    let mut snap = TimelineViewSnapshot::default();
    let strips = |s: &TimelineViewSnapshot| s.lanes.iter().map(|l| l.strips.len()).sum::<usize>();

    // Antes: 2 tracks, e a estrutura montada (1 lane na cena, 3 strips, 3 containers).
    snap.rebuild(&mut timeline, &ph, true);
    assert_eq!(snap.tracks.len(), 2, "duas tracks antes do delete");
    snap.rebuild(&mut timeline, &ph, false);
    assert_eq!(
        (snap.lanes.len(), strips(&snap), snap.containers.len()),
        (1, 3, 3)
    );

    // Deleta o objeto e roda o frame.
    sim.world_mut().despawn(obj);
    assert!(
        run_frame(&mut sim, &mut timeline),
        "ultimo objeto animado deletado => reset, e o bridge e' avisado para rebobinar"
    );

    // O que o painel mostra: NADA sobra — nem tracks, nem lanes, nem strips, nem containers.
    snap.rebuild(&mut timeline, &ph, true);
    assert_eq!(snap.tracks.len(), 0, "as tracks do objeto somem com ele");
    snap.rebuild(&mut timeline, &ph, false);
    assert_eq!(
        (snap.lanes.len(), strips(&snap), snap.containers.len()),
        (0, 0, 0),
        "containers e arranjo desaparecem COM o objeto (Enio, 2026-07-22)"
    );

    // E o documento é o de fábrica, com o ajuste de projeto preservado.
    assert_eq!(timeline.doc.clips().len(), 1, "um clip fresco");
    assert!(timeline.doc.active_loop().is_none(), "o loop foi embora");
    assert!(
        (timeline.doc.fps_display - 12.0).abs() < f64::EPSILON,
        "o fps de exibicao e' ajuste de projeto, nao animacao do morto — sobrevive"
    );
}

/// **Deletar UM objeto entre VÁRIOS purga só o dele** — o trabalho dos outros é intocável.
///
/// O morto tinha tracks em DOIS clips (o caso que o `unbind` do painel não cobre: ele poda só
/// o clip ativo); a purga varre todos. A composição (containers/strips) fica, porque ainda há
/// um objeto animado usando o documento — reset aqui destruiria a animação do sobrevivente.
#[test]
fn deleting_one_of_two_animated_objects_keeps_the_others_work() {
    let mut sim = SimWorld::new();
    let mut timeline = TimelineState::new();
    let hero = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("hero")))
        .id();
    let extra = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("extra")))
        .id();

    // Os dois keyados no clip 0; o "extra" também num 2º clip.
    key_at(&mut timeline, hero.to_bits(), 0.0, 3.0);
    key_at(&mut timeline, extra.to_bits(), 0.0, -4.0);
    let second_clip = timeline.doc.add_clip("B".into());
    timeline.doc.set_active(second_clip);
    key_at(&mut timeline, extra.to_bits(), 0.0, -9.0);
    timeline.doc.set_active(0);
    // E um container no documento, para provar que a composição não é arrastada junto.
    // (Sem strips na cena, de propósito: o oráculo da pose lê o clip ativo direto.)
    timeline.doc.add_container("Walk".to_string());
    run_frame(&mut sim, &mut timeline);

    let dead_target = timeline
        .doc
        .binding_for(extra.to_bits(), PropKind::TranslationX)
        .expect("extra bound")
        .target;

    sim.world_mut().despawn(extra);
    assert!(
        !run_frame(&mut sim, &mut timeline),
        "sobrou objeto animado: purga sim, reset NAO"
    );

    assert!(
        timeline
            .doc
            .bindings()
            .iter()
            .all(|b| b.entity != extra.to_bits() && b.target != dead_target),
        "as bindings do morto sairam"
    );
    for (i, named) in timeline.doc.clips().iter().enumerate() {
        assert!(
            named.clip.track(dead_target).is_none(),
            "a track do morto sumiu do clip {i} — TODOS os clips, nao so' o ativo"
        );
    }
    assert!(
        !timeline.doc.containers().is_empty(),
        "a composicao fica: o sobrevivente ainda usa o documento"
    );
    // E a animação do sobrevivente segue dirigindo a pose dele.
    apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
    let x = sim
        .world()
        .get::<Transform>(hero)
        .expect("hero vivo")
        .translation
        .x;
    assert!(
        (x - 3.0).abs() < 1e-6,
        "a animacao do sobrevivente continua valendo (x={x}, esperado 3)"
    );
}

/// **O caminho de volta existe e tem dois degraus** — Ctrl+Z global (o objeto respawna com o
/// `Name` LITERAL do snapshot) + Ctrl+Z da timeline (o documento volta, com a binding
/// dormente) — e o heal do frame recola a animação no objeto renascido.
///
/// É o que a purga tem de comprar para poder destruir: um Delete acidental não pode custar a
/// animação para sempre. A ordem importa e o teste a segue: o objeto volta ANTES do undo da
/// timeline, senão o upkeep seguinte purgaria de novo o que o undo devolveu.
#[test]
fn the_purge_is_one_timeline_undo_step_and_the_undo_heals_back() {
    let mut sim = SimWorld::new();
    let mut timeline = TimelineState::new();
    let obj = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("LibraryDemo")))
        .id();
    crate::nest_smoke::build_library(&mut timeline.doc, obj.to_bits());
    run_frame(&mut sim, &mut timeline); // carimba o wire_id em vida

    sim.world_mut().despawn(obj);
    assert!(run_frame(&mut sim, &mut timeline), "reset");
    assert!(timeline.doc.containers().is_empty(), "resetou mesmo");

    // Ctrl+Z global: o undo restaura o `Name` literal do snapshot (nunca `unique_name`).
    let reborn = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("LibraryDemo")))
        .id();
    assert_ne!(reborn, obj, "um respawn da' bits novos");

    // Ctrl+Z da timeline: UM passo devolve o documento inteiro (purga + reset juntos).
    assert!(timeline.undo(), "a purga deixou um passo para desfazer");
    assert_eq!(
        timeline.doc.containers().len(),
        3,
        "um unico undo devolve a biblioteca inteira"
    );

    // O frame seguinte recola pelo nome — a animação volta a dirigir o objeto renascido.
    assert!(!run_frame(&mut sim, &mut timeline), "curou, nao resetou");
    let b = &timeline.doc.bindings()[0];
    assert_eq!(b.entity, reborn.to_bits(), "recolada no objeto renascido");
    assert!(!b.missing);
}
