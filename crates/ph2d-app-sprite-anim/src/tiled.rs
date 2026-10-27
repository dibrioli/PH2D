//! ⭐ **Draw Mode Tiled** ([plano GEF 14](../../../docs/GameEngineFuncional/14_tiled_com_tamanho_plano.md) D3/D5):
//! ligar/desligar o `TiledDraw` e mudar a Largura/Altura de uma sprite — o collider automático vai junto.

use crate::image_choice::ImageFacts;
use ph2d_ecs::{
    Entity, ScrollRepeat, SpriteAnimations, SpriteGrid, TextureRepeat, TiledDraw, UvTransform,
    World,
};
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
        Some(_) => InspectorDraw::Tiled { size: s.size },
        None => InspectorDraw::Simple,
    }
}

/// Liga (`on`) ou desliga o Tiled. Ligar: o ladrilho é o que o objecto mostra da imagem `facts`, nativo (sem factos, o
/// tamanho de agora), o tamanho fica, e o *Repeat*/UV à mão saem (o Tiled manda nas repetições). Desligar: a
/// imagem estica até ao tamanho. `false` se nada mudou.
pub fn set_tiled(world: &mut World, e: Entity, facts: Option<ImageFacts>, on: bool) -> bool {
    if draw_of(world, e) == InspectorDraw::Fixed {
        return false;
    }
    let Some(size) = world.get::<Sprite>(e).map(|s| s.size) else {
        return false;
    };
    let tem = world.get::<TiledDraw>(e).is_some();
    if tem == on {
        return false;
    }
    let mut ent = world.entity_mut(e);
    if on {
        // O ladrilho é o que o objecto MOSTRA (a grelha dele), nunca a célula do corte da imagem: uma parada
        // inteira repete a imagem inteira (dono 27/10, «ficou achatada»).
        let mostra = ent.get::<SpriteGrid>().map(|g| [g.hframes, g.vframes]);
        let tile = facts.map_or(size, |f| {
            let g = mostra.unwrap_or([1, 1]);
            [
                f.native_m[0] / g[0].max(1) as f32,
                f.native_m[1] / g[1].max(1) as f32,
            ]
        });
        ent.remove::<(TextureRepeat, UvTransform)>();
        ent.insert(TiledDraw { tile });
    } else {
        ent.remove::<TiledDraw>();
    }
    true
}

/// Muda um eixo (`0` = Width, `1` = Height) do tamanho de uma sprite Tiled para `m` metros. Um collider em
/// caixa que media o desenho (o automático) segue-o; um que o artista mudou à mão fica. `false` se nada mudou.
pub fn set_tiled_side(world: &mut World, e: Entity, axis: usize, m: f32) -> bool {
    if axis > 1 || !m.is_finite() || world.get::<TiledDraw>(e).is_none() {
        return false;
    }
    let Some(mut s) = world.get::<Sprite>(e).copied() else {
        return false;
    };
    let novo = m.max(MIN_SIDE_M);
    let antes = s.size;
    if antes[axis] == novo {
        return false;
    }
    s.size[axis] = novo;
    let mut ent = world.entity_mut(e);
    ent.insert(s);
    if let Some(mut c) = ent.get_mut::<Collider>()
        && let ColliderShape::Cuboid { half_x, half_y } = c.shape
        && same(half_x, antes[0] * 0.5)
        && same(half_y, antes[1] * 0.5)
    {
        c.shape = ColliderShape::Cuboid {
            half_x: s.size[0] * 0.5,
            half_y: s.size[1] * 0.5,
        };
    }
    true
}

/// Iguais à precisão de um `f32` que passou por um campo de texto.
fn same(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-4 * b.abs().max(1.0)
}

#[cfg(test)]
#[path = "tiled_tests.rs"]
mod tests;
