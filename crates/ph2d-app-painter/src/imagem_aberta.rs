//! ⭐ **A imagem no Painter abre sem a repetição** (dono 06/11, ficha `docs/GameEngineFuncional/componentes/01_sprite.md`):
//! a lei é de `ph2d_sprite_screen::imagem_aberta`; aqui só se responde a quem o Painter em mãos edita.

use ph2d_ecs::{Entity, World};
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_painter::PainterTool;

/// ⭐ **A porta que a shell chama a cada quadro, antes do extract.** Sem Painter em mãos, a vista sai sozinha.
pub fn publica(world: &mut World, tools: &mut ToolRegistry) {
    let aberta = tools
        .active_mut()
        .filter(|t| t.id() == ToolId::new("painter"))
        .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        .and_then(|p| Some((Entity::try_from_bits(p.bound_document()?)?, p.canvas_size())));
    ph2d_sprite_screen::imagem_aberta::publica(world, aberta);
}

/// ⭐ **A folha visível mudou com o Painter ligado?** Uma vez por quadro, antes do bind; a lei é
/// [`ph2d_sprite_screen::imagem_aberta::troca`] (auditoria 08/11).
pub fn folha_trocou(world: &World, tools: &mut ToolRegistry) -> Option<(u64, Option<u8>)> {
    let ligada = tools
        .active_mut()
        .filter(|t| t.id() == ToolId::new("painter"))
        .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        .and_then(|p| p.bound_document());
    ph2d_sprite_screen::imagem_aberta::troca(world, ligada)
}
