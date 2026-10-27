//! ⭐ **Draw Mode Tiled** ([plano GEF 14](../../../docs/GameEngineFuncional/14_tiled_com_tamanho_plano.md) D3/D5):
//! a Largura/Altura de uma sprite que repete — o collider automático vai junto. Ligá-lo é o uso «Repeated»
//! ([`crate::image_use::set_use`]).

use ph2d_ecs::{Entity, ScrollRepeat, SpriteAnimations, TiledDraw, World};
use ph2d_editor_core::InspectorDraw;
use ph2d_physics_ecs::{Collider, ColliderShape};
use ph2d_render::Sprite;

/// O menor lado (m) que a Largura/Altura aceita: um bloco de tamanho nulo não se desenha nem colide.
pub const MIN_SIDE_M: f32 = 0.01;

/// O Draw Mode que o Inspector mostra de `e`. Sem escolha numa camada de fundo (`ScrollRepeat`, outra lei) e
/// numa sprite animada.
#[must_use]
pub fn draw_of(world: &World, e: Entity) -> InspectorDraw {
    let Some(s) = world.get::<Sprite>(e) else {
        return InspectorDraw::Fixed;
    };
    if world.get::<ScrollRepeat>(e).is_some() || world.get::<SpriteAnimations>(e).is_some() {
        return InspectorDraw::Fixed;
    }
    match world.get::<TiledDraw>(e) {
        Some(t) => InspectorDraw::Tiled {
            copies: std::array::from_fn(|i| {
                if t.tile[i] > 0.0 {
                    s.size[i] / t.tile[i]
                } else {
                    1.0
                }
            }),
        },
        None => InspectorDraw::Simple,
    }
}

/// Muda um eixo (`0` = Width, `1` = Height) de uma sprite Tiled para `copias` cópias da imagem (dono 27/10: o
/// Inspector conta cópias, 1 × 1 = a imagem inteira uma vez). Um collider em caixa que media o desenho (o
/// automático) segue-o; um que o artista mudou à mão fica. `false` se nada mudou.
pub fn set_tiled_side(world: &mut World, e: Entity, axis: usize, copias: f32) -> bool {
    let Some(tile) = world.get::<TiledDraw>(e).map(|t| t.tile[axis.min(1)]) else {
        return false;
    };
    if axis > 1 || !copias.is_finite() {
        return false;
    }
    let Some(mut s) = world.get::<Sprite>(e).copied() else {
        return false;
    };
    let novo = (copias * tile).max(MIN_SIDE_M);
    let antes = s.size;
    if antes[axis] == novo {
        return false;
    }
    s.size[axis] = novo;
    world.entity_mut(e).insert(s);
    follow_auto_collider(world, e, antes, s.size);
    true
}

/// O collider em caixa que media o desenho de `e` com o tamanho `antes` (o automático) passa a medir `depois`; um
/// que o artista mudou à mão fica.
pub(crate) fn follow_auto_collider(
    world: &mut World,
    e: Entity,
    antes: [f32; 2],
    depois: [f32; 2],
) {
    if let Some(mut c) = world.get_mut::<Collider>(e)
        && let ColliderShape::Cuboid { half_x, half_y } = c.shape
        && same(half_x, antes[0] * 0.5)
        && same(half_y, antes[1] * 0.5)
        && antes != depois
    {
        c.shape = ColliderShape::Cuboid {
            half_x: depois[0] * 0.5,
            half_y: depois[1] * 0.5,
        };
    }
}

/// Iguais à precisão de um `f32` que passou por um campo de texto.
fn same(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-4 * b.abs().max(1.0)
}

#[cfg(test)]
#[path = "tiled_tests.rs"]
mod tests;
