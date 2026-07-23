//! **Sob PINGPONG a lane NÃO é cíclica** (Enio, 2026-07-23): o playhead REFLETE nas
//! bordas da faixa e nunca cruza a emenda do loop, então a pose que o FIM do loop
//! deixa não pode aparecer no gap de abertura — *"Se estamos num PingPong e no
//! retorno ao início há um gap, ele deve parar exatamente na posição inicial da
//! strip … ao encontrar a strip o objeto ainda está no lugar onde estava antes do
//! gap iniciar. Porque deveriam haver saltos bizarros no pingpong?"*
//!
//! O bug que isto pina: `stack_frames` entregava a FAIXA ao `hold_at` sem consultar
//! o MODO, então PingPong armado (que guarda a faixa no doc como todo loop) fazia o
//! gap inicial mostrar a pose do fim — o salto no frame 1 do smoke de 2026-07-23,
//! destampado pelo fix do additive de container (antes o mesmo hold contribuía um
//! zero silencioso). O irmão: ROOTED num container, o frame 0 usava o loop da CENA
//! — o relógio errado bracketing uma pilha que ele não conhece.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{ClipLane, ClipStrip, PropKind, StripSource, TimelineDoc, apply_from_doc};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// One sprite at rest, one clip ramping x from 10 at t=0 to 20 at t=2.
fn scene() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(10.0),
        Interp::Linear,
    );
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(2.0),
        AnimValue::Float(20.0),
        Interp::Linear,
    );
    (world, doc, e)
}

fn x_of(world: &World, e: u64) -> f32 {
    world
        .get::<Transform>(Entity::from_bits(e))
        .unwrap()
        .translation
        .x
}

/// The scene's stack: one lane, one strip `[1, 3)` playing the clip — an initial
/// gap `[0, 1)` under whatever loop the test arms.
fn stack(doc: &mut TimelineDoc) {
    let mut lane = ClipLane::new("Lane 1");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 1.0, 3.0, 2.0));
    doc.stack_mut().push(lane);
}

/// PingPong armed: scrubbing into the initial gap writes NOTHING — the object
/// stays exactly where the strip left it, which is what lets the reflected
/// playhead cross the gap and find the object still at the strip's first pose.
/// The same range as a WRAP loop (ping-pong off) does hold the loop-end pose in
/// that gap — the seamless-wrap design already approved with fades — so this
/// test also pins that the fix did not overreach.
#[test]
fn a_ping_pong_gap_never_shows_the_loops_end() {
    let (mut world, mut doc, e) = scene();
    stack(&mut doc);
    doc.set_active_loop_for(false, Some((0.0, 3.0)));
    doc.set_active_ping_pong_for(false, true);

    apply_from_doc(&mut world, &mut doc, 2.0); // mid-strip: clip t 1.0
    assert!((x_of(&world, e) - 15.0).abs() < 1e-4);
    apply_from_doc(&mut world, &mut doc, 0.5); // the initial gap
    let x = x_of(&world, e);
    assert!(
        (x - 15.0).abs() < 1e-4,
        "x = {x}: under PING-PONG the opening gap must write nothing (the object \
         rests where it was); 20.0 means the loop-end wrap-hold fired on a loop \
         the playhead never wraps"
    );

    doc.set_active_ping_pong_for(false, false); // the SAME range, now a WRAP loop
    apply_from_doc(&mut world, &mut doc, 0.5);
    let x = x_of(&world, e);
    assert!(
        (x - 20.0).abs() < 1e-4,
        "x = {x}: under a WRAP loop the opening gap holds what the loop's end \
         leaves asserting (20.0) — the approved seamless-wrap design must survive"
    );
}

/// ROOTED (the Containers editing solo), frame 0 is the container's interior and
/// the loop that governs its holds is the CONTAINER's own — never the scene's
/// Arrange loop, which is another clock's bracket. Three poses, one gap:
/// scene loop armed + container loop unset = nothing written; container's own
/// WRAP loop = the interior's end pose; container's own PINGPONG = nothing again.
#[test]
fn a_rooted_interior_wraps_by_the_containers_own_loop_not_the_scenes() {
    let (mut world, mut doc, e) = scene();
    assert_eq!(doc.add_container("C".to_string()), 0);
    let mut inner = ClipLane::new("inner");
    inner.insert(ClipStrip::new(StripSource::Clip(0), 1.0, 2.0, 1.0));
    doc.container_stack_mut(0).unwrap().push(inner);
    // The scene's own Arrange loop, WRAP, bracketing everything.
    doc.set_active_loop_for(false, Some((0.0, 8.0)));
    doc.set_active_ping_pong_for(false, false);

    ph2d_timeline::apply_container(&mut world, &mut doc, 0, 1.5, |_| false); // clip t 0.5
    assert!((x_of(&world, e) - 12.5).abs() < 1e-4);
    ph2d_timeline::apply_container(&mut world, &mut doc, 0, 0.5, |_| false); // interior gap
    let x = x_of(&world, e);
    assert!(
        (x - 12.5).abs() < 1e-4,
        "x = {x}: the scene's Arrange loop must not reach a rooted interior's \
         holds (15.0 means the wrong clock's wrap-hold fired)"
    );

    doc.set_container_loop(0, Some((0.0, 2.0)), false); // the container's OWN wrap
    ph2d_timeline::apply_container(&mut world, &mut doc, 0, 0.5, |_| false);
    let x = x_of(&world, e);
    assert!(
        (x - 15.0).abs() < 1e-4,
        "x = {x}: the container's own WRAP loop must make its interior cyclic \
         (hold the interior's end pose, 15.0, in the opening gap)"
    );

    doc.set_container_loop(0, Some((0.0, 2.0)), true); // its own PINGPONG
    ph2d_timeline::apply_container(&mut world, &mut doc, 0, 1.5, |_| false);
    ph2d_timeline::apply_container(&mut world, &mut doc, 0, 0.5, |_| false);
    let x = x_of(&world, e);
    assert!(
        (x - 12.5).abs() < 1e-4,
        "x = {x}: a container's ping-pong gap writes nothing, exactly like the scene's"
    );
}
