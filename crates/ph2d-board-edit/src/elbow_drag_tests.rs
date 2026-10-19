//! W23 B5(1) — arrastar o segmento do meio de uma seta cotovelo, pelo editor (as caixas dos
//! oráculos, doc 13 §1).

use super::*;
use crate::BoardCursor;
use ph2d_board_model::{BoardOp, Route};

/// Duas caixas desencontradas (R1 0,0 100×100 · R2 400,300 100×100) e uma seta cotovelo entre os
/// centros, seleccionada.
fn elbow(w: &mut World) -> ElementId {
    w.rect([0.0, 0.0, 100.0, 100.0]);
    w.rect([400.0, 300.0, 100.0, 100.0]);
    let id = w.arrow_drag([50.0, 50.0], [450.0, 350.0], NONE).unwrap();
    let mut el = w.el(id).clone();
    el.connector_mut().unwrap().route = Route::Elbow;
    let _ = BoardOp::Put(el).apply(&mut w.doc);
    id
}

fn segment(w: &mut World) -> ([f64; 2], bool) {
    let o = w.ed.overlay(&mut w.doc, &mut w.ts);
    assert!(
        o.wires[0].mids.is_empty(),
        "o trecho em Z não tem a bolinha que cria ponto"
    );
    o.wires[0].segments[0]
}

/// ⭐ A pega do segmento do meio leva-o na PERPENDICULAR (o y do ponteiro não conta), sem criar
/// ponto de ajuste, e é UM passo de desfazer; um toque sem arrastar não muda nada; o cursor é ↔.
#[test]
fn dragging_the_middle_of_an_elbow_moves_it_across_in_one_step() {
    let mut w = world();
    let id = elbow(&mut w);
    let (seg, vertical) = segment(&mut w);
    assert!(vertical && close(seg[0], 250.0), "nasce a meio: {seg:?}");
    assert_eq!(
        w.ed.cursor_at(&w.doc, at(seg[0], seg[1], NONE)),
        Some(BoardCursor::ResizeEw)
    );
    w.click(seg, NONE);
    assert_eq!(
        w.el(id).connector().unwrap().mid(0),
        None,
        "um toque não fixa nada"
    );
    w.drag(seg, [seg[0] + 60.0, seg[1] + 45.0], NONE);
    let c = w.el(id).connector().unwrap().clone();
    assert!(c.waypoints.is_empty(), "nenhum ponto de ajuste");
    assert!(c.mid(0).is_some());
    let (moved, _) = segment(&mut w);
    assert!(close(moved[0], 310.0), "x = {}", moved[0]);
    assert!(
        close(moved[1], seg[1]),
        "o segmento não anda no seu próprio eixo"
    );
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.el(id).connector().unwrap().mid(0), None, "um passo");
    assert!(close(segment(&mut w).0[0], 250.0));
}

/// ⭐ O duplo clique no segmento fixo larga-o (volta ao meio de nascença), UM passo — o idioma do
/// Excalidraw (o botão oco); num segmento de nascença não faz nada de seta.
#[test]
fn a_double_click_lets_the_middle_go() {
    let mut w = world();
    let id = elbow(&mut w);
    let (seg, _) = segment(&mut w);
    w.drag(seg, [seg[0] + 60.0, seg[1]], NONE);
    let (fixed, _) = segment(&mut w);
    assert!(w.ed.double_click(
        &mut w.doc,
        &mut w.h,
        &mut w.ts,
        at(fixed[0], fixed[1], NONE)
    ));
    assert_eq!(w.el(id).connector().unwrap().mid(0), None);
    assert!(close(segment(&mut w).0[0], 250.0));
    assert!(w.cmd(Command::Undo));
    assert!(
        w.el(id).connector().unwrap().mid(0).is_some(),
        "desfazer devolve-o"
    );
}
