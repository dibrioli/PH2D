//! **The point gizmo** — a single grabbable dot at a world anchor.
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

use super::camera::world_to_screen_px;
use super::hit::ids;
use super::paint::HANDLE_SIZE_PX;
use crate::interaction::HitIndex;
use crate::zones::Rect;
use ph2d_tokens::Theme;
use ph2d_vector::{Circle, Color as VelloColor, Point, VectorScene};

/// A single point handle to draw at a world anchor — the joint-anchor gizmo.
///
/// Carries only the camera fields the projection needs. A point has no bbox and
/// no rotation, which is exactly why it could not be a [`super::GizmoView`]: that
/// type is a rotated box, and there is nothing here to rotate.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointGizmoView {
    /// World position of the handle = the selected entity's
    /// `Transform.translation` (the joint's authored anchor).
    pub anchor_world: [f32; 2],
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

/// Amber — the joint overlay's colour, so the grabbable dot reads as "the thing
/// you already see in the overlay, now grab it" rather than as a new element.
/// Theme-independent for the same reason the pivot ring is (the meaning does not
/// change between Forge / Workshop / Sunstone / Blueprint).
fn anchor_color() -> VelloColor {
    VelloColor::from_rgba8(0xFA, 0xBF, 0x40, 0xFF) // matches `JOINT_RGBA` in the physics overlay
}

/// Draw the point handle at its world anchor and register its hit rect, so a
/// canvas Down on it is recognised (`GIZMO_JOINT_ANCHOR`) and opens a translate
/// drag of the selected entity.
pub fn paint_point_gizmo(
    scene: &mut VectorScene,
    view: &PointGizmoView,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let _ = theme; // colour is theme-independent (see `anchor_color`)
    let s = world_to_screen_px(
        view.camera_center,
        view.camera_height_world,
        view.window_w,
        view.window_h,
        view.anchor_world,
    );
    // Generous grab: the hit rect is a full handle, larger than the visual dot,
    // so the point is as catchable as any gizmo handle.
    let hit = Rect::new(
        s[0] - HANDLE_SIZE_PX * 0.5,
        s[1] - HANDLE_SIZE_PX * 0.5,
        HANDLE_SIZE_PX,
        HANDLE_SIZE_PX,
    );
    hit_index.register(ids::GIZMO_JOINT_ANCHOR, hit);
    // Filled dot — a joint's anchor handle is always the active grab when the
    // joint is selected (there is no other handle to disambiguate it from).
    let dot = Circle::new(
        Point::new(f64::from(s[0]), f64::from(s[1])),
        f64::from(JOINT_ANCHOR_DOT_PX),
    );
    scene.inner_mut().fill(
        ph2d_vector::Fill::NonZero,
        ph2d_vector::Affine::IDENTITY,
        anchor_color(),
        None,
        &dot,
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
