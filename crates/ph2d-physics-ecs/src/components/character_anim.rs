//! **As animações do personagem** (`docs/GameEngineFuncional/05_N1_plano.md` §3) — que animação toca
//! em cada estado do [`super::PlatformPlayer`]: parado, a correr, a subir num salto, a cair.
//!
//! Só CONFIG (os nomes das tags e o limiar); o estado lê-se da vista do player a cada quadro
//! ([`crate::PhysicsBridge::player_view`]) e quem escreve o animador é a ponte da família do jogo.
//!
//! ⚠️ **Componente registado NOVO ⇒ `PROJECT_SCHEMA` sobe** (e os dois espelhos do registo).

use bevy_ecs::prelude::Component;
use ph2d_platformer::{FootingKind, PlayerView};
use serde::{Deserialize, Serialize};

/// **Que animação toca em cada estado.** Um nome que a lista de animações não tem não toca — o
/// animador fica sem tag e o Inspector di-lo.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharacterAnimations {
    /// No chão, quieto.
    pub idle: String,
    /// No chão, a andar.
    pub run: String,
    /// No ar, a subir.
    pub jump: String,
    /// No ar, a descer.
    pub fall: String,
    /// A partir de que velocidade (m/s, relativa ao chão) andar no chão é correr.
    pub run_from: f32,
    /// Espelha a imagem quando o personagem olha para a esquerda.
    pub flip: bool,
}

impl Default for CharacterAnimations {
    fn default() -> Self {
        Self {
            idle: "idle".into(),
            run: "run".into(),
            jump: "jump".into(),
            fall: "fall".into(),
            run_from: 0.5,
            flip: true,
        }
    }
}

/// O estado do personagem que escolhe a animação. ⚠️ Gravado nas `AnimRules` pelo índice: append-only.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterPose {
    Idle,
    Run,
    Jump,
    Fall,
}

impl CharacterPose {
    pub const ALL: [Self; 4] = [Self::Idle, Self::Run, Self::Jump, Self::Fall];

    /// A lei: no chão corre se a velocidade relativa ao chão passa de `run_from`; no ar (ou numa
    /// rampa íngreme demais para ficar de pé) sobe = salta, desce = cai.
    #[must_use]
    pub fn of(view: &PlayerView, run_from: f32) -> Self {
        match view.footing {
            FootingKind::Ground => {
                let rel = view.velocity[0] - view.ground_velocity[0];
                if rel.abs() >= run_from.max(0.0) {
                    Self::Run
                } else {
                    Self::Idle
                }
            }
            FootingKind::Airborne | FootingKind::Steep => {
                if view.velocity[1] > 0.0 {
                    Self::Jump
                } else {
                    Self::Fall
                }
            }
        }
    }
}

impl CharacterAnimations {
    /// O nome da animação deste estado.
    #[must_use]
    pub fn tag(&self, pose: CharacterPose) -> &str {
        match pose {
            CharacterPose::Idle => &self.idle,
            CharacterPose::Run => &self.run,
            CharacterPose::Jump => &self.jump,
            CharacterPose::Fall => &self.fall,
        }
    }
}

#[cfg(test)]
#[path = "character_anim_tests.rs"]
mod tests;
