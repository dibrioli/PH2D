//! Os testes da [`super::ScrollList`] e da [`super::AxisBar`].

use super::super::scrollbar::track_rect;
use super::axis_bar::swap;
use super::*;
use crate::hold_repeat::{STEPPER_HOLD_INITIAL_DELAY_NS, STEPPER_REPEAT_INTERVAL_NS};

/// A barra horizontal é a vertical com os eixos trocados: o polegar anda em `x`.
#[test]
fn a_horizontal_bar_is_the_vertical_one_on_its_side() {
    let b = AxisBar {
        track: Rect::new(10.0, 100.0, 400.0, 6.0),
        horizontal: true,
        content: 2000.0,
        visible: 500.0,
        arrows: false,
        step: 0.0,
    };
    let t0 = b.thumb(0.0);
    assert_eq!((t0.x, t0.y, t0.h), (10.0, 100.0, 6.0));
    assert_eq!(b.thumb(b.max()).x + b.thumb(b.max()).w, 410.0);
    assert_eq!(
        b.press((t0.x + 1.0, 103.0), 0.0),
        0.0,
        "no polegar a rolagem fica"
    );
    assert_eq!(
        b.press((410.0, 103.0), 0.0),
        b.max(),
        "o fim da calha é o fim"
    );
    let s = b.drag((20.0, 0.0), 0.0, (120.0, 50.0));
    assert!(s > 0.0 && s < b.max(), "só o x conta: {s}");
}

/// D17 — a margem horizontal é a vertical deitada: a mesma espessura, as mesmas setas e o
/// mesmo polegar, transpostos; a margem fica no fim de través (em baixo / à direita).
#[test]
fn a_horizontal_gutter_is_the_vertical_one_transposed() {
    let area = Rect::new(12.0, 40.0, 600.0, 300.0);
    let (vb, vt) = AxisBar::gutter(area, false, 23.0, 9.0);
    let (hb, ht) = AxisBar::gutter(swap(area), true, 23.0, 9.0);
    assert_eq!((hb, ht), (swap(vb), swap(vt)));
    assert_eq!(
        (vt.x, vt.w),
        (12.0 + 600.0 - 23.0 + 7.0, 9.0),
        "7 + 9 + 7 à direita"
    );
    let (b, t) = AxisBar::gutter(area, true, 23.0, 9.0);
    assert_eq!(
        (b.y, b.h, b.x, b.w),
        (340.0 - 23.0, 23.0, 12.0, 600.0),
        "em baixo"
    );
    assert_eq!((t.y, t.h), (340.0 - 16.0, 9.0));
    let bar = |track, horizontal| AxisBar {
        track,
        horizontal,
        content: 2000.0,
        visible: 300.0,
        arrows: true,
        step: 40.0,
    };
    let (v, h) = (bar(vt, false), bar(ht, true));
    for s in [0.0, 300.0, v.max()] {
        assert_eq!(h.thumb(s), swap(v.thumb(s)));
    }
    assert_eq!(
        h.arrow_tris(),
        v.arrow_tris().map(|t| t.map(|(x, y)| (y, x)))
    );
    let end = (ht.x + ht.w - 1.0, ht.y + 4.0);
    assert_eq!(h.arrow_at(end), Some(1.0));
    assert_eq!(h.press(end, 0.0), 40.0, "a seta dá um passo");
}

/// As setas (D16): cada ponta rola `step` e o polegar anda só entre elas.
#[test]
fn an_arrow_scrolls_one_step_and_the_thumb_stays_between_them() {
    let b = AxisBar {
        track: Rect::new(0.0, 0.0, 9.0, 300.0),
        horizontal: false,
        content: 2000.0,
        visible: 300.0,
        arrows: true,
        step: 40.0,
    };
    assert_eq!(b.arrow_at((4.0, 2.0)), Some(-1.0));
    assert_eq!(b.arrow_at((4.0, 298.0)), Some(1.0));
    assert_eq!(b.arrow_at((4.0, 150.0)), None);
    assert_eq!(b.press((4.0, 298.0), 100.0), 140.0);
    assert_eq!(b.press((4.0, 2.0), 20.0), 0.0, "não passa do começo");
    assert_eq!(b.press((4.0, 298.0), b.max()), b.max(), "nem do fim");
    assert_eq!(b.thumb(0.0).y, 12.0, "a ponta de 12 / 9 da espessura");
    let t = b.thumb(b.max());
    assert_eq!(t.y + t.h, 288.0);
}

const D: u128 = STEPPER_HOLD_INITIAL_DELAY_NS;
const I: u128 = STEPPER_REPEAT_INTERVAL_NS;

/// D17 — a lei de segurar: nada antes do atraso; no atraso o 1.º passo, depois um a cada
/// intervalo; um quadro atrasado recebe todos os vencidos; fora do alvo deita-os fora e retoma.
#[test]
fn holding_repeats_after_the_delay_at_the_interval_and_pauses_off_target() {
    let mut h = HoldRepeat::new(1_000);
    assert_eq!(h.due(1_000 + D - 1, true), 0, "nada antes do atraso");
    assert_eq!(h.due(1_000 + D, true), 1, "o 1.º passo no atraso");
    assert_eq!(h.due(1_000 + D + I - 1, true), 0);
    assert_eq!(h.due(1_000 + D + 3 * I, true), 3, "os vencidos de uma vez");
    assert_eq!(h.due(1_000 + D + 5 * I, false), 0, "fora da seta, nada");
    assert_eq!(h.due(1_000 + D + 5 * I, true), 0, "e os de fora não voltam");
    assert_eq!(h.due(1_000 + D + 6 * I, true), 1, "de volta à seta, retoma");
}

/// A barra agarrada (a das folhas do Kanban): segurar ▼ repete até ao fim e pára lá; largar
/// (largar a `BarGrab`) acaba; sem seta, `tick` nada pede; o polegar segue o rato como o
/// arrasto da barra das listas ([`AxisBar::drag`]).
#[test]
fn a_grabbed_bar_holds_to_the_end_and_drags_like_the_list_bar() {
    let area = Rect::new(0.0, 0.0, 300.0, 200.0);
    let (band, b) = AxisBar::in_gutter(area, false, (23.0, 9.0), (900.0, 200.0, 40.0));
    assert_eq!((band.x, band.w), (277.0, 23.0), "a margem à direita");
    let down = (b.track.x + 4.0, b.track.y + b.track.h - 1.0);
    let (s, mut g) = b.grab(down, 0.0, 0);
    assert_eq!(s, 40.0, "o carregar é um passo");
    assert_eq!(b.tick(&mut g, s, D - 1), Some(40.0), "nada antes do atraso");
    assert_eq!(b.tick(&mut g, s, D + 2 * I), Some(160.0), "1 + 3 passos");
    assert_eq!(
        b.tick(&mut g, 160.0, D + 1_000 * I),
        Some(b.max()),
        "pára no fim"
    );
    assert_eq!(b.tick(&mut g, b.max(), D + 2_000 * I), Some(b.max()));
    let mid = (down.0, b.track.y + b.track.h / 2.0);
    assert_eq!(
        b.follow(&mut g, mid, 300.0),
        300.0,
        "com a seta, o rato só se move"
    );
    assert_eq!(
        b.tick(&mut g, 300.0, D + 3_000 * I),
        Some(300.0),
        "fora da seta, pausa"
    );
    let t = b.thumb(0.0);
    let at = (t.x + 1.0, t.y + 2.0);
    let (s0, mut g) = b.grab(at, 0.0, 0);
    assert_eq!(
        (s0, b.tick(&mut g, s0, D + I)),
        (0.0, None),
        "o polegar não repete"
    );
    for dy in [10.0, 57.0, 400.0] {
        let to = (at.0 + 30.0, at.1 + dy);
        assert_eq!(b.follow(&mut g, to, s0), b.drag(at, 0.0, to));
    }
}

fn list_with_arrows() -> (ScrollList, Rect) {
    let mut l = ScrollList::default();
    let body = Rect::new(0.0, 0.0, 200.0, 4.0 * ROW_H_PX);
    l.set(body, 20.0 * ROW_H_PX);
    l.track = Some(track_rect(body));
    let t = l.track.unwrap();
    (l, t)
}

/// D17 — a lista suspensa: segurar ▼ dá um passo no carregar, repete pela lei e pára ao largar;
/// sair da seta pausa; o polegar não fica agarrado.
#[test]
fn holding_the_dropdown_arrow_keeps_scrolling_until_release() {
    let (mut l, t) = list_with_arrows();
    let down = t.y + t.h - 1.0;
    l.press_bar(down, 0);
    assert_eq!(l.scroll, ROW_H_PX, "o carregar é um passo");
    assert!(l.grab.is_none(), "a seta não agarra o polegar");
    assert!(l.tick(D - 1));
    assert_eq!(l.scroll, ROW_H_PX, "nada antes do atraso");
    assert!(l.tick(D + 2 * I));
    assert_eq!(l.scroll, 4.0 * ROW_H_PX, "1 + 3 passos");
    l.drag_bar(t.y + t.h / 2.0);
    assert!(l.tick(D + 4 * I));
    assert_eq!(l.scroll, 4.0 * ROW_H_PX, "fora da seta, pausa");
    l.drag_bar(down);
    assert!(l.tick(D + 5 * I));
    assert_eq!(l.scroll, 5.0 * ROW_H_PX, "de volta, retoma");
    l.release();
    assert!(!l.tick(D + 50 * I), "largou: não pede mais quadros");
    assert_eq!(l.scroll, 5.0 * ROW_H_PX);
    l.press_bar(down, 0);
    assert!(l.tick(D + 1_000 * I));
    assert_eq!(l.scroll, l.max(), "pára no fim");
}
