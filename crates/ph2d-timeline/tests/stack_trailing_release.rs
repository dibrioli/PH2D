//! **Depois do ÚLTIMO strip a lane SOLTA** (Enio, 2026-07-23: *"no momento em que
//! coloco o clip na lane 2 o segundo clip da lane 1 não toca mais"*).
//!
//! O hold do gap existe para o vão ENTRE strips (o fade cruza a partir dele, e o
//! vão é determinístico); segurado para sempre, uma lane de cima com um strip
//! curto virava uma máscara eterna de influência 1 sobre tudo embaixo — colocar
//! um clip na lane 2 CALAVA o resto da lane 1. Agora a pergunta é "ainda vem
//! alguma coisa nesta lane?" (presença além de `t`): vem → segura; não vem →
//! silêncio, e as lanes de baixo aparecem. Sob loop que EMENDA a lane é cíclica
//! ("depois do último" É "antes do primeiro") e o hold do rabo fica — é a pose em
//! que o desenho da emenda se apoia.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{ClipLane, ClipStrip, PropKind, StripSource, TimelineDoc, apply_from_doc};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// O cenário do relato, em números: lane 1 com A (chapado 10, `[0,2)`) e B
/// (rampa 20→30, `[2,4)`); lane 2 com C (chapado 99, `[1,2)`).
fn scene() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();
    doc.add_clip("B".to_string());
    doc.add_clip("C".to_string());
    let key = |doc: &mut TimelineDoc, clip: usize, t: f64, v: f32| {
        let was = doc.active_index();
        doc.set_active(clip);
        doc.insert_key(
            e,
            PropKind::TranslationX,
            s(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
        doc.set_active(was);
    };
    key(&mut doc, 0, 0.0, 10.0);
    key(&mut doc, 0, 2.0, 10.0);
    key(&mut doc, 1, 0.0, 20.0);
    key(&mut doc, 1, 2.0, 30.0);
    key(&mut doc, 2, 0.0, 99.0);
    key(&mut doc, 2, 1.0, 99.0);
    let mut lane1 = ClipLane::new("Lane 1");
    lane1.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 2.0, 2.0));
    lane1.insert(ClipStrip::new(StripSource::Clip(1), 2.0, 4.0, 2.0));
    let mut lane2 = ClipLane::new("Lane 2");
    lane2.insert(ClipStrip::new(StripSource::Clip(2), 1.0, 2.0, 1.0));
    doc.stack_mut().push(lane1);
    doc.stack_mut().push(lane2);
    (world, doc, e)
}

fn x_of(world: &World, e: u64) -> f32 {
    world
        .get::<Transform>(Entity::from_bits(e))
        .unwrap()
        .translation
        .x
}

/// O relato ao pé da letra: com C na lane 2, o SEGUNDO clip da lane 1 tem de
/// tocar depois que C acaba. Antes do fix: x=99 (a pose segurada de C) de t=2 em
/// diante, para sempre — B nunca tocava.
#[test]
fn the_second_clip_below_plays_after_the_upper_lanes_strip_ends() {
    let (mut world, mut doc, e) = scene();
    apply_from_doc(&mut world, &mut doc, 1.5);
    assert!((x_of(&world, e) - 99.0).abs() < 1e-4, "C vivo: manda");
    apply_from_doc(&mut world, &mut doc, 2.5);
    let x = x_of(&world, e);
    assert!(
        (x - 22.5).abs() < 1e-4,
        "x = {x}: B (rampa, clip t 0.5 = 22.5) tem de tocar — 99.0 é a máscara \
         eterna do hold de C"
    );
    apply_from_doc(&mut world, &mut doc, 3.5);
    let x = x_of(&world, e);
    assert!((x - 27.5).abs() < 1e-4, "x = {x}: e segue tocando (27.5)");
}

/// Sob um loop que EMENDA, o rabo da lane segue segurando (a lane é cíclica e a
/// emenda se apoia nessa pose): o mesmo cenário, com o loop de Arrange armado em
/// wrap sobre `[0,4)`, mantém a pose de C no gap final da lane 2.
#[test]
fn under_a_wrap_loop_the_trailing_hold_stays() {
    let (mut world, mut doc, e) = scene();
    doc.set_active_loop_for(false, Some((0.0, 4.0)));
    doc.set_active_ping_pong_for(false, false);
    apply_from_doc(&mut world, &mut doc, 2.5);
    let x = x_of(&world, e);
    assert!(
        (x - 99.0).abs() < 1e-4,
        "x = {x}: sob wrap a lane 2 é cíclica e o rabo segura C (99) — é a pose \
         que a emenda revela do outro lado"
    );
}
