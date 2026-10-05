//! Os gates da ponte da ferramenta de osso (A14).

use super::*;
use ph2d_editor_core::floating_panel::FloatingPanel;
use ph2d_editor_core::tool::Tool;

struct Move;

impl Tool for Move {
    fn id(&self) -> ToolId {
        ToolId::new("move")
    }
    fn label(&self) -> &str {
        "move"
    }
    fn icon_slug(&self) -> &str {
        "move"
    }
    fn build_panel(&self) -> FloatingPanel {
        FloatingPanel::new(self.id(), "move")
    }
    fn is_default(&self) -> bool {
        true
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn registo() -> ToolRegistry {
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(ph2d_tool_bone::make());
    tools.activate_default();
    tools
}

/// ⭐⭐ **Armar o verbo PÕE a ferramenta de osso na mão** — com outra na mão, o clique do painel
/// morria nela (o defeito do dono: fora do Edit do vetor os ossos não se mexiam). Controlo: antes
/// do `arm` a mão é a de omissão.
#[test]
fn arming_a_verb_puts_the_bone_tool_in_hand() {
    let mut tools = registo();
    assert!(
        !in_hand(&tools),
        "controlo: nasce com a ferramenta de omissão"
    );
    assert!(arm(&mut tools, BoneAction::Weight));
    assert!(in_hand(&tools));
    assert_eq!(config(&mut tools).action, BoneAction::Weight);
    // O estado fica com a ferramenta fora da mão (o painel pinta o pincel dela).
    tools.activate_default();
    assert!(!in_hand(&tools));
    assert_eq!(config(&mut tools).action, BoneAction::Weight);
}

/// Sem a ferramenta registada o `arm` diz que não, e a mão não muda.
#[test]
fn arming_without_the_tool_registered_says_no() {
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.activate_default();
    assert!(!arm(&mut tools, BoneAction::Create));
    assert!(!in_hand(&tools));
}
