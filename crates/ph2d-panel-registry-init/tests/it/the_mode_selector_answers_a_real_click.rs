//! ⭐⭐ **O SELETOR DE MODO responde a um clique de VERDADE** (spec/06 F2) — pela porta do quadro
//! (`HeroScreen::apply_event`), com TODOS os painéis registados: um painel que consumisse o id da
//! linha calaria o clique, e nenhum gate da fundação o veria (lá o registo de painéis é vazio).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
use ph2d_editor_core::{HeroScreen, NodeId};

/// O chip abre o seletor; a linha *Paint Mode* chega ao barramento como o pedido que a fase drena,
/// e fecha o pulldown. (Mutação: o `object_mode_menu::apply` deixar de empurrar o pedido ⇒ RED.)
#[test]
fn the_chip_opens_the_selector_and_a_row_asks_for_the_mode() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo.mode.publish(Some(7), &[ObjectMode::Paint]);
    let menu = hero.gizmo.mode.menu(&mut hero.store);
    hero.store.publish_mode_menu(menu);

    hero.apply_event(WidgetEvent::Click(ph2d_editor_core::ids::area_menu_button(
        0,
    )));
    assert_eq!(
        hero.store.context_menu().map(|m| m.kind),
        Some(ContextMenuKind::AreaCommands { slot: 0 }),
        "o chip não abriu o seletor"
    );
    let _ = hero.bus.drain().count();
    hero.apply_event(WidgetEvent::Click(ph2d_editor_core::ids::OBJECT_MODE_PAINT));
    let asked: Vec<ModeRequest> = hero
        .bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::ObjectMode(r) => Some(r),
            _ => None,
        })
        .collect();
    assert_eq!(asked, vec![ModeRequest::Enter(ObjectMode::Paint)]);
    assert!(
        hero.store.context_menu().is_none(),
        "o pulldown ficou aberto"
    );
}

/// ⭐ As linhas do seletor estão REGISTADAS — senão nascem mortas sob o dedo (o Up não emite
/// `Click`), que é o modo de falha mais caro desta casa.
#[test]
fn every_mode_row_is_registered() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let hero = HeroScreen::new(NodeId(1));
    for m in ObjectMode::ALL {
        assert!(hero.store.contains(m.row_id()), "{m:?}: linha sem registo");
    }
}
