//! ADR-0113 W2 — publica o estado da ferramenta Flip (ativa? + estilo de brush)
//! do registry para o `App`, de modo que o `input_dispatch` decida capturar o
//! canvas e asse o traço **sem downcast** (o downcast concreto vive AQUI,
//! allowlistado — mesmo padrão do `vector_bridge` / `painter_bridge`).
//!
//! Roda na mesma fase do `vector_bridge` (depois do drain de ActivateTool, antes
//! do paint): uma tool recém-ativada já é vista neste frame. A tool persiste no
//! registry ativa ou não, então o estilo sobrevive a troca de ferramenta.

use ph2d_editor::{ToolId, ToolRegistry};
use ph2d_tool_flip::{FlipStyleSnapshot, FlipTool};

/// `(flip_active, style)` — `flip_active` = a Flip é a tool ATIVA; `style` = o
/// snapshot de brush/modo (presente sempre que a tool está registrada).
pub(crate) fn publish(tools: &mut ToolRegistry) -> (bool, Option<FlipStyleSnapshot>) {
    let flip_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("flip"));
    let style = tools
        .tool_by_id_mut(&ToolId::new("flip"))
        .and_then(|t| t.as_any_mut().downcast_mut::<FlipTool>())
        .map(|t| t.ui_snapshot());
    (flip_active, style)
}
