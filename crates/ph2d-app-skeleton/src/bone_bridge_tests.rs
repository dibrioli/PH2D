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

/// ⭐⭐ **Dono 01/11: escolher um osso existente abre o *Transform*, nunca *Create*/*Weight*.**
/// O osso que o *Create* acabou de fazer não tira o artista do *Create*; e fora da mão a
/// ferramenta não é armada (o Object fica Object, A14).
#[test]
fn choosing_an_existing_bone_opens_transform_but_a_newborn_keeps_create() {
    for de in [BoneAction::Create, BoneAction::Weight] {
        let mut tools = registo();
        assert!(arm(&mut tools, de));
        let mut m = crate::state::SkeletonState::default();
        assert!(on_focus(&mut tools, &mut m, Some(7)));
        assert_eq!(
            config(&mut tools).action,
            BoneAction::Transform,
            "de {de:?}"
        );
    }
    // O recém-nascido: o Create fica.
    let mut tools = registo();
    assert!(arm(&mut tools, BoneAction::Create));
    let mut m = crate::state::SkeletonState::default();
    crate::reveal::on_birth(&mut m.osso_revelado, 9);
    assert!(!on_focus(&mut tools, &mut m, Some(9)));
    assert_eq!(config(&mut tools).action, BoneAction::Create, "o osso novo");
    // ⛔ Fora da mão: nada é armado.
    let mut tools = registo();
    let mut m = crate::state::SkeletonState::default();
    assert!(on_focus(&mut tools, &mut m, Some(7)));
    assert!(
        !in_hand(&tools),
        "escolher um osso não põe a ferramenta na mão"
    );
}
