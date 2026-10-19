//! W23 B5(2) — os textos ao longo da linha de uma seta, pelo editor (doc 14): arrastar, duplo
//! clique, apagar e deitá-los, cada um UM passo.

use super::*;
use ph2d_board_model::{BoardOp, Route, TextTurn};

/// Duas caixas lado a lado (0,0 e 400,0, 160×100) e uma seta RECTA entre os centros com «sim» a meio,
/// sem selecção: a linha vai de x = 160 a x = 400 em y = 50.
fn labelled(w: &mut World) -> ElementId {
    w.rect([0.0, 0.0, 160.0, 100.0]);
    w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 50.0], NONE).unwrap();
    let mut el = w.el(id).clone();
    let c = el.connector_mut().unwrap();
    c.route = Route::Straight;
    c.captions = vec![Caption::new("sim", 0.5)];
    let _ = BoardOp::Put(el).apply(&mut w.doc);
    w.cmd(Command::Escape);
    id
}

fn caption_at(w: &mut World, id: ElementId, i: usize) -> [f64; 2] {
    let c = w.el(id).connector().unwrap().clone();
    let r = w.ed.routes(&w.doc).get(id).unwrap().clone();
    r.caption_frame(c.captions[i].at, c.turn).center
}

/// ⭐ Carregar num texto selecciona a seta e arrastá-lo leva-o AO LONGO da linha: assenta no ponto
/// dela mais perto do ponteiro (o y do ponteiro não o tira da linha), a linha não muda, e é UM
/// passo de desfazer; um toque só selecciona. (Um pouco ao lado do centro: a bolinha do meio da
/// linha, que cria um ponto de ajuste, fica por cima do texto.)
#[test]
fn dragging_a_caption_slides_it_along_the_line_in_one_step() {
    let mut w = world();
    let id = labelled(&mut w);
    let c0 = caption_at(&mut w, id, 0);
    let grab = [c0[0] + 12.0, c0[1] + 3.0];
    w.click(grab, NONE);
    assert_eq!(w.ed.selection().iter().copied().collect::<Vec<_>>(), [id]);
    assert_eq!(w.el(id).connector().unwrap().captions[0].at, 0.5);
    let line = w.ed.routes(&w.doc).get(id).unwrap().polyline();
    w.drag(grab, [grab[0] + 60.0, grab[1] + 80.0], NONE);
    let c = w.el(id).connector().unwrap().clone();
    assert!(c.waypoints.is_empty(), "nenhum ponto de ajuste");
    assert_eq!(w.ed.routes(&w.doc).get(id).unwrap().polyline(), line);
    assert!(c.captions[0].at > 0.5);
    let moved = caption_at(&mut w, id, 0);
    assert!(
        (moved[0] - (c0[0] + 60.0)).abs() < 1e-2,
        "andou o que o ponteiro andou: {moved:?}"
    );
    assert!((moved[1] - 50.0).abs() < 1e-2, "fica na linha: {moved:?}");
    assert!(w.cmd(Command::Undo));
    assert_eq!(
        w.el(id).connector().unwrap().captions[0].at,
        0.5,
        "um passo"
    );
}

/// ⭐ O duplo clique na LINHA, longe dos textos, escreve um texto NOVO ali (o Miro); terminar guarda-o
/// como UM passo. O duplo clique num texto escreve NELE.
#[test]
fn a_double_click_on_the_line_starts_a_new_caption_there() {
    let mut w = world();
    let id = labelled(&mut w);
    let p = [220.0, 50.0];
    w.click(p, NONE);
    assert!(w.ed.double_click(&mut w.doc, &mut w.h, &mut w.ts, at(p[0], p[1], NONE)));
    w.ed.text_input(&mut w.doc, &mut w.ts, "não");
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::Commit);
    let caps = w.el(id).connector().unwrap().captions.clone();
    assert_eq!(caps.len(), 2);
    assert_eq!(caps[0], Caption::new("sim", 0.5), "o outro fica");
    assert_eq!(caps[1].text, "não");
    let born = caption_at(&mut w, id, 1);
    assert!(
        (born[0] - p[0]).abs() < 1e-2 && (born[1] - p[1]).abs() < 1e-2,
        "{born:?}"
    );
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.el(id).connector().unwrap().captions.len(), 1, "um passo");
    let c0 = caption_at(&mut w, id, 0);
    let on = [c0[0] + 10.0, c0[1]];
    assert!(w.ed.double_click(&mut w.doc, &mut w.h, &mut w.ts, at(on[0], on[1], NONE)));
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::SelectAll);
    w.ed.text_input(&mut w.doc, &mut w.ts, "talvez");
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::Commit);
    assert_eq!(
        w.el(id).connector().unwrap().captions,
        vec![Caption::new("talvez", 0.5)]
    );
}

/// ⭐ Apagar as letras todas de um texto apaga o texto (nenhum fica vazio na seta); um duplo clique
/// na linha que não escreve nada não deixa texto nem passo de desfazer.
#[test]
fn an_emptied_caption_goes_away() {
    let mut w = world();
    let id = labelled(&mut w);
    w.h = History::default();
    let p = [220.0, 50.0];
    assert!(w.ed.double_click(&mut w.doc, &mut w.h, &mut w.ts, at(p[0], p[1], NONE)));
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::Commit);
    assert_eq!(w.el(id).connector().unwrap().captions.len(), 1);
    assert!(!w.h.can_undo(), "nada mudou, nada a desfazer");
    assert!(w.cmd(Command::EditText));
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::SelectAll);
    w.ed.text_key(
        &mut w.doc,
        &mut w.h,
        &mut w.ts,
        TextKey::Backspace { word: false },
    );
    w.ed.text_key(&mut w.doc, &mut w.h, &mut w.ts, TextKey::Commit);
    assert!(w.el(id).connector().unwrap().captions.is_empty());
    assert!(w.cmd(Command::Undo));
    assert_eq!(
        w.el(id).connector().unwrap().captions,
        vec![Caption::new("sim", 0.5)]
    );
}

/// ⭐ O bloco inteiro de um texto apanha o clique, rodado com ele: ao longo de uma seta VERTICAL o
/// fim de um texto comprido está por cima da linha; deitado na horizontal, está de lado — longe da
/// linha, e mesmo assim selecciona a seta.
#[test]
fn a_click_on_the_far_end_of_a_caption_selects_the_arrow() {
    let mut w = world();
    w.rect([0.0, 0.0, 100.0, 100.0]);
    w.rect([0.0, 500.0, 100.0, 100.0]);
    let id = w.arrow_drag([50.0, 50.0], [50.0, 550.0], NONE).unwrap();
    let mut el = w.el(id).clone();
    let c = el.connector_mut().unwrap();
    c.route = Route::Straight;
    c.captions = vec![Caption::new("um texto bem comprido", 0.5)];
    c.turn = TextTurn::Level;
    let _ = BoardOp::Put(el).apply(&mut w.doc);
    w.cmd(Command::Escape);
    let c0 = caption_at(&mut w, id, 0);
    let side = [c0[0] + 60.0, c0[1]];
    w.click(side, NONE);
    assert_eq!(w.ed.selection().iter().copied().collect::<Vec<_>>(), [id]);
    assert!(w.cmd(Command::TextTurn(TextTurn::Along)));
    assert_eq!(w.el(id).connector().unwrap().turn, TextTurn::Along);
    w.cmd(Command::Escape);
    w.click(side, NONE);
    assert!(
        w.ed.selection().is_empty(),
        "ao longo da linha já não está ali"
    );
    let along = [c0[0], c0[1] + 60.0];
    w.click(along, NONE);
    assert_eq!(w.ed.selection().iter().copied().collect::<Vec<_>>(), [id]);
    assert!(w.cmd(Command::Undo));
    assert_eq!(
        w.el(id).connector().unwrap().turn,
        TextTurn::Level,
        "deitar é UM passo"
    );
}
