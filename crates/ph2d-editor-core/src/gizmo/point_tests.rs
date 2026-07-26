//! **The point handles draw where the anchors are, and are grabbable there.**
//!
//! Split out of `point.rs` when W-J2b turned the single dot into a list (LOC).

use super::*;
use crate::interaction::HitIndex;

fn view(handles: Vec<PointHandle>) -> PointGizmoView {
    PointGizmoView {
        handles,
        snap_world: None,
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 1000.0,
        window_h: 1000.0,
        canvas: Rect::new(0.0, 0.0, 1000.0, 1000.0),
    }
}

fn a(key: u64, world: [f32; 2]) -> PointHandle {
    PointHandle {
        key,
        kind: PointHandleKind::AnchorA,
        world,
    }
}

fn b(key: u64, world: [f32; 2]) -> PointHandle {
    PointHandle {
        key,
        kind: PointHandleKind::AnchorB,
        world,
    }
}

/// Paint into a fresh index + map and hand both back.
fn paint(v: &PointGizmoView) -> (HitIndex, BTreeMap<NodeId, PointHandle>) {
    let mut scene = VectorScene::new();
    let mut hits = HitIndex::default();
    let mut map = BTreeMap::new();
    paint_point_gizmo(&mut scene, v, Theme::default(), &mut hits, &mut map);
    (hits, map)
}

fn screen(v: &PointGizmoView, w: [f32; 2]) -> [f32; 2] {
    world_to_screen_px(
        v.camera_center,
        v.camera_height_world,
        v.window_w,
        v.window_h,
        w,
    )
}

/// **A Down on the dot's screen position hits its handle, and the map says
/// whose it is.**
///
/// The whole point of the gizmo: the anchor must be grabbable on the canvas.
/// The hit is registered at the anchor's PROJECTED position, so it tracks the
/// joint under pan/zoom the same way every other gizmo handle does.
///
/// Mutation-tested: dropping the `hit_index.register` call leaves nothing to
/// hit, and this goes red — the dot would paint but never be draggable.
#[test]
fn the_anchor_dot_is_hittable_where_it_is_drawn() {
    let v = view(vec![a(7, [2.0, 1.0])]);
    let (hits, map) = paint(&v);
    let s = screen(&v, [2.0, 1.0]);
    let id = hits.hit(s[0], s[1]).expect(
        "a Down on the anchor's screen position did not hit the joint-anchor handle — the \
         pivot would be undraggable on the canvas",
    );
    assert_eq!(map.get(&id).copied(), Some(a(7, [2.0, 1.0])));
    // And a point far away misses it (the hit is a handle, not the whole canvas).
    assert_eq!(hits.hit(s[0] + 200.0, s[1] + 200.0), None);
}

/// **The B end is grabbable where it is drawn.** Without its own hit rect the
/// second anchor would paint and never take a drag.
///
/// Mutation-tested: painting only the A pass goes red.
#[test]
fn the_b_handle_is_hittable_where_it_is_drawn() {
    let v = view(vec![a(3, [0.0, 0.0]), b(3, [2.0, 1.0])]);
    let (hits, map) = paint(&v);

    let s = screen(&v, [2.0, 1.0]);
    let hit_b = hits
        .hit(s[0], s[1])
        .expect("the B anchor must be grabbable");
    assert_eq!(map[&hit_b].kind, PointHandleKind::AnchorB);

    let sa = screen(&v, [0.0, 0.0]);
    let hit_a = hits.hit(sa[0], sa[1]).expect("and A at its own position");
    assert_eq!(map[&hit_a].kind, PointHandleKind::AnchorA);
}

/// **A coincident pair is still two handles.** A Pin at rest anchors both
/// bodies at the same world point, so the two marks land on each other — A
/// takes the inner square and B the band outside it.
///
/// This is the gate that fails if the registration order is swapped (B last
/// would swallow A entirely, and the pivot would become undraggable on every
/// Pin and Weld in the scene — i.e. on the common case).
#[test]
fn a_coincident_pair_gives_a_the_centre_and_b_the_band() {
    let v = view(vec![a(11, [1.0, -1.0]), b(11, [1.0, -1.0])]);
    let (hits, map) = paint(&v);
    let s = screen(&v, [1.0, -1.0]);

    let centre = hits.hit(s[0], s[1]).expect("dead centre belongs to A");
    assert_eq!(
        map[&centre].kind,
        PointHandleKind::AnchorA,
        "dead centre belongs to A"
    );

    let band = hits
        .hit(s[0] + JOINT_ANCHOR_RING_PX - 1.0, s[1])
        .expect("the band outside A's square must still reach B");
    assert_eq!(
        map[&band].kind,
        PointHandleKind::AnchorB,
        "the band outside A's square must reach B, or a Pin's B end could never be grabbed"
    );
}

/// **Two joints are four handles with four distinct ids, each answering for
/// itself.** This is what makes "every joint has handles" a feature rather than
/// a way to drag the wrong joint.
///
/// Mutation-tested: dropping `key` from [`point_handle_id`] (so all A handles
/// share one id) makes the map hold ONE entry per side and the first joint's
/// dot resolve to the second joint — the shell would author an anchor on a
/// joint the artist never touched.
#[test]
fn two_joints_are_four_handles_with_four_distinct_ids() {
    let v = view(vec![
        a(101, [-3.0, 0.0]),
        b(101, [-1.0, 0.0]),
        a(202, [1.0, 0.0]),
        b(202, [3.0, 0.0]),
    ]);
    let (hits, map) = paint(&v);
    assert_eq!(map.len(), 4, "four handles must own four distinct hit ids");

    for h in &v.handles {
        let s = screen(&v, h.world);
        let id = hits
            .hit(s[0], s[1])
            .expect("every published handle must be grabbable at its own position");
        assert_eq!(
            map.get(&id).copied(),
            Some(*h),
            "the hit at {:?} resolved to {:?}, not to the handle drawn there — a drag would \
             author the wrong joint's anchor",
            h.world,
            map.get(&id)
        );
    }
}

/// **No handles, nothing registered.** The empty list is never published (the
/// shell hands out `None`), but the painter must not invent a mark for it.
#[test]
fn an_empty_list_paints_nothing() {
    let (hits, map) = paint(&view(vec![]));
    assert_eq!(hits.hit(500.0, 500.0), None);
    assert!(map.is_empty());
}

/// **Every mark is at least as grabbable as it is visible.**
///
/// A dot drawn bigger than the rect that catches it is a dot the artist clicks
/// on and nothing happens — the failure mode of *making the marks bigger*
/// (Enio, 2026-07-25) if only the drawing half moves. Pinned per side, plus the
/// nesting that makes a coincident pair two handles.
///
/// Mutation-tested: leaving A's hit half at the old `HANDLE_SIZE_PX * 0.5` (6)
/// while the dot draws at 9 goes red here.
#[test]
fn the_hit_rects_are_never_smaller_than_the_marks() {
    assert!(
        hit_half_px(PointHandleKind::AnchorA) >= JOINT_ANCHOR_DOT_PX,
        "A's dot is drawn larger than its hit rect — visible and ungrabbable"
    );
    assert!(
        hit_half_px(PointHandleKind::AnchorB) >= JOINT_ANCHOR_RING_PX,
        "B's ring is drawn larger than its hit rect — visible and ungrabbable"
    );
    assert!(
        hit_half_px(PointHandleKind::AnchorB) - hit_half_px(PointHandleKind::AnchorA) >= 4.0,
        "the band between the two squares is where a coincident pair's B end is grabbed; \
         narrower than a few pixels and a Pin's B anchor is unreachable in practice"
    );
    assert!(
        SNAP_CROSS_PX > hit_half_px(PointHandleKind::AnchorB),
        "the snap crosshair must reach past the outermost handle, or the mark that explains \
         the magnet is hidden under the marks it explains"
    );
}

/// **The snap crosshair draws and takes no hit.** It is a readout — a mark that
/// says *this is why the dot stopped* — and a readout that swallows the pointer
/// would steal the drag it is describing.
#[test]
fn the_snap_mark_is_drawn_without_taking_the_pointer() {
    let mut v = view(vec![a(1, [0.0, 0.0])]);
    v.snap_world = Some([3.0, 3.0]);
    let (hits, map) = paint(&v);
    let s = screen(&v, [3.0, 3.0]);
    assert_eq!(
        hits.hit(s[0], s[1]),
        None,
        "the snap crosshair must not register a hit"
    );
    assert_eq!(map.len(), 1, "only the A handle registers");
}

/// **The dot moves with the anchor.** Two anchors project to two different
/// screen positions, and the hit follows — so the handle sits on the joint, not
/// at a fixed screen spot.
#[test]
fn the_hit_follows_the_anchor() {
    for anchor in [[0.0, 0.0], [3.0, -2.0]] {
        let v = view(vec![a(5, anchor)]);
        let (hits, map) = paint(&v);
        let s = screen(&v, anchor);
        let id = hits.hit(s[0], s[1]).expect("hit at the projected anchor");
        assert_eq!(map[&id].world, anchor);
    }
}

/// **Where an anchor and a parameter grip land on each other, the ANCHOR wins.**
///
/// A limit wall can be grabbed anywhere along its tick and a length ring
/// anywhere on its circle; an anchor is a single point with nowhere else to go.
/// So the anchors register LAST (`PAINT_ORDER`) and the backwards walk of
/// `HitIndex::hit` hands them the shared pixel.
///
/// Mutation-tested: moving the anchors to the front of `PAINT_ORDER` makes the
/// grip swallow the dot, and this goes red.
#[test]
fn an_anchor_beats_a_parameter_grip_on_a_shared_pixel() {
    let p = |kind| PointHandle {
        key: 42,
        kind,
        world: [1.0, 1.0],
    };
    let v = view(vec![
        p(PointHandleKind::AnchorA),
        p(PointHandleKind::Length),
        p(PointHandleKind::LimitMin),
    ]);
    let (hits, map) = paint(&v);
    let s = screen(&v, [1.0, 1.0]);
    let id = hits.hit(s[0], s[1]).expect("something is there");
    assert_eq!(
        map[&id].kind,
        PointHandleKind::AnchorA,
        "a parameter grip took the anchor's own pixel — the anchor has nowhere \
         else to be grabbed, the grip has its whole line"
    );
}
