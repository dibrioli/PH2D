//! Gates do despacho dos botões de criar forma vetorial — o módulo FILHO de `rail_vector_tools.rs`,
//! num DIRETÓRIO pela razão do `rail_painter_tools/tests.rs`.

use super::*;
use crate::action_bus::EditorAction;

fn pressed(hero: &HeroScreen) -> Vec<NodeId> {
    ids::VECTOR_RAIL_TOOL_IDS
        .into_iter()
        .filter(|id| matches!(hero.store.button_state(*id), Some(ButtonState::Pressed)))
        .collect()
}

/// ⭐⭐ GATE — o clique pede a ferramenta do vetor e deixa o id no canal de pick (é o vetor que o
/// lê, com a ferramenta já na mão); um id alheio não é tomado.
#[test]
fn a_rail_click_asks_for_the_vector_tool_and_leaves_its_pick() {
    for id in ids::VECTOR_RAIL_TOOL_IDS {
        let mut hero = HeroScreen::new(NodeId(1));
        super::super::super::left_rail::populate(&mut hero.store);
        assert!(apply(&mut hero, WidgetEvent::Click(id)));
        let asked = hero
            .bus
            .drain()
            .any(|a| matches!(a, EditorAction::ActivateTool { tool_id } if tool_id == "vector"));
        assert!(asked, "o botão {id:?} não pediu a ferramenta");
        assert_eq!(hero.store.take_command_pick(), Some(id));
    }
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(!apply(&mut hero, WidgetEvent::Click(ids::TOOL_TRANSLATE)));
}

/// ⭐⭐ GATE — o aceso é DERIVADO: um só, o pedido; `None` apaga todos.
#[test]
fn the_lit_button_is_the_one_the_hand_holds() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    assert!(pressed(&hero).is_empty(), "nasce com a caneta acesa");
    sync(&mut hero.store, Some(ids::VECTOR_RAIL_SHAPE));
    assert_eq!(pressed(&hero), vec![ids::VECTOR_RAIL_SHAPE]);
    assert!(any_pressed(&hero.store));
    sync(&mut hero.store, Some(ids::VECTOR_RAIL_PEN));
    assert_eq!(pressed(&hero), vec![ids::VECTOR_RAIL_PEN]);
    sync(&mut hero.store, None);
    assert!(pressed(&hero).is_empty());
}

/// ⭐⭐ GATE — Mover/rodar/escalar com a caneta do trilho na mão larga-a (o botão diz o que faz);
/// sem ela, não pede nada.
#[test]
fn a_transform_click_drops_the_rail_pen() {
    let mut hero = HeroScreen::new(NodeId(1));
    super::super::super::left_rail::populate(&mut hero.store);
    let cancels = |hero: &mut HeroScreen| {
        assert!(super::super::rail_tools::apply(
            hero,
            WidgetEvent::Click(ids::TOOL_ROTATE)
        ));
        hero.bus
            .drain()
            .any(|a| matches!(a, EditorAction::CancelActiveTool))
    };
    assert!(
        !cancels(&mut hero),
        "sem caneta na mão, o Rodar largou a ferramenta"
    );
    sync(&mut hero.store, Some(ids::VECTOR_RAIL_PEN));
    assert!(cancels(&mut hero), "a caneta ficou na mão depois do Rodar");
}
