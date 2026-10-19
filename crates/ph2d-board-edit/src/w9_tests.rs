//! W9 (a fidelidade ao Excalidraw) no editor: o que a auditoria de 2 lentes achou, cada um com o teste
//! que o via vermelho — a ponta presa com o vão continua presa ao agarrá-la com zoom, a cópia de um
//! elemento importado treme de outra maneira, a nota com a letra importada mede-se nessa letra; e
//! (W10) o realce do arrasto de uma ponta sem o ponto no foco.

use super::*;

/// ⭐ Agarrar a PONTA de uma seta presa e mexê-la 1 px com zoom NÃO a solta: a ponta está a `g` da
/// forma, e a faixa de ligação em px fica mais fina que o vão com zoom (auditoria W9, lente 2: a
/// partir de ~230 % a ponta soltava-se e a seta deixava de seguir a caixa).
#[test]
fn grabbing_a_bound_end_with_zoom_keeps_it_bound() {
    let mut w = world();
    w.rect([0.0, 0.0, 160.0, 100.0]);
    let b = w.rect([400.0, 0.0, 160.0, 100.0]);
    let id = w.arrow_drag([80.0, 50.0], [480.0, 50.0], NONE).unwrap();
    let tip = w.ed.routes(&w.doc).get(id).unwrap().ends()[1];
    // Zoom 4: um px de ecrã são 0,25 do mundo.
    let px = 0.25;
    let p = |x: f64, y: f64| Pointer {
        world: [x, y],
        px,
        mods: NONE,
    };
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, p(tip[0], tip[1]), VIEW);
    w.ed.pointer_move(&mut w.doc, &mut w.ts, p(tip[0] - px, tip[1]));
    w.ed.pointer_up(&mut w.doc, &mut w.h, p(tip[0] - px, tip[1]));
    assert!(
        matches!(w.ends(id)[1], End::Bound { target, .. } if target == b),
        "a ponta soltou-se: {:?}",
        w.ends(id)[1]
    );
}

/// ⭐ Duplicar um elemento com a semente IMPORTADA de um `.excalidraw` dá uma cópia que treme de outra
/// maneira (como no Excalidraw, que sorteia uma nova) — o original fica com a dele (auditoria W9).
#[test]
fn a_copy_of_an_imported_element_trembles_its_own_way() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 50.0, 50.0]);
    let mut el = w.el(id).clone();
    el.set_imported_seed(Some(4242));
    BoardOp::Put(el).apply(&mut w.doc);
    assert_eq!(w.el(id).seed(), 4242);
    assert!(w.cmd(Command::Duplicate));
    let dup = *w.ed.selection().iter().next().unwrap();
    assert_ne!(dup, id);
    assert_ne!(w.el(dup).seed(), 4242, "a cópia treme igual ao original");
    assert!(w.cmd(Command::Copy));
    assert!(w.cmd(Command::Paste));
    let pasted = *w.ed.selection().iter().next().unwrap();
    assert_ne!(w.el(pasted).seed(), w.el(dup).seed());
    assert_eq!(w.el(id).seed(), 4242, "o original fica com a dele");
}

/// ⭐ Uma nota com a letra À MÃO que veio de um `.excalidraw` (`Look::letter`) mede a altura nessa
/// letra — a MESMA em que se desenha (auditoria W9: media-se na da interface, e a nota deixava de
/// abraçar o texto).
#[test]
fn a_note_with_the_imported_hand_letter_measures_in_that_letter() {
    use ph2d_board_layout::{Face, text_height};
    let mut w = world();
    let c = ph2d_board_model::Rgba(ph2d_board_model::STICKY_COLORS[1]);
    let mut st = Style::new(Some(c), None, c.readable_ink());
    st.look.letter = ph2d_board_model::Letter::Hand;
    let side = ph2d_board_model::STICKY_SIDE;
    let shape = ph2d_board_model::Shape {
        kind: ShapeType::Sticky,
        style: st,
        text: ph2d_board_model::RichText::plain(&"palavras compridas ".repeat(30)),
    };
    let el = Element::new_shape(
        w.doc.mint_id(),
        w.doc.z_on_top(),
        shape,
        [0.0, 0.0, side, side],
    );
    let s = el.shape().unwrap();
    let base = s.kind.note_aspect().map_or(el.h, |a| el.w * a);
    let tw = ph2d_board_geom::text_rect(s.kind, el.w, base)[2] as f32;
    let size = s.style.font_size as f32;
    let face = |hand: bool| Face {
        hand,
        ..Face::default()
    };
    let hand = text_height(&mut w.ts, &s.text, size, tw, face(true));
    let sans = text_height(&mut w.ts, &s.text, size, tw, face(false));
    assert!(
        (hand - sans).abs() > 1.0,
        "a régua não distingue as letras: {hand} · {sans}"
    );
    let want = ph2d_board_geom::height_for_text(s.kind, base, f64::from(hand));
    let got = crate::notes::fitted_height(&mut w.ts, &el);
    assert!(
        (got - want).abs() < 1e-6,
        "mediu {got}, a letra desenhada pede {want}"
    );
}

/// ⭐ W10 — o REALCE ao arrastar uma ponta para a faixa junto de uma forma é o do Excalidraw (medido no
/// next a meio do arrasto: `saidas/w10_R_ponta_banda_diag.next.meio.png`, `w10_R_ponta_fora10`): o
/// contorno realçado e a seta já parada onde vai ficar, a `g` do contorno — e NENHUM ponto no foco
/// (o nosso marcava-o no contorno e a ponta parava a `g` dali: dois sítios para uma ponta, auditoria
/// W9, lente 2). Os pontos que o next mostra são os do encaixe no meio do lado, outra coisa.
#[test]
fn dragging_an_end_into_the_band_highlights_the_shape_and_the_arrow_shows_where_it_stops() {
    let mut w = world();
    let r = w.rect([500.0, 100.0, 200.0, 120.0]);
    let id = w.arrow_drag([330.0, 380.0], [460.0, 380.0], NONE).unwrap();
    w.ed.pointer_down(
        &mut w.doc,
        &mut w.h,
        &mut w.ts,
        at(460.0, 380.0, NONE),
        VIEW,
    );
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(478.0, 255.0, NONE));
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(497.0, 130.0, NONE));
    let o = w.ed.overlay(&mut w.doc, &mut w.ts);
    assert_eq!(
        o.target,
        Some(Target { element: r }),
        "só a forma: nenhum ponto no foco"
    );
    let tip = w.ed.routes(&w.doc).get(id).unwrap().ends()[1];
    let el = w.el(r);
    let (_, d) = ph2d_board_geom::nearest_on_outline(el, tip).unwrap();
    assert!(
        !ph2d_board_geom::inside(el, tip),
        "a ponta ao vivo entrou na forma"
    );
    assert!(
        (d - ph2d_board_route::gap(el)).abs() < 1e-3,
        "a seta ao vivo pára a {d:.3}, a lei diz {}",
        ph2d_board_route::gap(el)
    );
}

/// ⭐ W10 (auditoria, lente 2) — o botão «Rascunho ↔ Final» com um texto ABERTO fecha-o primeiro: a
/// escrita é um passo e a troca outro (antes o 1.º desfazer levava os dois e a edição aberta media-se
/// na letra velha).
#[test]
fn the_sketch_button_closes_the_open_text_first() {
    let mut w = world();
    let id = w.rect([0.0, 0.0, 160.0, 100.0]);
    let mut el = w.el(id).clone();
    el.style_mut().look.letter = ph2d_board_model::Letter::Sans;
    BoardOp::Put(el).apply(&mut w.doc);
    w.ed.select(&w.doc, []);
    assert!(w.ed.open_text(&mut w.doc, id));
    assert!(w.ed.text_input(&mut w.doc, &mut w.ts, "x"));
    w.ed.toggle_sketch(&mut w.doc, &mut w.h, false);
    assert!(
        w.ed.overlay(&mut w.doc, &mut w.ts).text.is_none(),
        "o texto ficou aberto"
    );
    assert_eq!(w.el(id).style().look.letter, ph2d_board_model::Letter::Auto);
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.el(id).style().look.letter, ph2d_board_model::Letter::Sans);
    assert_eq!(
        w.el(id).shape().unwrap().text.as_str(),
        "x",
        "o 1.º desfazer levou também a escrita"
    );
}

/// ⭐ W10 (prova de mutação) — o botão alcança também quem JÁ está no modo novo: uma forma à mão com a
/// letra fixa do ficheiro, num quadro com outra ainda final, passa à letra do modo (a sobrevivente: o
/// filtro só pela `sketch` deixava-a com a letra fixa).
#[test]
fn the_sketch_button_reaches_a_sketched_element_with_a_fixed_letter() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 160.0, 100.0]);
    let b = w.rect([300.0, 0.0, 160.0, 100.0]);
    let mut el = w.el(a).clone();
    el.style_mut().sketch = true;
    el.style_mut().look.letter = ph2d_board_model::Letter::Sans;
    BoardOp::Put(el).apply(&mut w.doc);
    w.ed.select(&w.doc, []);
    assert_eq!(w.ed.toggle_sketch(&mut w.doc, &mut w.h, false), Some(true));
    assert!(w.el(a).style().sketch && w.el(b).style().sketch);
    assert_eq!(w.el(a).style().look.letter, ph2d_board_model::Letter::Auto);
}

/// ⭐ Decisão do dono (10/10, a pergunta da W10 §9) — uma NOTA importada com a letra à mão fixa do
/// ficheiro volta ao modo com o botão: rascunho ⇒ à mão, e o final seguinte ⇒ a letra comum (antes o
/// botão não alcançava notas e ela ficava à mão para sempre).
#[test]
fn the_sketch_button_gives_an_imported_notes_letter_back_to_the_mode() {
    let mut w = world();
    w.ed.tool = Tool::Shape(ShapeType::Sticky);
    w.click([0.0, 0.0], NONE);
    let note = *w.ed.selection().iter().next().unwrap();
    let mut el = w.el(note).clone();
    el.style_mut().look.letter = ph2d_board_model::Letter::Hand;
    BoardOp::Put(el).apply(&mut w.doc);
    assert!(crate::hand_lettered(w.el(note)));
    w.ed.select(&w.doc, []);
    assert_eq!(w.ed.toggle_sketch(&mut w.doc, &mut w.h, false), Some(true));
    assert_eq!(
        w.el(note).style().look.letter,
        ph2d_board_model::Letter::Auto
    );
    assert!(crate::hand_lettered(w.el(note)));
    assert_eq!(w.ed.toggle_sketch(&mut w.doc, &mut w.h, true), Some(false));
    assert!(!crate::hand_lettered(w.el(note)), "a nota ficou à mão");
}
