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
        self.vivo = settings;
        settings.apply_to(&mut self.world);
        self.ring.clear();
    }

    /// O que o mundo corre agora — o autorado, ou o que a curva da Cena pôs por cima (F30 §B).
    pub fn settings_vivos(&self) -> PhysicsSettings {
        self.vivo
    }

    /// ⭐ **A curva da Cena muda o mundo VIVO** (F30 §B) — o gémeo do `retune_body`: só quando o
    /// valor muda, e ⛔ SEM limpar o anel (uma curva é função do tempo: o checkpoint de antes
    /// continua a ser deste documento). O autorado ([`Self::settings`], o que o save grava e o
    /// painel edita) não muda. Devolve `true` se o mundo mudou.
    pub fn conduzir_settings(&mut self, s: PhysicsSettings) -> bool {
        let s = s.clamped();
        if s == self.vivo {
            return false;
        }
        self.vivo = s;
        s.apply_to(&mut self.world);
        true
    }

    /// **Os pontos de retoma** de um mundo privado (o «Simular» de uma macro): um checkpoint a cada
    /// `stride` tiques, até `budget_bytes`. O documento fica no [`ph2d_physics`] por omissão
    /// (`checkpoint_stride`). Limpa o anel: os pontos guardados seguiam outra política.
    pub fn set_checkpoint_policy(&mut self, stride: u64, budget_bytes: usize) {
        self.ring.set_policy(stride, budget_bytes);
        self.ring.clear();
        self.clear_state_ring();
    }
}
