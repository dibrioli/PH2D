//! **One-way (jump-through) platforms** — the physics hook that makes a collider
//! solid from one side only.
//!
//! rapier ships the primitive itself (`ContactModificationContext::update_as_oneway_platform`),
//! including the allowed/forbidden **hysteresis** that keeps a body from popping while
//! it straddles the surface. This module is the integration: which collider is a
//! platform, and which direction is its solid side.
//!
//! ## How the flag reaches the hook
//!
//! A hook is `&self` and sees only the contact context, so the "this collider is a
//! platform" bit travels in the collider's own `user_data` ([`ONE_WAY_BIT`]) — which is
//! exactly what rapier provides `user_data` for — alongside
//! `ActiveHooks::MODIFY_SOLVER_CONTACTS`, without which the hook is never called for
//! that pair. Both are set in `PhysicsWorld::spawn_body` from `BodyDesc::one_way`.
//!
//! ## ⚠️ The allowed normal lives in COLLIDER1's frame
//!
//! `update_as_oneway_platform` tests `manifold.local_n1`, the contact normal in
//! **collider1's** local space pointing toward collider1's exterior. The platform may
//! be collider1 **or** collider2 — rapier does not order the pair for us — so a
//! constant `+Y` is only right in the first case. Passing `-Y` for the second (as the
//! rapier demo does for its axis-aligned fixture) silently assumes the two colliders
//! share an orientation, which a rotated platform or a tumbling body breaks.
//!
//! So the direction is derived, not assumed:
//!
//! ```text
//! allowed_local_n1 = R1⁻¹ · (s · platform_world_up),   s = +1 if platform is collider1
//!                                                       s = −1 if platform is collider2
//! ```
//!
//! When the platform IS collider1 this reduces to its own local `+Y` exactly, so there
//! is ONE formula and no special case — and it stays correct for a platform at any
//! angle meeting a body at any angle.

use rapier2d::dynamics::RigidBodySet;
use rapier2d::geometry::{ColliderSet, SolverFlags};
use rapier2d::na::Vector2;
use rapier2d::pipeline::{ContactModificationContext, PairFilterContext, PhysicsHooks};

/// The `user_data` bit that marks a collider as a one-way platform. `user_data` is
/// rapier's per-collider payload and this crate uses no other bit of it, so a plain
/// mask is enough; a second consumer would take the next bit.
pub const ONE_WAY_BIT: u128 = 1 << 0;

/// How far the contact normal may tilt from the platform's solid side and still count
/// as "landing on it", in radians.
///
/// The distinction the hook has to make is **up versus down** — the two cases are 180°
/// apart — so the threshold only has to split that, and a generous cone is what keeps
/// a body landing slightly tilted (or contacting near an edge, where the normal fans)
/// from dropping through. A quarter turn is that: everything on the solid half-space
/// lands, everything approaching from the far side passes.
///
/// rapier's own demo uses `0.1` rad (5.7°), which is tuned to a flat box settling
/// squarely on a flat platform; on a ball, or near a rim, that cone is narrow enough to
/// flip a legitimate landing to "forbidden".
pub const ALLOWED_ANGLE: f32 = std::f32::consts::FRAC_PI_4;

/// Is this collider a one-way platform?
fn is_one_way(colliders: &ColliderSet, handle: rapier2d::geometry::ColliderHandle) -> bool {
    colliders
        .get(handle)
        .is_some_and(|c| c.user_data & ONE_WAY_BIT != 0)
}

/// The physics hooks the world steps with. **Stateless** — everything it needs is on
/// the collider (the `user_data` bit) or in the contact context (the poses), so it
/// holds no mirror of the scene that could go stale, and it is trivially `Send + Sync`.
///
/// A world with no one-way collider never reaches [`Self::modify_solver_contacts`]:
/// rapier only calls it for pairs where a collider carries
/// `ActiveHooks::MODIFY_SOLVER_CONTACTS`. That is what makes installing these hooks
/// byte-identical for every scene authored before one-way platforms existed.
pub struct OneWayHooks;

impl PhysicsHooks for OneWayHooks {
    /// Left at rapier's default deliberately: a one-way platform still needs its
    /// contacts COMPUTED (that is how we learn which side the body is on) — it is the
    /// solver contacts that get cleared, one manifold at a time, below.
    fn filter_contact_pair(&self, _context: &PairFilterContext) -> Option<SolverFlags> {
        Some(SolverFlags::COMPUTE_IMPULSES)
    }

    fn modify_solver_contacts(&self, context: &mut ContactModificationContext) {
        // Which of the pair is the platform? If both are (two one-way platforms
        // touching), collider1 wins — an arbitrary but deterministic tie-break, and the
        // pair is degenerate anyway: each is trying to be solid to the other from a
        // different side.
        let (platform, platform_is_c1) = if is_one_way(context.colliders, context.collider1) {
            (context.collider1, true)
        } else if is_one_way(context.colliders, context.collider2) {
            (context.collider2, false)
        } else {
            // The hook fires for any pair where EITHER collider asked for it; a pair
            // that got here without a platform has nothing to modify.
            return;
        };

        let Some(plat) = context.colliders.get(platform) else {
            return;
        };
        let Some(c1) = context.colliders.get(context.collider1) else {
            return;
        };
        // The platform's solid side, in world space: its own local +Y, rotated by its
        // pose. Deriving it from the pose is why a rotated platform is one-way along
        // its OWN axis rather than along the world's.
        let world_up = plat.position().rotation * Vector2::y();
        // Into collider1's frame, flipped when the platform is collider2 (the normal
        // then points from the body TOWARD the platform). See the module docs.
        let signed = if platform_is_c1 { world_up } else { -world_up };
        let allowed_local_n1 = c1.position().rotation.inverse_transform_vector(&signed);

        context.update_as_oneway_platform(&allowed_local_n1, ALLOWED_ANGLE);
    }
}

/// Kept so the hook's own type-level contract (`Send + Sync`, required by rapier's
/// non-wasm `PhysicsHooks`) is a compile error to break rather than a runtime surprise.
#[allow(dead_code)]
fn _hooks_are_send_sync() {
    fn assert<T: Send + Sync>() {}
    assert::<OneWayHooks>();
    let _ = std::mem::size_of::<RigidBodySet>();
}
