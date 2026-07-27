//! The seam between the Physics **world panel** and the `PhysicsBridge`.
//!
//! ⚠️ Not to be confused with [`physics_bridge`](super::physics_bridge), which
//! steps the SIMULATION at the Playhead tick, in an entirely different phase of
//! the frame. This one publishes what the panel paints and applies what the
//! artist did. Two names, two jobs.
//!
//! ## Why this panel is not tool-gated
//!
//! Every other docked panel here mirrors a tool's activation
//! (`hero.panel_visibility.insert("vector", vector_active)`). Physics is the
//! world/scene-settings category (ADR-0131 D8): it belongs to the document, not
//! to a tool, so **the artist owns its visibility** and this bridge never
//! writes it. That also means no `LAST_ACTIVE` edge-trigger and no Inspector
//! takeover — there is no activation edge to trigger on, and stealing the
//! Inspector slot on a panel the artist opened deliberately would take away the
//! thing they were probably looking at.

use ph2d_editor::screens::hero::HeroScreen;
use ph2d_panel_physics::{PhysicsIntent, PhysicsSnapshot};
use ph2d_physics_ecs::{InteractionSettings, PhysicsBridge};

/// Publish the world state for `paint`, then apply whatever the artist did.
///
/// Returns the new value for `App.show_colliders` — the panel does not own that
/// flag, the shell does (the `B` key toggles the same one), so the toggle comes
/// back as a request rather than a write.
pub(crate) fn dispatch(
    hero: &mut HeroScreen,
    physics: &mut PhysicsBridge,
    show_colliders: bool,
    interaction: &mut InteractionSettings,
) -> bool {
    // ── 1. Publish. Every row reads this; the panel keeps no copy. ──
    ph2d_panel_physics::set_current_physics(Some(PhysicsSnapshot {
        settings: physics.settings(),
        // Displayed, never owned (ADR-0131 D4): the world scale is a PROJECT
        // setting, edited in Settings → Pixels per Meter.
        pixels_per_meter: hero.project.pixels_per_meter,
        show_colliders,
        body_count: physics.body_count(),
        // The interaction tool (W-Hand). Runtime-only state of the SHELL — it
        // rides this snapshot because the panel paints it, not because it is a
        // world setting; `ph2d_physics_ecs::interaction` says why it is never
        // persisted.
        interaction: *interaction,
    }));

    // ── 2. Apply. The panel queued intents during event dispatch. ──
    let mut colliders = show_colliders;
    for intent in ph2d_panel_physics::drain_intents() {
        match intent {
            // `set_settings` clamps, pushes into rapier, and drops the scrub
            // cache — and is a no-op when nothing changed, which matters here
            // because this runs every frame.
            PhysicsIntent::SetSettings(s) => physics.set_settings(s),
            PhysicsIntent::ToggleColliders => colliders = !colliders,
            // Clamped on the way in, by the same door the model uses: a NaN
            // stiffness would reach the solver and poison the pose, the
            // `Transform` and the determinism hash.
            PhysicsIntent::SetInteraction(s) => *interaction = s.clamped(),
        }
    }
    colliders
}
