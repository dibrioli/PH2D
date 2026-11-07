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

/// Uma aba de família que toma tudo e conta o que lhe chegou: a lei é da porta, vale para toda família.
#[derive(Default)]
struct Greedy(std::rc::Rc<std::cell::Cell<u32>>);

impl crate::doc_surface::DocumentSurface for Greedy {
    fn tabs(&self) -> Vec<crate::doc_surface::DocTab> {
        vec![crate::doc_surface::DocTab {
            id: 1,
            title: String::new(),
            dirty: false,
        }]
    }
    fn paint(
        &mut self,
        _: u64,
        _: Rect,
        _: &mut ph2d_vector::VectorScene,
        _: &mut TextSystem,
        _: &Theme,
    ) {
    }
    fn pointer(
        &mut self,
        _: u64,
        _: crate::doc_surface::PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        _: f32,
        _: f32,
    ) -> bool {
        self.0.set(self.0.get() + 1);
        // O carregar do meio cai fora do documento dela, como o canto de uma área vazia.
        !(kind == PointerKind::Down && button == PointerButton::Middle)
    }
    fn pointer_move(
        &mut self,
        _: u64,
        _: crate::doc_surface::PointerInput<'_>,
        _: f32,
        _: f32,
    ) -> bool {
        self.0.set(self.0.get() + 1);
        true
    }
    fn wheel(&mut self, _: u64, _: f32, _: f32, _: f32, _: bool) -> bool {
        false
    }
    fn key(
        &mut self,
        _: u64,
        _: &mut TextSystem,
        _: crate::doc_surface::KeyInput<'_>,
    ) -> crate::doc_surface::KeyOutcome {
        crate::doc_surface::KeyOutcome::default()
    }
    fn compose(&mut self, _: u64, _: crate::doc_surface::Compose<'_>) -> bool {
        false
    }
    fn close(&mut self, _: u64) {}
    fn undo(&mut self, _: u64, _: bool) -> bool {
        false
    }
    fn pointer_look(&self, _: u64, _: f32, _: f32) -> Option<crate::doc_surface::PointerLook> {
        Some(crate::doc_surface::PointerLook::Hand)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn moved(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    TS.with_borrow_mut(|ts| {
        let input = Input {
            text: ts,
            mods: Modifiers::default(),
            now_ns: 0,
        };
        pointer_move(hero, input, x, y)
    })
}

/// ⭐ Premido no chrome e largado sobre uma aba que toma tudo: a aba não ouve nada do gesto, e volta a
/// ouvir depois dele. É a porta de TODAS as famílias (Kanban, Code, Planner, Som).
#[test]
fn a_press_on_chrome_stays_chrome_over_any_family_tab() {
    use PointerButton::{Primary, Secondary};
    use PointerKind::{Down, Up};
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    hero.documents
        .install_surface(Box::new(Greedy(calls.clone())));
    hero.documents
        .activate_surface(crate::doc_surface::SurfaceTab { family: 0, id: 1 });
    let (x, y) = (500.0, 300.0);

    assert!(!press(&mut hero, Down, Primary, x, y, false));
    assert!(!moved(&mut hero, x, y), "o movimento é do chrome");
    assert!(pointer_look(&hero, x, y).is_none(), "o ponteiro da aba");
    // Outro botão, premido e largado a meio, não solta o gesto do primeiro.
    assert!(press(&mut hero, Down, Secondary, x, y, true));
    assert!(press(&mut hero, Up, Secondary, x, y, true));
    assert_eq!(calls.get(), 2, "só os do outro botão");
    assert!(!moved(&mut hero, x, y), "o primeiro continua premido");
    assert!(
        !press(&mut hero, Up, Primary, x, y, true),
        "o largar é do chrome"
    );
    assert_eq!(calls.get(), 2, "a aba ouviu o gesto do chrome");

    assert!(
        moved(&mut hero, x, y),
        "acabado o gesto, a aba volta a ouvir"
    );
    assert!(press(&mut hero, Down, Primary, x, y, true));
    assert!(
        press(&mut hero, Up, Primary, x, y, true),
        "o gesto dela é dela"
    );
    assert_eq!(calls.get(), 5);

    // Carregar sobre a área livre que ela não tomou não é chrome: o largar continua a ir-lhe.
    assert!(!press(&mut hero, Down, PointerButton::Middle, x, y, true));
    assert!(press(&mut hero, Up, PointerButton::Middle, x, y, true));
    assert_eq!(calls.get(), 7);
}

/// O quadro segue a mesma porta: a vista não anda sob o arrasto de um controlo.
#[test]
fn a_press_on_chrome_stays_chrome_over_a_board() {
    let mut hero = with_board();
    let (x, y) = (500.0, 300.0);
    assert!(!press(
        &mut hero,
        PointerKind::Down,
        PointerButton::Middle,
        x,
        y,
        false
    ));
    assert!(!moved(&mut hero, x + 40.0, y));
    assert!(!press(
        &mut hero,
        PointerKind::Up,
        PointerButton::Middle,
        x,
        y,
        true
    ));
    assert_eq!(hero.documents.chrome_press, None);
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
