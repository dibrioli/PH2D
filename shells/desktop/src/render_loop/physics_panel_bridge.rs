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
use ph2d_physics_ecs::{InputTape, InteractionSettings, PhysicsBridge};

use super::run_stash::{self, RunVerb};

/// As duas fitas que este painel mostra e move (W25) — a corrida VIVA do
/// documento e a que espera por um desfazer.
///
/// ⚠️ **Um par emprestado, e não uma cópia:** a autoridade é o `App`, que é
/// também quem grava. Publicar um comprimento e aplicar a troca noutro lugar
/// faria o botão descrever uma corrida e descartar outra.
pub(crate) struct RunTapes<'a> {
    /// A corrida gravada — a que viaja no arquivo e que o Bake replaya.
    pub live: &'a mut InputTape,
    /// A corrida descartada, guardada na sessão.
    pub stash: &'a mut InputTape,
    /// O passo do relógio fixo, para dizer os comprimentos em SEGUNDOS.
    ///
    /// ⚠️ **A MESMA régua que a §14 usa** (`fixed_step.fixed_dt()`): um readout
    /// com outro passo diria outra duração para a mesma corrida, e as duas
    /// vistas passariam a discordar sobre um número que nenhuma delas inventou.
    pub fixed_dt: f64,
}

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
    run: RunTapes<'_>,
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
        // W25: os dois números da corrida, derivados das DUAS fitas — cada um da
        // SUA. Trocá-los é o erro que ninguém pega lendo, e é por isso que os
        // dois são afirmados por nome no gate.
        recorded_run_seconds: (run.live.len() as f64 * run.fixed_dt) as f32,
        discarded_run_seconds: (run.stash.len() as f64 * run.fixed_dt) as f32,
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
            // ⚠️ **A MESMA porta que a §14 atravessa** (W25). Uma segunda cópia
            // do `mem::take` faria a mesma coisa hoje — e é essa forma que
            // apodrece no dia em que o descarte ganhar um caso especial.
            PhysicsIntent::ClearRun => {
                run_stash::apply(RunVerb::Discard, &mut *run.live, &mut *run.stash);
            }
            PhysicsIntent::RestoreRun => {
                run_stash::apply(RunVerb::Restore, &mut *run.live, &mut *run.stash);
            }
        }
    }
    colliders
}
