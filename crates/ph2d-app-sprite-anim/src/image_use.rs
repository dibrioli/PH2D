//! ⭐ **O USO de uma imagem num objecto** ([plano GEF 13](../../../docs/GameEngineFuncional/13_isolamento_por_uso_plano.md)
//! D4/D5): parada · repetida (bloco, fundo) · animada. O Inspector mostra só os campos do uso; uma imagem
//! animada num bloco que repete é um conflito que o PH2D diz (a Unity corta em silêncio).

use crate::image_choice::ImageFacts;
use ph2d_ecs::{
    AnimationTag, Entity, RepeatMode, ScrollRepeat, SpriteAnimations, SpriteAnimator, SpriteGrid,
    TextureRepeat, UvTransform, World,
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

/// Quantas células tem a imagem.
#[must_use]
pub fn cells(facts: ImageFacts) -> u32 {
    facts.slicing.map_or(1, |s| s.grid[0].max(1) * s.grid[1].max(1))
}

/// O conflito de `e` com a imagem `facts` que ele mostra.
#[must_use]
pub fn conflict(world: &World, e: Entity, facts: ImageFacts) -> Option<ImageConflict> {
    let n = cells(facts);
    (n > 1 && image_use(world, e) == Some(ImageUse::Repeated))
        .then_some(ImageConflict::AnimatedImageRepeated { cells: n })
}

/// O nome da animação que [`make_animated`] cria.
pub const LOOP_TAG: &str = "loop";

/// ⭐ **Make it Animated** (plano GEF 13 D5): o objecto deixa de repetir e toca todas as células da imagem em
/// laço. O tamanho passa a uma célula, à escala de UM ladrilho de antes pela largura (a imagem não se deforma).
/// `false` se a imagem tem uma célula só, ou o objecto já anima.
pub fn make_animated(world: &mut World, e: Entity, facts: ImageFacts) -> bool {
    let Some(slicing) = facts.slicing else {
        return false;
    };
    let n = cells(facts);
    if n < 2 || image_use(world, e).is_none_or(|u| u == ImageUse::Animated) {
        return false;
    }
    let Some(sprite) = world.get::<Sprite>(e).copied() else {
        return false;
    };
    let ladrilho = match (world.get::<ScrollRepeat>(e), world.get::<UvTransform>(e)) {
        (Some(r), _) if r.tile[0] > 0.0 => r.tile[0],
        (_, Some(uv)) if uv.scale[0] > 0.0 => sprite.size[0] / uv.scale[0],
        _ => sprite.size[0],
    };
    let celula = [
        facts.native_m[0] / slicing.grid[0].max(1) as f32,
        facts.native_m[1] / slicing.grid[1].max(1) as f32,
    ];
    let escala = if celula[0] > 0.0 { ladrilho / celula[0] } else { 1.0 };
    let mut tags = SpriteAnimations::new();
    if tags.insert(AnimationTag::new(LOOP_TAG, 0, n - 1)).is_err() {
        return false;
    }
    let mut animator = SpriteAnimator::new(LOOP_TAG);
    animator.autoplay = true;
    let mut ent = world.entity_mut(e);
    ent.remove::<(TextureRepeat, UvTransform, ScrollRepeat)>();
    ent.insert((
        Sprite {
            size: [celula[0] * escala, celula[1] * escala],
            ..sprite
        },
        SpriteGrid {
            hframes: slicing.grid[0].max(1),
            vframes: slicing.grid[1].max(1),
            frame: 0,
        },
        tags,
        animator,
    ));
    true
}

#[cfg(test)]
#[path = "image_use_gate_tests.rs"]
mod gate_tests;
