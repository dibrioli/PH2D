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

/// ⭐⭐ **A linha *Sculpt Mode* do seletor de uma PEÇA chega ao barramento** (spec/06 F3) — com todos
/// os painéis registados, pela porta do quadro. ⚠️ O CONTROLO é a ordem das linhas: Object à frente,
/// depois as da família na ordem que ela declara.
#[test]
fn the_sculpt_row_of_a_piece_asks_for_sculpt() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo
        .mode
        .publish(Some(9), &[ObjectMode::Sculpt, ObjectMode::Paint]);
    let menu = hero.gizmo.mode.menu(&mut hero.store);
    assert_eq!(
        menu.as_ref().map(|m| m.faces.clone()),
        Some(vec![
            "Object Mode".to_string(),
            "Sculpt Mode".to_string(),
            "Paint Mode".to_string()
        ])
    );
    hero.store.publish_mode_menu(menu);
    hero.apply_event(WidgetEvent::Click(ph2d_editor_core::ids::area_menu_button(
        0,
    )));
    let _ = hero.bus.drain().count();
    hero.apply_event(WidgetEvent::Click(
        ph2d_editor_core::ids::OBJECT_MODE_SCULPT,
    ));
    let asked: Vec<ModeRequest> = hero
        .bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::ObjectMode(r) => Some(r),
            _ => None,
        })
        .collect();
    assert_eq!(asked, vec![ModeRequest::Enter(ObjectMode::Sculpt)]);
}

fn image_selected() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo.mode.publish(Some(7), &[ObjectMode::Paint]);
    let menu = hero.gizmo.mode.menu(&mut hero.store);
    hero.store.publish_mode_menu(menu);
    hero
}

/// ⭐ **`Ctrl+Tab` abre a LISTA dos modos** (spec/06 §3.2) — o seletor do cabeçalho, e o 2.º toque
/// fecha; o `Tab` continua a ser o pedido de alternar. (Mutação: `mode_key` ignorar `list` ⇒ RED.)
#[test]
fn ctrl_tab_opens_the_mode_list_and_a_second_closes_it() {
    use ph2d_editor_core::screens::hero::mode_drive::mode_key;
    let mut hero = image_selected();
    mode_key(&mut hero, true);
    assert_eq!(
        hero.store.context_menu().map(|m| m.kind),
        Some(ContextMenuKind::AreaCommands { slot: 0 }),
        "o Ctrl+Tab não abriu a lista"
    );
    assert!(
        hero.bus
            .drain()
            .all(|a| !matches!(a, EditorAction::ObjectMode(_)))
    );
    mode_key(&mut hero, true);
    assert!(
        hero.store.context_menu().is_none(),
        "o 2.º Ctrl+Tab não fechou"
    );
    mode_key(&mut hero, false);
    let asked: Vec<_> = hero
        .bus
        .drain()
        .filter(|a| matches!(a, EditorAction::ObjectMode(_)))
        .collect();
    assert!(
        matches!(
            asked.as_slice(),
            [EditorAction::ObjectMode(ModeRequest::Toggle)]
        ),
        "o Tab deixou de alternar"
    );
}

/// ⛔ **Sem seletor, o `Ctrl+Tab` não abre o pulldown do MÓDULO** que ficou no `slot` 0. CONTROLO: o
/// pulldown existe e o clique nele abre-o — só a tecla é que não é dele.
#[test]
fn ctrl_tab_without_a_selector_opens_nothing() {
    use ph2d_editor_core::interaction::AreaMenu;
    use ph2d_editor_core::screens::hero::mode_drive::mode_key;
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.publish_mode_menu(None);
    hero.store
        .set_area_commands(vec![AreaMenu::default()], Vec::new());
    assert!(!hero.store.has_mode_selector());
    mode_key(&mut hero, true);
    assert!(
        hero.store.context_menu().is_none(),
        "abriu o menu do módulo"
    );
    hero.apply_event(WidgetEvent::Click(ph2d_editor_core::ids::area_menu_button(
        0,
    )));
    assert!(
        hero.store.context_menu().is_some(),
        "controlo: o chip não abre"
    );
}

/// ⭐ **A shell liga o `Tab` COM e SEM o Ctrl à mesma porta** — um braço `if !cmd_chord` (o de antes)
/// deixaria o `Ctrl+Tab` cair adiante e a lista seria um atalho sem tecla.
#[test]
fn the_shell_hands_tab_and_ctrl_tab_to_the_mode_key() {
    const HANDLERS: &str = include_str!("../../../../shells/desktop/src/input_handlers.rs");
    let arm = HANDLERS
        .find("KeyCode::Tab =>")
        .expect("o braço do Tab sem guarda sumiu");
    let body = &HANDLERS[arm..arm + 300];
    assert!(
        body.contains("mode_key(hero, cmd_chord)"),
        "o Tab não chega à porta do modo com o Ctrl"
    );
}

/// ⭐ **Um chip TRANSBORDADO abre sob o `⋯`** — o `Ctrl+Tab` e a paleta abrem-no sem ponteiro, e
/// sem rect o menu caía no canto da área. CONTROLO: com o rect do chip, é sob o chip.
#[test]
fn an_overflowed_selector_opens_under_the_overflow_chip() {
    use ph2d_editor_core::screens::hero::mode_drive::mode_key;
    use ph2d_editor_core::zones::Rect;
    let mut hero = image_selected();
    hero.hit_index.register(
        ph2d_editor_core::ids::TOOL_BAR_OVERFLOW,
        Rect::new(300.0, 10.0, 20.0, 20.0),
    );
    mode_key(&mut hero, true);
    let m = hero.store.context_menu().expect("não abriu");
    assert_eq!((m.x, m.y), (300.0, 30.0));
    hero.store.close_context_menu();
    hero.hit_index.register(
        ph2d_editor_core::ids::area_menu_button(0),
        Rect::new(120.0, 10.0, 40.0, 20.0),
    );
    mode_key(&mut hero, true);
    let m = hero.store.context_menu().expect("não abriu");
    assert_eq!((m.x, m.y), (120.0, 30.0));
}
