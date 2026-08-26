//! Guards for the **contact channel** ([`HIT_COL`], doc 89 folha 13 P1) — the fact that a
//! collision leaves behind something a downstream node can read.
//!
//! A CHILD of `tests.rs`, so `flat()`/`collide_pt` are the same fixtures the shape gates use.

use super::*;

/// The `hit` column of a cooked stream. Panics when absent, because "absent" and "zero" are the
/// two answers this channel must never confuse — a gate that quietly read `0` for a missing
/// column would pass with the whole feature deleted.
fn hits(s: &Stream) -> Vec<f32> {
    match s.get(HIT_COL) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("no `{HIT_COL}` column"),
    }
}

/// **A touch is observable and a miss is not** — the capability in one assertion.
///
/// Two elements, one below the floor and one well above it, in the SAME stream: the collider
/// cannot answer with a constant. Before this channel existed the two came out of the node
/// distinguishable only by `P`/`vel`, which the step rewrites every tick anyway.
///
/// FALSIFIED by a channel that reports contact everywhere (a flag written outside the branch):
/// the flier would read non-zero.
#[test]
fn a_touch_writes_the_hit_channel_and_a_miss_leaves_it_zero() {
    let s = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, -3.0], [0.0, 5.0]]))
        .with("vel", Column::Vec2(vec![[0.0, -1.0], [0.0, -1.0]]));
    let out = collide_pt(&s, SHAPE_PLANE, -2.0, [0.0, 0.0], 2.0, 0.0, 0.0);
    let h = hits(&out);
    assert!(h[0] > 0.0, "o que tocou tem de dizer que tocou: {h:?}");
    assert_eq!(h[1], 0.0, "o que está caindo no ar não tocou nada: {h:?}");
}

/// **The number IS the depth it pushed out**, in world units — not a 1.
///
/// One number carries both facts (*touched* is `> 0` by construction, *how hard* is the value),
/// and this is the half that a bare flag would lose. The element sits a full unit under a floor
/// at `-2`, so the collider owes it exactly `1.0` of push-out.
///
/// FALSIFIED by a flag: `1.0` here is the depth by coincidence, so the fixture uses a depth that
/// is NOT one — `0.25`.
#[test]
fn the_hit_is_the_depth_the_collider_pushed_out() {
    for depth in [1.0f32, 0.25, 3.5] {
        let s = one([0.0, -2.0 - depth], [0.0, -1.0]);
        let out = collide_pt(&s, SHAPE_PLANE, -2.0, [0.0, 0.0], 2.0, 0.0, 0.0);
        assert_eq!(hits(&out)[0], depth, "a profundidade empurrada é o número");
    }
}

/// **A CHAIN reports the DEEPEST contact of the tick, never the last collider's** — the whole
/// reason the binding is `ReadWrite` and not `Write`.
///
/// Colliders compose (demo 29 stacks a ramp and a wall), and with a plain write the channel
/// would answer *"did the LAST one touch me"* — a fact about the order the artist wired the
/// graph rather than about the world, and one that changes when they re-wire it.
///
/// FALSIFIED by a plain `Write`: the deep first contact is overwritten by the shallow second,
/// and by a second collider that touches nothing the channel is cleared outright.
#[test]
fn a_chain_of_colliders_reports_the_deepest_contact_not_the_last() {
    // Deep contact first (1.0 under a floor at -2), then a shallow one (0.1).
    let deep = collide_pt(
        &one([0.0, -3.0], [0.0, 0.0]),
        SHAPE_PLANE,
        -2.0,
        [0.0; 2],
        2.0,
        0.0,
        0.0,
    );
    assert_eq!(hits(&deep)[0], 1.0);
    let shallow = collide_pt(&deep, SHAPE_PLANE, -1.9, [0.0; 2], 2.0, 0.0, 0.0);
    assert_eq!(
        hits(&shallow)[0],
        1.0,
        "o mais fundo do TIQUE, não o último colisor"
    );

    // And a collider the element never reaches must not erase what the one before it saw.
    let far = collide_pt(&deep, SHAPE_PLANE, -100.0, [0.0; 2], 2.0, 0.0, 0.0);
    assert_eq!(hits(&far)[0], 1.0, "quem não tocou não apaga quem tocou");
}

/// **A response the node REFUSED reports no contact.**
///
/// The finite guard drops the whole response, so `P` and `vel` come out untouched — nothing
/// happened, and the channel describes what HAPPENED, not what was attempted. A `hit` written
/// outside that guard would tell a downstream kill to remove an element the collider never
/// managed to move.
///
/// FALSIFIED by writing the channel before the guard.
#[test]
fn a_refused_response_reports_no_contact() {
    let s = one([0.0, -3.0], [f32::NAN, 0.0]);
    let out = collide_pt(&s, SHAPE_PLANE, -2.0, [0.0, 0.0], 2.0, 0.0, 0.0);
    let (p, _) = read(&out);
    assert_eq!(p, [0.0, -3.0], "a resposta foi recusada: nada se moveu");
    assert_eq!(hits(&out)[0], 0.0, "…logo nada tocou");
}

/// **Contact is `> 0` BY CONSTRUCTION, in every shape** — the property that lets one number
/// carry two facts without them ever disagreeing.
///
/// Each shape's test returns `Some` exactly when the depth it computes is strictly positive, so
/// there is no state in which the collider responds and the channel reads zero. Asserted over
/// the three shapes because it is a claim about the SHAPE TABLE, not about the plane.
#[test]
fn every_shape_that_responds_reports_a_strictly_positive_depth() {
    let cases = [
        // (shape, height, centre, radius, a point inside the surface)
        (SHAPE_PLANE, -2.0f32, [0.0f32, 0.0], 2.0f32, [0.0f32, -2.5]),
        (SHAPE_DISC, 0.0, [0.0, 0.0], 2.0, [0.5, 0.0]),
        (SHAPE_BOWL, 0.0, [0.0, 0.0], 2.0, [3.0, 0.0]),
    ];
    for (shape, height, c, radius, p) in cases {
        let out = collide_pt(&one(p, [0.0, 0.0]), shape, height, c, radius, 0.0, 0.0);
        let h = hits(&out)[0];
        assert!(h > 0.0, "shape {shape}: respondeu e reportou {h}");
    }
}

/// **The `Point` collider still writes the channel** — a document that never touched the radius
/// control gets the observable too.
///
/// The channel is the node's OUTPUT SHAPE, not a feature of one radius mode: a stream whose
/// columns depend on which mode is selected would make every downstream reader conditional on a
/// control it does not know about.
#[test]
fn the_channel_exists_whatever_the_radius_mode_is() {
    for part in [
        (RADIUS_POINT, 0.0f32, 0.0f32),
        (RADIUS_FIXED, 0.25, 1.0),
        (RADIUS_SIZE, 0.0, 1.0),
    ] {
        let out = collide(
            &one([0.0, 5.0], [0.0, 0.0]),
            SHAPE_PLANE,
            -2.0,
            [0.0, 0.0],
            2.0,
            0.0,
            0.0,
            part,
            flat(),
            (0.0, 0),
        );
        assert_eq!(hits(&out).len(), 1, "modo {}: a coluna existe", part.0);
    }
}
