//! [`BodyDefaults`] — the world-level values every new body is born with.
//!
//! ## Why these are a WORLD setting even though rapier stores them per-body
//!
//! rapier's [`IntegrationParameters`] has no global damping and no global sleep
//! thresholds — both live on the rigid body ([`RigidBodyActivation`], and the
//! builder's `linear_damping`/`angular_damping`). Measured, not assumed: the
//! parameter struct's public fields are `dt`, the contact/joint spring terms,
//! the solver iteration counts and the CCD knobs, and nothing else.
//!
//! Exposing them as world settings anyway is the idiom every 2D engine ships —
//! Godot's *Project Settings → Physics → 2D* carries `default_linear_damp`,
//! `default_angular_damp`, `sleep_threshold_linear`, `sleep_threshold_angular`
//! and `time_before_sleep`; Unity's *Physics 2D* carries the sleep tolerances.
//! An artist asking "how much drag does this world have?" is asking one
//! question, and answering it body-by-body would be a worse product.
//!
//! ⚠️ **So today there is exactly ONE door for each of these numbers**, and that
//! is the whole reason this type is safe. If a per-body damping override is ever
//! added, it MUST arrive with a combine mode (Godot's `damp_mode`
//! Combine/Replace) — a second field that silently wins over this one is the
//! "two doors to the same question" failure this repo keeps paying for.
//!
//! [`IntegrationParameters`]: rapier2d::dynamics::IntegrationParameters
//! [`RigidBodyActivation`]: rapier2d::dynamics::RigidBodyActivation

use rapier2d::dynamics::{RigidBodyActivation, RigidBodySet};

/// World-level defaults stamped onto every body [`super::PhysicsWorld::spawn_body`]
/// creates, and re-stamped onto every live body by
/// [`super::PhysicsWorld::set_body_defaults`].
///
/// Plain `Copy` data — no rapier types cross this boundary, so the ECS bridge
/// can build one without depending on rapier (the same rule [`super::BodyDesc`]
/// follows).
///
/// **Every field defaults to the value rapier itself uses**, so a world that
/// never touches this struct simulates byte-identically to one built before it
/// existed. That is not a claim, it is a gate:
/// `the_body_defaults_are_the_ones_rapier_already_used`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyDefaults {
    /// Linear drag. `0.0` (rapier's own) = vacuum; higher slows translation.
    pub linear_damping: f32,
    /// Angular drag. `0.0` (rapier's own) = spins forever.
    pub angular_damping: f32,
    /// Linear speed below which a body may fall asleep (normalized — rapier
    /// multiplies it by `IntegrationParameters::length_unit`). rapier: `0.4`.
    pub sleep_linear_threshold: f32,
    /// Angular speed below which a body may fall asleep. rapier: `0.5`.
    pub sleep_angular_threshold: f32,
    /// How long a body must stay under both thresholds before sleeping,
    /// in seconds. rapier: `2.0`.
    pub time_until_sleep: f32,
}

impl BodyDefaults {
    /// rapier's own defaults, read from rapier rather than transcribed — a
    /// literal here would silently drift the day the dependency bumps, and the
    /// whole point of this struct is that leaving it alone changes nothing.
    pub fn rapier() -> Self {
        Self {
            // `RigidBodyBuilder` starts both damping factors at zero (its
            // `Default` leaves the fields unset).
            linear_damping: 0.0,
            angular_damping: 0.0,
            sleep_linear_threshold: RigidBodyActivation::default_normalized_linear_threshold(),
            sleep_angular_threshold: RigidBodyActivation::default_angular_threshold(),
            time_until_sleep: RigidBodyActivation::default_time_until_sleep(),
        }
    }

    /// Stamp these values onto every body currently in `bodies`.
    ///
    /// Called when the settings CHANGE, because a value the artist just typed
    /// has to describe the world they are looking at — not only the bodies that
    /// happen to be spawned after it.
    ///
    /// ⚠️ **Sleeping bodies are woken**, and that is not politeness. A sleeping
    /// body is not integrated at all, so a threshold that now says it should be
    /// awake would never be consulted — the setting would appear to do nothing
    /// until something else happened to disturb the body. Waking a settled stack
    /// is cheap and self-correcting: the bodies are already at rest, so they
    /// re-settle on the next step.
    pub(crate) fn apply_to_all(&self, bodies: &mut RigidBodySet) {
        // Iteration order does not matter here: this writes the SAME values to
        // every body and reads nothing, so the result is order-independent
        // (unlike spawning, which assigns handles).
        for (_, body) in bodies.iter_mut() {
            self.apply_to(body);
            body.wake_up(true);
        }
    }

    /// Stamp these values onto one body.
    pub(crate) fn apply_to(&self, body: &mut rapier2d::dynamics::RigidBody) {
        body.set_linear_damping(self.linear_damping);
        body.set_angular_damping(self.angular_damping);
        let act = body.activation_mut();
        act.normalized_linear_threshold = self.sleep_linear_threshold;
        act.angular_threshold = self.sleep_angular_threshold;
        act.time_until_sleep = self.time_until_sleep;
    }
}

impl Default for BodyDefaults {
    fn default() -> Self {
        Self::rapier()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapier2d::dynamics::RigidBodyBuilder;

    /// **The defaults are rapier's, and the oracle is RAPIER — not us.**
    ///
    /// This lives here, as a unit test, for one reason: an integration test
    /// can only reach `BodyDefaults::rapier()`, so every comparison it could
    /// make is this function against itself. That version of this gate was
    /// written first and stayed **green** while `linear_damping` was mutated
    /// to `0.05` — the same trap the audio line documented twice (a fast path
    /// measured against itself). The only oracle that can fail is a body
    /// rapier built and nobody configured.
    ///
    /// What it defends: adding this struct must cost every existing project
    /// exactly nothing. A "nicer" default here silently re-simulates all art.
    #[test]
    fn the_defaults_are_the_ones_rapier_hands_out_untouched() {
        let untouched = RigidBodyBuilder::dynamic().build();
        let ours = BodyDefaults::rapier();

        assert_eq!(
            ours.linear_damping,
            untouched.linear_damping(),
            "our default linear damping is not the one rapier gives an \
             unconfigured body"
        );
        assert_eq!(
            ours.angular_damping,
            untouched.angular_damping(),
            "our default angular damping is not the one rapier gives an \
             unconfigured body"
        );
        let act = untouched.activation();
        assert_eq!(
            ours.sleep_linear_threshold, act.normalized_linear_threshold,
            "our default sleep speed threshold is not rapier's"
        );
        assert_eq!(
            ours.sleep_angular_threshold, act.angular_threshold,
            "our default sleep spin threshold is not rapier's"
        );
        assert_eq!(
            ours.time_until_sleep, act.time_until_sleep,
            "our default sleep delay is not rapier's"
        );
    }
}
