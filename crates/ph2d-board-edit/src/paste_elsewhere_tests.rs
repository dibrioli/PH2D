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
    assert_eq!(
        root_a, root_b,
        "a fixtura precisa do MESMO número nos dois quadros"
    );
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
    assert_eq!(
        a.el(copy).frame,
        None,
        "a cópia caiu dentro de um frame alheio"
    );
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
    assert_eq!(
        a.el(copy).frame,
        Some(g),
        "a cópia ficou fora do frame onde caiu"
    );
}

/// Um documento CLONADO é outro quadro (outra origem): colar nele é colar noutro quadro.
#[test]
fn a_cloned_board_is_another_board() {
    let (a, _) = W::new();
    let twin = a.doc.clone();
    assert_ne!(twin.origin(), a.doc.origin());
    assert_eq!(twin, a.doc, "o conteúdo é o mesmo");
}

/// ⭐ W24 — o que vai para OUTRO quadro chega inteiro ENTRE SI: a seta presa às CÓPIAS das duas caixas,
/// a caixa dentro da cópia do frame, o filho pendurado na cópia da raiz — e nada aponta um vizinho de lá
/// com o mesmo número. Pelas duas portas: o `Ctrl+C`/`Ctrl+V` e o «Copy to board…» (`lift`/`land`).
#[test]
fn a_copy_to_another_board_keeps_its_arrows_frames_and_map() {
    use ph2d_board_model::{Anchor, Connector, Element, End, Files, Rgba, Route, Style};
    for door in ["paste", "land"] {
        let (mut a, root) = W::new();
        let kid = a.child("A");
        let f = frame(&mut a, [1_000.0, 1_000.0, 800.0, 500.0]);
        let inner = rect(&mut a, [1_100.0, 1_100.0, 100.0, 100.0]);
        let s1 = rect(&mut a, [3_000.0, 0.0, 100.0, 100.0]);
        let s2 = rect(&mut a, [3_400.0, 0.0, 100.0, 100.0]);
        let ink = Rgba([20, 20, 20, 255]);
        let bound = |target| End::Bound {
            target,
            anchor: Anchor::Center,
        };
        let c = Connector::new(
            bound(s1),
            bound(s2),
            Route::Straight,
            Style::new(None, Some(ink), ink),
        );
        let wire = a.doc.mint_id();
        let el = Element::new_connector(wire, a.doc.z_on_top(), c);
        let _ = BoardOp::Put(el).apply(&mut a.doc);
        assert_eq!(a.el(inner).frame, Some(f));
        a.ed.commit_text(&mut a.doc, &mut a.h);
        a.ed.select(&a.doc, [root, kid, f, s1, s2, wire]);
        // O destino tem os MESMOS números (outro mapa, um frame e caixas), para que um id esquecido
        // aponte um estranho em vez de falhar.
        let (mut b, root_b) = W::new();
        let _ = b.child("B");
        let g = frame(&mut b, [9_000.0, 9_000.0, 800.0, 500.0]);
        for x in [0.0, 300.0, 600.0] {
            rect(&mut b, [x, 9_000.0, 100.0, 100.0]);
        }
        assert_eq!(
            (root_b, g),
            (root, f),
            "a fixtura precisa dos MESMOS números"
        );
        let before: BTreeSet<ElementId> = b.doc.live().map(|e| e.id).collect();
        let doc = if door == "paste" {
            assert!(a.cmd(Command::Copy));
            move_to(&mut a, &mut b);
            assert!(a.cmd(Command::Paste));
            &a.doc
        } else {
            let parcel = a.ed.lift(&a.doc, &Files::default());
            let mut files = Files::default();
            assert!(a.ed.land(&mut b.doc, &mut b.h, &mut files, &parcel, [0.0; 2]));
            &b.doc
        };
        let new: BTreeSet<ElementId> = doc
            .live()
            .map(|e| e.id)
            .filter(|id| !before.contains(id))
            .collect();
        assert_eq!(new.len(), 7, "{door}: as sete cópias");
        let get = |id| doc.get(id).unwrap();
        let only = |p: &dyn Fn(&Element) -> bool| -> Vec<ElementId> {
            new.iter().copied().filter(|&id| p(get(id))).collect()
        };
        let [frame_c] = only(&|e| matches!(e.kind, ph2d_board_model::ElementKind::Frame(_)))[..]
        else {
            panic!("{door}: um frame");
        };
        let [wire_c] = only(&|e| e.connector().is_some())[..] else {
            panic!("{door}: uma seta");
        };
        let roots = only(&|e| e.mind().is_some_and(|m| m.parent.is_none()));
        let kids = only(&|e| e.mind().is_some_and(|m| m.parent.is_some()));
        assert_eq!((roots.len(), kids.len()), (1, 1), "{door}: o mapa");
        assert_eq!(
            get(kids[0]).mind().unwrap().parent,
            Some(roots[0]),
            "{door}: o filho na cópia da raiz"
        );
        let boxes = only(&|e| e.shape().is_some() && e.mind().is_none());
        let in_frame: Vec<_> = boxes
            .iter()
            .filter(|&&id| get(id).frame == Some(frame_c))
            .collect();
        assert_eq!(in_frame.len(), 1, "{door}: a caixa na cópia do frame");
        let c = get(wire_c).connector().unwrap();
        let ends: Vec<ElementId> = [c.start, c.end]
            .into_iter()
            .filter_map(|e| match e {
                End::Bound { target, .. } => Some(target),
                End::Free(_) => None,
            })
            .collect();
        assert_eq!(ends.len(), 2, "{door}: a seta continua presa");
        assert!(
            ends[0] != ends[1]
                && ends
                    .iter()
                    .all(|t| boxes.contains(t) && !in_frame.contains(&t)),
            "{door}: presa às CÓPIAS das duas caixas: {ends:?} · {boxes:?}"
        );
        for &id in &new {
            let e = get(id);
            let refs = [e.frame, e.link(), e.mind().and_then(|m| m.parent)];
            assert!(
                refs.into_iter().flatten().all(|r| new.contains(&r)),
                "{door}: {id:?} aponta um vizinho do destino: {refs:?}"
            );
        }
    }
}
