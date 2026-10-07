//! A CANETA (W4) pelo caminho de quem a usa — as leis do Miro (fontes em `ink.rs`).

use super::*;
use ph2d_board_model::{Pen, ShapeType};

impl World {
    /// Um traço da caneta pelos pontos `pts` (carregar no 1.º, arrastar pelos outros, largar).
    fn stroke(&mut self, pts: &[[f64; 2]]) {
        let p = |q: [f64; 2]| at(q[0], q[1], NONE);
        self.ed
            .pointer_down(&mut self.doc, &mut self.h, &mut self.ts, p(pts[0]), VIEW);
        for q in &pts[1..] {
            self.ed.pointer_move(&mut self.doc, &mut self.ts, p(*q));
        }
        let last = pts[pts.len() - 1];
        self.ed.pointer_up(&mut self.doc, &mut self.h, p(last));
    }

    fn inks(&self) -> Vec<&Element> {
        self.doc.live().filter(|el| el.ink().is_some()).collect()
    }

    /// O que se VÊ: os elementos vivos sem o contador de versão (o desfazer repõe o conteúdo; a
    /// versão e as lápides são da colaboração e só andam para a frente).
    fn content(doc: &BoardDoc) -> Vec<Element> {
        doc.live()
            .map(|e| Element {
                version: 0,
                ..e.clone()
            })
            .collect()
    }

    fn line(&mut self, y: f64) -> ElementId {
        self.ed.tool = Tool::Pen(Pen::Pen);
        let before: Vec<ElementId> = self.inks().iter().map(|e| e.id).collect();
        let pts: Vec<[f64; 2]> = (0..=20).map(|i| [f64::from(i) * 10.0, y]).collect();
        self.stroke(&pts);
        self.inks()
            .into_iter()
            .map(|e| e.id)
            .find(|id| !before.contains(id))
            .expect("nasceu um traço")
    }
}

#[test]
fn the_pen_draws_one_stroke_through_the_pointer_and_one_undo_step_removes_it() {
    let mut w = world();
    let id = w.line(50.0);
    let el = w.el(id);
    let pts = crate::world_points(el);
    assert_eq!(pts.len(), 21, "um ponto por movimento");
    assert!(close(pts[0][0], 0.0) && close(pts[20][0], 200.0) && close(pts[7][1], 50.0));
    let preset = w.ed.pen.current(Pen::Pen);
    assert_eq!(el.style().stroke, Some(preset.color));
    assert_eq!(el.style().stroke_width, preset.width);
    assert!(!el.ink().unwrap().pressure, "o rato não mede a pressão");
    assert_eq!(w.ed.tool, Tool::Pen(Pen::Pen), "a caneta fica na mão (Miro)");
    assert!(w.cmd(Command::Undo));
    assert!(w.inks().is_empty());
    assert!(w.cmd(Command::Redo));
    assert_eq!(w.inks().len(), 1);
}

#[test]
fn the_highlighter_draws_with_its_own_preset() {
    let mut w = world();
    w.ed.pen.preset_mut(Pen::Highlighter, 0).width = 24.0;
    w.ed.tool = Tool::Pen(Pen::Highlighter);
    w.stroke(&[[0.0, 0.0], [50.0, 0.0], [100.0, 10.0]]);
    let el = w.inks()[0];
    assert_eq!(el.ink().unwrap().pen, Pen::Highlighter);
    assert_eq!(el.style().stroke_width, 24.0);
    assert_eq!(w.ed.pen.last, Pen::Highlighter, "o `P` volta a pegar no marcador");
}

#[test]
fn the_eraser_deletes_whole_pen_strokes_it_touches_and_nothing_else() {
    let mut w = world();
    let shape = w.rect([80.0, 80.0, 60.0, 60.0]);
    let a = w.line(0.0);
    let b = w.line(200.0);
    let before = World::content(&w.doc);
    w.ed.tool = Tool::Eraser { precise: false };
    // Passa pelo meio do traço `a` e pela forma, longe de `b`.
    w.stroke(&[[100.0, -40.0], [100.0, 40.0], [100.0, 120.0]]);
    assert!(w.doc.get(a).is_none(), "o traço tocado sai INTEIRO");
    assert!(w.doc.get(b).is_some(), "o que não tocou fica");
    assert!(w.doc.get(shape).is_some(), "a borracha apaga só desenho da caneta (Miro)");
    assert!(w.cmd(Command::Undo), "UM passo");
    assert_eq!(World::content(&w.doc), before);
}

#[test]
fn the_precision_eraser_cuts_only_where_it_passes_and_undo_puts_the_stroke_back() {
    let mut w = world();
    let a = w.line(0.0);
    let before = World::content(&w.doc);
    w.ed.tool = Tool::Eraser { precise: true };
    w.stroke(&[[100.0, -40.0], [100.0, 40.0]]);
    assert!(w.doc.get(a).is_none());
    let pieces = w.inks();
    assert_eq!(pieces.len(), 2, "o traço partiu-se em dois");
    let reach = w.ed.metrics().eraser * PX + w.ed.pen.current(Pen::Pen).width / 2.0;
    for p in &pieces {
        for q in crate::world_points(p) {
            assert!((q[0] - 100.0).abs() > reach - 1e-9, "nenhum ponto ficou debaixo da borracha");
        }
    }
    let xs: Vec<f64> = pieces.iter().map(|p| crate::world_points(p)[0][0]).collect();
    assert!(xs.contains(&0.0), "o pedaço da esquerda começa onde o traço começava");
    // Uma 2.ª passagem no mesmo gesto corta um pedaço que o gesto criou: continua UM passo.
    w.stroke(&[[30.0, -40.0], [30.0, 40.0]]);
    assert_eq!(w.inks().len(), 3);
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.inks().len(), 2);
    assert!(w.cmd(Command::Undo));
    assert_eq!(World::content(&w.doc), before, "o traço volta inteiro, com o mesmo id");
}

#[test]
fn the_laser_never_touches_the_document_and_its_trail_fades_away() {
    let mut w = world();
    let before = w.doc.rev();
    w.ed.tool = Tool::Laser;
    w.stroke(&[[0.0, 0.0], [10.0, 0.0], [20.0, 5.0]]);
    assert_eq!(w.doc.rev(), before, "nenhuma operação no documento");
    assert!(!w.cmd(Command::Undo), "nem um passo de desfazer");
    let now = std::time::Instant::now();
    let trail = w.ed.laser_trail(now);
    assert!(trail.len() >= 3);
    assert!(trail.iter().all(|(_, life)| *life > 0.0 && *life <= 1.0));
    assert!(w.ed.laser_trail(now + crate::LASER_LIFE).is_empty(), "um segundo depois sumiu");
}

#[test]
fn escape_while_drawing_takes_the_stroke_back_without_a_step() {
    let mut w = world();
    w.ed.tool = Tool::Pen(Pen::Pen);
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(0.0, 0.0, NONE), VIEW);
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(40.0, 0.0, NONE));
    assert_eq!(w.inks().len(), 1, "desenha-se AO VIVO");
    assert!(w.cmd(Command::Escape));
    assert!(w.inks().is_empty());
    assert!(!w.ed.is_busy());
    assert!(!w.cmd(Command::Undo));
}

#[test]
fn a_stroke_is_selected_by_clicking_on_it_and_moves_like_a_shape() {
    let mut w = world();
    let a = w.line(100.0);
    w.ed.tool = Tool::Select;
    w.click([55.0, 130.0], NONE);
    assert!(w.ed.selection().is_empty(), "ao lado do traço não apanha");
    w.click([55.0, 101.0], NONE);
    assert_eq!(w.ed.selection().iter().copied().collect::<Vec<_>>(), vec![a]);
    assert!(w.ed.frame(&w.doc).is_some(), "um traço tem moldura e pegas");
    let x0 = w.el(a).x;
    w.drag([55.0, 101.0], [75.0, 141.0], NONE);
    assert!(close(w.el(a).x, x0 + 20.0));
    assert!(close(crate::world_points(w.el(a))[0][1], 140.0), "os pontos vão com a caixa");
}

#[test]
fn the_sketch_button_flips_the_selection_or_else_the_whole_board_but_never_notes_or_ink() {
    let mut w = world();
    let r1 = w.rect([0.0, 0.0, 100.0, 60.0]);
    let r2 = w.rect([300.0, 0.0, 100.0, 60.0]);
    w.ed.tool = Tool::Shape(ShapeType::Sticky);
    w.click([600.0, 0.0], NONE);
    let note = *w.ed.selection().iter().next().unwrap();
    let ink = w.line(400.0);
    // Com selecção: só ela, e o quadro não muda de modo.
    w.ed.select(&w.doc, [r1]);
    assert_eq!(w.ed.toggle_sketch(&mut w.doc, &mut w.h, false), None);
    assert!(w.el(r1).style().sketch && !w.el(r2).style().sketch);
    // Sem selecção: o quadro inteiro passa a rascunho (havia uma final), e o que nascer também.
    w.ed.select(&w.doc, []);
    assert_eq!(w.ed.toggle_sketch(&mut w.doc, &mut w.h, false), Some(true));
    assert!(w.el(r1).style().sketch && w.el(r2).style().sketch);
    assert!(!w.el(note).style().sketch && !w.el(ink).style().sketch, "a nota é papel; a caneta já é mão");
    let r3 = w.rect([0.0, 200.0, 100.0, 60.0]);
    assert!(w.el(r3).style().sketch, "num quadro em rascunho as formas nascem à mão");
    // E volta: tudo à mão ⇒ final.
    w.ed.select(&w.doc, []);
    assert_eq!(w.ed.toggle_sketch(&mut w.doc, &mut w.h, true), Some(false));
    assert!(!w.el(r1).style().sketch && !w.el(r3).style().sketch);
    assert!(w.cmd(Command::Undo), "UM passo por troca");
    assert!(w.el(r1).style().sketch && w.el(r3).style().sketch);
}

#[test]
fn the_eraser_ring_follows_the_pointer_only_with_an_eraser_in_hand() {
    let mut w = world();
    w.ed.hover(&w.doc, at(30.0, 40.0, NONE));
    assert_eq!(w.ed.overlay(&mut w.doc, &mut w.ts).eraser, None);
    w.ed.tool = Tool::Eraser { precise: true };
    w.ed.hover(&w.doc, at(30.0, 40.0, NONE));
    assert_eq!(w.ed.overlay(&mut w.doc, &mut w.ts).eraser, Some([30.0, 40.0]));
}
