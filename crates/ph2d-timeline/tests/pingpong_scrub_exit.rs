//! **O objeto NÃO salta ao scrubar exatamente na SAÍDA de uma strip com fade, sob
//! PingPong** (Enio, 2026-07-23): *"em pingpong funciona perfeitamente quando em
//! play … mas em pause, movendo manualmente o playhead o objeto dá um salto na
//! saída da strip da lane 2."*
//!
//! Mecanismo: no instante EXATO `t == t_end` de uma strip que fez fade-out, o peso
//! vivo já é 0 (o fade é meio-aberto, `weight_at` zera em `lead_end`), mas o `hold`
//! reancorava o frame CONGELADO da strip a peso CHEIO (a borda inclusiva
//! `lead_end >= t`). O Play varre por cima da borda (um tick quase nunca cai
//! exatamente nela); um scrub PAUSADO e frame-snapado estaciona nela, e sob
//! PingPong o loop é filtrado a `None` — nada suaviza a emenda. O corte inclusivo
//! agora vale só para corte DURO (o frame de fim de um container); uma strip que
//! desvanece SOLTA na própria borda.

use ph2d_anim::{AnimValue::Float, Interp::Linear, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    ClipLane, ClipStrip, PropKind::TranslationX, StripSource, TimelineDoc, apply_from_doc,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn x_of(world: &World, e: u64) -> f32 {
    world
        .get::<Transform>(Entity::from_bits(e))
        .unwrap()
        .translation
        .x
}

/// A sprite whose x ramps 10 -> 40 over [0, 6] s. A background lane plays the whole
/// ramp; an overlay lane (on TOP) plays [1, 3) with a fade on both sides. At t = 3
/// (the overlay's exit) the background reads x(3) = 25, while the overlay's frozen
/// last frame is x(2) = 20 — DISTINCT, so an exit spike to 20 is detectable.
fn scene() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();
    doc.insert_key(e, TranslationX, s(0.0), Float(10.0), Linear);
    doc.insert_key(e, TranslationX, s(6.0), Float(40.0), Linear);

    let mut bg = ClipLane::new("bg");
    bg.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 6.0, 6.0));
    doc.stack_mut().push(bg);

    let mut overlay = ClipLane::new("overlay");
    let mut ov = ClipStrip::new(StripSource::Clip(0), 1.0, 3.0, 2.0);
    ov.ease_in = 0.5;
    ov.ease_out = 0.5; // fade dos DOIS lados
    overlay.insert(ov);
    doc.stack_mut().push(overlay);
    (world, doc, e)
}

/// Under PingPong, a paused scrub that lands EXACTLY on the overlay's exit must not
/// jump: the pose at t = 3 is continuous with t just after it (both the background),
/// NOT the overlay's frozen frame.
#[test]
fn a_pingpong_scrub_does_not_jump_at_a_faded_strips_exit() {
    let (mut world, mut doc, e) = scene();
    doc.set_active_loop_for(false, Some((0.0, 6.0)));
    doc.set_active_ping_pong_for(false, true);

    // Just before the exit: the overlay is fading, the pose is near the background.
    apply_from_doc(&mut world, &mut doc, 3.0 - 1.0 / 60.0);
    let before = x_of(&world, e);
    // EXACTLY at the exit — the bug parks here.
    apply_from_doc(&mut world, &mut doc, 3.0);
    let at = x_of(&world, e);
    // Just after: the overlay is gone, the background shows.
    apply_from_doc(&mut world, &mut doc, 3.0 + 1.0 / 60.0);
    let after = x_of(&world, e);

    assert!(
        (at - after).abs() < 1.0,
        "the pose at the exit (t=3, x={at}) jumped away from just-after it \
         (x={after}) — the faded strip's frozen frame spiked in"
    );
    assert!(
        (at - before).abs() < 2.0,
        "and it is continuous with just-before the exit (x={before})"
    );
    // The background reads x(3) = 25; the overlay's frozen frame is x(2) = 20. The
    // pose at the exit must be the BACKGROUND, never the frozen overlay.
    assert!(
        (at - 25.0).abs() < 2.0,
        "x={at} at the exit: expected the background (~25), not the overlay's \
         frozen frame (~20)"
    );
}

/// **No overreach**: with the loop as a WRAP (ping-pong off), the seam design DOES
/// hold the trailing pose across the wrap — that is the approved seamless-loop
/// behaviour, and the exit-release fix must not touch it.
#[test]
fn a_wrap_loop_still_holds_the_trailing_pose() {
    let (mut world, mut doc, e) = scene();
    doc.set_active_loop_for(false, Some((0.0, 6.0)));
    doc.set_active_ping_pong_for(false, false); // WRAP

    apply_from_doc(&mut world, &mut doc, 3.0);
    let wrap = x_of(&world, e);
    // Under a wrap loop the lane is cyclic, so the trailing hold stays — the pose is
    // NOT the plain background here. (The exact value is the seam design's; we only
    // pin that the fix did not collapse it to the background.)
    assert!(
        (wrap - 25.0).abs() > 1.0,
        "wrap x={wrap}: the cyclic trailing hold must survive the exit fix"
    );
}
