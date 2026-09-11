// Prova que a cena do stack_smoke realmente cruza o objeto da esquerda pra direita.
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc};

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, p: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

#[test]
fn the_stack_demo_crosses_the_object_from_left_to_right() {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("StackDemo")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "Left".into());
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, -3.0);
    let right = doc.add_clip("Right".into());
    doc.set_active(right);
    key(doc, bits, PropKind::TranslationX, 0.0, 3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, 3.0);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).unwrap();
    doc.add_strip(lane, 0, 0.0, 3.0);
    doc.add_strip(lane, right, 2.0, 5.0);

    let x_at = |sim: &mut SimWorld, st: &mut TimelineState, t: f64| -> f32 {
        apply_from_doc(sim.world_mut(), &mut st.doc, t);
        let e = ph2d_ecs::Entity::from_bits(bits);
        sim.world().get::<Transform>(e).unwrap().translation.x
    };
    let x0 = x_at(&mut sim, &mut st, 0.5); // só Left
    let xmid = x_at(&mut sim, &mut st, 2.5); // meio do overlap
    let xend = x_at(&mut sim, &mut st, 3.5); // só Right
    eprintln!("x(0.5)={x0}  x(2.5)={xmid}  x(3.5)={xend}");
    assert!(x0 < -2.5, "comeca a ESQUERDA: {x0}");
    assert!(xend > 2.5, "termina a DIREITA: {xend}");
    assert!(
        x0 < xmid && xmid < xend,
        "o crossfade ATRAVESSA (monotonico): {x0} -> {xmid} -> {xend}"
    );
}
