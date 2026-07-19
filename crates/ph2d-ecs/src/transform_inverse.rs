//! **Going the other way** — from the hierarchy back to a local pose.
//!
//! [`crate::transform`] owns composition: a local [`Transform`] plus its
//! ancestors becomes a world one. This module owns the two questions that
//! composition alone cannot answer, and that anything computing in world space
//! has to ask:
//!
//! - *where is this entity, given its whole chain?* —
//!   [`parent_world_transform`] and its allocation-free twin;
//! - *what local pose would produce this world one?* —
//!   [`Transform::inverse_compose`], the exact inverse of `Transform::compose`.
//!
//! The pairing is the point. A physics solver answers in WORLD space while
//! `Transform` stores LOCAL, so a body parented to anything needs both
//! directions and needs them to agree; a scene that composes on the way in and
//! assigns raw on the way out is *stable* and wrong, drifting one parent-offset
//! per frame (`docs/Physics/BUGS_physics.md` #2).

use bevy_ecs::world::World;
use ph2d_core::Vec2;

use crate::ChildOf;
use crate::transform::Transform;
use bevy_ecs::entity::Entity;

pub fn parent_world_transform(world: &World, entity: Entity) -> Transform {
    parent_world_transform_into(world, entity, &mut Vec::new())
}

/// [`parent_world_transform`] with the ancestor buffer handed in, so a
/// per-frame caller allocates **nothing**.
///
/// The chain has to be collected before it can be folded — `ChildOf` walks
/// child→root and `compose` needs root→child — and the repo's composition is a
/// documented *approximation* (skew cascades additively, §2.2.1), so it is not
/// exactly associative in `f32`. That rules out the clever allocation-free
/// fold that accumulates on the way up: it would produce a *slightly different*
/// number than the plain function, and two answers to "where is this entity?"
/// is precisely the bug this family keeps producing. Same fold, same order,
/// borrowed buffer — `the_scratch_walk_is_the_plain_walk` pins the two
/// byte-for-byte.
///
/// `scratch` is cleared on entry; its contents on return are meaningless.
pub fn parent_world_transform_into(
    world: &World,
    entity: Entity,
    scratch: &mut Vec<Transform>,
) -> Transform {
    scratch.clear();
    let mut cur = world.get::<ChildOf>(entity).map(|c| c.parent());
    while let Some(p) = cur {
        if let Some(t) = world.get::<Transform>(p) {
            scratch.push(*t);
        }
        cur = world.get::<ChildOf>(p).map(|c| c.parent());
    }
    let mut acc = Transform::IDENTITY;
    for t in scratch.iter().rev() {
        acc = Transform::compose(acc, *t);
    }
    acc
}

/// The entity's own pose in WORLD space: its local [`Transform`] composed with
/// its whole ancestor chain.
///
/// `None` when the entity has no `Transform` — it is not placeable, so it has
/// no world pose to give.
///
/// ⚠️ **Anything that computes in world space must ask THIS**, not the raw
/// `Transform`. For a root entity the two are identical, which is exactly what
/// makes the mistake survive: every fixture built on root entities passes, and
/// the error appears only once something is parented — as a body that simulates
/// in one place and draws in another, or an overlay drawn a parent-offset away
/// from the sprite it is annotating. Both of those shipped
/// (`docs/Physics/BUGS_physics.md` #2).
#[must_use]
pub fn world_transform(world: &World, entity: Entity) -> Option<Transform> {
    world_transform_into(world, entity, &mut Vec::new())
}

/// [`world_transform`] with the ancestor buffer handed in, for callers that run
/// per frame and must not allocate.
#[must_use]
pub fn world_transform_into(
    world: &World,
    entity: Entity,
    scratch: &mut Vec<Transform>,
) -> Option<Transform> {
    let local = *world.get::<Transform>(entity)?;
    Some(Transform::compose(
        parent_world_transform_into(world, entity, scratch),
        local,
    ))
}

impl Transform {
    /// Every field is finite — no `NaN`, no `±inf`.
    ///
    /// Exists because a `Transform` is only safe to STORE if this holds: one
    /// non-finite field poisons the whole subtree's `GlobalTransform` through
    /// propagation (see [`Transform::compose`]'s `debug_assert`).
    #[inline]
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.translation.x.is_finite()
            && self.translation.y.is_finite()
            && self.rotation.is_finite()
            && self.scale.x.is_finite()
            && self.scale.y.is_finite()
            && self.skew_x.is_finite()
            && self.skew_y.is_finite()
    }

    /// The exact inverse of [`Transform::compose`]: given the `parent` and the
    /// composed `world`, recover the `child` that produced it.
    ///
    /// `inverse_compose(p, compose(p, c)) == c` — that round trip is the whole
    /// specification, and it is what the gate asserts over a swept space of
    /// rotations, scales and skews.
    ///
    /// **Why this exists.** A physics solver answers in WORLD space, and
    /// `Transform` is LOCAL. Writing a world pose straight into a child's
    /// `Transform` makes the renderer compose it *again* with the parent, so
    /// the body simulates in one place and draws in another — measured at a
    /// full parent-offset of divergence before this landed (see
    /// `BUGS_physics.md` #2). Anything that computes in world space and has to
    /// store the answer on a parented entity needs this direction.
    ///
    /// The algebra just runs `compose` backwards, in reverse order: undo the
    /// parent's translation, un-rotate, un-shear, un-scale, and subtract the
    /// angles that `compose` added.
    ///
    /// # Returns `None` when the result would not be finite
    ///
    /// Some parents destroy information: a **zero scale component** collapses
    /// the subtree onto a line or a point, and a **shear determinant of zero**
    /// (`tan(skew_x) · tan(skew_y) == 1`) does the same through the skew
    /// matrix — infinitely many child poses map to the same world one, so
    /// there is nothing to recover. Note that the shear case is *reachable*,
    /// not a corner: `SKEW_LIMIT` is `π/2 − 0.01`, so `tan` runs to ~100 and a
    /// (2, 0.5) pair sits comfortably inside the legal range.
    ///
    /// ⚠️ The guard is **"is every output finite?"**, not a comparison against
    /// zero and not a threshold. Testing `det == 0.0` is wrong twice over: it
    /// misses a determinant of `1e-30`, whose quotient overflows anyway, and
    /// it says nothing about a `NaN` that arrived in the *inputs*. Asking the
    /// result directly is the property the caller actually needs — *may I
    /// store this?* — and it needs no magic number to state
    /// ([[feedback_a_threshold_must_live_where_the_domain_is_empty]]).
    ///
    /// A merely **ill-conditioned** parent (tiny but non-zero determinant) is
    /// deliberately allowed through: it yields huge-but-finite local
    /// coordinates that `compose` maps straight back to the right world pose,
    /// so the pair stays self-consistent and the object draws where it belongs.
    /// Refusing it would be an arbitrary cutoff that breaks a working scene.
    ///
    /// Why refusal matters at all: `compose`'s own `debug_assert` spells out
    /// the cost of writing the alternative — **one corrupted angle poisons the
    /// entire subtree's `GlobalTransform`**, and signaling-vs-quiet `NaN` bit
    /// patterns drift cross-host. A caller that cannot recover a local pose
    /// must leave the `Transform` alone, not store a broken one.
    #[inline]
    #[must_use]
    pub fn inverse_compose(parent: Self, world: Self) -> Option<Self> {
        let tan_sx = libm::tanf(parent.skew_x);
        let tan_sy = libm::tanf(parent.skew_y);
        // Shear `[[1, tan_sx], [tan_sy, 1]]` — singular when its determinant
        // vanishes. Not tested against zero here; see the finiteness check
        // below, which covers this and everything else that can go wrong.
        let det = 1.0 - tan_sx * tan_sy;

        // `compose` ran: scale → shear → rotate → translate. Undo in reverse.
        let (sin, cos) = libm::sincosf(parent.rotation);
        let vx = world.translation.x - parent.translation.x;
        let vy = world.translation.y - parent.translation.y;
        // Un-rotate (transpose of a rotation is its inverse).
        let rx = vx * cos + vy * sin;
        let ry = -vx * sin + vy * cos;
        // Un-shear: the analytic inverse of `[[1, tan_sx], [tan_sy, 1]]`.
        let s_tx = (rx - ry * tan_sx) / det;
        let s_ty = (ry - rx * tan_sy) / det;
        // Un-scale.
        let out = Self {
            translation: Vec2::new(s_tx / parent.scale.x, s_ty / parent.scale.y),
            rotation: world.rotation - parent.rotation,
            scale: Vec2::new(
                world.scale.x / parent.scale.x,
                world.scale.y / parent.scale.y,
            ),
            skew_x: world.skew_x - parent.skew_x,
            skew_y: world.skew_y - parent.skew_y,
        };
        out.is_finite().then_some(out)
    }
}
