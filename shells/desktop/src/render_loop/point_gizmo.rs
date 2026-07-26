//! **Which anchors get a canvas handle** — the joint-anchor gizmo's publish rule.
//!
//! Extracted from `snapshots::publish` so the rule is gated HEADLESS: the
//! publish itself needs a camera, a present world and a live `HeroScreen`, none
//! of which a unit test has, but the decision is a pure function of the sim
//! world, the bridge and the clock.
//!
//! The point gizmo is `ph2d_editor::PointGizmoView` (editor-core): grabbable
//! dots at world anchors, for entities that are POINTS rather than boxes. A
//! physics joint is exactly that — it carries a `Transform` but no drawable
//! geometry, so `snapshots::build_view` returns `None` for it and it would
//! otherwise get no canvas handle at all.
//!
//! # Every joint, not the selected one (W-J2b)
//!
//! W-JointAnchor and W-J2 offered the handles only for the SELECTED joint, and
//! Enio's smoke named the cost (2026-07-25: *"precisam ser selecionados e
//! arrastável diretamente no canvas sem necessitar selecionar no hierarchy"*).
//! The two halves of that are one fact: a joint has **no sprite**, so a canvas
//! click can never reach it through `pick_sprites_at_world`, and a selection is
//! the only thing that made its dots appear — which means the dots were reachable
//! only by first finding the joint in the Hierarchy. A handle you must find
//! somewhere else before you can grab it is not on the canvas.
//!
//! So every joint publishes its handles, and grabbing one SELECTS its joint (the
//! shell's Down does that, so §12 opens on the joint you just grabbed).

use ph2d_ecs::{Entity, SimWorld};
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{JointSide, PhysicsBridge};
use ph2d_render::Camera2d;

use ph2d_editor::gizmo::{PointGizmoView, PointHandle, PointSide};

/// Every anchor that should be grabbable this frame, sorted by `(entity, side)`.
///
/// ⚠️ **Rest-only, and this closes a gap rather than opening one:**
/// `sync_joint_pivots` already declares in its own doc that "during play the dot
/// is not shown and the overlay draws the live solver anchors" — and that claim
/// was false, because nothing asked. During play the anchors the overlay draws
/// are the solver's, which is what the artist should be reading; the authored
/// handles are a rest-time thing, and a handle that took a drag against a
/// swinging body would author against a pose nobody chose.
///
/// Locked entities are skipped: `joint_anchor_drag::open_drag` refuses them, and
/// a handle that paints, registers a hit and then declines the gesture is worse
/// than one that is not offered.
///
/// The A end of a **dormant** joint is included — the bridge's door answers it
/// from the authored `Transform` — because that is precisely the joint the
/// artist is in the middle of fixing. Its B end has no body, so no anchor, so no
/// handle.
#[must_use]
pub(super) fn joint_anchor_handles(
    sim: &SimWorld,
    physics: &PhysicsBridge,
    at_rest: bool,
) -> Vec<PointHandle> {
    if !at_rest {
        return Vec::new();
    }
    let mut out: Vec<PointHandle> = Vec::new();
    for &e in physics.joint_entities() {
        if ph2d_ecs::is_locked_for_edit(sim.world(), e) {
            continue;
        }
        for (side, gizmo_side) in [(JointSide::A, PointSide::A), (JointSide::B, PointSide::B)] {
            if let Some(world) = physics.joint_anchor_world(sim, e, side) {
                out.push(PointHandle {
                    key: e.to_bits(),
                    side: gizmo_side,
                    world,
                });
            }
        }
    }
    // Deterministic display order. The hit ids are per-handle, so ordering
    // cannot decide WHO owns a pixel — it decides only which of two overlapping
    // joints paints on top, and that should not depend on archetype layout.
    out.sort_by_key(|h| (h.key, h.side));
    out
}

/// The point gizmo for this frame, or `None` when there is nothing to grab.
///
/// This function decides *whether* there are handles and where the camera is;
/// it never decides *where* an anchor is. That answer comes from the bridge's
/// anchor door (see [`joint_anchor_handles`]), and re-deriving it here would be
/// the second opinion that drifts — the failure W-AnchorFollow paid 1.771 m for.
#[must_use]
pub(super) fn build_point_view(
    handles: Vec<PointHandle>,
    camera: &Camera2d,
    window_size: WindowSize,
    snap: Option<[f32; 2]>,
) -> Option<PointGizmoView> {
    if handles.is_empty() {
        return None;
    }
    Some(PointGizmoView {
        handles,
        snap_world: snap,
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: window_size.width as f32,
        window_h: window_size.height as f32,
        canvas: ph2d_editor::zones::Rect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        ),
    })
}

/// The joint a hit id belongs to, and which end — read back from the map the
/// painter filled while registering.
///
/// The ids are keyed by entity bits (`ph2d_editor::gizmo::point_handle_id`), so
/// no static table can classify them; this is the same shape as the box gizmo's
/// `gizmo_hit_map` and for the same reason.
#[must_use]
pub(crate) fn resolve_anchor_hit(
    hit_map: &std::collections::BTreeMap<ph2d_editor::NodeId, PointHandle>,
    id: ph2d_editor::NodeId,
) -> Option<(Entity, JointSide)> {
    let h = hit_map.get(&id)?;
    Some((
        Entity::from_bits(h.key),
        match h.side {
            PointSide::A => JointSide::A,
            PointSide::B => JointSide::B,
        },
    ))
}

#[cfg(test)]
#[path = "point_gizmo_tests.rs"]
mod tests;
