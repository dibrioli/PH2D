use super::*;
use crate::ids;
use crate::interaction::WidgetEvent;
use ph2d_a11y::NodeId;

fn with_board() -> HeroScreen {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.last_canvas = Rect::new(100.0, 40.0, 800.0, 600.0);
    hero.apply_event(WidgetEvent::Click(ids::DOC_TAB_NEW));
    hero
}

fn zoom(hero: &HeroScreen) -> f64 {
    hero.documents.active_board().unwrap().camera.zoom
}

#[test]
fn the_wheel_zooms_the_board_only_inside_its_area() {
    let mut hero = with_board();
    assert!(wheel(&mut hero, 500.0, 300.0, 16.0));
    assert!(zoom(&hero) > 1.0, "roda para cima aproxima");
    let z = zoom(&hero);
    assert!(!wheel(&mut hero, 10.0, 10.0, 16.0), "fora da área a roda não é do quadro");
    assert_eq!(zoom(&hero), z);
}

#[test]
fn with_the_scene_tab_active_nothing_is_consumed() {
    let mut hero = with_board();
    hero.apply_event(WidgetEvent::Click(ids::DOC_TAB_SCENE));
    assert!(!wheel(&mut hero, 500.0, 300.0, 16.0));
    assert!(!pointer(&mut hero, PointerKind::Down, PointerButton::Primary, 500.0, 300.0, true));
    assert!(!pointer_move(&mut hero, 520.0, 300.0));
}

#[test]
fn dragging_moves_the_view_and_releasing_ends_it() {
    let mut hero = with_board();
    let cam0 = hero.documents.active_board().unwrap().camera;
    assert!(pointer(&mut hero, PointerKind::Down, PointerButton::Middle, 500.0, 300.0, true));
    assert!(pointer_move(&mut hero, 540.0, 290.0));
    let cam1 = hero.documents.active_board().unwrap().camera;
    assert_eq!(cam1.center_x, cam0.center_x - 40.0);
    assert_eq!(cam1.center_y, cam0.center_y + 10.0);
    assert!(pointer(&mut hero, PointerKind::Up, PointerButton::Middle, 540.0, 290.0, true));
    assert!(!pointer_move(&mut hero, 600.0, 290.0), "solto, o cursor já não arrasta");
}

#[test]
fn a_press_on_chrome_over_the_area_is_not_the_boards() {
    let mut hero = with_board();
    assert!(!pointer(&mut hero, PointerKind::Down, PointerButton::Primary, 500.0, 300.0, false));
    assert!(hero.documents.pan_from.is_none());
}
