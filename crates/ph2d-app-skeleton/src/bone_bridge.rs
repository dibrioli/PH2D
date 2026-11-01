//! ⭐ **A ponte da ferramenta de osso** — pôr a ferramenta na mão e ler o estado dela (A14). O
//! downcast fica confinado aqui, como no `ph2d_app_vec::vector_bridge`.

use ph2d_editor_core::floating_panel::ToolId;
use ph2d_editor_core::tool::ToolRegistry;
use ph2d_tool_bone::{BONE, BoneAction, BoneConfig, BoneTool};

fn tool(tools: &mut ToolRegistry) -> Option<&mut BoneTool> {
    tools
        .tool_by_id_mut(&ToolId::new(BONE))
        .and_then(|t| t.as_any_mut().downcast_mut::<BoneTool>())
}

/// A ferramenta de osso está na mão?
#[must_use]
pub fn in_hand(tools: &ToolRegistry) -> bool {
    tools.active().is_some_and(|t| t.id() == ToolId::new(BONE))
}

/// O estado da ferramenta — guardado mesmo fora da mão (o painel pinta o pincel de peso dela).
#[must_use]
pub fn config(tools: &mut ToolRegistry) -> BoneConfig {
    tool(tools).map(|t| t.config()).unwrap_or_default()
}

/// ⭐⭐ **Põe a ferramenta de osso na mão com `action` armado** — a porta ÚNICA de quem arma o verbo
/// de fora do painel (a aresta do foco, as cenas). Devolve `false` se a ferramenta não está
/// registada.
pub fn arm(tools: &mut ToolRegistry, action: BoneAction) -> bool {
    if !in_hand(tools) && !tools.set_active(&ToolId::new(BONE)) {
        return false;
    }
    tool(tools).map(|t| t.set_action(action)).is_some()
}

/// ⭐⭐ **Escolher um osso JÁ EXISTENTE abre o *Transform*** (dono 01/11): na aresta do foco
/// ([`crate::reveal::on_focus`]) com a ferramenta de osso na mão, o verbo passa a *Transform*. O osso
/// acabado de nascer não conta ([`crate::reveal::on_birth`] absorve a aresta) e a ferramenta fora
/// da mão não é armada (A14: o Object fica Object). Devolve se houve aresta.
pub fn on_focus(
    tools: &mut ToolRegistry,
    st: &mut crate::state::SkeletonState,
    foco: Option<u64>,
) -> bool {
    let novo = crate::reveal::on_focus(&mut st.osso_revelado, foco);
    if novo && in_hand(tools) && config(tools).action != BoneAction::Transform {
        let _ = arm(tools, BoneAction::Transform);
    }
    novo
}

#[cfg(test)]
#[path = "bone_bridge_tests.rs"]
mod tests;
