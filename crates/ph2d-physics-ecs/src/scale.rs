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
use ph2d_physics::ShapeDesc;

use crate::components::ColliderShape;

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
    }
}
