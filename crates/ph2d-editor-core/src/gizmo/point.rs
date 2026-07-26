//! **The point gizmo** — the grabbable dots at every joint's two world anchors.
//!
//! The three `GizmoView` publishers ([`super::paint_sprite_gizmo`] and friends)
//! are all BOXES with scale/rotate handles, built from drawable geometry (a
//! sprite quad, a vector bbox). A physics joint is a *point*: it carries a
//! `Transform` (its authored anchor) but no geometry, so it publishes no box and
//! — until this — had no canvas handle at all. Its anchor was authorable only by
//! typing into the Inspector's Position fields.
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
//!
//! # Every joint, not the selected one (W-J2b)
//!
//! The view carries a **list**. A joint has no sprite, so a canvas click could
//! never reach it through `pick_sprites_at_world` — which meant the only way to
//! get its handles on screen was to hunt for it in the Hierarchy first, and a
//! handle you must find somewhere else before you can grab it is a handle that
//! is not on the canvas at all (Enio, 2026-07-25).
//!
//! Several joints therefore register the same two kinds of handle in one frame,
//! and a hit id must say *which*. That question already has an answer here:
//! `gizmo::paint::keyed_handle_id` gives every EXTRA selection its own id space
//! by hashing the entity bits, and the shell resolves the hit through the map
//! the painter filled while painting. These dots do the same —
//! [`point_handle_id`] — for the same reason and with the same failure mode
//! avoided (a linear scrambler makes consecutive ids collide; see that
//! function's note).

use super::camera::world_to_screen_px;
use super::hit::ids;
use crate::interaction::HitIndex;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, BezPath, Circle, Color as VelloColor, Point, Stroke, VectorScene};
use std::collections::BTreeMap;

/// Which end of a joint a handle authors. Editor-core's own word for it — the
/// gizmo layer knows there are two ends and nothing else about physics; the
/// shell maps this onto `ph2d_physics_ecs::JointSide`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PointSide {
    /// The end attached to body A — the filled dot.
    A,
    /// The end attached to body B — the hollow ring.
    B,
}

/// One grabbable anchor: where it is, whose it is, and which end.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointHandle {
    /// The owning entity (`Entity::to_bits`), opaque here. It is what makes two
    /// joints' handles distinguishable, both in the hit id and in the map the
    /// shell reads back.
    pub key: u64,
    pub side: PointSide,
    /// World position of the anchor this handle authors.
    pub world: [f32; 2],
}

/// The point handles to draw this frame — the anchor gizmo.
///
/// Carries only the camera fields the projection needs. A point has no bbox and
/// no rotation, which is exactly why it could not be a [`super::GizmoView`]: that
/// type is a rotated box, and there is nothing here to rotate.
#[derive(Clone, Debug, PartialEq)]
pub struct PointGizmoView {
    /// Every anchor on screen, in a stable order. Empty is not published (the
    /// shell hands out `None` instead), so a non-empty list is the invariant.
    pub handles: Vec<PointHandle>,
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

/// Visual radius of the anchor dot, screen px.
///
/// Sized up from 6 on Enio's smoke of W-J2 (*"os círculos das pontas precisam
/// ser maiores"*): a mark you have to aim at is a mark you have to find first,
/// and these are now offered for every joint in the scene rather than for the
/// one already selected. 9 px puts the dot's extent at 1.5× the box gizmo's
/// `HANDLE_SIZE_PX` corner square (12), so it reads as a deliberate grab target
/// next to one rather than as a marker.
const JOINT_ANCHOR_DOT_PX: f32 = 9.0;

/// Visual radius of the B ring, screen px — outside A's dot, so a coincident
/// pair reads as one mark inside another rather than as one mark. Holds the
/// same 5:3 ratio to the dot that the pair had at 6/10, which is what keeps the
/// concentric reading legible at the new size.
const JOINT_ANCHOR_RING_PX: f32 = 15.0;

/// Stroke width of the B ring, screen px. Wider than the 1.5 the pair shipped
/// with, so the bigger circle keeps the same visual weight instead of thinning
/// into a hairline.
const JOINT_ANCHOR_RING_STROKE_PX: f64 = 2.0;

/// Arm length of the snap crosshair, screen px — past the B ring, so the mark is
/// legible even when both handles sit on the snapped point.
const SNAP_CROSS_PX: f32 = 20.0;

/// Half-extent of a handle's hit square, screen px, **by side**.
///
/// ⚠️ These are the VISUAL radii, deliberately: a mark drawn larger than the
/// rect that catches it is a dot the artist clicks and nothing happens. A takes
/// the inner square and B the band outside it, which is the whole of how a
/// coincident pair stays two handles.
const fn hit_half_px(side: PointSide) -> f32 {
    match side {
        PointSide::A => JOINT_ANCHOR_DOT_PX,
        PointSide::B => JOINT_ANCHOR_RING_PX,
    }
}

/// The hit id of one joint's handle — `canonical ^ hash(key)`.
///
/// ⚠️ **The multipliers are odd and DIFFERENT per side, and neither is the one
/// the box gizmo's extras use.** The failure this avoids is documented at
/// `gizmo::paint::keyed_handle_id`: a *linear* scrambler (`canonical ^ bits ^
/// CONST`) cancels when two ids are compared, so consecutive entity bits and
/// consecutive canonical ids collide constantly — which is how a click on one
/// sprite's handle came to resolve to a different sprite in 2026-06. Multiplying
/// is non-linear, so consecutive keys land far apart; using a distinct constant
/// per side means A and B hash independently rather than differing by the one
/// bit that separates their canonical ids.
#[must_use]
pub fn point_handle_id(key: u64, side: PointSide) -> NodeId {
    let (canonical, mul) = match side {
        PointSide::A => (ids::GIZMO_JOINT_ANCHOR, 0x_C2B2_AE3D_27D4_EB4F_u64),
        PointSide::B => (ids::GIZMO_JOINT_ANCHOR_B, 0x_D6E8_FEB8_6659_FD93_u64),
    };
    NodeId(canonical.0 ^ key.wrapping_mul(mul))
}

/// Amber — the joint overlay's colour, so the grabbable dot reads as "the thing
/// you already see in the overlay, now grab it" rather than as a new element.
/// Theme-independent for the same reason the pivot ring is (the meaning does not
/// change between Forge / Workshop / Sunstone / Blueprint).
fn anchor_color() -> VelloColor {
    VelloColor::from_rgba8(0xFA, 0xBF, 0x40, 0xFF) // matches `JOINT_RGBA` in the physics overlay
}

/// Draw every joint's anchor handles and register their hit rects, recording
/// `id -> handle` in `hit_map` so a Down can be resolved back to the joint and
/// the end it belongs to.
///
/// Order is load-bearing twice. The **snap crosshair first** (it is a backdrop
/// for the marks that sit on it), then **every B, then every A** —
/// `HitIndex::hit` walks backwards, so the last registration wins, and A must
/// win the square it shares with B on a coincident pair. Two passes rather than
/// per-joint interleaving: with one pass the next joint's B would be registered
/// after this joint's A and would swallow it wherever two joints overlap.
pub fn paint_point_gizmo(
    scene: &mut VectorScene,
    view: &PointGizmoView,
    theme: Theme,
    hit_index: &mut HitIndex,
    hit_map: &mut BTreeMap<NodeId, PointHandle>,
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
    for side in [PointSide::B, PointSide::A] {
        for h in view.handles.iter().filter(|h| h.side == side) {
            let s = project(h.world);
            let half = hit_half_px(side);
            let id = point_handle_id(h.key, side);
            hit_index.register(
                id,
                Rect::new(s[0] - half, s[1] - half, half * 2.0, half * 2.0),
            );
            hit_map.insert(id, *h);
            match side {
                // Hollow — the B end, in the same amber as A (module docs).
                PointSide::B => {
                    let ring = Circle::new(
                        Point::new(f64::from(s[0]), f64::from(s[1])),
                        f64::from(JOINT_ANCHOR_RING_PX),
                    );
                    scene.inner_mut().stroke(
                        &Stroke::new(JOINT_ANCHOR_RING_STROKE_PX),
                        Affine::IDENTITY,
                        anchor_color(),
                        None,
                        &ring,
                    );
                }
                // Filled dot — the A end.
                PointSide::A => {
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
            }
        }
    }
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
#[path = "point_tests.rs"]
mod tests;
