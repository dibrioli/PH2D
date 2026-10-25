//! ⭐ **A animação segue o estado do personagem** (`docs/GameEngineFuncional/05_N1_plano.md` §3) — a
//! ponte entre a vista do player ([`PhysicsBridge::player_view`]) e o animador da folha.
//!
//! Corre depois do passo da física. O animador é o do próprio personagem ou, se ele não tem, o do 1.º
//! filho que tem (a imagem pode ser uma peça dentro do corpo, como na Godot).

use ph2d_ecs::{Children, Entity, SpriteAnimator, World};
use ph2d_physics_ecs::{CharacterAnimations, CharacterPose, PhysicsBridge};
use ph2d_render::Sprite;

/// Quem tem o animador que este personagem conduz.
fn alvo(world: &World, e: Entity) -> Option<Entity> {
    if world.get::<SpriteAnimator>(e).is_some() {
        return Some(e);
    }
    world
        .get::<Children>(e)?
        .iter()
        .copied()
        .find(|c| world.get::<SpriteAnimator>(*c).is_some())
}

/// ⭐ Escreve, em cada personagem com [`CharacterAnimations`] e vista do player, a animação do estado
/// (reiniciada só quando o estado muda) e o espelho pelo lado para que ele olha.
pub fn anima_personagens(world: &mut World, physics: &PhysicsBridge) {
    let mut q = world.query::<(Entity, &CharacterAnimations)>();
    let pedidos: Vec<(Entity, String, Option<bool>)> = q
        .iter(world)
        .filter_map(|(e, anims)| {
            let view = physics.player_view(e)?;
            let pose = CharacterPose::of(view, anims.run_from);
            let flip = anims.flip.then_some(view.facing < 0.0);
            Some((alvo(world, e)?, anims.tag(pose).to_owned(), flip))
        })
        .collect();
    for (alvo, tag, flip) in pedidos {
        if let Some(mut a) = world.get_mut::<SpriteAnimator>(alvo)
            && a.current != tag
        {
            a.current = tag;
            a.elapsed_ticks = 0;
            a.pingpong_reverse = false;
            a.repeat_count = 0;
        }
        if let (Some(flip), Some(mut s)) = (flip, world.get_mut::<Sprite>(alvo))
            && s.flip_x != flip
        {
            s.flip_x = flip;
        }
    }
}

#[cfg(test)]
#[path = "personagem_tests.rs"]
mod tests;
