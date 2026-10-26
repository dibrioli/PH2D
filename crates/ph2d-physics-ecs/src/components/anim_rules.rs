//! **As regras de animação** (`docs/GameEngineFuncional/08_regras_de_animacao_plano.md`): «quando ___ →
//! toca ___», por ordem — a de cima ganha.
//!
//! Componente IRMÃO de [`super::CharacterAnimations`] (o postcard é posicional; um campo novo lá subiria
//! o `PROJECT_SCHEMA`). Ausente com `CharacterAnimations` presente = as 4 regras de sempre, DERIVADAS
//! ([`AnimRules::of_character`]) — um ficheiro antigo abre e toca igual sem migrar nada.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use super::{CharacterAnimations, CharacterPose};
use crate::HealthEventKind;

/// Um facto de vida DESTE objecto, lido do `HealthEvent` com `target == self`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeEvent {
    Damaged,
    Died,
    Healed,
    Dodged,
}

impl LifeEvent {
    pub const ALL: [Self; 4] = [Self::Damaged, Self::Died, Self::Healed, Self::Dodged];

    /// O facto de um `HealthEventKind` (um golpe que só tocou o escudo não é dano na vida).
    #[must_use]
    pub fn of(kind: HealthEventKind) -> Option<Self> {
        Some(match kind {
            HealthEventKind::Damaged { .. } => Self::Damaged,
            HealthEventKind::Died => Self::Died,
            HealthEventKind::Healed { .. } => Self::Healed,
            HealthEventKind::Dodged => Self::Dodged,
            HealthEventKind::Shielded { .. } => return None,
        })
    }
}

/// **A condição de uma regra.** ⚠️ Append-only (postcard grava o índice da variante).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimWhen {
    /// O estado do personagem (só com `PlatformPlayer`).
    Pose(CharacterPose),
    /// Uma ação do mapa de teclas premida (o NOME da ação, ex. `"down"`).
    Action(String),
    /// Um facto de vida deste objecto, neste quadro.
    Life(LifeEvent),
    /// Um sinal com este nome ouvido neste quadro.
    Signal(String),
}

impl AnimWhen {
    /// Um facto de um quadro só: a regra SEGURA a sua animação até ela fechar uma volta.
    #[must_use]
    pub fn is_instant(&self) -> bool {
        matches!(self, Self::Life(_) | Self::Signal(_))
    }
}

/// Uma regra: quando `when` vale, toca a animação `play` (o nome da tag).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimRule {
    pub when: AnimWhen,
    pub play: String,
}

/// **A lista de regras, por ordem: a de cima ganha.**
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimRules(pub Vec<AnimRule>);

impl AnimRules {
    /// As 4 regras de um `CharacterAnimations` (parado · a correr · a subir · a cair).
    #[must_use]
    pub fn of_character(c: &CharacterAnimations) -> Self {
        Self(
            CharacterPose::DERIVED
                .iter()
                .filter_map(|&p| {
                    Some(AnimRule {
                        when: AnimWhen::Pose(p),
                        play: c.tag(p)?.to_owned(),
                    })
                })
                .collect(),
        )
    }

    /// As regras que valem: as gravadas, senão as derivadas do personagem. `None` = sem regras.
    #[must_use]
    pub fn effective(
        rules: Option<&Self>,
        character: Option<&CharacterAnimations>,
    ) -> Option<Self> {
        rules.cloned().or_else(|| character.map(Self::of_character))
    }

    /// Renomear uma animação leva as regras que a tocam. Devolve se alguma mudou.
    pub fn rename_play(&mut self, old: &str, new: &str) -> bool {
        let mut moved = false;
        for r in &mut self.0 {
            if r.play == old {
                new.clone_into(&mut r.play);
                moved = true;
            }
        }
        moved
    }
}

#[cfg(test)]
#[path = "anim_rules_tests.rs"]
mod tests;
