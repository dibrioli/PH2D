//! **Uma track dormente não pode adotar o PRÓXIMO objeto** (Enio, 2026-07-22).
//!
//! Report: *"Ao deletar um objeto os containers e arranjo na timeline não desaparecem
//! imediatamente com ele. E quando crio outro objeto após deletar o primeiro, a timeline não
//! funciona para o segundo objeto."*
//!
//! O mecanismo é a colisão de duas regras que, sozinhas, estão certas:
//!
//! 1. **A binding sobrevive ao objeto** (`timeline_persist::upkeep`, 2026-07-11) — deletar
//!    esconde as rows, e o Ctrl+Z global RESPAWNA a entidade com bits novos e o mesmo `Name`,
//!    então a track se recola pelo nome. Sem isso, delete+undo perdia a animação (o undo global
//!    não carrega o `TimelineDoc`).
//! 2. **O nome novo é único entre os VIVOS** (`name_unique::unique_name`) — um objeto deletado
//!    devolve o nome dele ao pote.
//!
//! Juntas: o objeto morto libera "Sprite", o objeto NOVO nasce "Sprite", e o `upkeep` — que não
//! sabe distinguir *"o mesmo objeto voltou pelo undo"* de *"outro objeto pegou o nome vago"* —
//! cola a track órfã nele. O segundo objeto nasce dirigido pela animação do primeiro: a pose é
//! reescrita todo frame, e é isso que se vê como *"a timeline não funciona para ele"*.

use crate::name_unique::unique_name;
use crate::timeline_persist::upkeep;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc, apply_intent};

/// **Um frame, na ordem do `timeline_bridge::run`**: apply (que marca as bindings órfãs) e
/// depois upkeep (que cura o que dá para curar e publica a reserva de nomes).
///
/// Os testes abaixo dirigem ISTO entre os gestos do artista, em vez de chamar as metades soltas:
/// a ordem é o que torna a reserva fresca quando o próximo objeto nasce, e um teste que a
/// pulasse estaria medindo uma sequência que o produto não executa.
fn run_frame(sim: &mut SimWorld, timeline: &mut TimelineState) {
    apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
    upkeep(timeline, sim.world_mut());
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

/// **O gate do report** — o segundo objeto nasce LIVRE da animação do primeiro.
///
/// Dirige a sequência exata do artista: cria, anima, deleta, cria de novo. O oráculo é a POSE:
/// o objeto novo é posto em x = 50 e tem de continuar lá depois do apply do frame. Se a track
/// órfã o adotou, o apply a reescreve para o valor keyado no objeto MORTO — e é isso que o
/// artista descreve como a timeline não funcionar para ele.
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

    // 2. E o deleta. A binding fica dormente — por design (delete + Ctrl+Z tem de curar).
    sim.world_mut().despawn(first);
    run_frame(&mut sim, &mut timeline);
    assert!(
        timeline.doc.bindings()[0].missing,
        "a track fica dormente ao perder o objeto — este e' o design de 2026-07-11"
    );

    // 3. Cria OUTRO objeto. Não é o primeiro voltando: é um objeto novo, na posição em que o
    //    artista o largou.
    let second_name = unique_name(&mut sim, "Sprite");
    let second = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(50.0, 0.0)),
            Name::new(second_name.clone()),
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
         devia estar onde o artista o largou (50). Nome do 1o: {first_name:?}; do 2o: \
         {second_name:?}",
        pose.x
    );
    assert!(
        timeline.doc.bindings()[0].missing,
        "a track do objeto morto continua dormente — ela nao tem dono vivo"
    );
}

/// **A timeline PASSA a funcionar para o segundo objeto** — a outra metade do report.
///
/// O arranjo (lanes, strips, containers) sobrevive ao objeto **de propósito**: são assets do
/// DOCUMENTO, não do sprite — o mesmo motivo por que apagar uma layer no After Effects não
/// apaga a comp. O que fazia isso parecer quebrado era o sequestro: o objeto novo nascia
/// dirigido pela animação do morto, então nada que o artista fizesse com ele pegava.
///
/// Com o sequestro morto, a montagem que sobreviveu **serve** o objeto novo: ele keya no clip
/// ativo, e as strips que já estavam lá tocam essa animação. O oráculo é a POSE dele ao longo
/// do tempo — não a contagem de rows, que diria que existe uma track sem dizer que ela move
/// alguma coisa.
#[test]
fn the_second_object_can_be_animated_by_the_arrangement_that_survived() {
    let mut sim = SimWorld::new();
    let mut timeline = TimelineState::new();

    // O primeiro objeto: animado, e depois deletado.
    let n1 = unique_name(&mut sim, "Sprite");
    let first = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(n1)))
        .id();
    key_at(&mut timeline, first.to_bits(), 0.0, -7.0);
    run_frame(&mut sim, &mut timeline);
    sim.world_mut().despawn(first);
    run_frame(&mut sim, &mut timeline);

    // O segundo objeto — nasce "Sprite (1)", porque a track dormente reserva "Sprite" (medido).
    let n2 = unique_name(&mut sim, "Sprite");
    assert_eq!(n2, "Sprite (1)", "a reserva empurra o nome do 2o objeto");
    let second = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(n2)))
        .id();
    key_at(&mut timeline, second.to_bits(), 0.0, 10.0);
    key_at(&mut timeline, second.to_bits(), 1.0, 20.0);
    run_frame(&mut sim, &mut timeline);

    let x_at = |sim: &mut SimWorld, timeline: &mut TimelineState, t: f64| {
        apply_from_doc(sim.world_mut(), &mut timeline.doc, t);
        sim.world()
            .get::<Transform>(second)
            .expect("o segundo objeto")
            .translation
            .x
    };
    assert!(
        (x_at(&mut sim, &mut timeline, 0.0) - 10.0).abs() < 1e-6,
        "a animacao DELE deve valer em t=0"
    );
    assert!(
        (x_at(&mut sim, &mut timeline, 1.0) - 20.0).abs() < 1e-6,
        "…e em t=1: a timeline funciona para o segundo objeto"
    );
}

/// **E a cura de delete+undo continua funcionando** — o controle deste fix.
///
/// A regra 1 (a binding sobrevive ao objeto) é a razão de o dormente existir, e um fix que a
/// quebrasse trocaria um bug por outro: o Ctrl+Z devolveria o objeto sem a animação dele. O
/// undo global RESTAURA o `Name` do snapshot (não passa por `unique_name`), então o objeto
/// volta com o nome exato — e é por aí que a track o reencontra.
#[test]
fn delete_then_undo_still_heals_the_track() {
    let mut sim = SimWorld::new();
    let mut timeline = TimelineState::new();

    let name = unique_name(&mut sim, "Sprite");
    let first = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name.clone())))
        .id();
    key_at(&mut timeline, first.to_bits(), 0.0, -7.0);
    run_frame(&mut sim, &mut timeline);

    sim.world_mut().despawn(first);
    run_frame(&mut sim, &mut timeline);
    assert!(timeline.doc.bindings()[0].missing);

    // O undo respawna: bits NOVOS, o `Name` LITERAL do snapshot (nunca `unique_name`).
    let reborn = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name)))
        .id();
    assert_ne!(reborn, first, "um respawn da' bits novos");
    assert_eq!(
        upkeep(&mut timeline, sim.world_mut()),
        1,
        "a track reencontra o objeto que voltou — o delete+undo NAO pode regredir"
    );
    assert_eq!(timeline.doc.bindings()[0].entity, reborn.to_bits());
}
/// **O que a timeline MOSTRA some com o objeto; o que a timeline É sobrevive** — o sintoma 1,
/// medido e pinado.
///
/// Report: *"os containers e arranjo não desaparecem imediatamente com ele"*. A medição separa
/// duas coisas que o painel desenha lado a lado:
///
/// - **As TRACKS do objeto** (as rows keyáveis, na aba Keys) SOMEM — elas descrevem *este
///   objeto*, e ele morreu. É o comportamento certo, e é o que o `apply` marca `missing`.
/// - **A estrutura de composição** (lanes, strips, containers, na aba Arrange/Containers)
///   FICA — porque containers são **assets do DOCUMENTO**, não do sprite (ADR-0133, a regra do
///   precomp do After Effects: apagar uma layer não apaga a comp). Deletar o sprite não pode
///   destruir a biblioteca de peças que outros objetos podem usar. Quem limpa isso é a lixeira
///   da aba Containers, em cascata.
///
/// Este gate pina a fronteira: se um dia as tracks pararem de sumir, ou os containers passarem
/// a sumir, um dos dois lados do report reabre.
#[test]
fn the_objects_tracks_vanish_but_the_composition_structure_survives() {
    use ph2d_timeline::TimelineViewSnapshot;

    let mut sim = SimWorld::new();
    let obj = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("LibraryDemo")))
        .id();
    let mut timeline = TimelineState::new();
    crate::nest_smoke::build_library(&mut timeline.doc, obj.to_bits());

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
    apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
    upkeep(&mut timeline, sim.world_mut());

    // As tracks SOMEM (o objeto que elas descrevem morreu).
    snap.rebuild(&mut timeline, &ph, true);
    assert_eq!(snap.tracks.len(), 0, "as tracks do objeto somem com ele");

    // A estrutura de composição FICA — assets do documento, não do sprite (ADR-0133).
    snap.rebuild(&mut timeline, &ph, false);
    assert_eq!(
        (snap.lanes.len(), strips(&snap), snap.containers.len()),
        (1, 3, 3),
        "lanes/strips/containers sao assets do DOCUMENTO — deletar o sprite nao os apaga"
    );
}
