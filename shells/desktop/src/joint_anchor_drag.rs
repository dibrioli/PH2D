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
use ph2d_editor::gizmo::PointHandleKind;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{PhysicsBridge, ShapeDesc};
use ph2d_render::Camera2d;

use crate::App;
use ph2d_editor::screens::hero::JointFieldEdit;
use ph2d_physics_ecs::PhysicsJoint;

use crate::render_loop::inspector_joint::joint_with_edit;
use crate::render_loop::point_gizmo::anchor_side;

/// A point-handle drag in flight — an anchor, a limit wall, or a length ring.
#[derive(Copy, Clone, Debug)]
pub(crate) struct JointAnchorDrag {
    /// The joint entity being authored (`Entity::to_bits`).
    pub(crate) joint_bits: u64,
    /// What the grabbed handle authors.
    pub(crate) kind: PointHandleKind,
    /// What the press was holding, in the coordinate the value lives in.
    grab: Grab,
    /// The candidate the anchor snapped to this frame, for the gizmo to mark.
    /// `None` while free, and always `None` for a parameter grip (see
    /// `advance_joint_anchor_drag`).
    pub(crate) snap: Option<[f32; 2]>,
}

/// The press-time offset between the cursor and the value, **in the value's own
/// coordinate**.
///
/// Every handle in this editor is grab-relative — pressing 5 px off a dot must
/// not teleport it 5 px — and for these three that means three different
/// quantities, which is why this is an enum rather than a vector with two
/// unused components. A limit is an angle, so its grab offset is an angle; a
/// length is a radius, so its grab offset is a radius.
#[derive(Copy, Clone, Debug)]
enum Grab {
    /// Anchor: the world vector from the cursor to the anchor.
    World([f32; 2]),
    /// Limit wall: the angle (rad) from the cursor's bearing to the wall's.
    Angle(f32),
    /// Length ring: the signed difference between the ring's radius and the
    /// cursor's distance from the anchor.
    Radius(f32),
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

/// Open a drag on `kind` of `joint`, or `None` if there is nothing to author
/// (a locked entity, an end with no body, a parameter this joint does not have).
///
/// ⚠️ **The joint comes from the HIT, not from the selection** (W-J2b). Every
/// joint publishes handles now, so "which joint" is answered by the map the
/// painter filled while registering the dots — asking the selection instead
/// would author the selected joint's value from a click on a different one's
/// handle, silently.
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
    joint: ph2d_ecs::Entity,
    pointer: (f32, f32),
    kind: PointHandleKind,
) -> Option<JointAnchorDrag> {
    if ph2d_ecs::is_locked_for_edit(sim.world(), joint) {
        return None;
    }
    let cursor = camera.screen_to_world(pointer, window);
    let grab = if let Some(side) = anchor_side(kind) {
        let anchor = physics.joint_anchor_world(sim, joint, side)?;
        Grab::World([anchor[0] - cursor[0], anchor[1] - cursor[1]])
    } else {
        // A parameter grip measures against the joint's LIVE geometry, which is
        // the same `JointView` the overlay drew the arc and the ring from.
        let v = physics.joint_views().find(|v| v.entity == joint)?;
        let held = param_value(&v, kind)?;
        match kind {
            PointHandleKind::Length => Grab::Radius(held - distance(v.anchor_a, cursor)),
            // The wall's bearing minus the cursor's, so the wall does not jump
            // to the cursor on press.
            _ => Grab::Angle(wrap_pi((v.angle_a + held) - bearing(v.anchor_a, cursor))),
        }
    };
    Some(JointAnchorDrag {
        joint_bits: joint.to_bits(),
        kind,
        grab,
        snap: None,
    })
}

/// The value a parameter grip currently sits at: a limit (rad, relative) or the
/// ring's radius (m). `None` when this joint does not have that parameter — the
/// same `JointView` fields the overlay asks before drawing them, so a grip can
/// never be opened on something that is not on screen.
fn param_value(v: &ph2d_physics_ecs::JointView, kind: PointHandleKind) -> Option<f32> {
    match kind {
        PointHandleKind::LimitMin => v.limits.map(|l| l[0]),
        PointHandleKind::LimitMax => v.limits.map(|l| l[1]),
        PointHandleKind::Length => v.length,
        PointHandleKind::AnchorA | PointHandleKind::AnchorB => None,
    }
}

/// World bearing of `p` seen from `from`, radians.
///
/// ⚠️ `libm`, the crate-wide convention — though the reason the rest of the
/// engine has it does **not** apply here: this number is produced by a human
/// gesture and stored as authored state, not derived per-tick inside the sim, so
/// it never reaches the determinism hash by a path a replay repeats. Using the
/// pinned one anyway costs nothing and keeps one answer to "how do we take an
/// angle".
fn bearing(from: [f32; 2], p: [f32; 2]) -> f32 {
    libm::atan2f(p[1] - from[1], p[0] - from[0])
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    (b[0] - a[0]).hypot(b[1] - a[1])
}

/// `x` folded into `(-pi, pi]`.
fn wrap_pi(x: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut v = x % tau;
    if v > std::f32::consts::PI {
        v -= tau;
    } else if v <= -std::f32::consts::PI {
        v += tau;
    }
    v
}

/// `x` moved by whole turns to the nearest branch of `near`.
///
/// Without it a wall dragged across the ±pi cut jumps to the other side of the
/// arc: the raw bearing wraps, the stored limit does not, and the drag would
/// suddenly be authoring a value a full turn away from the one under the cursor.
fn unwrap_near(x: f32, near: f32) -> f32 {
    near + wrap_pi(x - near)
}
impl App {
    /// Follow the cursor with the open drag. A no-op when none is in flight.
    ///
    /// Three arms, one gesture. The **anchor** arm writes through the bridge's
    /// anchor door (the value is a place, and the bridge owns where a place is);
    /// the two **parameter** arms write the joint COMPONENT through the SAME
    /// funnel the §12 number rows use (`inspector_joint::joint_with_edit`), so
    /// posing a limit and typing one cannot disagree about clamping, about
    /// units, or about which field they mean.
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
        let entity = ph2d_ecs::Entity::from_bits(drag.joint_bits);
        drag.snap = match drag.grab {
            Grab::World(off) => {
                let free = [cursor[0] + off[0], cursor[1] + off[1]];
                let side = anchor_side(drag.kind).expect("a world grab is an anchor's");
                let (target, snap) = if ctrl {
                    let mut cands = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
                    let n = gfx
                        .physics
                        .joint_snap_targets(&gfx.sim, entity, side, &mut cands);
                    let threshold = SNAP_PX * gfx.camera.height_world / window_size.height as f32;
                    match nearest_within(&cands[..n], free, threshold) {
                        Some(p) => (p, Some(p)),
                        None => (free, None),
                    }
                } else {
                    (free, None)
                };
                gfx.physics
                    .set_joint_anchor_world(&mut gfx.sim, entity, side, target);
                snap
            }
            // ⚠️ No magnet on a parameter grip, and that is a gap with a name
            // rather than an omission: an angle would want a STEP (15°, say) and
            // a length would want the scene grid, and neither candidate set
            // exists here. The anchor's nine collider points do not answer
            // either question.
            Grab::Angle(off) => {
                let (anchor_a, angle_a) = anchor_of(&gfx.physics, entity);
                let raw = bearing(anchor_a, cursor) + off - angle_a;
                write_limit(&mut gfx.sim, entity, drag.kind, raw);
                None
            }
            Grab::Radius(off) => {
                let (anchor_a, _) = anchor_of(&gfx.physics, entity);
                let len = distance(anchor_a, cursor) + off;
                write_length(&mut gfx.sim, entity, len);
                None
            }
        };
        self.joint_anchor_drag = Some(drag);
    }
}

/// **Pose one wall of the limit arc**, in radians relative to body A.
///
/// Two decisions live here, and both are about what an ARC is.
///
/// ⚠️ **The dragged wall is UNWRAPPED against the value it is replacing**, so
/// crossing the ±pi cut moves the wall by the small amount the cursor moved
/// rather than jumping a whole turn to the other side of the arc.
///
/// ⚠️ **A wall STOPS at its sibling; it never passes it.** `PhysicsJoint::clamped`
/// *swaps* inverted limits — correct for a typed pair (a hinge limited to
/// `min > max` is a weld nobody asked for), and wrong for a gesture: the swap
/// would hand the artist the OTHER wall mid-drag, and the hand that was widening
/// the arc would start narrowing it with nothing on screen saying why. A wall
/// that stops is what every editor with a range does, and it is what the arc
/// itself shows: the two ends cannot cross.
fn write_limit(sim: &mut SimWorld, joint: ph2d_ecs::Entity, kind: PointHandleKind, raw: f32) {
    let Some(&current) = sim.world().get::<PhysicsJoint>(joint) else {
        return;
    };
    let (held, other) = match kind {
        PointHandleKind::LimitMin => (current.limit_min, current.limit_max),
        _ => (current.limit_max, current.limit_min),
    };
    let want = unwrap_near(raw, held);
    let walled = if matches!(kind, PointHandleKind::LimitMin) {
        want.min(other)
    } else {
        want.max(other)
    };
    let edit = if matches!(kind, PointHandleKind::LimitMin) {
        JointFieldEdit::LimitMinDeg(walled.to_degrees())
    } else {
        JointFieldEdit::LimitMaxDeg(walled.to_degrees())
    };
    write_edit(sim, joint, current, edit);
}

/// **Pose the length ring** — a spring's rest length or a rope's maximum, by
/// KIND, because the ring is one geometry naming two fields (`JointView.length`
/// is one field for the same reason: it is the same question to the drawing).
fn write_length(sim: &mut SimWorld, joint: ph2d_ecs::Entity, len: f32) {
    let Some(&current) = sim.world().get::<PhysicsJoint>(joint) else {
        return;
    };
    let edit = match current.kind {
        ph2d_physics_ecs::JointKind::Spring => JointFieldEdit::RestLength(len),
        ph2d_physics_ecs::JointKind::Rope => JointFieldEdit::MaxLength(len),
        // Pin/Weld have no ring, so no grip is ever published for them; a drag
        // that got here would be authoring a field the joint does not use.
        _ => return,
    };
    write_edit(sim, joint, current, edit);
}

/// The write itself: through the §12 funnel, then IN PLACE.
///
/// In place — not the editor command queue — for the same reason `set_joint_body`
/// is: this runs inside a pointer handler, mid-frame, and the global diff-based
/// undo captures the result. The whole drag is one undo step because
/// `post_frame_undo` suppresses while a button is held.
fn write_edit(
    sim: &mut SimWorld,
    joint: ph2d_ecs::Entity,
    current: PhysicsJoint,
    edit: JointFieldEdit,
) {
    let Some(next) = joint_with_edit(current, edit) else {
        return;
    };
    if next != current
        && let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(joint)
    {
        *j = next;
    }
}

/// The joint's live `(anchor_a, angle_a)` — the frame every angular parameter is
/// measured in, read from the same `JointView` the arc was drawn from.
fn anchor_of(physics: &PhysicsBridge, joint: ph2d_ecs::Entity) -> ([f32; 2], f32) {
    physics
        .joint_views()
        .find(|v| v.entity == joint)
        .map_or(([0.0, 0.0], 0.0), |v| (v.anchor_a, v.angle_a))
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

impl JointAnchorDrag {
    /// The limit this drag is posing right now — `(joint, radians relative to
    /// body A)` — or `None` when it is not a limit drag.
    ///
    /// Read from the COMPONENT rather than remembered on the drag: the write
    /// already happened this frame and went through `clamped()` plus the wall
    /// against the sibling, so the ghost shows the value the joint HAS, not the
    /// one the cursor asked for. A ghost standing where the solver will not let
    /// the body stop would be a promise the simulation breaks.
    #[must_use]
    pub(crate) fn posed_limit(&self, sim: &SimWorld) -> Option<(ph2d_ecs::Entity, f32)> {
        let joint = ph2d_ecs::Entity::from_bits(self.joint_bits);
        let j = sim.world().get::<PhysicsJoint>(joint)?;
        match self.kind {
            PointHandleKind::LimitMin => Some((joint, j.limit_min)),
            PointHandleKind::LimitMax => Some((joint, j.limit_max)),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "joint_anchor_drag_tests.rs"]
mod tests;
