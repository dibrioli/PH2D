//! **Dragging a joint anchor** — one gesture, both ends (W-J2).
//!
//! The A dot used to open a generic `GizmoDragKind::Translate` of the joint
//! entity, because the joint's `Transform` *was* its anchor. W-AnchorFollow made
//! the anchor body-local and demoted the `Transform` to a derived display value,
//! and W-J2 added a second handle for body B — which has no `Transform` at all.
//! A Translate could never author it.
//!
//! So both handles now open **this** drag, and it speaks through the bridge's
//! anchor door (`ph2d_physics_ecs::PhysicsBridge::set_joint_anchor_world`). The
//! two ends run the same code with a different [`JointSide`], which is the point:
//! *two handles, one behaviour*. Reaching the same result by two different paths
//! is how they would come to disagree about snapping, about undo, about which
//! frame the write lands in.
//!
//! # What this deliberately does NOT do any more
//!
//! It does not clear `PhysicsJoint::anchored`. That sentinel re-derives **both**
//! locals from the seed policy, so dragging A would have thrown away a B anchor
//! the artist had just placed. A reposition knows its side and its world point,
//! so it writes that local directly (see `bridge::anchors`).
//!
//! # Undo
//!
//! Nothing here touches the undo queue: `post_frame_undo` suppresses while a
//! pointer button is held and records the diff once on release, so a whole drag
//! is one global step — the same way moving a sprite is.

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{JointSide, PhysicsBridge, ShapeDesc};
use ph2d_render::Camera2d;

use crate::App;

/// An anchor drag in flight.
#[derive(Copy, Clone, Debug)]
pub(crate) struct JointAnchorDrag {
    /// The joint entity being authored (`Entity::to_bits`).
    pub(crate) joint_bits: u64,
    /// Which end the grabbed handle belongs to.
    pub(crate) side: JointSide,
    /// Anchor position minus cursor position at the press, in world. The drag is
    /// **grab-relative** like every other handle in the editor: pressing 5 px off
    /// the dot's centre must not teleport the anchor 5 px.
    grab_offset: [f32; 2],
    /// The candidate the anchor snapped to this frame, for the gizmo to mark.
    /// `None` while free.
    pub(crate) snap: Option<[f32; 2]>,
}

/// Snap radius in **screen** pixels, converted to world at the current zoom so
/// the magnet feels the same however far in the artist is.
///
/// 14 px, which is not a fresh guess: it is the number the pivot handle's
/// Ctrl-snap already uses (`input_dispatch::gizmo_drag`, the MovePivot branch).
/// Two point handles in the same editor snapping at two different distances
/// would be two answers to one question. It is also comfortably smaller than the
/// gap between neighbouring candidates at any zoom a body is authored at — a
/// 0.5 m box at the default camera puts its corners 50 px apart — so the free
/// space between targets stays reachable.
const SNAP_PX: f32 = 14.0;

/// Open an anchor drag on `side` of the selected joint, or `None` if there is
/// nothing to author (no selection, a locked entity, an end with no body).
///
/// ⚠️ Takes the four pieces of `AppGfx` it needs rather than `&AppGfx`, because
/// the Down handler that calls it already holds a `&mut` into `gfx.hero_screen`
/// — field-level borrows are disjoint, a whole-struct reborrow is not.
#[must_use]
pub(crate) fn open_drag(
    physics: &PhysicsBridge,
    sim: &SimWorld,
    camera: &Camera2d,
    window: WindowSize,
    selection: Option<u64>,
    pointer: (f32, f32),
    side: JointSide,
) -> Option<JointAnchorDrag> {
    let bits = selection?;
    let entity = ph2d_ecs::Entity::from_bits(bits);
    if ph2d_ecs::is_locked_for_edit(sim.world(), entity) {
        return None;
    }
    let anchor = physics.joint_anchor_world(sim, entity, side)?;
    let cursor = camera.screen_to_world(pointer, window);
    Some(JointAnchorDrag {
        joint_bits: bits,
        side,
        grab_offset: [anchor[0] - cursor[0], anchor[1] - cursor[1]],
        snap: None,
    })
}

impl App {
    /// Follow the cursor with the open anchor drag, snapping while Ctrl is held.
    /// A no-op when no anchor drag is in flight.
    pub(crate) fn advance_joint_anchor_drag(&mut self) {
        let Some(mut drag) = self.joint_anchor_drag else {
            return;
        };
        // Ctrl (Cmd on macOS) is the editor's snap modifier — the same key the
        // pivot handle reads, so one gesture means one thing.
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        let pointer = self.last_pointer;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let window_size = gfx.surface.size();
        let cursor = gfx.camera.screen_to_world(pointer, window_size);
        let free = [
            cursor[0] + drag.grab_offset[0],
            cursor[1] + drag.grab_offset[1],
        ];
        let entity = ph2d_ecs::Entity::from_bits(drag.joint_bits);
        let (target, snap) = if ctrl {
            let mut cands = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
            let n = gfx
                .physics
                .joint_snap_targets(&gfx.sim, entity, drag.side, &mut cands);
            let threshold = SNAP_PX * gfx.camera.height_world / window_size.height as f32;
            match nearest_within(&cands[..n], free, threshold) {
                Some(p) => (p, Some(p)),
                None => (free, None),
            }
        } else {
            (free, None)
        };
        gfx.physics
            .set_joint_anchor_world(&mut gfx.sim, entity, drag.side, target);
        drag.snap = snap;
        self.joint_anchor_drag = Some(drag);
    }
}

/// The candidate closest to `p` within `threshold` world units, or `None`.
///
/// Nearest-wins rather than first-within-range: candidates crowd together on a
/// small body at a low zoom, and "whichever was listed first" would make the
/// magnet's choice depend on the order a shape happens to enumerate its corners.
#[must_use]
pub(crate) fn nearest_within(
    candidates: &[[f32; 2]],
    p: [f32; 2],
    threshold: f32,
) -> Option<[f32; 2]> {
    let mut best: Option<([f32; 2], f32)> = None;
    let limit = threshold * threshold;
    for &c in candidates {
        let (dx, dy) = (c[0] - p[0], c[1] - p[1]);
        let d2 = dx * dx + dy * dy;
        if d2 <= limit && best.is_none_or(|(_, b)| d2 < b) {
            best = Some((c, d2));
        }
    }
    best.map(|(c, _)| c)
}

#[cfg(test)]
mod tests {
    use super::nearest_within;

    /// **The nearest candidate wins, not the first one listed.** A shape
    /// enumerates its corners in a fixed order, and taking the first match would
    /// make the magnet's answer depend on that order rather than on the cursor.
    ///
    /// Mutation-tested: returning on the first candidate within range makes this
    /// pick `[1.0, 0.0]` instead of `[0.2, 0.0]`.
    #[test]
    fn the_nearest_candidate_wins() {
        let cands = [[1.0, 0.0], [0.2, 0.0], [-3.0, 0.0]];
        assert_eq!(nearest_within(&cands, [0.0, 0.0], 2.0), Some([0.2, 0.0]));
    }

    /// **Out of range is no snap.** The magnet has to let go, or the anchor
    /// could never be placed between two candidates.
    #[test]
    fn nothing_within_range_does_not_snap() {
        let cands = [[5.0, 0.0], [0.0, 5.0]];
        assert_eq!(nearest_within(&cands, [0.0, 0.0], 1.0), None);
        assert!(nearest_within(&[], [0.0, 0.0], 1000.0).is_none());
    }

    /// **The threshold is inclusive at the boundary**, so a candidate exactly at
    /// the radius still catches — a strict comparison would make the magnet's
    /// edge depend on float noise.
    #[test]
    fn a_candidate_exactly_at_the_radius_catches() {
        assert_eq!(
            nearest_within(&[[2.0, 0.0]], [0.0, 0.0], 2.0),
            Some([2.0, 0.0])
        );
    }
}
