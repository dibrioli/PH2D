//! ⭐ **A imagem no Painter abre sem a repetição** (dono 06/11, ficha `docs/GameEngineFuncional/componentes/01_sprite.md`):
//! com o Painter em mãos, a imagem que ele edita desenha-se como UMA cópia inteira, sem ladrilhos nem corte. Ao
//! largar, volta sozinha ao que estava: a mesma lei de regresso do [`crate::skin_suspend`], nada a repor.
//!
//! ⚠️ **É uma vista, não uma edição:** publica-se no recurso [`ImagemAberta`], que os componentes autorados nunca
//! veem. O desenho lê-o no extract e em `ph2d_ecs::resolve_uv_xform`; o ponteiro e o chrome leem-no por
//! `ph2d_sprite_screen::sprite_mostrada`.

use ph2d_ecs::{Entity, ImagemAberta, ScrollRepeat, SpriteRegion, TiledDraw, UvTransform, World};
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_render::Sprite;
use ph2d_tool_painter::PainterTool;

/// **O quad de UMA cópia da imagem inteira de `e`**, à escala a que o objecto a mostra (`canvas` = o tamanho dela em
/// px). Numa grelha é uma célula: quem desenha desdobra-a. O centro fica onde o bloco tem o dele: a âncora não muda.
#[must_use]
pub fn quad_da_imagem_inteira(world: &World, e: Entity, canvas: (u32, u32)) -> Option<[f32; 2]> {
    let size = world.get::<Sprite>(e)?.size;
    if let Some(t) = world.get::<TiledDraw>(e) {
        return Some(t.tile);
    }
    let fundo = world.get::<ScrollRepeat>(e).map(|r| r.tile);
    let uv = world.get::<UvTransform>(e).map(|u| u.scale);
    let area = world.get::<SpriteRegion>(e).map(|r| r.rect);
    let px = [canvas.0, canvas.1];
    Some(std::array::from_fn(|i| {
        if let Some(t) = fundo.map(|t| t[i]).filter(|t| *t > 0.0) {
            t
        } else if let Some(k) = uv.map(|k| k[i]).filter(|k| *k > 0.0) {
            size[i] / k
        } else if let Some(w) = area.map(|r| r[2 + i]).filter(|w| *w > 0.0 && px[i] > 0) {
            size[i] * px[i] as f32 / w
        } else {
            size[i]
        }
    }))
}

/// A imagem que o Painter em mãos edita, e o tamanho dela.
fn aberta(tools: &mut ToolRegistry) -> Option<(Entity, (u32, u32))> {
    let tool = tools.active_mut()?;
    if tool.id() != ToolId::new("painter") {
        return None;
    }
    let painter = tool.as_any_mut().downcast_mut::<PainterTool>()?;
    let bits = painter.bound_document()?;
    Some((Entity::try_from_bits(bits)?, painter.canvas_size()))
}

/// ⭐ **A porta que a shell chama a cada quadro, antes do extract.** Sem Painter em mãos o recurso sai.
pub fn publica(world: &mut World, tools: &mut ToolRegistry) {
    let agora = aberta(tools).and_then(|(entity, canvas)| {
        let quad = quad_da_imagem_inteira(world, entity, canvas)?;
        Some(ImagemAberta { entity, quad })
    });
    match agora {
        Some(a) => {
            if world.get_resource::<ImagemAberta>() != Some(&a) {
                world.insert_resource(a);
            }
        }
        None => {
            world.remove_resource::<ImagemAberta>();
        }
    }
}

#[cfg(test)]
#[path = "imagem_aberta_tests.rs"]
mod tests;
