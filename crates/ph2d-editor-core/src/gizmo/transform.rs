//! Gizmo transform math: drag-state-to-Transform delta.
//!
//! Extracted from monolithic `gizmo.rs` in Wave 6+7 Phase 1.B.

use super::camera::{GizmoCamera, GizmoModifiers, GizmoSnap};
use super::drag::{GizmoDragKind, GizmoDragState, TransformSnapshot};

/// Given the in-progress drag + the live camera state, compute the
/// new Transform the host should write into SimWorld this frame.
///
/// `modifiers` carries the live keyboard modifier state (Shift / Ctrl
/// / Alt). `snap` carries per-project quantization steps from
/// `ProjectSettings`. Both arguments default to "no behavior change"
/// when constructed via `Default::default()`.
///
/// `world_snap_fn` is an OPTIONAL closure the host wires up to the
/// active grid snap subsystem (`GridSnapState::snap_world`). When
/// present AND the drag is a Scale (Corner / Edge) — i.e. the user
/// is dragging a corner / edge handle to a specific world-space
/// target — the cursor world position is rewritten through the
/// closure BEFORE the scale ratio is computed. This lets the dragged
/// corner land on a grid vertex / center / corner per the user's
/// selected snap target. Pass `None` to keep the cursor unsnapped.
/// `FnMut` accepted because `GridSnapState::snap_world` takes `&mut
/// self` (it lazily caches origin offsets).
///
/// Pure function — no I/O, no allocation (closure dispatch only).
/// Tested directly against canonical input/output cases in `tests::*`
/// below.
pub fn compute_gizmo_transform(
    drag: &GizmoDragState,
    camera: &GizmoCamera,
    modifiers: GizmoModifiers,
    snap: GizmoSnap,
    world_snap_fn: Option<&mut dyn FnMut([f32; 2]) -> [f32; 2]>,
) -> TransformSnapshot {
    let raw_world = camera.screen_to_world(drag.cursor_screen);
    // Apply the host's grid snap to the cursor BEFORE the math when
    // the drag is a Scale — that way the snapped corner anchors at
    // a grid vertex / center / corner. Translate / Rotate ignore the
    // closure (Translate runs its own grid-snap pass in the host on
    // the final transform; Rotate has no spatial snap target).
    let now_world = match (drag.kind, world_snap_fn) {
        (GizmoDragKind::ScaleCorner { .. } | GizmoDragKind::ScaleEdge { .. }, Some(f)) => {
            f(raw_world)
        }
        _ => raw_world,
    };
    match drag.kind {
        GizmoDragKind::Translate => {
            let dx = now_world[0] - drag.start_cursor_world[0];
            let dy = now_world[1] - drag.start_cursor_world[1];
            let mut new_x = drag.start_transform.translation[0] + dx;
            let mut new_y = drag.start_transform.translation[1] + dy;
            // Ctrl/Cmd: quantize the resulting position (NOT the
            // delta) to the snap_move grid. Snapping the delta would
            // drift with cursor jitter near the threshold; snapping
            // the final position is what Photoshop/Figma users
            // expect.
            if modifiers.ctrl && snap.move_meters > 0.0 {
                new_x = quantize(new_x, snap.move_meters);
                new_y = quantize(new_y, snap.move_meters);
            }
            TransformSnapshot {
                translation: [new_x, new_y],
                rotation: drag.start_transform.rotation,
                scale: drag.start_transform.scale,
            }
        }
        GizmoDragKind::ScaleCorner { dx_sign, dy_sign } => {
            // Pivot stays fixed at the opposite-corner location.
            // We let the user drag the corner to where the cursor
            // is; the scale factor is the ratio of the new corner-
            // pivot vector vs the original one — measured along the
            // sprite's LOCAL axes (rotated by `start_transform.
            // rotation`), not world axes. Without the inverse-rot
            // step a rotated sprite scales uniformly even when the
            // user drags along one local axis only.
            let rot = drag.start_transform.rotation;
            let cos_r = rot.cos();
            let sin_r = rot.sin();
            let start_dx = drag.start_cursor_world[0] - drag.pivot_world[0];
            let start_dy = drag.start_cursor_world[1] - drag.pivot_world[1];
            let now_dx = now_world[0] - drag.pivot_world[0];
            let now_dy = now_world[1] - drag.pivot_world[1];
            let start_local_x = start_dx * cos_r + start_dy * sin_r;
            let start_local_y = -start_dx * sin_r + start_dy * cos_r;
            let now_local_x = now_dx * cos_r + now_dy * sin_r;
            let now_local_y = -now_dx * sin_r + now_dy * cos_r;
            // Guard against zero-length start vector (degenerate
            // case where the user clicks exactly on the pivot —
            // shouldn't be reachable through normal UI but defensive
            // either way).
            let mut ratio_x = if start_local_x.abs() > 1e-6 {
                now_local_x / start_local_x
            } else {
                1.0
            };
            let mut ratio_y = if start_local_y.abs() > 1e-6 {
                now_local_y / start_local_y
            } else {
                1.0
            };
            // Shift: lock aspect ratio. Pick the ratio with the
            // larger absolute deviation from 1.0 so a small drag on
            // one axis doesn't override the user's intent on the
            // other.
            if modifiers.shift {
                let dev_x = (ratio_x - 1.0).abs();
                let dev_y = (ratio_y - 1.0).abs();
                let chosen = if dev_x > dev_y { ratio_x } else { ratio_y };
                ratio_x = chosen;
                ratio_y = chosen;
            }
            let scale_x = (drag.start_transform.scale[0] * ratio_x).max(0.001);
            let scale_y = (drag.start_transform.scale[1] * ratio_y).max(0.001);
            // Translation re-anchor: when the opposite corner is the
            // pivot (default — NOT Ctrl/Cmd), the sprite center must
            // shift so that the new opposite-corner world position
            // STAYS at `pivot_world`. Without this step the sprite
            // scales around its center even though the math used
            // opposite-corner ratios, which is what the user saw as
            // "the opposite corner moves with the drag". Center
            // anchor (Ctrl/Cmd held) keeps translation unchanged —
            // scaling in place is the intended behavior there.
            let translation = if drag.anchor_is_center {
                drag.start_transform.translation
            } else {
                opposite_anchor_translation(
                    drag.pivot_world,
                    drag.sprite_half_intrinsic,
                    [-dx_sign, -dy_sign],
                    [scale_x, scale_y],
                    drag.start_transform.rotation,
                )
            };
            TransformSnapshot {
                translation,
                rotation: drag.start_transform.rotation,
                scale: [scale_x, scale_y],
            }
        }
        GizmoDragKind::ScaleEdge { axis, sign } => {
            // Axis-only scale: one component changes, the other
            // sticks to its start value. Like ScaleCorner, the ratio
            // is measured in the sprite's LOCAL frame so the active
            // axis matches the user's perceived "X edge" / "Y edge"
            // even when the sprite is rotated.
            let axis = axis.min(1) as usize;
            let rot = drag.start_transform.rotation;
            let cos_r = rot.cos();
            let sin_r = rot.sin();
            let start_dx = drag.start_cursor_world[0] - drag.pivot_world[0];
            let start_dy = drag.start_cursor_world[1] - drag.pivot_world[1];
            let now_dx = now_world[0] - drag.pivot_world[0];
            let now_dy = now_world[1] - drag.pivot_world[1];
            let start_local = if axis == 0 {
                start_dx * cos_r + start_dy * sin_r
            } else {
                -start_dx * sin_r + start_dy * cos_r
            };
            let now_local = if axis == 0 {
                now_dx * cos_r + now_dy * sin_r
            } else {
                -now_dx * sin_r + now_dy * cos_r
            };
            let ratio = if start_local.abs() > 1e-6 {
                now_local / start_local
            } else {
                1.0
            };
            let mut scale = drag.start_transform.scale;
            scale[axis] = (drag.start_transform.scale[axis] * ratio).max(0.001);
            // Shift on an edge handle also locks AR — copy the
            // ratio to the other axis (Figma semantics).
            if modifiers.shift {
                let other = (axis + 1) % 2;
                scale[other] = (drag.start_transform.scale[other] * ratio).max(0.001);
            }
            // Translation re-anchor: same logic as ScaleCorner. The
            // opposite-edge midpoint stays fixed at `pivot_world` —
            // its local offset is along the dragged axis only.
            let opposite_local_sign = if axis == 0 {
                [-sign, 0.0]
            } else {
                [0.0, -sign]
            };
            let translation = if drag.anchor_is_center {
                drag.start_transform.translation
            } else {
                opposite_anchor_translation(
                    drag.pivot_world,
                    drag.sprite_half_intrinsic,
                    opposite_local_sign,
                    scale,
                    drag.start_transform.rotation,
                )
            };
            TransformSnapshot {
                translation,
                rotation: drag.start_transform.rotation,
                scale,
            }
        }
        GizmoDragKind::Rotate => {
            // atan2 in world coords; subtract start angle from now
            // angle, add to start rotation.
            let start_angle = (drag.start_cursor_world[1] - drag.pivot_world[1])
                .atan2(drag.start_cursor_world[0] - drag.pivot_world[0]);
            let now_angle =
                (now_world[1] - drag.pivot_world[1]).atan2(now_world[0] - drag.pivot_world[0]);
            let mut rotation = drag.start_transform.rotation + (now_angle - start_angle);
            // Shift: snap to `snap.rotate_deg` increments. Converts
            // degrees to radians once before quantize.
            if modifiers.shift && snap.rotate_deg > 0.0 {
                let step_rad = snap.rotate_deg.to_radians();
                rotation = quantize(rotation, step_rad);
            }
            TransformSnapshot {
                translation: drag.start_transform.translation,
                rotation,
                scale: drag.start_transform.scale,
            }
        }
    }
}

/// Compute the world-space pivot for a Scale drag (Corner / Edge).
///
/// Default (`use_center_anchor = false`): pivot at the OPPOSITE corner
/// (for `ScaleCorner`) or the OPPOSITE edge midpoint (for `ScaleEdge`)
/// of the sprite's bounding box. Dragging a corner stretches the box
/// from the anchored opposite side — the same anchor model
/// Photoshop / Figma / Blender use.
///
/// `use_center_anchor = true`: pivot at the sprite center (its
/// `translation`). Dragging a corner scales the box uniformly around
/// the center. Bound to the Ctrl / Cmd modifier in the host.
///
/// `sprite_half_size` is the sprite's INTRINSIC half-extent in local
/// frame (i.e. `Sprite::size * 0.5`, before `Transform::scale`). The
/// helper multiplies by `snap.scale` and rotates by `snap.rotation`
/// to project to world.
///
/// `Translate` and `Rotate` ignore the toggle and always pivot on the
/// sprite center (those drags don't have a natural opposite anchor).
pub fn anchor_pivot_world(
    kind: GizmoDragKind,
    sprite_half_size: [f32; 2],
    snap: TransformSnapshot,
    use_center_anchor: bool,
) -> [f32; 2] {
    let local = if use_center_anchor {
        return snap.translation;
    } else {
        match kind {
            GizmoDragKind::ScaleCorner { dx_sign, dy_sign } => [
                -dx_sign * sprite_half_size[0],
                -dy_sign * sprite_half_size[1],
            ],
            GizmoDragKind::ScaleEdge { axis, sign } => {
                if axis == 0 {
                    [-sign * sprite_half_size[0], 0.0]
                } else {
                    [0.0, -sign * sprite_half_size[1]]
                }
            }
            // Translate / Rotate don't have an "opposite" anchor —
            // fall back to the sprite center.
            _ => return snap.translation,
        }
    };
    // Local → scaled local → rotated → world.
    let scaled_x = local[0] * snap.scale[0];
    let scaled_y = local[1] * snap.scale[1];
    let cos_r = snap.rotation.cos();
    let sin_r = snap.rotation.sin();
    let rotated_x = scaled_x * cos_r - scaled_y * sin_r;
    let rotated_y = scaled_x * sin_r + scaled_y * cos_r;
    [
        snap.translation[0] + rotated_x,
        snap.translation[1] + rotated_y,
    ]
}

/// Compute the sprite-center translation that keeps `pivot_world`
/// fixed at the OPPOSITE anchor's world position under a new scale.
///
/// `opposite_local_sign` are the (sign_x, sign_y) of the
/// opposite-corner local offset (e.g. `[-1, -1]` for the SW corner
/// when the user drags the NE corner). Pass `0.0` on an axis that
/// doesn't move (Edge handles use one zero component).
///
/// Math (no rotation case): `pivot_world = new_translation +
/// opposite_local_sign * sprite_half_intrinsic * new_scale`. Solve
/// for `new_translation`. With rotation, the local offset rotates
/// by `rotation` (world +X = (cos, sin), world +Y = (-sin, cos))
/// before subtracting from `pivot_world`.
pub(super) fn opposite_anchor_translation(
    pivot_world: [f32; 2],
    sprite_half_intrinsic: [f32; 2],
    opposite_local_sign: [f32; 2],
    new_scale: [f32; 2],
    rotation: f32,
) -> [f32; 2] {
    let local_x = opposite_local_sign[0] * sprite_half_intrinsic[0] * new_scale[0];
    let local_y = opposite_local_sign[1] * sprite_half_intrinsic[1] * new_scale[1];
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    let rotated_x = local_x * cos_r - local_y * sin_r;
    let rotated_y = local_x * sin_r + local_y * cos_r;
    [pivot_world[0] - rotated_x, pivot_world[1] - rotated_y]
}

/// Quantize `value` to the nearest multiple of `step`. `step <= 0`
/// returns the value unchanged (defensive — the UI gates the
/// modifier on a positive step before reaching here).
pub(super) fn quantize(value: f32, step: f32) -> f32 {
    if step > 0.0 {
        (value / step).round() * step
    } else {
        value
    }
}
