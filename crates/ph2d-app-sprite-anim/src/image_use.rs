//! ⭐ **O USO de uma imagem num objecto** ([plano GEF 13](../../../docs/GameEngineFuncional/13_isolamento_por_uso_plano.md)
//! D4/D5): parada · repetida (bloco, fundo) · animada. O Inspector mostra só os campos do uso; uma imagem
//! animada num bloco que repete é um conflito que o PH2D diz (a Unity corta em silêncio).

use crate::image_choice::ImageFacts;
use ph2d_ecs::{
    Entity, RepeatMode, ScrollRepeat, SpriteAnimations, TextureRepeat, World,
};
use ph2d_render::Sprite;

/// Para que serve a imagem neste objecto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageUse {
    /// Uma imagem (ou uma célula dela) parada.
    Still,
    /// A imagem repete-se: um bloco (*Repeat*) ou uma camada de fundo (`ScrollRepeat`).
    Repeated,
    /// As células da imagem tocam como animação.
    Animated,
}

/// O uso de `e`; `None` se não é uma sprite. Animada ganha: as animações mandam no que se desenha.
#[must_use]
pub fn image_use(world: &World, e: Entity) -> Option<ImageUse> {
    world.get::<Sprite>(e)?;
    if world.get::<SpriteAnimations>(e).is_some() {
        return Some(ImageUse::Animated);
    }
    let repete = world.get::<ScrollRepeat>(e).is_some()
        || matches!(
            world.get::<TextureRepeat>(e),
            Some(TextureRepeat(RepeatMode::Enabled | RepeatMode::Mirror))
        );
    Some(if repete {
        ImageUse::Repeated
    } else {
        ImageUse::Still
    })
}

/// O que não bate certo entre a imagem e o uso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageConflict {
    /// Uma imagem com `cells` células num objecto que a repete: o bloco mostra só uma.
    AnimatedImageRepeated { cells: u32 },
}

/// O conflito de `e` com a imagem `facts` que ele mostra.
#[must_use]
pub fn conflict(_world: &World, _e: Entity, _facts: ImageFacts) -> Option<ImageConflict> {
    None
}

/// ⭐ **Make it Animated**: o objecto deixa de repetir e toca as células da imagem em laço. `false` se não há
/// nada a fazer.
pub fn make_animated(_world: &mut World, _e: Entity, _facts: ImageFacts) -> bool {
    false
}

#[cfg(test)]
#[path = "image_use_gate_tests.rs"]
mod gate_tests;
