//! **The Position channel end to end** (ADR-0141): a scalar track whose value is a
//! distance, a trajectory on the binding, and an object that follows it.
//!
//! These drive `apply_from_doc` — the product's own pass — rather than the geometry
//! in isolation, because the thing worth pinning is that a *distance* reaches the
//! Transform as a *place*.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{MotionPath, PathAnchor, PropKind, TimelineDoc, apply_from_doc};

/// An L-shaped path: right 10, then up 10, with square corners. Its total length is
/// 20 and every point on it is trivially checkable by hand, which is what makes it a
/// good oracle for "did the object go where the distance says".
fn ell() -> MotionPath {
    MotionPath::new(vec![
        PathAnchor::corner([0.0, 0.0]),
        PathAnchor::corner([10.0, 0.0]),
        PathAnchor::corner([10.0, 10.0]),
    ])
}

/// A world with one entity, and a document with a Position binding on it carrying
/// `path` plus the keys `(t, s)`.
fn rig(path: MotionPath, keys: &[(f64, f32)]) -> (World, Entity, TimelineDoc) {
    let mut w = World::new();
    let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
    let mut doc = TimelineDoc::new();
    for &(t, s) in keys {
        doc.insert_key(
            e.to_bits(),
            PropKind::Position,
            RationalTime::from_seconds(t),
            AnimValue::Float(s),
            Interp::Linear,
        );
    }
    let i = doc
        .bindings()
        .iter()
        .position(|b| b.prop == PropKind::Position)
        .expect("the key bound one");
    doc.bindings_mut()[i].path = Some(path);
    (w, e, doc)
}

fn pos(w: &World, e: Entity) -> [f32; 2] {
    let xf = w.get::<Transform>(e).unwrap();
    [xf.translation.x, xf.translation.y]
}

fn near(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3
}

/// **The channel, whole.** The track holds distances; the object lands on the
/// trajectory at those distances. Mid-track it is around the CORNER — which is the
/// point of the whole design: a straight line in the graph editor is not a straight
/// line on the canvas.
#[test]
fn a_position_track_walks_the_object_along_its_path() {
    // 0 s at the start, 2 s at the far end (20 units of travel).
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);

    apply_from_doc(&mut w, &mut doc, 0.0);
    assert!(near(pos(&w, e), [0.0, 0.0]), "start: {:?}", pos(&w, e));

    // A quarter of the time, a quarter of the DISTANCE: 2.5 units along the first
    // leg. ⚠️ This sample is NOT at a symmetric fraction on purpose — at halves and
    // at segment boundaries a curve walked by its PARAMETER agrees with one walked
    // by arc length, so a gate that only checked those would pass on an engine that
    // never measured anything (it did: the mutation that swaps `inv_arclen` for
    // `s / span` survived this file until this line existed).
    apply_from_doc(&mut w, &mut doc, 0.25);
    assert!(near(pos(&w, e), [2.5, 0.0]), "quarter: {:?}", pos(&w, e));

    // Half the time, half the DISTANCE: 10 units along an L is the corner.
    apply_from_doc(&mut w, &mut doc, 1.0);
    let mid = pos(&w, e);
    assert!(near(mid, [10.0, 0.0]), "halfway is the corner, got {mid:?}");

    // Three quarters: 15 units — 10 across, then 5 up the second leg.
    apply_from_doc(&mut w, &mut doc, 1.5);
    assert!(near(pos(&w, e), [10.0, 5.0]), "{:?}", pos(&w, e));

    apply_from_doc(&mut w, &mut doc, 2.0);
    assert!(near(pos(&w, e), [10.0, 10.0]), "{:?}", pos(&w, e));
}

/// **The trajectory is not a straight line, and a linear track proves it.** Between
/// the two keys the value interpolates linearly, so a channel that had quietly
/// become "lerp the two endpoints" would put the object on the DIAGONAL. It is the
/// same failure the FLIP tween documented one module over: a lerp cuts the chord.
#[test]
fn the_object_follows_the_curve_and_not_the_chord_between_its_ends() {
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);
    apply_from_doc(&mut w, &mut doc, 1.0);
    let mid = pos(&w, e);
    let chord = [5.0, 5.0]; // where a lerp of the two endpoints would land
    let off_chord = ((mid[0] - chord[0]).powi(2) + (mid[1] - chord[1]).powi(2)).sqrt();
    assert!(
        off_chord > 5.0,
        "the object sat {off_chord:.3} from the straight-line answer — that is the \
         straight-line answer, and the path was ignored"
    );
}

/// **A Position binding's `rest` is a DISTANCE**, captured by projecting the authored
/// pose onto the path — never a coordinate, and never zero.
///
/// Zero is the start of the trajectory. `rest` exists precisely so a lane that only
/// partially covers has something honest to fade in *from* (ADR-0115 R5), and the
/// failure it was invented to prevent — the sprite flying to an origin nobody chose —
/// is exactly what a zero here would reintroduce.
#[test]
fn a_position_bindings_rest_is_where_the_pose_sits_on_the_path_not_zero() {
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);
    // Park the object beside the second leg, level with 7 units up it — i.e. 17
    // units along the path — before the timeline has ever written to it.
    w.get_mut::<Transform>(e).unwrap().translation = Vec2::new(10.6, 7.0);

    apply_from_doc(&mut w, &mut doc, 0.0);

    let rest = doc.bindings()[0]
        .rest
        .expect("captured on the first live frame");
    assert!(
        (rest - 17.0).abs() < 1e-2,
        "rest is {rest:.4}; the pose sits 17 units along this path"
    );
    assert!(
        rest > 1.0,
        "a rest of ~0 means the base is the START of the trajectory, which is the \
         'fly to the origin' bug this field exists to prevent"
    );
}

/// A Position binding with no trajectory yet writes NOTHING — the same silence an
/// empty track gets. A default would be a place nobody authored.
#[test]
fn a_position_binding_without_a_path_leaves_the_object_alone() {
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);
    doc.bindings_mut()[0].path = None;
    w.get_mut::<Transform>(e).unwrap().translation = Vec2::new(3.0, 4.0);

    apply_from_doc(&mut w, &mut doc, 1.0);

    assert_eq!(pos(&w, e), [3.0, 4.0], "the object must not have moved");
    assert_eq!(
        doc.bindings()[0].rest,
        None,
        "and no rest was invented for a path that does not exist"
    );
}

/// A distance past the end of the trajectory means "at the end" — a key can outlive
/// the anchor that justified it (the artist deletes a leg), and the honest answer is
/// the last point on the path, not a panic and not the origin.
#[test]
fn a_distance_past_the_end_of_the_path_holds_at_the_end() {
    let (mut w, e, mut doc) = rig(ell(), &[(0.0, 0.0), (2.0, 500.0)]);
    apply_from_doc(&mut w, &mut doc, 2.0);
    assert!(near(pos(&w, e), [10.0, 10.0]), "{:?}", pos(&w, e));
}

/// The Position track survives the file, and so does its trajectory.
#[test]
fn a_position_binding_and_its_path_round_trip_through_the_document() {
    let (_, _, doc) = rig(ell(), &[(0.0, 0.0), (2.0, 20.0)]);
    let bytes = doc.to_bytes().unwrap();
    let back = TimelineDoc::from_bytes(&bytes).unwrap();
    let path = back.bindings()[0]
        .path
        .as_ref()
        .expect("the trajectory came back");
    assert_eq!(path.anchors(), ell().anchors());
    assert!(
        (path.length() - 20.0).abs() < 1e-9,
        "the derived table was rebuilt on load: {}",
        path.length()
    );
}
