//! ⭐ **«Animating»** (doc 17 item 9, B7): com `held`, PARAR a agulha não promove a pose a documento
//! (a agulha não grava passo) e uma escrita por cima não substitui a pose de entrada (o laranja some
//! no Done): o autorado do ledger é a cena de entrada do princípio ao fim.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld};
use ph2d_timeline::PropKind;

fn rig() -> (SimWorld, Entity, TimelineDoc) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Keyed")))
        .id();
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0_f32), (4.0, 10.0)] {
        doc.insert_key(
            e.to_bits(),
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    (sim, e, doc)
}

fn quadro(sim: &mut SimWorld, doc: &mut TimelineDoc, drive: &mut PreviewDrive, held: bool) {
    let before = state_of_bindings(sim.world(), doc);
    ph2d_timeline::apply_from_doc(sim.world_mut(), doc, 2.0);
    declare_timeline_writes_held(sim.world(), &before, drive, held);
    drive.settle();
}

fn autorado(drive: &PreviewDrive, e: Entity) -> Option<Driven> {
    drive.authored(e.to_bits(), Driver::SolverPose)
}

/// *Mutação: `held` ignorado (o modo de hoje) ⇒ a agulha parada larga a condução — vermelho.*
#[test]
fn while_animating_a_parked_playhead_is_not_an_edit() {
    let (mut sim, e, mut doc) = rig();
    let entrada = Driven::SolverPose(*sim.world().get::<Transform>(e).unwrap());
    let mut drive = PreviewDrive::default();
    for _ in 0..10 {
        quadro(&mut sim, &mut doc, &mut drive, true);
    }
    assert!(
        sim.world().get::<Transform>(e).unwrap().translation.x > 4.0,
        "controlo positivo: a curva moveu o objeto"
    );
    assert_eq!(
        autorado(&drive, e),
        Some(entrada),
        "a agulha parada promoveu a pose a documento — cada ida da agulha seria um passo"
    );
    sim.world_mut()
        .get_mut::<Transform>(e)
        .unwrap()
        .translation
        .x = 99.0;
    quadro(&mut sim, &mut doc, &mut drive, true);
    assert_eq!(
        autorado(&drive, e),
        Some(entrada),
        "a escrita sem chave virou documento — o Done não devolveria a cena de entrada"
    );
}

/// Controlo: sem a sessão, parar a agulha larga a condução (a pose vira documento, como hoje).
#[test]
fn without_the_session_a_parked_playhead_becomes_the_document() {
    let (mut sim, e, mut doc) = rig();
    let mut drive = PreviewDrive::default();
    for _ in 0..3 {
        quadro(&mut sim, &mut doc, &mut drive, false);
    }
    assert_eq!(autorado(&drive, e), None);
}

/// Uma escrita sem chave num campo que a Timeline NÃO conduz (a rotação, só `x` tem curva)
/// sobrevive ao Done: o autorado adopta-a e o `x` conduzido continua o da entrada. *Mutação: o
/// `driven_held` sem a «outra mão» por campo ⇒ a rotação some — vermelho.*
#[test]
fn while_animating_an_edit_on_an_undriven_field_survives_done() {
    let (mut sim, e, mut doc) = rig();
    let entrada = *sim.world().get::<Transform>(e).unwrap();
    let mut drive = PreviewDrive::default();
    for _ in 0..3 {
        quadro(&mut sim, &mut doc, &mut drive, true);
    }
    sim.world_mut().get_mut::<Transform>(e).unwrap().rotation = 0.7;
    for _ in 0..3 {
        quadro(&mut sim, &mut doc, &mut drive, true);
    }
    let Some(Driven::SolverPose(a)) = autorado(&drive, e) else {
        panic!("a condução continua")
    };
    assert!(
        (a.rotation - 0.7).abs() < 1e-6,
        "a rotação sumiu no Done: {a:?}"
    );
    assert_eq!(
        a.translation, entrada.translation,
        "o x conduzido volta à entrada"
    );
}
