//! As curas da auditoria de duas lentes da W6 (08/10), cada uma presa ao cenário que a lente
//! reproduziu — com o arnês do mapa ([`super::W`]).

use super::*;
use ph2d_board_model::{Anchor, Connector, End, Route, ShapeType};

fn visible(w: &W, id: ElementId) -> bool {
    w.doc.live().any(|e| e.id == id)
}

/// A1: duplicar um nó e carregar `Enter` no original — duas chaves iguais faziam o `between`
/// entrar em pânico. Cada irmão tem a sua chave, e o irmão novo fica no meio.
#[test]
fn a_duplicated_node_has_its_own_order_and_enter_after_it_works() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    w.select(a);
    assert!(w.cmd(Command::Duplicate));
    let kids = w.doc.mind_children(root);
    assert_eq!(kids.len(), 2);
    let keys: BTreeSet<_> = kids
        .iter()
        .map(|k| w.el(*k).mind().unwrap().order.clone())
        .collect();
    assert_eq!(keys.len(), 2, "chaves próprias");
    w.select(a);
    assert!(w.cmd(Command::EditText));
    assert!(w.cmd(Command::MindSibling));
    assert_eq!(w.doc.mind_children(root).len(), 3);
}

/// A2: uma selecção MISTA (um nó + uma nota) arrastada move a nota; o nó volta ao layout.
#[test]
fn a_mixed_selection_drag_moves_the_note() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    w.ed.commit_text(&mut w.doc, &mut w.h);
    w.ed.tool = Tool::Shape(ShapeType::Sticky);
    w.click([3_000.0, 3_000.0], NONE);
    let note = *w.sel().iter().next().unwrap();
    let c = w.el(note).center();
    let ay = w.el(a).y;
    w.ed.select(&w.doc, [a, note]);
    w.drag(c, [c[0] + 300.0, c[1]], NONE);
    assert!(
        (w.el(note).center()[0] - c[0] - 300.0).abs() < 1e-6,
        "a nota andou"
    );
    assert!(
        (w.el(a).y - ay).abs() < 1e-9,
        "o nó arrumado voltou ao sítio"
    );
    let _ = root;
}

/// A2 (frame): apagar um frame leva os nós ESCONDIDOS de um ramo recolhido dentro dele; copiá-lo leva
/// os filhos escondidos também.
#[test]
fn a_frame_takes_the_collapsed_branch_inside_it() {
    let (mut w, _) = W::new();
    w.ed.commit_text(&mut w.doc, &mut w.h);
    w.ed.tool = Tool::Frame;
    w.drag([1_000.0, 1_000.0], [1_800.0, 1_500.0], NONE);
    let f = *w.sel().iter().next().unwrap();
    w.ed.tool = Tool::MindMap;
    w.click([1_200.0, 1_250.0], NONE);
    let a = w.child("A");
    let a1 = w.child("A1");
    w.select(a);
    assert!(w.cmd(Command::MindCollapse));
    w.select(f);
    assert!(w.cmd(Command::Copy));
    assert!(w.cmd(Command::Paste));
    let copies = w.doc.live_with_hidden().count();
    // «Plan» (o mapa do arnês, fora do frame) + o frame, a raiz, A e A1 — e a cópia dos quatro.
    assert_eq!(copies, 1 + 2 * 4, "a cópia do frame leva o filho escondido");
    w.select(f);
    assert!(w.cmd(Command::Delete));
    assert!(w.doc.get(a1).is_none(), "o escondido sai com o frame");
}

/// A4: o modelo de mapa inserido longe da origem não mora no frame que cobre a origem.
#[test]
fn a_template_map_lives_where_it_is_inserted() {
    let (mut w, _) = W::new();
    w.ed.commit_text(&mut w.doc, &mut w.h);
    w.ed.tool = Tool::Frame;
    w.drag([-300.0, -300.0], [300.0, 300.0], NONE);
    let ids = w.ed.insert_template(
        &mut w.doc,
        &mut w.h,
        crate::Template::MindMap,
        [8_000.0, 8_000.0],
        &|k| k.to_owned(),
    );
    let r = ids[0];
    assert!(
        std::iter::once(r)
            .chain(w.doc.mind_descendants(r))
            .all(|id| w.doc.get(id).unwrap().frame.is_none())
    );
}

/// A5: largar um nó sobre um nó RECOLHIDO abre-o (o largado não some).
#[test]
fn dropping_onto_a_collapsed_node_opens_it() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    w.child("A1");
    w.select(a);
    assert!(w.cmd(Command::MindCollapse));
    w.select(root);
    let b = w.child("Bee");
    w.select(b);
    let (from, to) = (w.el(b).center(), w.el(a).center());
    w.drag(from, to, NONE);
    assert_eq!(w.doc.mind_parent(b), Some(a));
    assert!(visible(&w, b), "o pai abriu-se");
}

/// A6/A11: um nó de mapa não se trava (o Miro), e por isso o `Delete` nunca deixa órfãos.
#[test]
fn a_mind_node_does_not_lock() {
    let (mut w, _) = W::new();
    let a = w.child("A");
    w.select(a);
    w.cmd(Command::Lock);
    assert!(!w.el(a).locked());
}

/// A7: colar um ramo recolhido não selecciona os escondidos.
#[test]
fn pasting_a_collapsed_branch_selects_only_what_shows() {
    let (mut w, _) = W::new();
    let a = w.child("A");
    w.child("A1");
    w.select(a);
    assert!(w.cmd(Command::MindCollapse));
    w.select(a);
    assert!(w.cmd(Command::Copy));
    assert!(w.cmd(Command::Paste));
    assert!(!w.sel().is_empty());
    assert!(w.sel().iter().all(|id| visible(&w, *id)));
}

/// A8: redimensionar um nó arruma o mapa no fim do gesto (o filho sai de dentro do pai alargado).
#[test]
fn resizing_a_node_settles_the_map() {
    let (mut w, _) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    w.select(a);
    let e = w.el(a).clone();
    let grab = [e.x + e.w, e.y + e.h / 2.0];
    w.drag(grab, [grab[0] + 120.0, grab[1]], NONE);
    let (a, a1) = (w.el(a).clone(), w.el(a1).clone());
    assert!((a1.x - (a.x + a.w + ph2d_board_model::MIND_LEVEL_GAP)).abs() < 1e-6);
}

/// A9: a cor de um ramo novo é a seguinte à do ÚLTIMO (apagar um do meio não a repete).
#[test]
fn a_new_branch_never_repeats_its_last_siblings_colour() {
    let (mut w, root) = W::new();
    w.child("A");
    w.select(root);
    let b = w.child("B");
    w.select(root);
    let c = w.child("C");
    w.select(b);
    assert!(w.cmd(Command::Delete));
    w.select(root);
    let d = w.child("D");
    assert_ne!(w.el(d).mind().unwrap().color, w.el(c).mind().unwrap().color);
}

/// A10: `Tab` num pai SEM auto layout põe o filho ao lado, nunca por cima.
#[test]
fn tab_on_a_manual_parent_does_not_cover_it() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    w.select(a);
    assert!(w.cmd(Command::MindAuto));
    let c = w.child("C");
    let (pa, pc) = (w.el(a).aabb(), w.el(c).aabb());
    let overlap = pa[0] < pc[2] && pc[0] < pa[2] && pa[1] < pc[3] && pc[1] < pa[3];
    assert!(!overlap, "{pa:?} × {pc:?}");
    let _ = root;
}

/// A11: colar um ramo cujo pai já não existe faz dele a raiz de um mapa novo (forma de raiz).
#[test]
fn a_branch_pasted_after_its_parent_is_gone_is_a_new_map() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    w.select(a1);
    assert!(w.cmd(Command::Copy));
    w.select(a);
    assert!(w.cmd(Command::Delete));
    assert!(w.cmd(Command::Paste));
    let pasted = *w.sel().iter().next().unwrap();
    assert_eq!(w.doc.mind_root(pasted), Some(pasted));
    assert_eq!(w.el(pasted).shape().unwrap().kind, ShapeType::Pill);
    let _ = root;
}

/// B5: um nó recolhido não tem «+» do lado do selo, e o selo ganha a um «+» a zoom alto.
#[test]
fn the_badge_side_has_no_plus() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    w.child("A1");
    w.select(a);
    assert!(w.cmd(Command::MindCollapse));
    w.select(a);
    assert!(
        w.ed.mind_plus(&w.doc).is_empty(),
        "o lado de fora é do selo"
    );
    let n = ph2d_board_model::hidden_count(&w.doc, a);
    let [x, y, bw, bh] = ph2d_board_model::badge_rect(&w.doc, a, digits(n)).unwrap();
    let p = Pointer {
        world: [x + bw / 2.0, y + bh / 2.0],
        px: 0.5,
        mods: NONE,
    };
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, p, VIEW);
    w.ed.pointer_up(&mut w.doc, &mut w.h, p);
    assert_eq!(w.doc.mind_children(a).len(), 1, "o selo só expande");
    assert!(!w.el(a).mind().unwrap().collapsed);
    assert_eq!(
        w.sel(),
        BTreeSet::from([a]),
        "e selecciona o nó (o Ctrl+↑ seguinte fecha ESTE)"
    );
    let _ = root;
}

/// B1: `Grow` (o `Ctrl+seta` das formas) recusa um nó — o clone levava o pai e a chave dele.
#[test]
fn grow_refuses_a_mind_node() {
    let (mut w, _) = W::new();
    let a = w.child("A");
    w.select(a);
    let n = w.doc.live_len();
    assert!(!w.cmd(Command::Grow([1.0, 0.0])));
    assert_eq!(w.doc.live_len(), n);
}

/// B10: empurrar nós arrumados não deixa um passo de desfazer vazio.
#[test]
fn nudging_laid_nodes_leaves_no_empty_step() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let b = w.sibling("B");
    w.ed.commit_text(&mut w.doc, &mut w.h);
    w.ed.select(&w.doc, [a, b]);
    let ay = w.el(a).y;
    w.cmd(Command::Nudge([0.0, 10.0]));
    assert!((w.el(a).y - ay).abs() < 1e-9);
    assert!(w.cmd(Command::Undo));
    assert_eq!(
        w.el(b).shape().unwrap().text.as_str(),
        "",
        "o último passo é o texto de B, não um empurrão vazio"
    );
    let _ = root;
}

/// Uma seta presa a um nó escondido some com ele: o clique não a apanha.
#[test]
fn an_arrow_to_a_hidden_node_is_not_hit() {
    let (mut w, root) = W::new();
    let a = w.child("A");
    let a1 = w.child("A1");
    w.ed.commit_text(&mut w.doc, &mut w.h);
    let c = Connector::new(
        End::Free([5_000.0, 5_000.0]),
        End::Bound {
            target: a1,
            anchor: Anchor::Center,
        },
        Route::Straight,
        Style::new(None, Some(Rgba([0, 0, 0, 255])), Rgba([0, 0, 0, 255])),
    );
    let arrow = Element::new_connector(w.doc.mint_id(), w.doc.z_on_top(), c);
    let id = arrow.id;
    w.h.apply(&mut w.doc, vec![BoardOp::Put(arrow)]);
    w.select(a);
    assert!(w.cmd(Command::MindCollapse));
    assert!(w.doc.connector_hidden(w.doc.get(id).unwrap()));
    let p = Pointer {
        world: [5_000.0, 5_000.0],
        px: 1.0,
        mods: NONE,
    };
    assert_ne!(w.ed.hit(&w.doc, p), Some(id));
    let _ = root;
}
