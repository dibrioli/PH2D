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

thread_local! {
    static TS: std::cell::RefCell<TextSystem> = std::cell::RefCell::new(TextSystem::without_system_fonts());
}

/// O `pointer` da shell, com uma letra de teste e sem modificadores.
fn press(
    hero: &mut HeroScreen,
    kind: PointerKind,
    button: PointerButton,
    x: f32,
    y: f32,
    on_canvas: bool,
) -> bool {
    TS.with_borrow_mut(|ts| {
        let input = Input {
            text: ts,
            mods: Modifiers::default(),
            now_ns: 0,
        };
        pointer(hero, input, kind, button, x, y, on_canvas)
    })
}

fn drag_to(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    TS.with_borrow_mut(|ts| {
        let input = Input {
            text: ts,
            mods: Modifiers::default(),
            now_ns: 0,
        };
        pointer_move(hero, input, x, y)
    })
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
    assert!(
        !wheel(&mut hero, 10.0, 10.0, 16.0),
        "fora da área a roda não é do quadro"
    );
    assert_eq!(zoom(&hero), z);
}

#[test]
fn with_the_scene_tab_active_nothing_is_consumed() {
    let mut hero = with_board();
    hero.apply_event(WidgetEvent::Click(ids::DOC_TAB_SCENE));
    assert!(!wheel(&mut hero, 500.0, 300.0, 16.0));
    assert!(!press(
        &mut hero,
        PointerKind::Down,
        PointerButton::Primary,
        500.0,
        300.0,
        true
    ));
    assert!(!drag_to(&mut hero, 520.0, 300.0));
}

#[test]
fn dragging_moves_the_view_and_releasing_ends_it() {
    let mut hero = with_board();
    let cam0 = hero.documents.active_board().unwrap().camera;
    assert!(press(
        &mut hero,
        PointerKind::Down,
        PointerButton::Middle,
        500.0,
        300.0,
        true
    ));
    assert!(drag_to(&mut hero, 540.0, 290.0));
    let cam1 = hero.documents.active_board().unwrap().camera;
    assert_eq!(cam1.center_x, cam0.center_x - 40.0);
    assert_eq!(cam1.center_y, cam0.center_y + 10.0);
    assert!(press(
        &mut hero,
        PointerKind::Up,
        PointerButton::Middle,
        540.0,
        290.0,
        true
    ));
    assert!(
        !drag_to(&mut hero, 600.0, 290.0),
        "solto, o cursor já não arrasta"
    );
}

#[test]
fn a_press_on_chrome_over_the_area_is_not_the_boards() {
    let mut hero = with_board();
    assert!(!press(
        &mut hero,
        PointerKind::Down,
        PointerButton::Primary,
        500.0,
        300.0,
        false
    ));
    assert!(hero.documents.pan_from.is_none());
}

/// O estilo de nascença grava a TINTA DO DOCUMENTO (nunca a do tema em que a forma nasceu): uma cena
/// aberta num tema escuro tem de se ler no claro (smoke do dono, 07/10).
#[test]
fn the_default_style_stores_the_document_ink_not_the_theme_one() {
    let ink = ph2d_board_model::Rgba(ph2d_board_model::DEFAULT_INK);
    let s = super::default_style();
    assert_eq!(s.stroke, Some(ink));
    assert_eq!(s.text_color, ink);
}
