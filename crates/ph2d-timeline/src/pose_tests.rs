//! **`pose_at` É o apply, sem escrever.** Estes gates pinam a equivalência campo a campo
//! (ADR-0142): `pose_at(e, t)` tem de dar o MESMO `Transform` que `{ apply_from_doc em t;
//! read Transform }` — byte a byte, porque é a mesma porta. Se o scaffolding do `pose_at`
//! (o corte do clip, o `remapped_time`, a decisão de auto-orient, o skip de track vazia)
//! divergir do apply, o onion desenha o fantasma onde o objeto NÃO estaria — a doença
//! [[feedback_derived_coordinate_seed_must_match_sample]] que este módulo pagou 3×.

use crate::{MotionPath, PathAnchor, PropKind, TimelineDoc, apply_from_doc, pose_at};
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform, World};

/// Todos os campos de pose, byte a byte (é a MESMA porta ⇒ identidade exata, não ε).
#[track_caller]
fn same(a: &Transform, b: &Transform, ctx: &str) {
    let f = |x: f32, y: f32, name: &str| {
        assert_eq!(x.to_bits(), y.to_bits(), "{ctx}: {name}: {x} != {y}");
    };
    f(a.translation.x, b.translation.x, "tx");
    f(a.translation.y, b.translation.y, "ty");
    f(a.rotation, b.rotation, "rot");
    f(a.scale.x, b.scale.x, "sx");
    f(a.scale.y, b.scale.y, "sy");
    f(a.skew_x, b.skew_x, "skew_x");
    f(a.skew_y, b.skew_y, "skew_y");
}

fn key(doc: &mut TimelineDoc, e: u64, prop: PropKind, t: f64, v: f32, interp: Interp) {
    doc.insert_key(
        e,
        prop,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        interp,
    );
}

/// `pose_at` num mundo em repouso vs `apply` num mundo gêmeo. Os dois partem da MESMA pose
/// de repouso e sobrepõem os mesmos campos ⇒ o resultado tem de bater exatamente.
#[track_caller]
fn assert_equivalent(build: impl Fn() -> (World, u64, TimelineDoc), times: &[f64], ctx: &str) {
    for &t in times {
        let (mut wa, e, mut da) = build();
        apply_from_doc(&mut wa, &mut da, t);
        let after_apply = *wa
            .get::<Transform>(Entity::try_from_bits(e).unwrap())
            .unwrap();

        let (wb, e2, db) = build();
        let via_pose = pose_at(&wb, &db, e2, t).unwrap();

        same(&after_apply, &via_pose, &format!("{ctx} @ t={t}"));
    }
}

#[test]
fn pose_at_matches_apply_on_every_scalar_channel() {
    // Um objeto que translada, gira e escala ao mesmo tempo — os cinco canais de
    // sprite-transform sobre uma pose de repouso que NÃO é a identidade (para provar que
    // os campos não-dirigidos ficam de pé e os dirigidos são sobrepostos).
    let build = || {
        let mut w = World::new();
        let mut xf = Transform::from_translation(Vec2::new(3.0, -2.0));
        xf.rotation = 0.5;
        xf.scale = Vec2::new(2.0, 0.5);
        let e = w.spawn(xf).id();
        let mut doc = TimelineDoc::new();
        let b = e.to_bits();
        key(
            &mut doc,
            b,
            PropKind::TranslationX,
            0.0,
            0.0,
            Interp::Linear,
        );
        key(
            &mut doc,
            b,
            PropKind::TranslationX,
            4.0,
            10.0,
            Interp::Linear,
        );
        key(
            &mut doc,
            b,
            PropKind::TranslationY,
            0.0,
            0.0,
            Interp::Linear,
        );
        key(
            &mut doc,
            b,
            PropKind::TranslationY,
            4.0,
            -8.0,
            Interp::Linear,
        );
        key(&mut doc, b, PropKind::Rotation, 0.0, 0.0, Interp::Linear);
        key(&mut doc, b, PropKind::Rotation, 4.0, 3.0, Interp::Linear);
        key(&mut doc, b, PropKind::ScaleX, 0.0, 1.0, Interp::Linear);
        key(&mut doc, b, PropKind::ScaleX, 4.0, 3.0, Interp::Linear);
        key(&mut doc, b, PropKind::ScaleY, 0.0, 1.0, Interp::Linear);
        key(&mut doc, b, PropKind::ScaleY, 4.0, 0.25, Interp::Linear);
        (w, b, doc)
    };
    assert_equivalent(&build, &[0.0, 1.0, 2.5, 4.0, 5.0], "scalars");
}

#[test]
fn pose_at_matches_apply_for_a_position_path_with_auto_orient() {
    // O canal Position (distância → ponto) COM auto-orient: `pose_at` tem de escrever a
    // MESMA translação E a MESMA rotação da tangente que o apply. Uma curva em L para a
    // tangente virar de fato.
    let build = || {
        let mut w = World::new();
        let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let b = e.to_bits();
        let mut doc = TimelineDoc::new();
        doc.bind(b, PropKind::Position);
        key(&mut doc, b, PropKind::Position, 0.0, 0.0, Interp::Linear);
        key(&mut doc, b, PropKind::Position, 4.0, 24.0, Interp::Linear);
        doc.bindings_mut()[0].path = Some(MotionPath::new(vec![
            PathAnchor::corner([0.0, 0.0]),
            PathAnchor::corner([12.0, 0.0]),
            PathAnchor::corner([12.0, 12.0]),
        ]));
        doc.set_auto_orient(b, true);
        (w, b, doc)
    };
    assert_equivalent(&build, &[0.0, 1.0, 2.0, 3.0, 4.0], "position+orient");
}

#[test]
fn pose_at_honours_the_time_remap_clock() {
    // Com um Time Remap 2× (0→0, 2→4) a track de X é lida no tempo-FONTE, não no do
    // playhead. Se `pose_at` amostrasse no `clip_t` cru em vez do `remapped_time`, o
    // fantasma nasceria no lugar errado — este é o gate que pega essa divergência.
    let build = || {
        let mut w = World::new();
        let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let b = e.to_bits();
        let mut doc = TimelineDoc::new();
        key(
            &mut doc,
            b,
            PropKind::TranslationX,
            0.0,
            0.0,
            Interp::Linear,
        );
        key(
            &mut doc,
            b,
            PropKind::TranslationX,
            4.0,
            10.0,
            Interp::Linear,
        );
        key(&mut doc, b, PropKind::TimeRemap, 0.0, 0.0, Interp::Linear);
        key(&mut doc, b, PropKind::TimeRemap, 2.0, 4.0, Interp::Linear);
        (w, b, doc)
    };
    assert_equivalent(&build, &[0.0, 0.5, 1.0, 1.5, 2.0], "time-remap");
}

#[test]
fn pose_at_honours_the_clip_duration_cut() {
    // A duração autorada do clip corta o relógio: além do corte, tanto o apply quanto o
    // `pose_at` congelam no fim. Um fantasma além do fim tem de ler o MESMO instante
    // clampado que o objeto vivo lê.
    let build = || {
        let mut w = World::new();
        let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let b = e.to_bits();
        let mut doc = TimelineDoc::new();
        key(
            &mut doc,
            b,
            PropKind::TranslationX,
            0.0,
            0.0,
            Interp::Linear,
        );
        key(
            &mut doc,
            b,
            PropKind::TranslationX,
            4.0,
            10.0,
            Interp::Linear,
        );
        doc.set_clip_length_override(doc.active_index(), Some(2.0));
        (w, b, doc)
    };
    // t=3 e t=5 estão além do corte de 2 s — os dois têm de congelar no mesmo x.
    assert_equivalent(&build, &[1.0, 2.0, 3.0, 5.0], "clip-cut");
}

#[test]
fn entity_key_times_are_the_deduped_union_across_tracks() {
    let mut doc = TimelineDoc::new();
    let e = 9_u64;
    // X keyado em 0,1,2 ; Y keyado em 1,3 ; TimeRemap em 0,4 (NÃO conta — é o relógio).
    for (p, t) in [
        (PropKind::TranslationX, 0.0),
        (PropKind::TranslationX, 1.0),
        (PropKind::TranslationX, 2.0),
        (PropKind::TranslationY, 1.0), // alinhado com o X em t=1 ⇒ UMA pose
        (PropKind::TranslationY, 3.0),
        (PropKind::TimeRemap, 0.0),
        (PropKind::TimeRemap, 4.0),
    ] {
        key(&mut doc, e, p, t, 0.0, Interp::Linear);
    }
    let times = crate::entity_key_times(&doc, e);
    assert_eq!(
        times,
        vec![0.0, 1.0, 2.0, 3.0],
        "união deduplicada, sem o Time Remap"
    );
}

#[test]
fn animated_entities_lists_each_entity_once() {
    let mut w = World::new();
    let e1 = w
        .spawn(Transform::from_translation(Vec2::ZERO))
        .id()
        .to_bits();
    let e2 = w
        .spawn(Transform::from_translation(Vec2::ZERO))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    // e1 tem X e Y keyados (dois bindings, uma entidade); e2 só X.
    key(
        &mut doc,
        e1,
        PropKind::TranslationX,
        0.0,
        0.0,
        Interp::Linear,
    );
    key(
        &mut doc,
        e1,
        PropKind::TranslationX,
        4.0,
        10.0,
        Interp::Linear,
    );
    key(
        &mut doc,
        e1,
        PropKind::TranslationY,
        0.0,
        0.0,
        Interp::Linear,
    );
    key(
        &mut doc,
        e1,
        PropKind::TranslationY,
        4.0,
        5.0,
        Interp::Linear,
    );
    key(
        &mut doc,
        e2,
        PropKind::TranslationX,
        0.0,
        0.0,
        Interp::Linear,
    );
    key(
        &mut doc,
        e2,
        PropKind::TranslationX,
        4.0,
        3.0,
        Interp::Linear,
    );
    let mut got = crate::animated_entities(&doc);
    got.sort_unstable();
    let mut want = vec![e1, e2];
    want.sort_unstable();
    assert_eq!(
        got, want,
        "cada entidade animada aparece exatamente uma vez"
    );
}
