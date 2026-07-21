//! **The collider half of a spawn** — [`build_collider`].
//!
//! Split out of `world.rs` for the 700-LOC cap (W-OneWay). `spawn_body` is two
//! questions: what RIGID BODY is this, and what COLLIDER hangs off it. This is the
//! second — every `BodyDesc` field that describes the SHAPE and its surface, in one
//! place, so a new collider property has an obvious home.

use rapier2d::geometry::{Collider, ColliderBuilder};
use rapier2d::na::Vector2;
use rapier2d::pipeline::ActiveHooks;
use rapier2d::prelude::nalgebra;

use super::desc::BodyDesc;
use super::oneway;
use super::shape::{ShapeDesc, capsule_vertices, ellipse_vertices};

/// Turn a [`BodyDesc`]'s collider half into a rapier `Collider`.
///
/// Every branch here is byte-neutral at its default, which is what lets each wave
/// append a collider property without re-simulating existing art.
pub(super) fn build_collider(desc: &BodyDesc) -> Collider {
    let shape = match desc.shape {
        ShapeDesc::Ball { radius } => ColliderBuilder::ball(radius),
        ShapeDesc::Cuboid { half_x, half_y } => ColliderBuilder::cuboid(half_x, half_y),
        // A true capsule: rapier has it natively, so a uniformly-scaled
        // character collider is exact (and rounder than any polygon).
        ShapeDesc::Capsule {
            half_height,
            radius,
        } => ColliderBuilder::capsule_y(half_height, radius),
        // Non-uniformly scaled capsule → elliptical caps, which no solver
        // represents exactly; same convex-polygon treatment as the ellipse.
        ShapeDesc::Stadium {
            half_height,
            rx,
            ry,
        } => {
            let pts: Vec<_> = capsule_vertices(half_height, rx, ry)
                .into_iter()
                .map(|[x, y]| nalgebra::Point2::new(x, y))
                .collect();
            ColliderBuilder::convex_polyline(pts).unwrap_or_else(|| {
                ColliderBuilder::capsule_y(half_height.max(f32::MIN_POSITIVE), rx.max(ry))
            })
        }
        ShapeDesc::Ellipse { rx, ry } => {
            let pts: Vec<_> = ellipse_vertices(rx, ry)
                .into_iter()
                .map(|[x, y]| nalgebra::Point2::new(x, y))
                .collect();
            // `convex_polyline` returns None only on a degenerate ring
            // (an axis scaled to ~0). That is not a shape a real sprite
            // produces, but a `None` here must not panic the spawn — fall
            // back to a ball of the larger half-extent so the body still
            // exists and collides.
            ColliderBuilder::convex_polyline(pts)
                .unwrap_or_else(|| ColliderBuilder::ball(rx.max(ry).max(f32::MIN_POSITIVE)))
        }
    };
    // Mass source: an explicit override (`Some(m)` → `.mass(m)`, kg, ignoring
    // density — Unity's manual mass) or auto (`None` → `.density(d)`, mass =
    // density × area, rapier's own default and byte-identical to before this
    // existed). Exactly one is set, never both — they are the same quantity by
    // two roads. The angular inertia is derived from the shape either way.
    let shape = match desc.mass_override {
        Some(m) => shape.mass(m),
        None => shape.density(desc.density),
    };
    shape
        .restitution(desc.restitution)
        .friction(desc.friction)
        // How this collider's restitution/friction combine with another's on
        // contact. `Average` on both is rapier's own default (byte-identical to
        // before this existed); `Max` makes a superball bounce off ANY floor —
        // rapier resolves a contact with `rule1.max(rule2)`, so the more
        // energetic of the two colliders wins. Rides the `BodyDesc`, so a rewind
        // re-arms it.
        .restitution_combine_rule(desc.material.restitution)
        .friction_combine_rule(desc.material.friction)
        // A sensor passes through (no contact forces) but the narrow phase
        // still records its overlaps — read back by `intersecting_body_pairs`.
        .sensor(desc.is_sensor)
        // The collider's position relative to its body. `[0, 0]` centres it on
        // the body (rapier's default, byte-identical to before this existed);
        // rapier rotates this translation with the body, so an offset foot-box
        // turns with the character. Scale is already folded in by the caller.
        .translation(Vector2::new(desc.offset[0], desc.offset[1]))
        // One-way (jump-through) platform. The flag reaches the stateless hook as a
        // `user_data` bit, and `MODIFY_SOLVER_CONTACTS` is what makes rapier call
        // the hook for this pair at all — without it the bit is inert. A collider
        // that is not one-way sets neither, so it never reaches the hook and is
        // byte-identical to before this existed (see `oneway`).
        .user_data(if desc.one_way { oneway::ONE_WAY_BIT } else { 0 })
        .active_hooks(if desc.one_way {
            ActiveHooks::MODIFY_SOLVER_CONTACTS
        } else {
            ActiveHooks::empty()
        })
        .build()
}
