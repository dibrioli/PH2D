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
    /// Agachado (plano 09). Não é uma pose EXCLUSIVA: [`Self::of`] continua a dar a base
    /// (parado/a correr…) e esta vale ao mesmo tempo, lida de [`PlayerView::crouching`].
    Crouch,
    /// Agachado E a correr (plano 10) — o «a andar agachado».
    CrouchWalk,
}

impl CharacterPose {
    pub const ALL: [Self; 6] = [
        Self::Idle,
        Self::Run,
        Self::Jump,
        Self::Fall,
        Self::Crouch,
        Self::CrouchWalk,
    ];
    /// As 4 que o [`CharacterAnimations`] nomeia — as regras derivadas de um ficheiro antigo.
    pub const DERIVED: [Self; 4] = [Self::Idle, Self::Run, Self::Jump, Self::Fall];
    /// As do agachar, pela ordem das regras derivadas (a de cima ganha): cada uma só entra se o objecto
    /// tem a tag de [`Self::conventional_tag`].
    pub const CROUCH_DERIVED: [Self; 2] = [Self::CrouchWalk, Self::Crouch];

    /// A tag que a regra derivada desta pose toca por CONVENÇÃO — só as do agachar, que o
    /// [`CharacterAnimations`] não nomeia (as da folha de fábrica, `ph2d_render::HERO_SHEET_TAGS`).
    #[must_use]
    pub const fn conventional_tag(self) -> Option<&'static str> {
        match self {
            Self::Crouch => Some("crouch"),
            Self::CrouchWalk => Some("crouch_walk"),
            Self::Idle | Self::Run | Self::Jump | Self::Fall => None,
        }
    }

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
    /// O nome da animação deste estado; `None` para uma pose que ele não nomeia (agachado).
    #[must_use]
    pub fn tag(&self, pose: CharacterPose) -> Option<&str> {
        match pose {
            CharacterPose::Idle => Some(&self.idle),
            CharacterPose::Run => Some(&self.run),
            CharacterPose::Jump => Some(&self.jump),
            CharacterPose::Fall => Some(&self.fall),
            CharacterPose::Crouch | CharacterPose::CrouchWalk => None,
        }
    }
}

#[cfg(test)]
#[path = "character_anim_tests.rs"]
mod tests;
