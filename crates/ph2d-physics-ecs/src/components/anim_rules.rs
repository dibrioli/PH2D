//! **As regras de animação** (`docs/GameEngineFuncional/08_regras_de_animacao_plano.md`): «quando ___ →
//! toca ___», por ordem — a de cima ganha.
//!
//! Componente IRMÃO de [`super::CharacterAnimations`] (o postcard é posicional; um campo novo lá subiria
//! o `PROJECT_SCHEMA`). Ausente com `CharacterAnimations` presente = as 4 regras de sempre, DERIVADAS
//! ([`AnimRules::of_character`]) — um ficheiro antigo abre e toca igual sem migrar nada.

use bevy_ecs::prelude::Component;
use ph2d_ecs::{Children, Entity, SpriteAnimations, SpriteAnimator, World};
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
        Self::of_character_with(c, |_| false)
    }

    /// As regras derivadas de um objecto cujas animações respondem a `has_tag`: no topo as do agachar
    /// ([`CharacterPose::CROUCH_DERIVED`]) cuja tag de convenção existe, depois as 4. Sem essas tags
    /// (todo ficheiro antigo) ⇒ exactamente [`Self::of_character`].
    #[must_use]
    pub fn of_character_with(c: &CharacterAnimations, has_tag: impl Fn(&str) -> bool) -> Self {
        let crouch = CharacterPose::CROUCH_DERIVED.iter().filter_map(|&p| {
            let tag = p.conventional_tag().filter(|t| has_tag(t))?;
            Some((p, tag))
        });
        let base = CharacterPose::DERIVED
            .iter()
            .filter_map(|&p| Some((p, c.tag(p)?)));
        Self(
            crouch
                .chain(base)
                .map(|(p, tag)| AnimRule {
                    when: AnimWhen::Pose(p),
                    play: tag.to_owned(),
                })
                .collect(),
        )
    }

    /// As regras que valem: as gravadas, senão as derivadas do personagem (`has_tag` = as animações do
    /// objecto que anima). `None` = sem regras.
    #[must_use]
    pub fn effective(
        rules: Option<&Self>,
        character: Option<&CharacterAnimations>,
        has_tag: impl Fn(&str) -> bool,
    ) -> Option<Self> {
        rules
            .cloned()
            .or_else(|| character.map(|c| Self::of_character_with(c, has_tag)))
    }

    /// ⭐ As regras que valem no objecto `e`: as gravadas, senão as derivadas do seu personagem com as tags
    /// do objecto que ANIMA ([`anim_target`]) — a mesma resposta para o cartão e para o jogo.
    #[must_use]
    pub fn of_entity(world: &World, e: Entity) -> Option<Self> {
        let tags = anim_target(world, e).and_then(|t| world.get::<SpriteAnimations>(t));
        Self::effective(
            world.get::<Self>(e),
            world.get::<CharacterAnimations>(e),
            |name| tags.is_some_and(|l| l.get(name).is_some()),
        )
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

/// Quem tem o animador que o objecto `e` conduz: o próprio ou, se ele não tem, o 1.º filho que tem (a imagem
/// pode ser uma peça dentro do corpo, como na Godot).
#[must_use]
pub fn anim_target(world: &World, e: Entity) -> Option<Entity> {
    if world.get::<SpriteAnimator>(e).is_some() {
        return Some(e);
    }
    world
        .get::<Children>(e)?
        .iter()
        .copied()
        .find(|c| world.get::<SpriteAnimator>(*c).is_some())
}

#[cfg(test)]
#[path = "anim_rules_tests.rs"]
mod tests;
