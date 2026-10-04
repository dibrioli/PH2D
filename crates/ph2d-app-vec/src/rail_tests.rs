//! Gates dos botões de criar no trilho — o lado do vetor (o clique e o aceso, com a ferramenta de
//! verdade); o despacho tem os seus em `ph2d_editor_core::…::chrome::rail_vector_tools`.

use super::*;
use ph2d_editor_core::ToolId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::widget::ButtonState;
use ph2d_tool_vector::VectorTool;

fn hero() -> HeroScreen {
    HeroScreen::new(NodeId(1))
}

fn mode(tools: &mut ToolRegistry) -> Option<DrawMode> {
    crate::vector_mode::tool_mut(tools).map(|t| t.mode())
}

fn lit(hero: &HeroScreen) -> Vec<NodeId> {
    ids::VECTOR_RAIL_TOOL_IDS
        .into_iter()
        .filter(|id| {
            matches!(
                hero.store.get(*id),
                Some(InteractiveState::Button {
                    state: ButtonState::Pressed
                })
            )
        })
        .collect()
}

/// ⭐ GATE — a tabela é uma bijecção com as ferramentas de criar, e todas são do modo Object.
#[test]
fn every_rail_button_is_one_create_tool() {
    for (id, m) in RAIL {
        assert_eq!(mode_of(id), Some(m));
        assert_eq!(button_of(m), Some(id));
        assert_eq!(
            m.object_mode(),
            ph2d_editor_core::object_mode::ObjectMode::Object,
            "{m:?} no trilho de Object"
        );
    }
    assert_eq!(button_of(DrawMode::Node), None);
}

/// ⭐⭐ GATE — **o clique real até à mão**: o clique deixa o pick; enquanto a ferramenta não está na
/// mão ele ESPERA (não se perde); com ela na mão vira o modo, e o botão acende. Trocar de modo por
/// outra porta apaga-o.
#[test]
fn a_rail_click_puts_the_create_tool_in_hand_and_lights_it() {
    let mut hero = hero();
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(VectorTool::default()));
    hero.apply_event(WidgetEvent::Click(ids::VECTOR_RAIL_PENCIL));
    drive(&mut hero, &mut tools);
    assert_eq!(
        mode(&mut tools),
        None,
        "a ferramenta ainda não chegou à mão"
    );
    assert!(tools.set_active(&ToolId::new("vector")));
    drive(&mut hero, &mut tools);
    assert_eq!(mode(&mut tools), Some(DrawMode::Pencil), "o pick perdeu-se");
    assert_eq!(lit(&hero), vec![ids::VECTOR_RAIL_PENCIL]);
    crate::vector_mode::tool_mut(&mut tools)
        .expect("na mão")
        .set_mode(DrawMode::Node);
    drive(&mut hero, &mut tools);
    assert!(
        lit(&hero).is_empty(),
        "o trilho acende uma ferramenta que a mão não tem"
    );
}
