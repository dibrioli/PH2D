//! **Dragging a joint anchor** — one gesture, both ends (W-J2).
//!
//! Both handles open this drag and write through the bridge's anchor door
//! (`PhysicsBridge::set_joint_anchor_world`): two handles, one behaviour.
//! It never clears `PhysicsJoint::anchored` (that re-derives BOTH locals).
//! Undo: none here — `post_frame_undo` records the whole drag as one step.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::gizmo::PointHandleKind;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{PhysicsBridge, ShapeDesc};
use ph2d_render::Camera2d;

use crate::{CanvasCtx, physics_state::PhysicsState};
use ph2d_editor_core::screens::hero::JointFieldEdit;
use ph2d_physics_ecs::PhysicsJoint;

use crate::anchor_side::anchor_side;
use crate::joint::joint_with_edit;

/// A point-handle drag in flight — an anchor, a limit wall, or a length ring.
#[derive(Copy, Clone, Debug)]
pub struct JointAnchorDrag {
    /// The joint entity being authored (`Entity::to_bits`).
    pub(crate) joint_bits: u64,
    /// What the grabbed handle authors.
    pub(crate) kind: PointHandleKind,
    /// What the press was holding, in the coordinate the value lives in.
    grab: Grab,
    /// The candidate snapped to this frame; `None` while free or on a parameter grip.
    pub snap: Option<[f32; 2]>,
}

/// The press-time offset cursor→value, **in the value's own coordinate**
/// (grab-relative: pressing off a dot must not teleport it).
#[derive(Copy, Clone, Debug)]
pub enum Grab {
    /// Anchor: the world vector from the cursor to the anchor.
    World([f32; 2]),
    /// Limit wall: the angle (rad) from the cursor's bearing to the wall's.
    Angle(f32),
    /// Length ring: ring radius minus the cursor's distance from the anchor.
    Radius(f32),
    /// Stroke end of a rail: world vector cursor→end (a free POINT, like [`Grab::World`]).
    /// ⚠️ A Slider's limits are metres, never [`Grab::Angle`] (`limits_in_metres`).
    Rail([f32; 2]),
    /// Limitador de corda: metros de corda entre o autorado e a projeção do
    /// cursor (W-RopeStop). Escalar: a marca ANDA SOBRE a corda.
    Along(f32),
}

/// Snap radius in **screen** px (converted to world at the current zoom) — the
/// same 14 px the pivot handle's Ctrl-snap uses (`input_dispatch::gizmo_drag`).
pub const SNAP_PX: f32 = 14.0;

/// Open a drag on `kind` of `joint`, or `None` if there is nothing to author
/// (a locked entity, an end with no body, a parameter this joint does not have).
/// ⚠️ The joint comes from the HIT, never the selection (W-J2b). Takes pieces of
/// `AppGfx`, not `&AppGfx`: the caller holds a `&mut` into `gfx.hero_screen`.
#[must_use]
pub fn open_drag(
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
    } else if kind.is_wheel() {
        wheel::open_grab(sim, joint, kind, cursor)?
    } else if let Some(side) = kind.rope_stop_side() {
        // ⚠️ Geometria da PONTE, número do ECS; `stop_at_point` inverte `stop_mark`.
        let leg = physics.rope_stop_legs(joint)[side]?;
        let held = stops_of(sim, joint)[side];
        Grab::Along(held - ph2d_physics_ecs::stop_at_point(&leg, cursor))
    } else {
        // Measured against the same live `JointView` the overlay drew.
        let v = physics.joint_views().find(|v| v.entity == joint)?;
        let held = param_value(&v, kind)?;
        match kind {
            PointHandleKind::Length => Grab::Radius(held - distance(v.anchor_a, cursor)),
            // A rail's stroke is a LENGTH; `limits_in_metres` is the shared door.
            _ if v.kind.limits_in_metres() => {
                let axis = v.axis?;
                let end = [
                    v.anchor_a[0] + axis[0] * held,
                    v.anchor_a[1] + axis[1] * held,
                ];
                Grab::Rail([end[0] - cursor[0], end[1] - cursor[1]])
            }
            // Wall bearing minus cursor bearing: no jump on press.
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

/// A parameter grip's current value (limit rad / ring m); `None` when the joint
/// lacks it — the same `JointView` fields the overlay draws from.
fn param_value(v: &ph2d_physics_ecs::JointView, kind: PointHandleKind) -> Option<f32> {
    match kind {
        PointHandleKind::LimitMin => v.limits.map(|l| l[0]),
        PointHandleKind::LimitMax => v.limits.map(|l| l[1]),
        PointHandleKind::Length => v.length,
        // Pontos de MUNDO (âncora, roldana) e limitadores (lidos por `stops_of`).
        PointHandleKind::AnchorA
        | PointHandleKind::AnchorB
        | PointHandleKind::WheelCentre
        | PointHandleKind::WheelRim
        | PointHandleKind::WheelRimOut
        | PointHandleKind::RopeStopA
        | PointHandleKind::RopeStopB
        | PointHandleKind::JumpApex => None,
    }
}

/// World bearing of `p` seen from `from`, radians (`libm`, the crate convention).
fn bearing(from: [f32; 2], p: [f32; 2]) -> f32 {
    libm::atan2f(p[1] - from[1], p[0] - from[0])
}

pub(crate) fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
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

/// `x` moved by whole turns to the nearest branch of `near` (no jump at ±pi).
fn unwrap_near(x: f32, near: f32) -> f32 {
    near + wrap_pi(x - near)
}
// ⛔ Era um `impl crate::App`; agora só [`CanvasCtx`] — nenhum método novo no `AppHost`.

/// Follow the cursor with the open drag. A no-op when none is in flight.
///
/// Anchors write through the bridge's anchor door; parameters write the joint
/// through the §12 funnel (`joint_with_edit`), so posing and typing agree.
/// ⚠️ `ctrl` chega RESOLVIDO pela shell (o modificador de encaixe), nunca cru.
pub fn advance_joint_anchor_drag(
    st: &mut PhysicsState,
    ctx: &mut CanvasCtx<'_>,
    ctrl: bool,
    pointer: (f32, f32),
) {
    let Some(mut drag) = st.joint_anchor_drag else {
        return;
    };
    let window_size = ctx.window;
    let cursor = ctx.camera.screen_to_world(pointer, window_size);
    let entity = ph2d_ecs::Entity::from_bits(drag.joint_bits);
    drag.snap = match drag.grab {
        Grab::World(off) => {
            let free = [cursor[0] + off[0], cursor[1] + off[1]];
            // Um ímã para âncora e roldana; muda só de quem é o collider. Roldana
            // de CENÁRIO não tem candidato: a porta devolve zero (sem ramo especial).
            let mut cands = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
            let n = if !ctrl {
                0
            } else if drag.kind.is_wheel() {
                ctx.bridge.wheel_snap_targets(&*ctx.sim, entity, &mut cands)
            } else {
                let side = anchor_side(drag.kind).expect("a world grab is an anchor's");
                ctx.bridge
                    .joint_snap_targets(&*ctx.sim, entity, side, &mut cands)
            };
            let threshold = SNAP_PX * ctx.camera.height_world / window_size.height as f32;
            let (target, snap) = match nearest_within(&cands[..n], free, threshold) {
                Some(p) => (p, Some(p)),
                None => (free, None),
            };
            if drag.kind.is_wheel() {
                wheel::move_wheel(ctx.sim, entity, target);
            } else {
                let side = anchor_side(drag.kind).expect("a world grab is an anchor's");
                ctx.bridge
                    .set_joint_anchor_world(ctx.sim, entity, side, target);
            }
            snap
        }
        // ⚠️ No magnet on parameter grips: a named gap (would need a step/grid).
        Grab::Angle(off) => {
            let (anchor_a, angle_a) = anchor_of(ctx.bridge, entity);
            let raw = bearing(anchor_a, cursor) + off - angle_a;
            write_limit(ctx.sim, entity, drag.kind, raw);
            None
        }
        Grab::Radius(off) => {
            // Raio = distância do cursor ao CENTRO (âncora → joint; roda → roda).
            if drag.kind.is_wheel_radius() {
                wheel::resize_wheel(ctx.sim, entity, drag.kind, cursor, off);
            } else {
                let (anchor_a, _) = anchor_of(ctx.bridge, entity);
                let len = distance(anchor_a, cursor) + off;
                write_length(ctx.sim, entity, len);
            }
            None
        }
        Grab::Rail(off) => {
            // Anchor from the view the overlay DREW the rail from.
            if let Some(anchor) = ctx
                .bridge
                .joint_views()
                .find(|v| v.entity == entity)
                .map(|v| v.anchor_a)
            {
                let end = [cursor[0] + off[0], cursor[1] + off[1]];
                write_rail_end(ctx.sim, entity, drag.kind, anchor, end);
            }
            None
        }
        // O LIMITADOR anda SOBRE a corda; a perna é relida VIVA a cada frame.
        // ⚠️ Sem ímã, como no `Grab::Angle`.
        Grab::Along(off) => {
            if let Some(side) = drag.kind.rope_stop_side()
                && let Some(leg) = ctx.bridge.rope_stop_legs(entity)[side]
            {
                let s = (ph2d_physics_ecs::stop_at_point(&leg, cursor) + off).clamp(0.0, leg.len);
                write_stop(ctx.sim, entity, side, s);
            }
            None
        }
    };
    st.joint_anchor_drag = Some(drag);
}

/// **Aim the rail AND set its stroke, from one dragged point** (W-J6c).
///
/// Anchor→end is the axis; its length is that end of the stroke. The dragged
/// end keeps the SIDE (sign) its value had. ⚠️ The axis is the joint's own
/// `Transform::rotation` (same field as the §0 Rotation row). Degenerate (end
/// ON the anchor): axis kept, stroke → 0, never `atan2(0,0)`.
fn write_rail_end(
    sim: &mut SimWorld,
    joint: ph2d_ecs::Entity,
    kind: PointHandleKind,
    anchor: [f32; 2],
    end: [f32; 2],
) {
    let Some(&current) = sim.world().get::<PhysicsJoint>(joint) else {
        return;
    };
    let held = if matches!(kind, PointHandleKind::LimitMin) {
        current.limit_min
    } else {
        current.limit_max
    };
    // A zero value takes its handle's usual side.
    let side = if held < 0.0 || (held == 0.0 && matches!(kind, PointHandleKind::LimitMin)) {
        -1.0f32
    } else {
        1.0
    };
    let (dx, dy) = (end[0] - anchor[0], end[1] - anchor[1]);
    let len = dx.hypot(dy);
    if len > 1e-4 {
        // The axis points at a forward end and away from a rearward one.
        let ang = libm::atan2f(side * dy, side * dx);
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(joint) {
            t.rotation = ang;
        }
    }
    write_limit(sim, joint, kind, side * len);
}

/// **Pose one end of the range** — a wall of a hinge's arc (radians relative to
/// body A) or an end of a rail's stroke (metres along the axis).
///
/// `raw` is in the COMPONENT's unit; [`crate::joint::limit_out`] converts it.
/// ⚠️ Unwrapped against the held value only when it is an ANGLE (a stroke has no turns).
/// ⚠️ An end STOPS at its sibling — never the swap `clamped` does for typed pairs.
/// ⚠️ The UNIT is the kind's: degrees written as metres was a runaway loop.
fn write_limit(sim: &mut SimWorld, joint: ph2d_ecs::Entity, kind: PointHandleKind, raw: f32) {
    let Some(&current) = sim.world().get::<PhysicsJoint>(joint) else {
        return;
    };
    let (held, other) = match kind {
        PointHandleKind::LimitMin => (current.limit_min, current.limit_max),
        _ => (current.limit_max, current.limit_min),
    };
    let want = if current.kind.limits_in_metres() {
        raw
    } else {
        unwrap_near(raw, held)
    };
    let walled = if matches!(kind, PointHandleKind::LimitMin) {
        want.min(other)
    } else {
        want.max(other)
    };
    // The SAME door the §12 row reads from.
    let ui = crate::joint::limit_out(current.kind, walled);
    let edit = if matches!(kind, PointHandleKind::LimitMin) {
        JointFieldEdit::LimitMin(ui)
    } else {
        JointFieldEdit::LimitMax(ui)
    };
    write_edit(sim, joint, current, edit);
}

/// **Pose the length ring** — rest or max length, by the kind's `length_field`.
fn write_length(sim: &mut SimWorld, joint: ph2d_ecs::Entity, len: f32) {
    let Some(&current) = sim.world().get::<PhysicsJoint>(joint) else {
        return;
    };
    // ⚠️ `length_field`, nunca uma lista de tipos (o Rod ficava mudo assim).
    let edit = match current.kind.length_field() {
        Some(ph2d_physics_ecs::LengthField::Rest) => JointFieldEdit::RestLength(len),
        Some(ph2d_physics_ecs::LengthField::Max) => JointFieldEdit::MaxLength(len),
        // Sem comprimento não há anel nem grip.
        None => return,
    };
    write_edit(sim, joint, current, edit);
}

/// The write itself: through the §12 funnel, then IN PLACE (the diff undo captures it).
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

/// The candidate closest to `p` within `threshold` world units (nearest wins, not first).
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
    /// body A)` — or `None` when it is not a limit drag. Read from the COMPONENT
    /// (post-clamp), so the ghost shows what the joint HAS.
    #[must_use]
    pub fn posed_limit(&self, sim: &SimWorld) -> Option<(ph2d_ecs::Entity, f32)> {
        let joint = ph2d_ecs::Entity::from_bits(self.joint_bits);
        let j = sim.world().get::<PhysicsJoint>(joint)?;
        match self.kind {
            PointHandleKind::LimitMin => Some((joint, j.limit_min)),
            PointHandleKind::LimitMax => Some((joint, j.limit_max)),
            _ => None,
        }
    }
}

#[path = "joint_anchor_drag_wheel.rs"]
mod wheel;

/// A metade do LIMITADOR (W-RopeStop): autora o `RopeStops` da CORDA.
#[path = "joint_anchor_drag_stop.rs"]
mod stop;

pub(crate) use stop::stops_of;
use stop::write_stop;

#[cfg(test)]
#[path = "joint_anchor_drag_tests.rs"]
mod tests;
