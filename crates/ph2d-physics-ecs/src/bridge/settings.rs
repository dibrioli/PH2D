//! **As settings de MUNDO da ponte** — gravidade, sub-passos, iterações,
//! frequência de contato, arrasto e sono.
//!
//! Irmão de `bridge.rs`, separado dele pelo cap de 700 LOC, e o corte é o que os
//! dois já vinham desenhando: aqui *o que este mundo É*, lá *o que ele FAZ por
//! frame*. As settings são um assunto com dono próprio (o painel `physics`) e um
//! ciclo próprio (viajam no arquivo de projeto, fora do `ProjectState`).

use super::PhysicsBridge;
use crate::PhysicsSettings;

impl PhysicsBridge {
    /// Set world gravity (m/s²). Default is `(0, -9.81)` (Y-up).
    pub fn set_gravity(&mut self, x: f32, y: f32) {
        self.set_settings(PhysicsSettings {
            gravity_x: x,
            gravity_y: y,
            ..self.settings
        });
    }

    /// The world's authored settings (what the panel paints, and what the
    /// project file stores).
    pub fn settings(&self) -> PhysicsSettings {
        self.settings
    }

    /// Replace the world's authored settings and push them into rapier.
    ///
    /// Clamps on the way in: a range that only lives in a slider is not a
    /// range, and this is also the door a loaded project file comes through.
    ///
    /// ⚠️ **Clears the checkpoint ring**, for the same reason gravity always
    /// did: every cached state was simulated under the OLD settings, so
    /// replaying from one would splice two different worlds together and
    /// publish the result as if nothing happened. Asked once, for all ten
    /// knobs, instead of per-knob — one door cannot forget one of them.
    pub fn set_settings(&mut self, settings: PhysicsSettings) {
        let settings = settings.clamped();
        if settings == self.settings {
            // Idempotent: the panel republishes every frame, and waking every
            // body (which `set_body_defaults` does) on a frame where nothing
            // changed would keep a settled stack from ever sleeping.
            return;
        }
        self.settings = settings;
        settings.apply_to(&mut self.world);
        self.ring.clear();
    }
}
