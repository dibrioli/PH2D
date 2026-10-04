//! ⭐ **As ferramentas de CRIAR no trilho da esquerda** (spec/06 F3 ▸ Vector; escolha do dono,
//! 03/10: o pill VECTOR saiu, e em Object o trilho dá Caneta · Lápis · Formas · Texto).
//!
//! O trilho mora na fundação, que não conhece o vetor: o clique pede a ferramenta (`ActivateTool`)
//! e deixa o id no canal de pick; aqui, com a ferramenta já na mão, ele vira o `DrawMode`.
//! ⚠️ Não por `PanelEvent` no mesmo clique: o `ActivateTool` só se aplica depois do dreno, e o
//! evento cairia na ferramenta que estava na mão.

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::{NodeId, ToolRegistry};
use ph2d_tool_vector::DrawMode;

/// Cada botão do trilho e a ferramenta que ele põe na mão — a tabela única dos dois sentidos.
pub const RAIL: [(NodeId, DrawMode); 4] = [
    (ids::VECTOR_RAIL_PEN, DrawMode::Pen),
    (ids::VECTOR_RAIL_PENCIL, DrawMode::Pencil),
    (ids::VECTOR_RAIL_SHAPE, DrawMode::Shape),
    (ids::VECTOR_RAIL_TEXT, DrawMode::Text),
];

/// A ferramenta de um botão do trilho.
#[must_use]
pub fn mode_of(id: NodeId) -> Option<DrawMode> {
    RAIL.iter().find(|(r, _)| *r == id).map(|(_, m)| *m)
}

/// O botão que acende com `mode` na mão.
#[must_use]
pub fn button_of(mode: DrawMode) -> Option<NodeId> {
    RAIL.iter().find(|(_, m)| *m == mode).map(|(id, _)| *id)
}

/// ⭐ **Em todo quadro:** o clique pendente vira o modo da ferramenta na mão, e o botão aceso é
/// DERIVADO do modo (outra porta — o painel, um atalho — pode tê-lo trocado).
pub fn drive(hero: &mut HeroScreen, tools: &mut ToolRegistry) {
    let mut tool = crate::vector_mode::tool_mut(tools);
    if let Some(t) = tool.as_deref_mut()
        && let Some(m) = hero
            .store
            .take_command_pick_if(|id| mode_of(id).is_some())
            .and_then(mode_of)
    {
        t.set_mode(m);
    }
    let pressed = tool.map(|t| t.mode()).and_then(button_of);
    ph2d_editor_core::screens::hero::chrome::rail_vector_sync(&mut hero.store, pressed);
}

#[cfg(test)]
#[path = "rail_tests.rs"]
mod tests;
