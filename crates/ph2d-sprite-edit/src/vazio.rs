//! O cartão vazio da Sprite: quem ainda não escolheu imagem e a imagem pintada no lugar dele.

use ph2d_ecs::{Children, Entity, SpritePixels, World};
use ph2d_render::{Sprite, SpriteSource, WHITE_TILE_KEY};

/// O quadrado branco de quem ainda não escolheu imagem (`Add ▸ Sprite`, `Static Body`)?
#[must_use]
pub fn sem_imagem(world: &World, e: Entity) -> bool {
    world.get::<SpritePixels>(e).is_none()
        && world.get::<Sprite>(e).is_some_and(
            |s| matches!(s.source, SpriteSource::Atlas { key } if key == WHITE_TILE_KEY),
        )
}

/// `e` tem filhos?
#[must_use]
pub fn tem_filhos(world: &World, e: Entity) -> bool {
    world.get::<Children>(e).is_some_and(|c| !c.is_empty())
}

/// A imagem pintada a partir do convite toma o lugar de `vazio`: a mesma posição, o mesmo nome, e o quadrado branco sai.
pub fn substitui_o_vazio(world: &mut World, vazio: Entity, nova: Entity) {
    if !sem_imagem(world, vazio) {
        return;
    }
    let posicao = world
        .get::<ph2d_ecs::Transform>(vazio)
        .map(|t| t.translation);
    if let (Some(p), Some(mut t)) = (posicao, world.get_mut::<ph2d_ecs::Transform>(nova)) {
        t.translation = p;
    }
    world.despawn(vazio);
}

#[cfg(test)]
#[path = "vazio_tests.rs"]
mod tests;
