//! **The world scale reaches the collider.**
//!
//! A sprite scaled 2× draws at twice the size — its quad multiplies by the
//! `Transform`'s scale. Until this module, the collider did not: `body_desc`
//! read `col.shape` verbatim, so a scaled sprite kept an authored-size
//! collider, and the physical body disagreed with the drawn one (ADR-0131 W6,
//! reported by Enio). Unity and Godot inherit the transform's scale into the
//! collider for the same reason; so does this.
//!
//! ## One door, two consumers
//!
//! [`scaled_shape`] is the *single* place that decides what a scaled collider
//! becomes. Both the bridge (→ `ShapeDesc` → rapier) and the overlay (→ the
//! drawn wireframe) call it, so they cannot disagree about whether a scaled
//! ball is a circle or an ellipse — the recurring failure of this line is two
//! readers of one fact drifting apart, and a resolution that lived in each
//! caller would be exactly that.
//!
//! ## Why an ellipse
//!
//! Scale is **per-axis** (`Transform::scale` is a `Vec2`). A `Cuboid` takes
//! that natively — a box is a box under per-axis scale. A `Ball` cannot: under
//! *non-uniform* scale a circle is genuinely an ellipse on screen, and rapier
//! has no ellipse. Collapsing it back to a circle (pick an axis, as Unity
//! does) would make the collider disagree with the visible sprite — the very
//! thing the collider outline exists to prevent. So a non-uniform ball becomes
//! [`ShapeDesc::Ellipse`], realised downstream as a convex polygon. A *uniform*
//! ball stays a `Ball`: an exact circle is cheaper and rounder than any
//! polygon, and — because unit scale is `(1, 1)` — it is what makes an
//! unscaled body byte-identical to before this module existed.

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics::{
    BodyDesc, CoefficientCombineRule, CombineRules, DampingDesc, RigidBodyType, ShapeDesc,
};

use crate::components::{
    BodyKind, Collider, ColliderShape, CombineRule, DampMode, DampingOverride, MaterialCombine,
    RigidBody,
};

/// Map the ECS-native, serde-safe [`CombineRule`] onto rapier's
/// `CoefficientCombineRule` — the ONE place the two enums meet, so they cannot
/// disagree (the `BodyKind` → `RigidBodyType` discipline). The discriminants
/// match by construction; the match is spelled out so a reordering of either
/// enum fails to compile rather than silently remapping a rule.
fn combine_to_rapier(r: CombineRule) -> CoefficientCombineRule {
    match r {
        CombineRule::Average => CoefficientCombineRule::Average,
        CombineRule::Min => CoefficientCombineRule::Min,
        CombineRule::Multiply => CoefficientCombineRule::Multiply,
        CombineRule::Max => CoefficientCombineRule::Max,
    }
}

/// Resolve an authored [`ColliderShape`] under a **world** scale into the
/// rapier-facing [`ShapeDesc`] the solver (and the overlay) should use.
///
/// `scale` is the composed world scale of the body's entity (from
/// `ph2d_ecs::world_transform`), so a body under a scaled parent inherits the
/// parent's scale — the collider ends up where the sprite is drawn.
///
/// Sign: scale can be negative (a flip). Only magnitude sets a collider's
/// size — a box/circle/ellipse is mirror-symmetric — so every axis is taken
/// through `abs()`.
///
/// The uniform/non-uniform split for a ball is decided by **exact** equality
/// of `|sx|` and `|sy|`. That is the honest threshold: unit scale (and any
/// deliberate uniform scale) lands on the exact-circle branch and stays
/// byte-identical to today, while anything the artist actually squashed
/// becomes an ellipse. There is no tuning constant to drift.
#[must_use]
pub fn scaled_shape(shape: ColliderShape, scale: Vec2) -> ShapeDesc {
    let sx = scale.x.abs();
    let sy = scale.y.abs();
    match shape {
        ColliderShape::Cuboid { half_x, half_y } => ShapeDesc::Cuboid {
            half_x: half_x * sx,
            half_y: half_y * sy,
        },
        ColliderShape::Ball { radius } => {
            if sx == sy {
                ShapeDesc::Ball {
                    radius: radius * sx,
                }
            } else {
                ShapeDesc::Ellipse {
                    rx: radius * sx,
                    ry: radius * sy,
                }
            }
        }
        // Same uniform/non-uniform split as the ball, for the same reason: a
        // capsule squashed on one axis has ELLIPTICAL caps, which no solver
        // represents exactly, so it degrades to the convex `Stadium` rather
        // than keeping circular caps that would no longer match the sprite.
        // The straight segment always follows the Y scale; the caps follow both.
        ColliderShape::Capsule {
            half_height,
            radius,
        } => {
            if sx == sy {
                ShapeDesc::Capsule {
                    half_height: half_height * sy,
                    radius: radius * sx,
                }
            } else {
                ShapeDesc::Stadium {
                    half_height: half_height * sy,
                    rx: radius * sx,
                    ry: radius * sy,
                }
            }
        }
    }
}

/// Translate the authored components + current WORLD pose into a plain
/// [`BodyDesc`] for `PhysicsWorld::spawn_body`. The one place ECS types meet
/// rapier's — everything downstream is rapier-free. Lives here beside
/// [`scaled_shape`], the door it uses for the collider shape.
/// `gravity_scale` is passed in rather than read here because it comes from the
/// **optional** [`crate::GravityScale`] component — a body without one takes the
/// neutral `1.0` — and the bridge is the half holding the `World` to look it up.
/// It rides the `BodyDesc` so `rewind_to`, which rebuilds from descriptors,
/// preserves it across a scrub (ADR-0131 W8).
/// `linvel`/`angvel` are the authored INITIAL velocity (world axes), from the
/// optional [`crate::InitialVelocity`] component — passed in for the same reason
/// as `gravity_scale`. World axes, not the parent's local frame: the body spawns
/// at its WORLD pose, so its launch is a world-space vector (the convention
/// Unity/Godot expose), and it rides the `BodyDesc` so a rewind re-arms it (W9).
/// `ccd` is the presence of the optional [`crate::Ccd`] marker (W-CCD) — passed
/// in for the same reason, and it rides the `BodyDesc` so a rewind re-arms it too.
/// `lock_rotation` is the presence of the optional [`crate::LockRotation`] marker —
/// same story again (Freeze Rotation; rides the `BodyDesc` for the rewind).
/// `lock_x`/`lock_y` are the presences of the optional [`crate::LockPositionX`]/
/// [`crate::LockPositionY`] markers (Freeze Position) — each an independent axis of
/// the same `LockedAxes` fold, riding the `BodyDesc` for the rewind like the rest.
/// `mass_override` is the optional [`crate::MassOverride`] component's value (`Some`
/// = explicit kg, ignoring density; `None` = auto mass) — a VALUED override like
/// `gravity_scale`, folded in here and riding the `BodyDesc` for the rewind.
/// `dominance` is the optional [`crate::Dominance`] component's value (`0` = neutral)
/// — the collision priority, folded in and riding the `BodyDesc` for the rewind.
/// `material` is the optional [`crate::MaterialCombine`] component (absent = both
/// `Average`) — the collider's friction/restitution combine policy, mapped to
/// rapier's `CoefficientCombineRule` here (the one door) and riding the `BodyDesc`.
/// `damping` is the optional [`crate::DampingOverride`] component (absent = the world
/// default drag) — mapped to `ph2d_physics::DampingDesc` here (`DampMode::Replace` →
/// `replace: true`, the one door) and riding the `BodyDesc` for the rewind.
/// `one_way` is the PRESENCE of the optional [`crate::OneWayPlatform`] marker — the
/// collider is solid only from its local +Y side; rides the `BodyDesc` for the rewind.
/// `effector` is the optional [`crate::AreaEffector`] component's force in newtons
/// (absent = a body that pushes nothing) — the force zone, riding the `BodyDesc` for
/// the rewind like the rest. The wrapper is the half that refuses it on a solid
/// collider (`effector::zone_force`), so this stays a plain hand-off.
///
/// The argument list grows one flag per per-body wave (gravity / velocity / ccd /
/// lock); each is an independent optional component the bridge reads and folds in,
/// so bundling them into a struct would only move the same fields behind one name.
#[allow(clippy::too_many_arguments)]
pub(crate) fn body_desc(
    rb: &RigidBody,
    col: &Collider,
    t: &Transform,
    gravity_scale: f32,
    linvel: [f32; 2],
    angvel: f32,
    ccd: bool,
    lock_rotation: bool,
    lock_x: bool,
    lock_y: bool,
    mass_override: Option<f32>,
    dominance: i8,
    material: MaterialCombine,
    damping: Option<DampingOverride>,
    one_way: bool,
    effector: Option<[f32; 2]>,
) -> BodyDesc {
    BodyDesc {
        gravity_scale,
        linvel,
        angvel,
        ccd,
        lock_rotation,
        lock_x,
        lock_y,
        mass_override,
        dominance,
        material: CombineRules {
            restitution: combine_to_rapier(material.restitution),
            friction: combine_to_rapier(material.friction),
        },
        // The optional damping override, mapped to the wrapper's plain `DampingDesc`
        // (the mode becomes the `replace` bool). `None` leaves the body on the world
        // default drag. This is the single ECS→wrapper mapping for damping.
        damping: damping.map(|d| DampingDesc {
            linear: d.linear,
            angular: d.angular,
            replace: matches!(d.mode, DampMode::Replace),
        }),
        one_way,
        effector,
        body_type: match rb.kind {
            BodyKind::Dynamic => RigidBodyType::Dynamic,
            BodyKind::Static => RigidBodyType::Fixed,
            // POSITION-based, not velocity-based: the scene hands us a POSE
            // (a `Transform` a curve wrote), never a velocity. rapier derives
            // the velocity from the pose it was aimed at, which is what makes
            // an animated body push instead of teleport.
            BodyKind::Kinematic => RigidBodyType::KinematicPositionBased,
        },
        x: t.translation.x,
        y: t.translation.y,
        rotation: t.rotation,
        density: col.density,
        restitution: col.restitution,
        friction: col.friction,
        layer: col.layer,
        is_sensor: col.is_sensor,
        // `t` is the WORLD transform (composed through the parent chain), so
        // `t.scale` is the world scale — the collider inherits a scaled parent's
        // scale, landing where the sprite is drawn. `scaled_shape` is the one
        // door the overlay reads too, so the wireframe cannot describe a
        // different size than the solver simulates.
        shape: scaled_shape(col.shape, t.scale),
        // The collider offset scaled into world units. **Signed** scale, not
        // `abs`: unlike the shape's half-extents (a size, always positive), the
        // offset is a POSITION, so a flip (`scale.x < 0`) must MIRROR it to the
        // other side — the foot-box of a flipped character moves with the flip.
        // rapier then rotates this relative translation with the body; the overlay
        // reads `col.offset` through the same signed scale so the two agree.
        offset: [col.offset[0] * t.scale.x, col.offset[1] * t.scale.y],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desc_with(scale: Vec2, offset: [f32; 2]) -> BodyDesc {
        let rb = RigidBody {
            kind: BodyKind::Dynamic,
        };
        let col = Collider {
            shape: ColliderShape::Ball { radius: 0.5 },
            offset,
            ..Collider::default()
        };
        let t = Transform {
            translation: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale,
            skew_x: 0.0,
            skew_y: 0.0,
        };
        body_desc(
            &rb,
            &col,
            &t,
            1.0,
            [0.0, 0.0],
            0.0,
            false,
            false,
            false,
            false,
            None,
            0,
            MaterialCombine::default(),
            None,
            false,
            None,
        )
    }

    #[test]
    fn the_collider_offset_scales_with_the_body() {
        // Unit scale is byte-identical to the authored offset.
        assert_eq!(
            desc_with(Vec2::new(1.0, 1.0), [0.7, -0.3]).offset,
            [0.7, -0.3]
        );
        // A 2×/3× scale scales the offset the same amount (it is a position in the
        // body's local frame, so it grows with the body — Unity does the same).
        assert_eq!(
            desc_with(Vec2::new(2.0, 3.0), [1.0, 1.0]).offset,
            [2.0, 3.0]
        );
    }

    #[test]
    fn a_flip_mirrors_the_collider_offset() {
        // ⚠️ SIGNED scale, not `abs`: a flip (`scale.x < 0`) MIRRORS the offset to
        // the other side — the foot-box of a flipped character moves with it. Using
        // `abs` (like the shape's half-extents do) would leave the offset on the
        // original side, and the collider would sit on the wrong half of the flip.
        assert_eq!(
            desc_with(Vec2::new(-1.0, 1.0), [1.0, 0.0]).offset,
            [-1.0, 0.0]
        );
        assert_eq!(
            desc_with(Vec2::new(1.0, -1.0), [0.0, 1.0]).offset,
            [0.0, -1.0]
        );
    }
}
