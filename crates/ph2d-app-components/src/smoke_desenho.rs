//! O desenho ajustado ao corpo, para as cenas de smoke.
//!
//! Um corpo que gira no sítio para encarar o movimento (herói com `RotationMode::ToMovement`,
//! projétil: `ProjectileLaw::face_velocity` é TRUE por omissão) tem de ser invariante à rotação —
//! uma bola: uma caixa que gira no sítio entra na parede sem nenhum teste de colisão. E uma bola
//! sob um quadrado tem os cantos 41 % fora. Logo o CORPO não muda; o desenho encaixa-se nele:
//! um disco, e (onde o rumo importa) um filho «Rumo» que marca a frente.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::{ChildOf, Name, Transform, Visibility};
use ph2d_render::{DISC_TILE_KEY, Sprite, WHITE_TILE_KEY};
use ph2d_core::Vec2;

/// Um disco de raio `raio` (diâmetro `2·raio`).
pub(crate) fn disco(raio: f32, rgba: [f32; 4]) -> Sprite {
    Sprite::atlas(DISC_TILE_KEY, [2.0 * raio, 2.0 * raio], rgba)
}

/// O filho que marca para onde o corpo olha (+X local). O canto mais longe fica a `0,982·raio`:
/// dentro do disco.
pub(crate) fn rumo(world: &mut World, pai: Entity, raio: f32, rgba: [f32; 4]) -> Entity {
    let escuro = [rgba[0] * 0.55, rgba[1] * 0.55, rgba[2] * 0.55, rgba[3]];
    world
        .spawn((
            Name::new("Rumo"),
            Transform::from_translation(Vec2::new(0.35 * raio, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.2 * raio, 0.5 * raio], escuro),
            Visibility::visible(),
            ChildOf(pai),
        ))
        .id()
}
