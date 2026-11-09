//! ⭐ **O USO de uma imagem num objecto** (planos GEF 13/15/16): parada · repetida · animada · região. Vive aqui, por
//! baixo da família, para o canvas ([`crate::use_frame`]) e o Inspector lerem a MESMA resposta.

use ph2d_ecs::{
    Entity, ImageRegion, RepeatMode, ScrollRepeat, SpriteAnimations, SpriteAnimator, TextureRepeat,
    TiledDraw, World,
};
use ph2d_render::Sprite;

/// Para que serve a imagem neste objecto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageUse {
    /// Uma imagem (ou uma célula dela) parada.
    Still,
    /// A imagem repete-se: um bloco (*Repeat*, Tiled) ou uma camada de fundo (`ScrollRepeat`).
    Repeated,
    /// As células da imagem tocam como animação.
    Animated,
    /// Só uma área da imagem ([`ImageRegion`] + a área no [`SpriteRegion`]).
    Region,
}

/// O uso de `e`; `None` se não é uma sprite. Animada ganha (as animações mandam no que se desenha), e um
/// animador sem animações já é animada: é o que a secção Animation mostra.
#[must_use]
pub fn image_use(world: &World, e: Entity) -> Option<ImageUse> {
    world.get::<Sprite>(e)?;
    if world.get::<SpriteAnimations>(e).is_some() || world.get::<SpriteAnimator>(e).is_some() {
        return Some(ImageUse::Animated);
    }
    if world.get::<ImageRegion>(e).is_some() {
        return Some(ImageUse::Region);
    }
    let repete = world.get::<ScrollRepeat>(e).is_some()
        || world.get::<TiledDraw>(e).is_some()
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
