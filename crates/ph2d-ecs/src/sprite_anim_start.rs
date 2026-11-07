//! **Com qual animação o objecto começa** ([ficha](../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md),
//! decisão 2). Separado do `current`: o ▶ do cartão é prévia e muda o `current`, nunca isto.
//!
//! Irmão de [`crate::SpriteAnimator`] pelo postcard posicional: projeto antigo não tem o blob e começa no `current`.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::{SimComponent, SpriteAnimations, SpriteAnimator};

/// O nome da animação com que o jogo e o abrir do projeto começam (só em «Auto»).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAnimStart(pub String);

impl SimComponent for SpriteAnimStart {}

impl SpriteAnimStart {
    /// A animação que vale: a escolhida, se a lista ainda a tem; senão o `current`.
    #[must_use]
    pub fn ou_current<'a>(
        start: Option<&'a Self>,
        animator: &'a SpriteAnimator,
        lib: &SpriteAnimations,
    ) -> &'a str {
        start
            .map(|s| s.0.as_str())
            .filter(|n| lib.get(n).is_some())
            .unwrap_or(&animator.current)
    }
}
