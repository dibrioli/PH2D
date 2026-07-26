//! **The point gizmo** — the grabbable dots at a joint's two world anchors.
//!
//! The three `GizmoView` publishers ([`super::paint_sprite_gizmo`] and friends)
//! are all BOXES with scale/rotate handles, built from drawable geometry (a
//! sprite quad, a vector bbox). A physics joint is a *point*: it carries a
//! `Transform` (its authored anchor) but no geometry, so it publishes no box and
//! — until this — had no canvas handle at all. Its anchor was authorable only by
//! typing into the Inspector's Position fields.
//!
//! This is that missing handle. The shell publishes a [`PointGizmoView`] for a
//! selected joint (its `Transform.translation`, in world); the painter draws a
//! dot and registers a [`super::ids::GIZMO_JOINT_ANCHOR`] hit. A Down on it opens
//! a plain [`super::GizmoDragKind::Translate`] drag of the joint ENTITY
//! (shell-side, keyed on the selection since a joint has no sprite to pick), and
//! the existing gizmo math moves its `Transform.translation`. So dragging the dot
//! moves the pivot, and `rebuild_from_rest` re-derives the joint's local anchors
//! from it — one global undo step, exactly like moving a sprite.
//!
//! # Two ends, one vocabulary (W-J2)
//!
//! A joint binds **two** bodies and each end attaches somewhere on its own, so
//! one handle could only ever author half of it. The second dot is body B's
//! anchor — and it is drawn in the **same amber**, as a hollow ring rather than
//! a filled dot, because the two are the same kind of thing at two ends. That is
//! the vocabulary the joint overlay already speaks (W-J1 draws A's ownership line
//! solid and B's dashed): *solid is A, open is B*, said once and meant twice. Two
//! hues would have claimed they are different kinds of thing.
//!
//! ⚠️ **A Pin at rest has both anchors at the SAME point** — two bodies sharing a
//! place is what a pin is. So the marks are drawn concentric and the hit rects
//! are nested: A takes the inner square, B the band outside it. Nudging one dot
//! aside to make room would draw an anchor where it is not.

use super::camera::world_to_screen_px;
use super::hit::ids;
use super::paint::HANDLE_SIZE_PX;
use crate::interaction::HitIndex;
use crate::zones::Rect;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, BezPath, Circle, Color as VelloColor, Point, Stroke, VectorScene};

/// The point handles to draw for the selected joint — the anchor gizmo.
///
/// Carries only the camera fields the projection needs. A point has no bbox and
/// no rotation, which is exactly why it could not be a [`super::GizmoView`]: that
/// type is a rotated box, and there is nothing here to rotate.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointGizmoView {
    /// World position of the A handle — the joint's authored anchor on body A,
    /// which is also the value the Inspector's Position fields edit.
    pub anchor_world: [f32; 2],
    /// World position of the B handle, or `None` when there is no body B to
    /// anchor to (a half-authored joint, a renamed body). An anchor with no body
    /// is not a place, so no handle is offered for it.
    pub anchor_b_world: Option<[f32; 2]>,
    /// The snap candidate the live drag has caught, if any — drawn as a
    /// crosshair through the dot so the artist can see *why* it stopped moving
    /// freely. `None` whenever nothing is snapped (including when no drag is in
    /// flight).
    pub snap_world: Option<[f32; 2]>,
    pub camera_center: [f32; 2],
    pub camera_height_world: f32,
    pub window_w: f32,
    pub window_h: f32,
    /// Canvas rect in screen px — carried for symmetry with [`super::GizmoView`]
    /// (a future scissor against chrome would read it; the dot ignores it today).
    pub canvas: Rect,
}

/// Visual radius of the anchor dot, screen px. Smaller than the [`HANDLE_SIZE_PX`]
/// hit box, so the point is easy to catch but reads as a small marker rather than
/// a fat button.
const JOINT_ANCHOR_DOT_PX: f32 = 6.0;

/// Visual radius of the B ring, screen px — outside A's dot, so a coincident
/// pair reads as one mark inside another rather than as one mark.
const JOINT_ANCHOR_RING_PX: f32 = 10.0;

/// Half-extent of the B handle's hit square, screen px. Twice A's, so the band
/// between them is B's grab zone when the two anchors coincide (A is registered
/// last and therefore wins the inner square).
const JOINT_ANCHOR_B_HALF_PX: f32 = HANDLE_SIZE_PX;

/// Arm length of the snap crosshair, screen px — past the B ring, so the mark is
/// legible even when both handles sit on the snapped point.
const SNAP_CROSS_PX: f32 = 14.0;

/// Amber — the joint overlay's colour, so the grabbable dot reads as "the thing
/// you already see in the overlay, now grab it" rather than as a new element.
/// Theme-independent for the same reason the pivot ring is (the meaning does not
/// change between Forge / Workshop / Sunstone / Blueprint).
fn anchor_color() -> VelloColor {
    VelloColor::from_rgba8(0xFA, 0xBF, 0x40, 0xFF) // matches `JOINT_RGBA` in the physics overlay
}

/// Draw the joint's anchor handles and register their hit rects, so a canvas
/// Down on one is recognised (`GIZMO_JOINT_ANCHOR` / `..._B`) and opens an
/// anchor drag of that end.
///
/// Order is load-bearing twice: the **snap crosshair first** (it is a backdrop
/// for the marks that sit on it), then **B, then A** — `HitIndex::hit` walks
/// backwards, so the last registration wins, and A must win the square it shares
/// with B on a coincident pair.
pub fn paint_point_gizmo(
    scene: &mut VectorScene,
    view: &PointGizmoView,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let _ = theme; // colour is theme-independent (see `anchor_color`)
    let project = |w: [f32; 2]| {
        world_to_screen_px(
            view.camera_center,
            view.camera_height_world,
            view.window_w,
            view.window_h,
            w,
        )
    };
    if let Some(snap) = view.snap_world {
        paint_snap_cross(scene, project(snap));
    }
    if let Some(b) = view.anchor_b_world {
        let s = project(b);
        hit_index.register(
            ids::GIZMO_JOINT_ANCHOR_B,
            Rect::new(
                s[0] - JOINT_ANCHOR_B_HALF_PX,
                s[1] - JOINT_ANCHOR_B_HALF_PX,
                JOINT_ANCHOR_B_HALF_PX * 2.0,
                JOINT_ANCHOR_B_HALF_PX * 2.0,
            ),
        );
        // Hollow — the B end, in the same amber as A (module docs).
        let ring = Circle::new(
            Point::new(f64::from(s[0]), f64::from(s[1])),
            f64::from(JOINT_ANCHOR_RING_PX),
        );
        scene.inner_mut().stroke(
            &Stroke::new(1.5),
            Affine::IDENTITY,
            anchor_color(),
            None,
            &ring,
        );
    }
    let s = project(view.anchor_world);
    // Generous grab: the hit rect is a full handle, larger than the visual dot,
    // so the point is as catchable as any gizmo handle.
    let hit = Rect::new(
        s[0] - HANDLE_SIZE_PX * 0.5,
        s[1] - HANDLE_SIZE_PX * 0.5,
        HANDLE_SIZE_PX,
        HANDLE_SIZE_PX,
    );
    hit_index.register(ids::GIZMO_JOINT_ANCHOR, hit);
    // Filled dot — the A end.
    let dot = Circle::new(
        Point::new(f64::from(s[0]), f64::from(s[1])),
        f64::from(JOINT_ANCHOR_DOT_PX),
    );
    scene.inner_mut().fill(
        ph2d_vector::Fill::NonZero,
        Affine::IDENTITY,
        anchor_color(),
        None,
        &dot,
    );
}

/// A crosshair through the snapped candidate — the only thing on screen that
/// says *the dot stopped here on purpose*. Without it a snap is indistinguishable
/// from a drag that will not track the cursor.
fn paint_snap_cross(scene: &mut VectorScene, s: [f32; 2]) {
    let (cx, cy) = (f64::from(s[0]), f64::from(s[1]));
    let arm = f64::from(SNAP_CROSS_PX);
    let mut path = BezPath::new();
    path.move_to(Point::new(cx - arm, cy));
    path.line_to(Point::new(cx + arm, cy));
    path.move_to(Point::new(cx, cy - arm));
    path.line_to(Point::new(cx, cy + arm));
    scene.inner_mut().stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        anchor_color(),
        None,
        &path,
    );
}

#[cfg(test)]
mod tests {
    //! **The point handle draws where the anchor is, and is grabbable there.**

    use super::*;
    use crate::interaction::HitIndex;

    fn view(anchor: [f32; 2]) -> PointGizmoView {
        PointGizmoView {
            anchor_world: anchor,
            anchor_b_world: None,
            snap_world: None,
            camera_center: [0.0, 0.0],
            camera_height_world: 10.0,
            window_w: 1000.0,
            window_h: 1000.0,
            canvas: Rect::new(0.0, 0.0, 1000.0, 1000.0),
        }
    }

    /// **A Down on the dot's screen position hits `GIZMO_JOINT_ANCHOR`.**
    ///
    /// The whole point of the wave: the anchor must be grabbable on the canvas.
    /// The hit is registered at the anchor's PROJECTED position, so it tracks the
    /// joint under pan/zoom the same way every other gizmo handle does.
    ///
    /// Mutation-tested: dropping the `hit_index.register` call leaves nothing to
    /// hit, and this goes red — the dot would paint but never be draggable.
    #[test]
    fn the_anchor_dot_is_hittable_where_it_is_drawn() {
        let v = view([2.0, 1.0]);
        let mut scene = VectorScene::new();
        let mut hits = HitIndex::default();
        paint_point_gizmo(&mut scene, &v, Theme::default(), &mut hits);

        let s = world_to_screen_px(
            v.camera_center,
            v.camera_height_world,
            v.window_w,
            v.window_h,
            v.anchor_world,
        );
        assert_eq!(
            hits.hit(s[0], s[1]),
            Some(ids::GIZMO_JOINT_ANCHOR),
            "a Down on the anchor's screen position did not hit the joint-anchor \
             handle — the pivot would be undraggable on the canvas"
        );
        // And a point far away misses it (the hit is a small handle, not the
        // whole canvas).
        assert_ne!(
            hits.hit(s[0] + 200.0, s[1] + 200.0),
            Some(ids::GIZMO_JOINT_ANCHOR)
        );
    }

    /// **The B end is grabbable where it is drawn** — the whole of W-J2's first
    /// half. Without its own hit rect the second anchor would paint and never
    /// take a drag.
    ///
    /// Mutation-tested: dropping the `GIZMO_JOINT_ANCHOR_B` registration goes red.
    #[test]
    fn the_b_handle_is_hittable_where_it_is_drawn() {
        let mut v = view([0.0, 0.0]);
        v.anchor_b_world = Some([2.0, 1.0]);
        let mut scene = VectorScene::new();
        let mut hits = HitIndex::default();
        paint_point_gizmo(&mut scene, &v, Theme::default(), &mut hits);

        let s = world_to_screen_px(
            v.camera_center,
            v.camera_height_world,
            v.window_w,
            v.window_h,
            [2.0, 1.0],
        );
        assert_eq!(
            hits.hit(s[0], s[1]),
            Some(ids::GIZMO_JOINT_ANCHOR_B),
            "the B anchor must be grabbable at its own position"
        );
        // And A is still the one at A.
        let a = world_to_screen_px(
            v.camera_center,
            v.camera_height_world,
            v.window_w,
            v.window_h,
            [0.0, 0.0],
        );
        assert_eq!(hits.hit(a[0], a[1]), Some(ids::GIZMO_JOINT_ANCHOR));
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
        let mut v = view([1.0, -1.0]);
        v.anchor_b_world = Some([1.0, -1.0]);
        let mut scene = VectorScene::new();
        let mut hits = HitIndex::default();
        paint_point_gizmo(&mut scene, &v, Theme::default(), &mut hits);

        let s = world_to_screen_px(
            v.camera_center,
            v.camera_height_world,
            v.window_w,
            v.window_h,
            [1.0, -1.0],
        );
        assert_eq!(
            hits.hit(s[0], s[1]),
            Some(ids::GIZMO_JOINT_ANCHOR),
            "dead centre belongs to A"
        );
        assert_eq!(
            hits.hit(s[0] + JOINT_ANCHOR_RING_PX, s[1]),
            Some(ids::GIZMO_JOINT_ANCHOR_B),
            "the band outside A's square must still reach B, or a Pin's B end \
             could never be grabbed"
        );
    }

    /// **No body B, no B handle.** A half-authored joint has nothing to anchor
    /// its second end to, and offering a handle for it would let the artist
    /// author a value that is thrown away by the first seed.
    #[test]
    fn a_joint_with_no_b_anchor_paints_one_handle() {
        let v = view([0.0, 0.0]);
        let mut scene = VectorScene::new();
        let mut hits = HitIndex::default();
        paint_point_gizmo(&mut scene, &v, Theme::default(), &mut hits);
        assert_eq!(
            hits.len(),
            1,
            "only the A handle should be registered when there is no B anchor"
        );
    }

    /// **The snap crosshair draws and takes no hit.** It is a readout — a mark
    /// that says *this is why the dot stopped* — and a readout that swallows the
    /// pointer would steal the drag it is describing.
    #[test]
    fn the_snap_mark_is_drawn_without_taking_the_pointer() {
        let mut v = view([0.0, 0.0]);
        v.snap_world = Some([3.0, 3.0]);
        let mut scene = VectorScene::new();
        let mut hits = HitIndex::default();
        paint_point_gizmo(&mut scene, &v, Theme::default(), &mut hits);
        let s = world_to_screen_px(
            v.camera_center,
            v.camera_height_world,
            v.window_w,
            v.window_h,
            [3.0, 3.0],
        );
        assert_eq!(
            hits.hit(s[0], s[1]),
            None,
            "the snap crosshair must not register a hit"
        );
        assert_eq!(hits.len(), 1, "only the A handle registers");
    }

    /// **The dot moves with the anchor.** Two anchors project to two different
    /// screen positions, and the hit follows — so the handle sits on the joint,
    /// not at a fixed screen spot.
    #[test]
    fn the_hit_follows_the_anchor() {
        for anchor in [[0.0, 0.0], [3.0, -2.0]] {
            let v = view(anchor);
            let mut scene = VectorScene::new();
            let mut hits = HitIndex::default();
            paint_point_gizmo(&mut scene, &v, Theme::default(), &mut hits);
            let s = world_to_screen_px(
                v.camera_center,
                v.camera_height_world,
                v.window_w,
                v.window_h,
                anchor,
            );
            assert_eq!(hits.hit(s[0], s[1]), Some(ids::GIZMO_JOINT_ANCHOR));
        }
    }
}
