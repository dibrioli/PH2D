//! **O transporte do container é o do CONTAINER** (Enio, 2026-07-22).
//!
//! Dois pedidos, um assunto: (1) editar as lanes de um container reproduz o INTERIOR dele,
//! não a cena; (2) o loop é independente em cada modo — clip, arrange e container. Estes
//! gates pinam o motor (a crate): o solo do interior (`apply_container`) e o isolamento dos
//! três loops (`SetContainerLoop` não vaza para a cena nem para o clip). A costura do shell
//! (qual relógio o transporte anda) tem gates próprios em `timeline_bridge_tests`.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_timeline::{
    PropKind, StackHost, StripSource, TimelineIntent as I, TimelineState, apply_container,
    apply_from_doc, apply_intent,
};

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        PropKind::TranslationX,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

fn x(sim: &SimWorld, bits: u64) -> f64 {
    f64::from(
        sim.world()
            .get::<Transform>(Entity::from_bits(bits))
            .unwrap()
            .translation
            .x,
    )
}

/// One object animated by the ACTIVE clip (ramp x: 0 -> 10 over 2 s). A container "Walk"
/// holds ONE strip of that clip over its interior `[0, 2)`, and the SCENE places the
/// container instance late, at `[5, 7)`. Returns `(sim, state, bits, container)`.
fn scene() -> (SimWorld, TimelineState, u64, usize) {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Mover")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "Step".into());
    key(doc, bits, 0.0, 0.0);
    key(doc, bits, 2.0, 10.0); // the active clip ramps x 0 -> 10

    let walk = doc.add_container("Walk".into());
    let host = StackHost::Container(walk);
    doc.add_lane_in(host, "in".into()).unwrap();
    doc.add_strip_to(host, 0, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();

    let lane = doc.add_lane("scene".into()).unwrap();
    doc.add_strip_to(
        StackHost::Document,
        lane,
        StripSource::Container(u16::try_from(walk).unwrap()),
        5.0,
        7.0,
    )
    .unwrap();
    (sim, st, bits, walk)
}

/// **O playback dentro de um container reproduz o INTERIOR dele, no relógio LOCAL** — não a
/// cena (Enio, 2026-07-22: *"o playback deve ser relativo ao container aberto em edição"*).
///
/// O oráculo é a POSE: no segundo 1 LOCAL do container o interior toca metade da rampa (x=5),
/// enquanto a cena, no segundo 1, não toca o container de todo (a instância só começa em 5 s)
/// — a pose ficaria em 0. São dois relógios distintos e este gate prova que o solo segue o
/// do container.
#[test]
fn apply_container_plays_the_interior_at_its_own_clock() {
    let (mut sim, mut st, bits, walk) = scene();

    // Segundo 1 LOCAL do container: metade da rampa.
    apply_container(sim.world_mut(), &mut st.doc, walk, 1.0, |_| false);
    assert!(
        (x(&sim, bits) - 5.0).abs() < 1e-4,
        "no segundo 1 DO CONTAINER a rampa está na metade (x=5), got {}",
        x(&sim, bits)
    );

    // Controle: a MESMA cena, no segundo 1 da TIMELINE, não toca o container (começa em 5 s)
    // — a pose é a de repouso (rest capturado = 0), bem longe de 5.
    let mut sim2 = SimWorld::new();
    let b2 = sim2
        .world_mut()
        .spawn((Transform::default(), Name::new("Mover")))
        .id()
        .to_bits();
    // rebind the doc's binding to this fresh entity id
    let mut st2 = st.clone();
    for bnd in st2.doc.bindings_mut() {
        bnd.entity = b2;
        bnd.missing = false;
    }
    apply_from_doc(sim2.world_mut(), &mut st2.doc, 1.0);
    assert!(
        (x(&sim2, b2) - 5.0).abs() > 1.0,
        "o solo do CONTAINER e o da CENA são relógios diferentes — a pose não coincide"
    );
}

/// **O loop do container é INDEPENDENTE do da cena e do clip** (Enio, 2026-07-22: *"o loop
/// deve ser independente em cada modo"*) — o vazamento que o report nomeou.
///
/// `SetContainerLoop` escreve SÓ o container; a `SetLoop` da cena e a do clip (Keys) ficam
/// intocadas, e vice-versa. Três loops, três donos.
#[test]
fn the_three_loops_are_independent() {
    let (_sim, mut st, _bits, walk) = scene();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);

    // Arma a cena e o clip primeiro (o Arrange e a Keys).
    st.keys_mode = false;
    apply_intent(
        &mut st,
        &mut ph,
        I::SetLoop {
            range: Some((0.0, 20.0)),
            ping_pong: false,
        },
    );
    st.keys_mode = true;
    apply_intent(
        &mut st,
        &mut ph,
        I::SetLoop {
            range: Some((0.25, 1.75)),
            ping_pong: true,
        },
    );
    st.keys_mode = false;

    // Agora o container, com um range que não colide com nenhum dos dois.
    apply_intent(
        &mut st,
        &mut ph,
        I::SetContainerLoop {
            container: walk,
            range: Some((0.5, 1.5)),
            ping_pong: false,
        },
    );

    // Cada um guarda o SEU, e só o seu.
    assert_eq!(
        st.doc.container_loop(walk),
        (Some((0.5, 1.5)), false),
        "o container guarda o loop dele"
    );
    assert_eq!(
        st.doc.active_loop_for(false),
        Some((0.0, 20.0)),
        "o loop da CENA não foi tocado pelo do container — o vazamento reportado"
    );
    assert_eq!(
        st.doc.active_loop_for(true),
        Some((0.25, 1.75)),
        "e o loop do CLIP (Keys) também não"
    );

    // E o inverso: mexer no container de novo não move os outros.
    apply_intent(
        &mut st,
        &mut ph,
        I::SetContainerLoop {
            container: walk,
            range: None,
            ping_pong: false,
        },
    );
    assert_eq!(st.doc.container_loop(walk), (None, false));
    assert_eq!(
        st.doc.active_loop_for(false),
        Some((0.0, 20.0)),
        "a cena segue firme"
    );
    assert_eq!(
        st.doc.active_loop_for(true),
        Some((0.25, 1.75)),
        "o clip também"
    );
}

/// **`SetContainerLoop` sincroniza o relógio que recebe, e SÓ ele** — o playhead do
/// container ganha o loop; um outro playhead qualquer não é tocado.
#[test]
fn set_container_loop_syncs_the_clock_it_is_handed() {
    let (_sim, mut st, _bits, walk) = scene();
    let mut container_ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut st,
        &mut container_ph,
        I::SetContainerLoop {
            container: walk,
            range: Some((0.5, 1.5)),
            ping_pong: true,
        },
    );
    assert_eq!(
        container_ph.loop_range(),
        Some((0.5, 1.5)),
        "o relógio recebeu o loop"
    );
    assert!(container_ph.is_ping_pong(), "com o modo do container");
}

/// **Um índice de container obsoleto é no-op** — o documento recusa em vez de entrar em pânico.
#[test]
fn set_container_loop_on_a_stale_index_is_a_no_op() {
    let (_sim, mut st, _bits, _walk) = scene();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    let before = st.doc.clone();
    apply_intent(
        &mut st,
        &mut ph,
        I::SetContainerLoop {
            container: 999,
            range: Some((0.0, 1.0)),
            ping_pong: false,
        },
    );
    assert_eq!(
        st.doc, before,
        "container inexistente: o documento não muda"
    );
}

/// **The expression pass runs in the CONTAINER edit view too, on the container's
/// LOCAL clock** (ADR-0144 follow-up). A driven `Y = time*10` on the Mover: at the
/// container's local second 1 the pass writes Y=10. (Mutation: drop the
/// `expr_pass::run` call in `apply_container` -> Y stays 0.)
#[test]
fn apply_container_runs_expressions_on_the_local_clock() {
    let (mut sim, mut st, bits, walk) = scene();
    let tgt = st.doc.bind(bits, PropKind::TranslationY);
    st.doc
        .bindings_mut()
        .iter_mut()
        .find(|b| b.target == tgt)
        .unwrap()
        .expr = Some("time*10".into());

    apply_container(sim.world_mut(), &mut st.doc, walk, 1.0, |_| false);
    let y = f64::from(
        sim.world()
            .get::<Transform>(Entity::from_bits(bits))
            .unwrap()
            .translation
            .y,
    );
    assert!(
        (y - 10.0).abs() < 1e-4,
        "the expr pass runs in the container at local t=1 (Y = time*10 = 10), got {y}"
    );
}

/// **Past the container's AUTHORED end an expression FREEZES with the keys** — it runs
/// on `container_cut`, not the raw local clock (the *expressions extrapolate the
/// container duration* report). With "Walk" cut to 1 s, a driven `Y = time*10` at local
/// t=3 writes Y=10 (frozen at the cut), not 30. (Mutation: raw `t` in `apply_container`
/// -> Y=30, extrapolated past the container's end.)
#[test]
fn an_expression_freezes_at_the_container_cut() {
    let (mut sim, mut st, bits, walk) = scene();
    st.doc.set_container_length_override(walk, Some(1.0));
    let tgt = st.doc.bind(bits, PropKind::TranslationY);
    st.doc
        .bindings_mut()
        .iter_mut()
        .find(|b| b.target == tgt)
        .unwrap()
        .expr = Some("time*10".into());

    apply_container(sim.world_mut(), &mut st.doc, walk, 3.0, |_| false);
    let y = f64::from(
        sim.world()
            .get::<Transform>(Entity::from_bits(bits))
            .unwrap()
            .translation
            .y,
    );
    assert!(
        (y - 10.0).abs() < 1e-4,
        "expr freezes at the container cut (time*10 clamped to local t=1 -> 10), got {y}"
    );
}

/// **A keyed prop's expression goes QUIET outside its strip window** — it rides the
/// strip like the keys do, instead of playing on forever (Report B: expressions must
/// obey the strip's position and size). The Walk strip covers [0,2); a driven
/// `X = value + 100` on the KEYED X drives INSIDE (t=0.5) and is SKIPPED OUTSIDE
/// (t=3), where the keys don't play either, so X keeps the pose it had — proven by a
/// sentinel that survives repeated applies (no drift). (Mutation: drop the
/// keyed-uncovered gate in `expr_pass` -> the expr runs at t=3, X = rest+100 = 100.)
#[test]
fn a_keyed_expression_is_quiet_outside_its_strip_window() {
    let (mut sim, mut st, bits, walk) = scene();
    let tgt = st.doc.bind(bits, PropKind::TranslationX); // X is keyed (ramp 0 -> 10)
    st.doc
        .bindings_mut()
        .iter_mut()
        .find(|b| b.target == tgt)
        .unwrap()
        .expr = Some("value + 100".into());

    // Inside the strip [0,2): the expression drives (composed value + 100 > 100).
    apply_container(sim.world_mut(), &mut st.doc, walk, 0.5, |_| false);
    let inside = x(&sim, bits);
    assert!(
        inside > 100.0,
        "inside the strip the keyed expr drives (got {inside})"
    );

    // Outside the strip window: the keyed expr is quiet, so X keeps its pose. A
    // sentinel must survive repeated applies — no play-outside, no drift.
    sim.world_mut()
        .get_mut::<Transform>(Entity::from_bits(bits))
        .unwrap()
        .translation
        .x = 42.0;
    for _ in 0..3 {
        apply_container(sim.world_mut(), &mut st.doc, walk, 3.0, |_| false);
    }
    let outside = x(&sim, bits);
    assert!(
        (outside - 42.0).abs() < 1e-4,
        "a keyed expr is quiet outside its strip (X holds 42; running it gives rest+100), got {outside}"
    );
}
