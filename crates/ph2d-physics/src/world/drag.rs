//! Air drag — the half of "slowing down" that **knows how big a body is**.
//!
//! ## Why this exists next to `BodyDefaults::linear_damping`
//!
//! They are different models, and the difference is exactly what a smoke
//! caught (Enio, 2026-07-18: *"Air Drag… todos os objetos grandes e pequenos
//! caem na mesma velocidade"*).
//!
//! rapier's `linear_damping` scales velocity by `1/(1 + d·dt)` — it is a
//! **uniform decay**, so mass and size cannot enter it. Measured: with damping
//! `2.0`, four boxes spanning a **25× mass range** all fell at
//! **4.8925 m/s**, identical to four decimals. That is the correct behaviour of
//! that knob (Godot's `default_linear_damp` and Unity's `linearDamping` are the
//! same thing), and it is a genuinely useful one — top-down friction, a scene
//! that should feel syrupy.
//!
//! It is simply **not air**. Air pushes on a body's cross-section and is
//! resisted by its mass, so the published drag equation
//!
//! ```text
//! F = ½ · ρ · Cd · A · |v| · v          (Newtonian / quadratic drag)
//! ```
//!
//! gives `a = F/m`, and for a 2D body of uniform density with side `s` we have
//! `A ∝ s` and `m ∝ s²`, hence **`a ∝ v²/s`**: the small body decelerates more
//! and settles at a lower terminal speed. That is the intuition every artist
//! brings ("a feather and a rock do not fall the same"), and it is the one this
//! module implements.
//!
//! The knob lumps `½·ρ·Cd` into a single coefficient, because those three are
//! never separately meaningful to someone tuning a game: what they can feel is
//! "how thick is the air".
//!
//! ⚠️ **Zero is byte-identical to not having this at all** — the early-out below
//! is what keeps the cross-OS C9 hashes from moving, and there is a gate on it.

use rapier2d::dynamics::RigidBodySet;
use rapier2d::geometry::ColliderSet;
use rapier2d::na::Vector2;

/// Apply one substep of quadratic drag to every dynamic body.
///
/// Called from inside [`super::PhysicsWorld::step`]'s substep loop — **per
/// substep, not per tick**, because a force applied once per tick would be
/// wrong by the substep count, and the whole reason sub-stepping exists here is
/// that the integrator sees smaller slices.
pub(crate) fn apply(bodies: &mut RigidBodySet, colliders: &ColliderSet, k: f32, dt: f32) {
    // Fast path for the common case (drag off): skip the whole body walk.
    //
    // ⚠️ It is **only** a fast path. Bit-identity at `k == 0` does not depend on
    // it — the force below would be the zero vector and the impulse a no-op, so
    // the contract is honoured twice, by this branch and by the arithmetic. That
    // is why a mutation removing this line leaves every gate green, and it is
    // expected: same shape as the flat-paint early-out in the impasto GPU light
    // pass ([[feedback_layered_defenses_need_per_layer_gates]] — ask what each
    // layer protects ALONE, and be willing to answer "nothing, it is speed").
    if k <= 0.0 {
        return;
    }
    for (_, body) in bodies.iter_mut() {
        if !body.is_dynamic() {
            continue;
        }
        let v = *body.linvel();
        let speed = v.norm();
        if speed <= 0.0 {
            continue;
        }
        let length = characteristic_length(body.colliders(), colliders);
        // F = -k · L · |v| · v — quadratic, opposing motion.
        let force: Vector2<f32> = v * (-k * length * speed);
        // ⚠️ **Impulse (F·dt), not `add_force`.** rapier's `add_force` adds a
        // CONSTANT force that persists until `reset_forces` is called, and the
        // pipeline never calls it — so applying it once per substep accumulated
        // over every substep of every tick. Measured, that was ~720× the
        // intended force by the third second: bodies slammed to a near-stop and
        // the terminal speeds came out non-monotonic in size (0.05 / 0.51 /
        // 0.52 / 0.01 m/s), which is what sent me looking.
        //
        // An impulse is exactly "this force, for this slice of time", carries no
        // state into the next substep, and leaves the user-force channel free
        // for anything that later wants to push a body around.
        body.apply_impulse(force * dt, false);
    }
}

/// The body's cross-section, as one number.
///
/// **Isotropic on purpose**: the physically exact area is the silhouette
/// projected perpendicular to the velocity, which makes drag depend on which
/// way a box happens to be pointing — a rotating crate would then breathe in
/// and out as it tumbled, and no artist would connect that to a slider called
/// "Air Drag". The mean AABB extent is rotation-invariant and reads as "how big
/// is this thing", which is the question the knob is actually answering.
///
/// A body with no collider has no cross-section, so it gets no drag (rather
/// than a fabricated default that would make it behave like some arbitrary
/// size).
fn characteristic_length(
    handles: &[rapier2d::geometry::ColliderHandle],
    colliders: &ColliderSet,
) -> f32 {
    let Some(collider) = handles.first().and_then(|h| colliders.get(*h)) else {
        return 0.0;
    };
    let extents = collider.shape().compute_local_aabb().extents();
    (extents.x + extents.y) * 0.5
}
