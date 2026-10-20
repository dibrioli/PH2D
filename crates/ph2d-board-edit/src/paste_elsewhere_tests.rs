//! ⭐ B15 (bloco 6, `docs/MiroClone/17_bloco6_plano.md` §2.4): colar NOUTRO quadro. Os ids são um
//! contador por quadro — dois quadros têm ambos o mesmo número —, e o que a cópia não levou (o frame,
//! o alvo do link, o pai do nó) não pode apontar o vizinho com esse número no quadro de destino.

use super::*;
use ph2d_board_model::{BoardOp, ShapeType};

/// O editor (e a cópia dele) passa ao quadro de `other`: os documentos e os históricos trocam.
fn move_to(w: &mut W, other: &mut W) {
    std::mem::swap(&mut w.doc, &mut other.doc);
    std::mem::swap(&mut w.h, &mut other.h);
}

fn rect(w: &mut W, bx: [f64; 4]) -> ElementId {
    w.ed.tool = Tool::Shape(ShapeType::Rectangle);
    w.drag([bx[0], bx[1]], [bx[0] + bx[2], bx[1] + bx[3]], CTRL);
    *w.sel().iter().next().expect("nasceu seleccionada")
}

fn frame(w: &mut W, bx: [f64; 4]) -> ElementId {
    w.ed.commit_text(&mut w.doc, &mut w.h);
    w.ed.tool = Tool::Frame;
    w.drag([bx[0], bx[1]], [bx[0] + bx[2], bx[1] + bx[3]], NONE);
    *w.sel().iter().next().expect("nasceu seleccionado")
}

fn pasted(w: &mut W) -> ElementId {
    assert!(w.cmd(Command::Paste));
    let s = w.sel();
    assert_eq!(s.len(), 1, "uma cópia seleccionada");
    *s.iter().next().unwrap()
}

#[test]
fn a_node_pasted_in_another_board_never_hangs_from_a_strangers_map() {
    let (mut a, root_a) = W::new();
    let kid = a.child("A");
    a.select(kid);
    assert!(a.cmd(Command::Copy));
    let (mut b, root_b) = W::new();
    assert_eq!(root_a, root_b, "a fixtura precisa do MESMO número nos dois quadros");
    b.ed.commit_text(&mut b.doc, &mut b.h);
    move_to(&mut a, &mut b);
    let copy = pasted(&mut a);
    assert_eq!(
        a.el(copy).mind().map(|m| m.parent),
        Some(None),
        "a cópia pendurou-se no mapa do outro quadro"
    );
    assert!(a.doc.mind_children(root_b).is_empty());
}

#[test]
fn a_shape_pasted_in_another_board_never_lands_in_a_strangers_frame() {
    let (mut a, _) = W::new();
    let f = frame(&mut a, [1_000.0, 1_000.0, 800.0, 500.0]);
    let r = rect(&mut a, [1_100.0, 1_100.0, 100.0, 100.0]);
    assert_eq!(a.el(r).frame, Some(f));
    a.select(r);
    assert!(a.cmd(Command::Copy));
    // Controlo: no MESMO quadro a cópia fica na casa do original.
    let home = pasted(&mut a);
    assert_eq!(a.el(home).frame, Some(f), "controlo: a casa do original");
    let (mut b, _) = W::new();
    let g = frame(&mut b, [5_000.0, 5_000.0, 800.0, 500.0]);
    assert_eq!(g, f, "a fixtura precisa do MESMO número nos dois quadros");
    move_to(&mut a, &mut b);
    let copy = pasted(&mut a);
    assert_eq!(a.el(copy).frame, None, "a cópia caiu dentro de um frame alheio");
}

#[test]
fn a_link_pasted_in_another_board_never_points_at_a_stranger() {
    let (mut a, _) = W::new();
    a.ed.commit_text(&mut a.doc, &mut a.h);
    let s1 = rect(&mut a, [1_000.0, 0.0, 100.0, 100.0]);
    let s2 = rect(&mut a, [1_300.0, 0.0, 100.0, 100.0]);
    let mut e = a.el(s2).clone();
    e.set_link(Some(s1));
    let _ = BoardOp::Put(e).apply(&mut a.doc);
    a.select(s2);
    assert!(a.cmd(Command::Copy));
    let (mut b, _) = W::new();
    b.ed.commit_text(&mut b.doc, &mut b.h);
    let t1 = rect(&mut b, [9_000.0, 0.0, 100.0, 100.0]);
    assert_eq!(t1, s1, "a fixtura precisa do MESMO número nos dois quadros");
    move_to(&mut a, &mut b);
    let copy = pasted(&mut a);
    assert_eq!(a.el(copy).link(), None, "o link aponta um estranho");
}

/// Noutro quadro, a cópia sem casa MORA no frame de lá onde caiu (como quem nasce lá).
#[test]
fn a_copy_pasted_in_another_board_lives_in_the_frame_it_lands_in() {
    let (mut a, _) = W::new();
    a.ed.commit_text(&mut a.doc, &mut a.h);
    let r = rect(&mut a, [1_100.0, 1_100.0, 100.0, 100.0]);
    assert_eq!(a.el(r).frame, None);
    a.select(r);
    assert!(a.cmd(Command::Copy));
    let (mut b, _) = W::new();
    let g = frame(&mut b, [-5_000.0, -5_000.0, 15_000.0, 15_000.0]);
    move_to(&mut a, &mut b);
    let copy = pasted(&mut a);
    assert_eq!(a.el(copy).frame, Some(g), "a cópia ficou fora do frame onde caiu");
}

/// Um documento CLONADO é outro quadro (outra origem): colar nele é colar noutro quadro.
#[test]
fn a_cloned_board_is_another_board() {
    let (a, _) = W::new();
    let twin = a.doc.clone();
    assert_ne!(twin.origin(), a.doc.origin());
    assert_eq!(twin, a.doc, "o conteúdo é o mesmo");
}
