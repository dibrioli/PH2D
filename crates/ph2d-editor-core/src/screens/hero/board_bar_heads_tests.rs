//! W23 B5 — a grelha de TODAS as pontas e o cursor sobre as pegas, pelo caminho real do ecrã.

use super::*;

/// ⭐ Com a seta seleccionada, o «…» das pontas do FIM abre a grelha das quinze; o losango (uma das
/// de UML que a barra estreita não tinha) aplica-se à ponta do fim e a grelha fecha; sem a grelha
/// aberta a ponta de UML não se clica.
#[test]
fn the_more_tips_grid_offers_every_head_of_the_catalog() {
    let mut t = T::new();
    t.two_boxes();
    t.click_bar(Item::Tool(Tool::Connector));
    let (a, b) = (t.at(0.26, 0.45), t.at(0.66, 0.45));
    t.drag(a, b);
    assert!(
        t.find(node(Item::Head(1, Head::Diamond))).is_none(),
        "fechada"
    );
    t.click_bar(Item::MoreHeads(1));
    for h in Head::ALL {
        assert!(
            t.find(node(Item::Head(1, h))).is_some(),
            "{h:?} não está na grelha"
        );
    }
    t.click_bar(Item::Head(1, Head::Diamond));
    assert_eq!(t.arrows()[0].connector().unwrap().heads[1], Head::Diamond);
    assert_eq!(t.arrows()[0].connector().unwrap().heads[0], Head::None);
    assert!(
        t.find(node(Item::Head(1, Head::ZeroOrMany))).is_none(),
        "fechou"
    );
    // A do início abre a sua: as pontas dela são as do início.
    t.click_bar(Item::MoreHeads(0));
    t.click_bar(Item::Head(0, Head::ZeroOrMany));
    assert_eq!(
        t.arrows()[0].connector().unwrap().heads[0],
        Head::ZeroOrMany
    );
}

/// ⭐ W23 B5 — sobre a pega da direita de uma caixa seleccionada o ecrã pede a seta ↔ (a janela muda o
/// cursor do sistema por esta porta); no meio da caixa, nada.
#[test]
fn the_screen_asks_for_the_resize_cursor_over_a_handle() {
    use ph2d_board_edit::{BoardCursor, Dir, Handle};
    let mut t = T::new();
    t.key(BoardKey::Char('r'), Modifiers::default(), Some("r"));
    let (a, b) = (t.at(0.2, 0.4), t.at(0.3, 0.5));
    t.drag(a, b);
    t.paint();
    let area = board_view::area_of(t.hero.last_canvas);
    let (board, live) = t.hero.documents.active_parts().unwrap();
    let ed = live.editor.as_ref().unwrap();
    let f = ed.frame(&board.doc).expect("a caixa seleccionada");
    let px = 1.0 / board.camera.zoom;
    let at = f.handle(Handle::Resize(Dir::E), px, ed.metrics());
    let [x, y] = board.camera.to_screen(area, at);
    let [cx, cy] = board.camera.to_screen(area, f.center);
    assert_eq!(
        board_view::cursor(&t.hero, x as f32, y as f32),
        Some(BoardCursor::ResizeEw)
    );
    assert_eq!(board_view::cursor(&t.hero, cx as f32, cy as f32), None);
}

/// ⭐ W23 B5 — o trackpad no idioma do Miro: dois dedos MOVEM a vista (o zoom fica), a pinça dá zoom
/// com o ponto sob o ponteiro parado; a roda do rato continua a dar zoom.
#[test]
fn two_fingers_pan_the_board_and_a_pinch_zooms_it() {
    use board_view::BoardScroll;
    let mut t = T::new();
    let (x, y) = t.at(0.5, 0.5);
    let cam = |t: &T| t.hero.documents.active_board().unwrap().camera;
    let c0 = cam(&t);
    assert!(board_view::scroll(
        &mut t.hero,
        x,
        y,
        BoardScroll::Pan([30.0, -20.0])
    ));
    let c1 = cam(&t);
    assert_eq!(c1.zoom, c0.zoom, "dois dedos não dão zoom");
    assert!((c1.center_x - (c0.center_x - 30.0 / c0.zoom)).abs() < 1e-9);
    assert!((c1.center_y - (c0.center_y + 20.0 / c0.zoom)).abs() < 1e-9);
    let area = board_view::area_of(t.hero.last_canvas);
    let under = c1.to_world(area, [f64::from(x), f64::from(y)]);
    assert!(board_view::scroll(
        &mut t.hero,
        x,
        y,
        BoardScroll::Pinch(1.5)
    ));
    let c2 = cam(&t);
    assert!(
        (c2.zoom - c1.zoom * 1.5).abs() < 1e-9,
        "a pinça: {} -> {}",
        c1.zoom,
        c2.zoom
    );
    let still = c2.to_world(area, [f64::from(x), f64::from(y)]);
    assert!((still[0] - under[0]).abs() < 1e-6 && (still[1] - under[1]).abs() < 1e-6);
    assert!(board_view::wheel(&mut t.hero, x, y, -16.0));
    assert!(cam(&t).zoom != c2.zoom, "a roda continua a dar zoom");
}
