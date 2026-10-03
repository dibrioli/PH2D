//! ⭐⭐ **As abas da peça pedem o MODO, com o clique de verdade** (spec/06 F3; report do dono 29/09).
//!
//! ⚠️ Mora na shell e não ao lado do seletor (`ph2d-panel-registry-init`): os painéis do Painter e da
//! escultura são features que só a SHELL liga, e sem eles a aba não existe no registo.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
use ph2d_editor_core::{HeroScreen, NodeId};

/// ⭐⭐ **A aba do Painter ao lado da escultura, em Sculpt, pede o Paint** (report do dono, 29/09,
/// pela porta do modo — spec/06 F3) — o clique de verdade na ABA, com todos os painéis registados.
/// ⚠️ O CONTROLO é a mesma aba em Object: ela só levanta o painel.
#[test]
fn the_painter_tab_beside_the_sculpt_asks_for_paint() {
    use ph2d_editor_core::screens::hero::slot_tabs::tab_node_id;
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    let tab = tab_node_id(ph2d_editor_core::ids::PAINTER_LAYERS_PANEL);
    let modes = [ObjectMode::Sculpt, ObjectMode::Paint];
    let pede = |hero: &mut HeroScreen| -> Vec<ModeRequest> {
        let _ = hero.bus.drain().count();
        assert!(
            hero.apply_event(WidgetEvent::Click(tab)),
            "a aba não tomou o clique"
        );
        hero.bus
            .drain()
            .filter_map(|a| match a {
                EditorAction::ObjectMode(r) => Some(r),
                _ => None,
            })
            .collect()
    };
    hero.gizmo.mode.publish(Some(9), &modes);
    assert!(
        pede(&mut hero).is_empty(),
        "o CONTROLO: em Object a aba pediu um modo"
    );
    hero.gizmo.mode.enter(9, ObjectMode::Sculpt);
    assert_eq!(pede(&mut hero), vec![ModeRequest::Enter(ObjectMode::Paint)]);
}
